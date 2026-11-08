//! Fornitore `ollama`: API nativa (`POST {base_url}/api/chat`, §6.1).
//!
//! Invia `stream: false`, `format: "json"` (se `json_mode`), `think` secondo
//! `ragionamento` (§5.4) e `options.num_ctx/temperature/num_predict`: così il
//! contesto è garantito senza configurare il server. Legge la risposta da
//! `message.content` (il ragionamento, se presente, arriva separato in
//! `message.thinking` e viene ignorato) e i token di prompt da
//! `prompt_eval_count`. Un modello non scaricato produce un messaggio che
//! suggerisce `ollama pull <modello>`; un modello che non supporta `think`
//! fa ripetere la richiesta senza quel campo.
//!
//! In streaming (testo progressivo) invia `stream: true` e ricompone il testo
//! dalle righe JSON della risposta; l'ultima (`done: true`) porta i conteggi.

use std::time::{Duration, Instant};

use serde_json::{Value, json};
use tracing::{debug, info};

use super::http::{
    ClientHttp, ParametriRifiutati, StileChiave, avviso_parametro, base_senza_barra, corpo_json,
    errore_da_stato, estratto, inviati,
};
use super::ragionamento::Ragionamento;
use super::{ErroreAi, Narratore, RichiestaAi, RispostaGrezza};
use crate::config::ConfigAi;

/// Parametri facoltativi che il server può rifiutare.
const FACOLTATIVI: [&str; 2] = ["think", "format"];

/// Client Ollama nativo.
#[derive(Debug)]
pub struct ClientOllama {
    http: ClientHttp,
    url: String,
    modello: String,
    temperatura: f32,
    max_token: u32,
    contesto_token: u32,
    ragionamento: Ragionamento,
    json_mode: bool,
    rifiutati: ParametriRifiutati,
}

