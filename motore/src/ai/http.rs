//! Parti comuni dei client HTTP (OpenAI/LM Studio, Anthropic, Gemini, Ollama):
//! timeout di connessione e di lettura separati, tentativi con backoff
//! esponenziale (1 s, 2 s, 4 s…) solo per 429, 5xx (compreso il 529
//! "overloaded" di Anthropic) ed errori di connessione, header di
//! autenticazione per fornitore, header aggiuntivi, traduzione degli errori
//! HTTP e gestione dei parametri rifiutati (400 che cita il parametro → si
//! riprova una volta senza, con un solo avviso).
//!
//! Dopo un timeout di lettura non si ritenta: l'errore
//! [`ErroreAi::TimeoutLettura`] risale al motore, che chiede al giocatore
//! (tramite il [`crate::canale::Canale`]) se riprovare.
//! La chiave API va solo nell'header di autenticazione e non viene mai scritta
//! nei log né nei messaggi d'errore.

use std::collections::BTreeSet;
use std::error::Error as _;
use std::io::{self, BufRead};
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

use reqwest::blocking::Client;
use reqwest::header::{HeaderName, HeaderValue};
use serde_json::Value;
use tracing::{info, warn};

use super::ErroreAi;
use crate::config::{ConfigAi, MASCHERATO};

/// Versione dell'API Anthropic inviata nell'header `anthropic-version` (§5.1).
pub const VERSIONE_ANTHROPIC: &str = "2023-06-01";

/// Risposta HTTP letta per intero.
#[derive(Debug, Clone)]
pub struct RispostaHttp {
    /// Codice di stato.
    pub stato: u16,
    /// Corpo della risposta.
    pub corpo: String,
}

/// Come si invia la chiave al server.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StileChiave {
    /// `Authorization: Bearer <chiave>` (OpenAI, LM Studio, Ollama).
    Bearer,
    /// `x-api-key: <chiave>` + `anthropic-version` (Anthropic).
    Anthropic,
    /// `x-goog-api-key: <chiave>` (Gemini).
    Google,
}

/// Client HTTP condiviso dai fornitori.
pub struct ClientHttp {
    client: Client,
    chiave: Option<String>,
    stile: StileChiave,
    header_extra: Vec<(HeaderName, HeaderValue)>,
    max_tentativi: u32,
    backoff_base: Duration,
    timeout_secondi: u64,
    locale: bool,
    host_porta: String,
}

impl std::fmt::Debug for ClientHttp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ClientHttp")
            .field("chiave", &self.chiave.as_ref().map(|_| MASCHERATO))
            .field("stile", &self.stile)
            .field("max_tentativi", &self.max_tentativi)
            .field("timeout_secondi", &self.timeout_secondi)
            .field("locale", &self.locale)
            .field("host_porta", &self.host_porta)
            .finish()
    }
}

impl ClientHttp {
    /// Crea il client dalla configurazione.
    pub fn nuovo(config: &ConfigAi, stile: StileChiave) -> Result<Self, ErroreAi> {
        let locale = config.server_locale();
        let mut builder = Client::builder()
            .connect_timeout(Duration::from_secs(config.timeout_connessione_secondi))
            .timeout(Duration::from_secs(config.timeout_secondi))
            .user_agent(concat!("avventura/", env!("CARGO_PKG_VERSION")));
        if locale {
            builder = builder.no_proxy();
        }
        let client = builder
            .build()
            .map_err(|e| ErroreAi::Configurazione(format!("client HTTP non creato: {e}")))?;
        let mut header_extra = Vec::new();
        for (nome, valore) in &config.header_extra {
            let n = HeaderName::from_bytes(nome.as_bytes()).map_err(|_| {
                ErroreAi::Configurazione(format!("nome di header non valido: «{nome}»"))
            })?;
            let v = HeaderValue::from_str(valore).map_err(|_| {
                ErroreAi::Configurazione(format!("valore non valido per l'header «{nome}»"))
            })?;
            header_extra.push((n, v));
        }
        Ok(ClientHttp {
            client,
            chiave: config.chiave.clone(),
            stile,
            header_extra,
            max_tentativi: config.max_tentativi.max(1),
            backoff_base: Duration::from_secs(1),
            timeout_secondi: config.timeout_secondi,
            locale,
            host_porta: config.host_porta(),
        })
    }

    /// Cambia l'attesa base del backoff (1 s di default; utile nei test).
    pub fn imposta_backoff(&mut self, base: Duration) {
        self.backoff_base = base;
    }

