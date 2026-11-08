//! Fornitore `anthropic`: API Messages (`POST {base_url}/v1/messages`, §6.1).
//!
//! - Header: `x-api-key` (solo se c'è una chiave) e sempre
//!   `anthropic-version: 2023-06-01`.
//! - Le istruzioni di sistema vanno nel campo `system`; i messaggi alternano
//!   `user` e `assistant` (anche nei tentativi di correzione).
//! - Con `json_mode` si usano gli output strutturati
//!   (`output_config.format = {"type": "json_schema", ...}`), senza header beta e
//!   senza "prefill". Gli output strutturati esigono `additionalProperties:
//!   false` su tutti gli oggetti, quindi si invia lo schema rigoroso
//!   ([`crate::ai::schema::schema_rigoroso`]), che rappresenta le mappe come
//!   elenchi di coppie.
//! - Ragionamento (§5.4, verificato sulla documentazione corrente): per
//!   `basso`/`medio`/`alto` si invia `thinking: {"type": "adaptive"}` e
//!   `output_config.effort` (`low`/`medium`/`high`); con il ragionamento attivo
//!   `temperature` non è ammessa e viene omessa. `budget_tokens` non si usa più
//!   (i modelli recenti lo rifiutano).
//! - I modelli più recenti rifiutano anche `temperature` senza ragionamento:
//!   come per ogni parametro facoltativo, un 400 che lo cita fa ripetere la
//!   richiesta senza, una sola volta per sessione.
//! - Testo: concatenazione dei blocchi `content[*]` di tipo `text` (i blocchi
//!   `thinking` vengono ignorati); token di prompt da `usage.input_tokens`.
//!   `stop_reason = "refusal"` produce un messaggio chiaro.
//! - In streaming (testo progressivo) invia `stream: true` e ricompone il
//!   testo dagli eventi `content_block_delta` di tipo `text_delta`.

use std::time::{Duration, Instant};

use serde_json::{Value, json};
use tracing::{debug, info};

use super::flusso::dati_sse;
use super::http::{
    ClientHttp, ParametriRifiutati, StileChiave, avviso_parametro, base_senza_barra, corpo_json,
    errore_da_stato, inviati,
};
use super::ragionamento::Ragionamento;
use super::{ErroreAi, Narratore, RichiestaAi, RispostaGrezza, Ruolo, schema, separa_sistema};
use crate::config::ConfigAi;

/// Parametri facoltativi che il server può rifiutare, in ordine di priorità.
const FACOLTATIVI: [&str; 4] = ["effort", "thinking", "temperature", "output_config"];

/// Client Anthropic.
#[derive(Debug)]
pub struct ClientAnthropic {
    http: ClientHttp,
    url: String,
    modello: String,
    temperatura: f32,
    max_token: u32,
    ragionamento: Ragionamento,
    json_mode: bool,
    rifiutati: ParametriRifiutati,
}

impl ClientAnthropic {
    /// Crea il client dalla configurazione.
    pub fn nuovo(config: &ConfigAi) -> Result<Self, ErroreAi> {
        let mut base = base_senza_barra(&config.base_url);
        if let Some(senza) = base.strip_suffix("/v1") {
            base = senza.to_string();
        }
        Ok(ClientAnthropic {
            http: ClientHttp::nuovo(config, StileChiave::Anthropic)?,
            url: format!("{base}/v1/messages"),
            modello: config.modello.clone(),
            temperatura: config.temperatura,
            max_token: config.max_token,
            ragionamento: config.ragionamento,
            json_mode: config.json_mode,
            rifiutati: ParametriRifiutati::default(),
        })
    }

    /// Cambia l'attesa base del backoff (per i test).
    pub fn con_backoff(mut self, base: Duration) -> Self {
        self.http.imposta_backoff(base);
        self
    }

