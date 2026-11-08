//! Fornitori `openai` e `lmstudio`: `POST {base_url}/chat/completions` (§6.1).
//!
//! - `openai`: con `json_mode` invia `response_format = {"type":"json_object"}`.
//! - `lmstudio`: la documentazione di LM Studio prevede solo
//!   `response_format` di tipo `json_schema`, quindi invia lo schema della
//!   risposta ([`crate::ai::schema::schema_risposta`]).
//!
//! `reasoning_effort` è inviato solo se `ragionamento` non è `predefinito`.
//! Se il server rifiuta un parametro facoltativo (400 che lo cita:
//! `response_format`, `reasoning_effort`, `temperature`) si riprova una volta
//! senza e si avvisa una sola volta. Se rifiuta `max_tokens` si passa a
//! `max_completion_tokens` per il resto della sessione. Legge
//! `choices[0].message.content` e `usage.prompt_tokens`.
//!
//! In streaming (testo progressivo) invia `stream: true` e
//! `stream_options.include_usage` (tolto se rifiutato) e ricompone il testo dai
//! `choices[0].delta.content` degli eventi `data:`.

use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use serde_json::{Value, json};
use tracing::{debug, info};

use super::flusso::dati_sse;
use super::http::{
    ClientHttp, ParametriRifiutati, StileChiave, avviso_parametro, base_senza_barra, corpo_json,
    errore_da_stato, inviati,
};
use super::ragionamento::Ragionamento;
use super::{ErroreAi, Narratore, RichiestaAi, RispostaGrezza, schema};
use crate::config::{ConfigAi, Fornitore};

/// Parametri facoltativi che il server può rifiutare.
const FACOLTATIVI: [&str; 4] = [
    "stream_options",
    "response_format",
    "reasoning_effort",
    "temperature",
];

/// Client compatibile OpenAI (anche LM Studio).
#[derive(Debug)]
pub struct ClientOpenAi {
    http: ClientHttp,
    fornitore: Fornitore,
    url: String,
    modello: String,
    temperatura: f32,
    max_token: u32,
    ragionamento: Ragionamento,
    json_mode: bool,
    usa_max_completion_tokens: AtomicBool,
    rifiutati: ParametriRifiutati,
}