    /// Vero se viene inviata una chiave.
    pub fn usa_chiave(&self) -> bool {
        self.chiave.is_some()
    }

    /// Vero se il server è locale.
    pub fn locale(&self) -> bool {
        self.locale
    }

    /// Invia un POST JSON con i tentativi automatici previsti.
    pub fn post_json(&self, url: &str, corpo: &Value) -> Result<RispostaHttp, ErroreAi> {
        self.invia(url, corpo, None)
    }

    /// Come [`ClientHttp::post_json`], per le risposte in streaming: se il
    /// server risponde 2xx, il corpo viene letto riga per riga e ogni riga
    /// (senza il fine riga) passa a `riga` appena arriva. Il corpo completo è
    /// comunque restituito. Le risposte d'errore si leggono per intero.
    /// Un errore a metà del corpo non viene ritentato (il testo è già in vista).
    pub fn post_json_flusso(
        &self,
        url: &str,
        corpo: &Value,
        riga: &mut dyn FnMut(&str),
    ) -> Result<RispostaHttp, ErroreAi> {
        self.invia(url, corpo, Some(riga))
    }

    fn invia(
        &self,
        url: &str,
        corpo: &Value,
        mut riga: Option<&mut dyn FnMut(&str)>,
    ) -> Result<RispostaHttp, ErroreAi> {
        let mut tentativo = 1;
        loop {
            let mut richiesta = self.client.post(url).json(corpo);
            if self.stile == StileChiave::Anthropic {
                richiesta = richiesta.header("anthropic-version", VERSIONE_ANTHROPIC);
            }
            if let Some(chiave) = &self.chiave {
                richiesta = match self.stile {
                    StileChiave::Bearer => richiesta.bearer_auth(chiave),
                    StileChiave::Anthropic => richiesta.header("x-api-key", chiave),
                    StileChiave::Google => richiesta.header("x-goog-api-key", chiave),
                };
            }
            for (nome, valore) in &self.header_extra {
                richiesta = richiesta.header(nome.clone(), valore.clone());
            }
            let esito = match richiesta.send() {
                Ok(r) => {
                    let stato = r.status().as_u16();
                    match riga.as_deref_mut() {
                        Some(f) if r.status().is_success() => {
                            return self.leggi_a_righe(stato, r, f);
                        }
                        _ => r.text().map(|corpo| RispostaHttp { stato, corpo }),
                    }
                }
                Err(e) => Err(e),
            };
            let ultimo = tentativo >= self.max_tentativi;
            match esito {
                Ok(r) if (r.stato == 429 || r.stato >= 500) && !ultimo => {
                    warn!(
                        stato = r.stato,
                        tentativo, "errore temporaneo del server, nuovo tentativo"
                    );
                }
                Ok(r) => return Ok(r),
                Err(e) => {
                    let errore = self.traduci_errore_rete(&e);
                    match &errore {
                        ErroreAi::Connessione(_) if !ultimo => {
                            warn!(tentativo, errore = %errore, "errore di connessione, nuovo tentativo");
                        }
                        _ => return Err(errore),
                    }
                }
            }
            let attesa = self.backoff_base * 2u32.saturating_pow(tentativo - 1);
            info!(
                attesa_ms = attesa.as_millis() as u64,
                "attesa prima del nuovo tentativo"
            );
            thread::sleep(attesa);
            tentativo += 1;
        }
    }

    /// Legge un corpo in streaming riga per riga.
    fn leggi_a_righe(
        &self,
        stato: u16,
        risposta: reqwest::blocking::Response,
        riga: &mut dyn FnMut(&str),
    ) -> Result<RispostaHttp, ErroreAi> {
        let mut lettore = io::BufReader::new(risposta);
        let mut corpo = String::new();
        let mut buffer = Vec::new();
        loop {
            buffer.clear();
            let letti = lettore
                .read_until(b'\n', &mut buffer)
                .map_err(|e| self.traduci_errore_io(&e))?;
            if letti == 0 {
                break;
            }
            let testo = String::from_utf8_lossy(&buffer);
            corpo.push_str(&testo);
            riga(testo.trim_end_matches(['\r', '\n']));
        }
        Ok(RispostaHttp { stato, corpo })
    }