    /// Corpo JSON della richiesta.
    pub fn corpo(&self, richiesta: &RichiestaAi) -> Value {
        let (sistema, altri) = separa_sistema(&richiesta.messaggi);
        let messaggi: Vec<Value> = altri
            .iter()
            .map(|m| {
                let ruolo = if m.ruolo == Ruolo::Assistant {
                    "assistant"
                } else {
                    "user"
                };
                json!({ "role": ruolo, "content": m.contenuto })
            })
            .collect();
        let mut corpo = json!({
            "model": self.modello,
            "max_tokens": self.max_token,
            "system": sistema,
            "messages": messaggi,
        });
        let effort = self
            .ragionamento
            .per_anthropic()
            .filter(|_| self.rifiutati.attivo("thinking"));
        let mut output_config = serde_json::Map::new();
        if self.json_mode && self.rifiutati.attivo("output_config") {
            output_config.insert(
                "format".into(),
                json!({ "type": "json_schema", "schema": schema::schema_rigoroso() }),
            );
        }
        if let Some(livello) = effort {
            corpo["thinking"] = json!({ "type": "adaptive" });
            if self.rifiutati.attivo("effort") && self.rifiutati.attivo("output_config") {
                output_config.insert("effort".into(), json!(livello));
            }
        } else if self.rifiutati.attivo("temperature") {
            corpo["temperature"] = json!(self.temperatura);
        }
        if !output_config.is_empty() {
            corpo["output_config"] = Value::Object(output_config);
        }
        corpo
    }
}

/// Testo e metadati di una risposta (letta per intero o ricomposta dallo streaming).
#[derive(Debug, Default)]
struct Raccolta {
    testo: String,
    fine: Option<String>,
    token_prompt: Option<u64>,
    token_generati: Option<u64>,
    errore: Option<String>,
}

impl Raccolta {
    /// Da una risposta letta per intero.
    fn da_valore(valore: &Value) -> Result<Self, ErroreAi> {
        let testo: String = valore["content"]
            .as_array()
            .ok_or_else(|| ErroreAi::Formato("manca content".to_string()))?
            .iter()
            .filter(|b| b["type"] == "text")
            .filter_map(|b| b["text"].as_str())
            .collect::<Vec<_>>()
            .join("");
        Ok(Raccolta {
            testo,
            fine: valore["stop_reason"].as_str().map(str::to_string),
            token_prompt: valore["usage"]["input_tokens"].as_u64(),
            token_generati: valore["usage"]["output_tokens"].as_u64(),
            errore: None,
        })
    }

    /// Un evento dello streaming: `message_start` (token di prompt),
    /// `content_block_delta` di tipo `text_delta` (testo; i `thinking_delta`
    /// si ignorano), `message_delta` (stop_reason e token generati), `error`.
    fn aggiungi_evento(&mut self, evento: &Value, pezzo: &mut dyn FnMut(&str)) {
        match evento["type"].as_str() {
            Some("message_start") => {
                self.token_prompt = evento["message"]["usage"]["input_tokens"].as_u64();
            }
            Some("content_block_delta") if evento["delta"]["type"] == "text_delta" => {
                if let Some(t) = evento["delta"]["text"].as_str() {
                    self.testo.push_str(t);
                    pezzo(t);
                }
            }
            Some("message_delta") => {
                if let Some(f) = evento["delta"]["stop_reason"].as_str() {
                    self.fine = Some(f.to_string());
                }
                if let Some(n) = evento["usage"]["output_tokens"].as_u64() {
                    self.token_generati = Some(n);
                }
            }
            Some("error") => {
                let e = &evento["error"];
                self.errore = Some(e["message"].as_str().unwrap_or(&e.to_string()).to_string());
            }
            _ => {}
        }
    }
}