impl ClientOllama {
    /// Crea il client dalla configurazione.
    pub fn nuovo(config: &ConfigAi) -> Result<Self, ErroreAi> {
        let mut base = base_senza_barra(&config.base_url);
        // Errore frequente: usare l'URL dell'API compatibile OpenAI.
        if let Some(senza) = base.strip_suffix("/v1") {
            base = senza.to_string();
        }
        Ok(ClientOllama {
            http: ClientHttp::nuovo(config, StileChiave::Bearer)?,
            url: format!("{base}/api/chat"),
            modello: config.modello.clone(),
            temperatura: config.temperatura,
            max_token: config.max_token,
            contesto_token: config.contesto_token,
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
        let mut opzioni = json!({
            "temperature": self.temperatura,
            "num_predict": self.max_token,
        });
        if self.contesto_token > 0 {
            opzioni["num_ctx"] = json!(self.contesto_token);
        }
        let mut corpo = json!({
            "model": self.modello,
            "stream": false,
            "options": opzioni,
            "messages": richiesta.messaggi,
        });
        if self.json_mode && self.rifiutati.attivo("format") {
            corpo["format"] = json!("json");
        }
        if let Some(think) = self.ragionamento.per_ollama()
            && self.rifiutati.attivo("think")
        {
            corpo["think"] = think;
        }
        corpo
    }
}

/// Testo e metadati di una risposta (letta per intero o ricomposta dallo streaming).
#[derive(Debug, Default)]
struct Raccolta {
    testo: String,
    contenuto: bool,
    fine: Option<String>,
    token_prompt: Option<u64>,
    token_generati: Option<u64>,
    errore: Option<String>,
}

impl Raccolta {
    /// Aggiunge una risposta completa o una riga dello streaming (stesso formato:
    /// `message.content`, e nell'ultima `done_reason` e i conteggi dei token).
    fn aggiungi(&mut self, valore: &Value, pezzo: &mut dyn FnMut(&str)) {
        if let Some(e) = valore.get("error") {
            self.errore = Some(e.as_str().unwrap_or(&e.to_string()).to_string());
            return;
        }
        if let Some(t) = valore["message"]["content"].as_str() {
            self.contenuto = true;
            if !t.is_empty() {
                self.testo.push_str(t);
                pezzo(t);
            }
        }
        if let Some(f) = valore["done_reason"].as_str() {
            self.fine = Some(f.to_string());
        }
        if let Some(n) = valore["prompt_eval_count"].as_u64() {
            self.token_prompt = Some(n);
        }
        if let Some(n) = valore["eval_count"].as_u64() {
            self.token_generati = Some(n);
        }
    }
}

impl ClientOllama {
    /// Richiesta completa; con `flusso` la risposta arriva in streaming (JSON a righe).
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
                num_ctx = self.contesto_token,
                messaggi = richiesta.messaggi.len(),
                streaming = flusso.is_some(),
                "richiesta al narratore (ollama)"
            );
            debug!(corpo = %corpo, "corpo della richiesta");
            let risposta = match flusso.as_deref_mut() {
                Some(pezzo) => self.http.post_json_flusso(&self.url, &corpo, &mut |riga| {
                    if let Ok(evento) = serde_json::from_str::<Value>(riga.trim()) {
                        raccolta.aggiungi(&evento, pezzo);
                    }
                })?,
                None => self.http.post_json(&self.url, &corpo)?,
            };
            info!(
                stato = risposta.stato,
                byte = risposta.corpo.len(),
                "risposta ricevuta (ollama)"
            );
            debug!(corpo = %risposta.corpo, "risposta grezza (ollama)");
            let minuscolo = risposta.corpo.to_lowercase();
            let modello_mancante = risposta.stato == 404
                || (risposta.stato >= 400
                    && minuscolo.contains("not found")
                    && minuscolo.contains("model"));
            if modello_mancante {
                return Err(ErroreAi::NonTrovato(format!(
                    "il modello «{m}» non è disponibile in Ollama: esegui `ollama pull {m}` \
                     (dettagli: {})",
                    estratto(&risposta.corpo),
                    m = self.modello
                )));
            }
            if let Some(parametro) = self
                .rifiutati
                .gestisci(&risposta, &inviati(&corpo, &FACOLTATIVI))
            {
                avvisi.push(avviso_parametro(&parametro));
                continue;
            }
            if risposta.stato >= 400 {
                return Err(errore_da_stato(risposta.stato, &risposta.corpo, &self.url));
            }
            if flusso.is_none() {
                raccolta.aggiungi(&corpo_json(&risposta)?, &mut |_| {});
            }
            if let Some(errore) = raccolta.errore.take() {
                return Err(ErroreAi::Formato(format!("errore di Ollama: {errore}")));
            }
            if !raccolta.contenuto {
                return Err(ErroreAi::Formato("manca message.content".to_string()));
            }
            let testo = raccolta.testo;
            if testo.trim().is_empty() && raccolta.fine.as_deref() == Some("length") {
                return Err(ErroreAi::Formato(
                    "la risposta è stata troncata da max_token prima di produrre testo: \
                     aumenta max_token o imposta ragionamento = \"spento\""
                        .to_string(),
                ));
            }
            return Ok(RispostaGrezza {
                testo,
                token_prompt: raccolta.token_prompt,
                token_generati: raccolta.token_generati,
                durata: inizio.elapsed(),
                avvisi,
            });
        }
    }
}

impl Narratore for ClientOllama {
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
        format!("ollama · {} · {}", self.modello, self.url)
    }

    fn json_mode_attivo(&self) -> Option<bool> {
        Some(self.json_mode && self.rifiutati.attivo("format"))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::ai::Messaggio;
    use crate::config::Fornitore;

    #[test]
    fn corpo_della_richiesta() {
        let config = ConfigAi {
            fornitore: Fornitore::Ollama,
            base_url: "http://localhost:11434/v1/".into(),
            modello: "gemma4:12b".into(),
            contesto_token: 16384,
            ragionamento: Ragionamento::Spento,
            ..ConfigAi::default()
        };
        let c = ClientOllama::nuovo(&config).unwrap();
        assert_eq!(c.url, "http://localhost:11434/api/chat");
        let r = RichiestaAi {
            messaggi: vec![Messaggio::utente("ciao")],
            contesto: Default::default(),
        };
        let corpo = c.corpo(&r);
        assert_eq!(corpo["stream"], false);
        assert_eq!(corpo["format"], "json");
        assert_eq!(corpo["think"], false);
        assert_eq!(corpo["options"]["num_ctx"], 16384);
        assert_eq!(corpo["options"]["num_predict"], 1200);

        let config = ConfigAi {
            contesto_token: 0,
            json_mode: false,
            ragionamento: Ragionamento::Predefinito,
            ..config
        };
        let corpo = ClientOllama::nuovo(&config).unwrap().corpo(&r);
        assert!(corpo.get("format").is_none());
        assert!(corpo.get("think").is_none());
        assert!(corpo["options"].get("num_ctx").is_none());
    }
}