    /// Errore di lettura del corpo in streaming.
    fn traduci_errore_io(&self, e: &io::Error) -> ErroreAi {
        if let Some(r) = e
            .get_ref()
            .and_then(|interno| interno.downcast_ref::<reqwest::Error>())
        {
            return self.traduci_errore_rete(r);
        }
        if e.kind() == io::ErrorKind::TimedOut {
            return ErroreAi::TimeoutLettura {
                secondi: self.timeout_secondi,
            };
        }
        ErroreAi::Connessione(e.to_string())
    }

    fn traduci_errore_rete(&self, e: &reqwest::Error) -> ErroreAi {
        if e.is_connect() {
            if self.locale && connessione_rifiutata(e) {
                return ErroreAi::ServerLocaleNonRisponde(self.host_porta.clone());
            }
            return ErroreAi::Connessione(descrivi(e));
        }
        if e.is_timeout() {
            return ErroreAi::TimeoutLettura {
                secondi: self.timeout_secondi,
            };
        }
        ErroreAi::Connessione(descrivi(e))
    }
}

/// Parametri facoltativi che il server ha rifiutato in questa sessione.
///
/// Quando una risposta 400 (o 422) cita il nome di un parametro facoltativo
/// ancora attivo, il parametro viene disattivato per il resto della sessione,
/// la richiesta si ripete una volta senza e si produce un solo avviso.
#[derive(Debug, Default)]
pub struct ParametriRifiutati {
    disattivati: Mutex<BTreeSet<String>>,
}

impl ParametriRifiutati {
    fn insieme(&self) -> std::sync::MutexGuard<'_, BTreeSet<String>> {
        // Un panic in un altro thread non rende i dati inconsistenti: è un insieme.
        self.disattivati
            .lock()
            .unwrap_or_else(|avvelenato| avvelenato.into_inner())
    }

    /// Vero se il parametro va ancora inviato.
    pub fn attivo(&self, nome: &str) -> bool {
        !self.insieme().contains(nome)
    }

    /// Disattiva un parametro (es. dopo averlo sostituito con un altro nome).
    pub fn disattiva(&self, nome: &str) {
        self.insieme().insert(nome.to_string());
    }

    /// Se la risposta rifiuta uno dei `candidati` (in ordine di priorità) ancora
    /// attivi, lo disattiva e restituisce il suo nome.
    pub fn gestisci(&self, risposta: &RispostaHttp, candidati: &[&str]) -> Option<String> {
        if !matches!(risposta.stato, 400 | 422) {
            return None;
        }
        let corpo = risposta.corpo.to_lowercase();
        // I messaggi di Google citano i campi in snake_case (`thinking_config`) o a parole
        // (`Thinking level MINIMAL is not supported`).
        let citato = |c: &str| {
            let snake = in_snake_case(c);
            corpo.contains(&c.to_lowercase())
                || corpo.contains(&snake)
                || corpo.contains(&snake.replace('_', " "))
        };
        let rifiutato = candidati.iter().find(|c| self.attivo(c) && citato(c))?;
        self.disattiva(rifiutato);
        warn!(parametro = %rifiutato, corpo = %estratto(&risposta.corpo), "parametro rifiutato dal server: nuovo tentativo senza");
        Some(rifiutato.to_string())
    }
}

/// `thinkingConfig` → `thinking_config`.
fn in_snake_case(nome: &str) -> String {
    let mut s = String::new();
    for c in nome.chars() {
        if c.is_ascii_uppercase() {
            s.push('_');
            s.push(c.to_ascii_lowercase());
        } else {
            s.push(c);
        }
    }
    s
}

/// Candidati effettivamente presenti nel corpo inviato (chiavi a qualsiasi
/// livello di annidamento): un parametro non inviato non può essere «rifiutato».
pub fn inviati<'a>(corpo: &Value, candidati: &[&'a str]) -> Vec<&'a str> {
    fn contiene(v: &Value, chiave: &str) -> bool {
        match v {
            Value::Object(m) => m.contains_key(chiave) || m.values().any(|x| contiene(x, chiave)),
            _ => false,
        }
    }
    candidati
        .iter()
        .copied()
        .filter(|c| contiene(corpo, c))
        .collect()
}

/// Avviso mostrato una volta quando un parametro viene tolto.
pub fn avviso_parametro(parametro: &str) -> String {
    format!(
        "Il server non accetta il parametro «{parametro}»: riprovo senza (per il resto della \
         sessione non verrà più inviato)."
    )
}