impl ClientAnthropic {
    /// Richiesta completa; con `flusso` la risposta arriva in streaming.
    fn esegui(
        &self,
        richiesta: &RichiestaAi,
        mut flusso: Option<&mut dyn FnMut(&str)>,
    ) -> Result<RispostaGrezza, ErroreAi> {
        let inizio = Instant::now();
        let mut avvisi = Vec::new();
        loop {
            let mut corpo = self.corpo(richiesta);
            let mut raccolta = Raccolta::default();
            if flusso.is_some() {
                corpo["stream"] = json!(true);
            }
            info!(
                url = %self.url,
                modello = %self.modello,
                messaggi = richiesta.messaggi.len(),
                streaming = flusso.is_some(),
                "richiesta al narratore (anthropic)"
            );
            debug!(corpo = %corpo, "corpo della richiesta");
            let risposta = match flusso.as_deref_mut() {
                Some(pezzo) => self.http.post_json_flusso(&self.url, &corpo, &mut |riga| {
                    if let Some(evento) =
                        dati_sse(riga).and_then(|d| serde_json::from_str::<Value>(d).ok())
                    {
                        raccolta.aggiungi_evento(&evento, pezzo);
                    }
                })?,
                None => self.http.post_json(&self.url, &corpo)?,
            };
            info!(
                stato = risposta.stato,
                byte = risposta.corpo.len(),
                "risposta ricevuta (anthropic)"
            );
            debug!(corpo = %risposta.corpo, "risposta grezza (anthropic)");
            if let Some(parametro) = self
                .rifiutati
                .gestisci(&risposta, &inviati(&corpo, &FACOLTATIVI))
            {
                // `temperature` rifiutata dai modelli recenti è normale: solo nel log.
                if parametro != "temperature" {
                    avvisi.push(avviso_parametro(&parametro));
                }
                continue;
            }
            if risposta.stato >= 400 {
                return Err(errore_da_stato(
                    risposta.stato,
                    &risposta.corpo,
                    &format!(
                        "controlla base_url e il nome del modello («{}»)",
                        self.modello
                    ),
                ));
            }
            if flusso.is_none() {
                raccolta = Raccolta::da_valore(&corpo_json(&risposta)?)?;
            } else if let Some(errore) = raccolta.errore.take() {
                return Err(ErroreAi::Formato(format!(
                    "errore durante lo streaming: {errore}"
                )));
            }
            let Raccolta {
                testo,
                fine,
                token_prompt,
                token_generati,
                ..
            } = raccolta;
            match fine.as_deref() {
                Some("refusal") => {
                    return Err(ErroreAi::Formato(
                        "il modello ha rifiutato di rispondere (stop_reason = refusal): \
                         riformula l'azione"
                            .to_string(),
                    ));
                }
                Some("max_tokens") if testo.trim().is_empty() => {
                    return Err(ErroreAi::Formato(
                        "la risposta è stata troncata da max_token prima di produrre testo: \
                         aumenta max_token o abbassa ragionamento"
                            .to_string(),
                    ));
                }
                _ => {}
            }
            return Ok(RispostaGrezza {
                testo,
                token_prompt,
                token_generati,
                durata: inizio.elapsed(),
                avvisi,
            });
        }
    }
}

impl Narratore for ClientAnthropic {
    fn turno(&self, richiesta: &RichiestaAi) -> Result<RispostaGrezza, ErroreAi> {
        self.esegui(richiesta, None)
    }

    fn turno_in_flusso(
        &self,
        richiesta: &RichiestaAi,
        pezzo: &mut dyn FnMut(&str),
    ) -> Result<RispostaGrezza, ErroreAi> {
        self.esegui(richiesta, Some(pezzo))
    }

    fn descrizione(&self) -> String {
        format!("anthropic · {} · {}", self.modello, self.url)
    }

    fn json_mode_attivo(&self) -> Option<bool> {
        Some(self.json_mode && self.rifiutati.attivo("output_config"))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::ai::Messaggio;
    use crate::config::Fornitore;

    fn config(ragionamento: Ragionamento) -> ConfigAi {
        ConfigAi {
            fornitore: Fornitore::Anthropic,
            base_url: "https://api.anthropic.com".into(),
            modello: "claude-sonnet-5-5".into(),
            ragionamento,
            ..ConfigAi::default()
        }
    }

    #[test]
    fn corpo_con_sistema_separato_e_output_strutturato() {
        let c = ClientAnthropic::nuovo(&config(Ragionamento::Predefinito)).unwrap();
        assert_eq!(c.url, "https://api.anthropic.com/v1/messages");
        let r = RichiestaAi {
            messaggi: vec![
                Messaggio::sistema("regole"),
                Messaggio::utente("u"),
                Messaggio::assistente("sbagliata"),
                Messaggio::utente("correggi"),
            ],
            contesto: Default::default(),
        };
        let corpo = c.corpo(&r);
        assert_eq!(corpo["system"], "regole");
        assert_eq!(corpo["messages"].as_array().unwrap().len(), 3);
        assert_eq!(corpo["messages"][1]["role"], "assistant");
        assert_eq!(corpo["output_config"]["format"]["type"], "json_schema");
        assert_eq!(corpo["temperature"], json!(0.7f32));
        assert!(corpo.get("thinking").is_none());
    }

    #[test]
    fn ragionamento_adattivo_senza_temperatura() {
        let c = ClientAnthropic::nuovo(&config(Ragionamento::Medio)).unwrap();
        let corpo = c.corpo(&RichiestaAi {
            messaggi: vec![Messaggio::utente("u")],
            contesto: Default::default(),
        });
        assert_eq!(corpo["thinking"]["type"], "adaptive");
        assert_eq!(corpo["output_config"]["effort"], "medium");
        assert!(corpo.get("temperature").is_none());
    }
}