impl ClientOpenAi {
    /// Crea il client dalla configurazione.
    pub fn nuovo(config: &ConfigAi) -> Result<Self, ErroreAi> {
        Ok(ClientOpenAi {
            http: ClientHttp::nuovo(config, StileChiave::Bearer)?,
            fornitore: config.fornitore,
            url: format!("{}/chat/completions", base_senza_barra(&config.base_url)),
            modello: config.modello.clone(),
            temperatura: config.temperatura,
            max_token: config.max_token,
            ragionamento: config.ragionamento,
            json_mode: config.json_mode,
            usa_max_completion_tokens: AtomicBool::new(false),
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
        let mut corpo = json!({
            "model": self.modello,
            "messages": richiesta.messaggi,
        });
        let campo_token = if self.usa_max_completion_tokens.load(Ordering::Relaxed) {
            "max_completion_tokens"
        } else {
            "max_tokens"
        };
        corpo[campo_token] = json!(self.max_token);
        if self.rifiutati.attivo("temperature") {
            corpo["temperature"] = json!(self.temperatura);
        }
        if self.json_mode && self.rifiutati.attivo("response_format") {
            corpo["response_format"] = if self.fornitore == Fornitore::Lmstudio {
                json!({
                    "type": "json_schema",
                    "json_schema": {
                        "name": "risposta_narratore",
                        "schema": schema::schema_risposta()
                    }
                })
            } else {
                json!({ "type": "json_object" })
            };
        }
        if let Some(livello) = self.ragionamento.per_openai()
            && self.rifiutati.attivo("reasoning_effort")
        {
            corpo["reasoning_effort"] = json!(livello);
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
        let scelta = &valore["choices"][0];
        let testo = scelta["message"]["content"]
            .as_str()
            .ok_or_else(|| ErroreAi::Formato("manca choices[0].message.content".to_string()))?
            .to_string();
        Ok(Raccolta {
            testo,
            fine: scelta["finish_reason"].as_str().map(str::to_string),
            token_prompt: valore["usage"]["prompt_tokens"].as_u64(),
            token_generati: valore["usage"]["completion_tokens"].as_u64(),
            errore: None,
        })
    }

    /// Un evento `data:` dello streaming (`choices[0].delta.content`, `usage`).
    fn aggiungi_evento(&mut self, evento: &Value, pezzo: &mut dyn FnMut(&str)) {
        if let Some(e) = evento.get("error") {
            self.errore = Some(e["message"].as_str().unwrap_or(&e.to_string()).to_string());
            return;
        }
        let scelta = &evento["choices"][0];
        if let Some(t) = scelta["delta"]["content"].as_str()
            && !t.is_empty()
        {
            self.testo.push_str(t);
            pezzo(t);
        }
        if let Some(f) = scelta["finish_reason"].as_str() {
            self.fine = Some(f.to_string());
        }
        if let Some(n) = evento["usage"]["prompt_tokens"].as_u64() {
            self.token_prompt = Some(n);
        }
        if let Some(n) = evento["usage"]["completion_tokens"].as_u64() {
            self.token_generati = Some(n);
        }
    }
}

impl ClientOpenAi {
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
                if self.rifiutati.attivo("stream_options") {
                    corpo["stream_options"] = json!({ "include_usage": true });
                }
            }
            info!(
                url = %self.url,
                fornitore = %self.fornitore,
                modello = %self.modello,
                messaggi = richiesta.messaggi.len(),
                streaming = flusso.is_some(),
                "richiesta al narratore"
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
                "risposta ricevuta"
            );
            debug!(corpo = %risposta.corpo, "risposta grezza");
            if risposta.stato == 400
                && !self.usa_max_completion_tokens.load(Ordering::Relaxed)
                && risposta.corpo.contains("max_tokens")
            {
                info!("il modello richiede max_completion_tokens al posto di max_tokens");
                self.usa_max_completion_tokens
                    .store(true, Ordering::Relaxed);
                continue;
            }
            if let Some(parametro) = self
                .rifiutati
                .gestisci(&risposta, &inviati(&corpo, &FACOLTATIVI))
            {
                avvisi.push(avviso_parametro(&parametro));
                continue;
            }
            if risposta.stato >= 400 {
                return Err(errore_da_stato(
                    risposta.stato,
                    &risposta.corpo,
                    &format!(
                        "controlla base_url ({}) e il nome del modello («{}»)",
                        self.url, self.modello
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
            if testo.trim().is_empty() && fine.as_deref() == Some("length") {
                return Err(ErroreAi::Formato(
                    "la risposta è stata troncata da max_token prima di produrre testo \
                     (il modello ha speso i token nel ragionamento?): aumenta max_token o \
                     imposta ragionamento = \"spento\""
                        .to_string(),
                ));
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

impl Narratore for ClientOpenAi {
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
        format!("{} · {} · {}", self.fornitore, self.modello, self.url)
    }

    fn json_mode_attivo(&self) -> Option<bool> {
        Some(self.json_mode && self.rifiutati.attivo("response_format"))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::ai::Messaggio;

    fn richiesta() -> RichiestaAi {
        RichiestaAi {
            messaggi: vec![Messaggio::sistema("s"), Messaggio::utente("u")],
            contesto: Default::default(),
        }
    }

    #[test]
    fn corpo_openai_e_lmstudio() {
        let mut config = ConfigAi {
            ragionamento: Ragionamento::Alto,
            ..ConfigAi::default()
        };
        let c = ClientOpenAi::nuovo(&config).unwrap();
        let corpo = c.corpo(&richiesta());
        assert_eq!(corpo["response_format"]["type"], "json_object");
        assert_eq!(corpo["reasoning_effort"], "high");
        assert_eq!(corpo["max_tokens"], 1200);
        assert_eq!(corpo["messages"][0]["role"], "system");

        config.fornitore = Fornitore::Lmstudio;
        config.ragionamento = Ragionamento::Predefinito;
        let corpo = ClientOpenAi::nuovo(&config).unwrap().corpo(&richiesta());
        assert_eq!(corpo["response_format"]["type"], "json_schema");
        assert_eq!(
            corpo["response_format"]["json_schema"]["schema"]["required"],
            json!(["narrazione"])
        );
        assert!(corpo.get("reasoning_effort").is_none());
    }
}