/// Messaggio di un errore reqwest con le cause (gli URL non contengono segreti:
/// le chiavi viaggiano solo negli header).
fn descrivi(e: &reqwest::Error) -> String {
    let mut testo = e.to_string();
    let mut causa = e.source();
    while let Some(c) = causa {
        testo.push_str(&format!(": {c}"));
        causa = c.source();
    }
    testo
}

fn connessione_rifiutata(e: &reqwest::Error) -> bool {
    let mut causa = e.source();
    while let Some(c) = causa {
        if let Some(io) = c.downcast_ref::<io::Error>()
            && io.kind() == io::ErrorKind::ConnectionRefused
        {
            return true;
        }
        causa = c.source();
    }
    // Alcune piattaforme non espongono l'io::Error: si cerca nel testo.
    let testo = descrivi(e).to_lowercase();
    testo.contains("refused") || testo.contains("rifiutata") || testo.contains("10061")
}

/// Traduce un codice HTTP di errore in [`ErroreAi`].
pub fn errore_da_stato(stato: u16, corpo: &str, contesto_404: &str) -> ErroreAi {
    match stato {
        401 | 403 => ErroreAi::Autenticazione { stato },
        404 => ErroreAi::NonTrovato(format!(
            "risorsa non trovata (HTTP 404): {contesto_404}. Dettagli: {}",
            estratto(corpo)
        )),
        _ => ErroreAi::Http {
            stato,
            estratto: estratto(corpo),
        },
    }
}

/// Interpreta il corpo JSON di una risposta riuscita.
pub fn corpo_json(risposta: &RispostaHttp) -> Result<Value, ErroreAi> {
    serde_json::from_str(&risposta.corpo)
        .map_err(|_| ErroreAi::Formato(format!("JSON non valido: {}", estratto(&risposta.corpo))))
}

/// Primi 300 caratteri del corpo, su una riga.
pub fn estratto(corpo: &str) -> String {
    let una_riga = corpo.split_whitespace().collect::<Vec<_>>().join(" ");
    if una_riga.chars().count() > 300 {
        format!("{}…", una_riga.chars().take(300).collect::<String>())
    } else {
        una_riga
    }
}

/// URL di base senza `/` finale.
pub fn base_senza_barra(url: &str) -> String {
    url.trim().trim_end_matches('/').to_string()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn traduzione_dei_codici() {
        assert!(matches!(
            errore_da_stato(401, "", ""),
            ErroreAi::Autenticazione { stato: 401 }
        ));
        assert!(matches!(
            errore_da_stato(404, "", "x"),
            ErroreAi::NonTrovato(_)
        ));
        assert!(matches!(
            errore_da_stato(500, "boom", ""),
            ErroreAi::Http { stato: 500, .. }
        ));
    }

    #[test]
    fn estratto_breve() {
        assert_eq!(estratto("a\n  b"), "a b");
        assert_eq!(estratto(&"x".repeat(400)).chars().count(), 301);
    }

    #[test]
    fn debug_senza_chiave() {
        let config = ConfigAi {
            chiave: Some("sk-SEGRETA".into()),
            ..ConfigAi::default()
        };
        let c = ClientHttp::nuovo(&config, StileChiave::Bearer).unwrap();
        assert!(!format!("{c:?}").contains("SEGRETA"));
        assert!(c.usa_chiave());
    }

    #[test]
    fn parametri_rifiutati_una_sola_volta() {
        let p = ParametriRifiutati::default();
        let r = RispostaHttp {
            stato: 400,
            corpo: r#"{"error":"Unsupported parameter: 'reasoning_effort'"}"#.into(),
        };
        assert_eq!(
            p.gestisci(&r, &["response_format", "reasoning_effort"]),
            Some("reasoning_effort".into())
        );
        assert!(!p.attivo("reasoning_effort"));
        assert_eq!(p.gestisci(&r, &["reasoning_effort"]), None);
        // Messaggio di Google a parole, senza il nome del campo.
        let google = RispostaHttp {
            stato: 400,
            corpo:
                r#"{"error":{"message":"Thinking level MINIMAL is not supported for this model."}}"#
                    .into(),
        };
        assert_eq!(
            p.gestisci(&google, &["thinkingLevel", "thinkingConfig"]),
            Some("thinkingLevel".into())
        );
        let r500 = RispostaHttp {
            stato: 500,
            corpo: "response_format".into(),
        };
        assert_eq!(p.gestisci(&r500, &["response_format"]), None);
    }
}
