//! La chiave API viene inviata nell'header previsto dal fornitore
//! (`Authorization`, `x-api-key`, `x-goog-api-key`) ma non compare mai nel log
//! (anche a livello debug), nell'output `Debug` della configurazione né nei
//! messaggi d'errore.
//!
//! È in un binario di test separato perché il log di `tracing` usa uno stato
//! globale che i test eseguiti in parallelo disturberebbero.

use std::io::Write;
use std::sync::{Arc, Mutex, mpsc};
use std::thread;

use avventura::ai::anthropic::ClientAnthropic;
use avventura::ai::gemini::ClientGemini;
use avventura::ai::openai::ClientOpenAi;
use avventura::ai::{ErroreAi, Messaggio, Narratore, RichiestaAi};
use avventura::config::{ConfigAi, Fornitore};

const CHIAVE: &str = "sk-prova-SEGRETISSIMA-123";

/// Buffer condiviso in cui il log del test viene scritto.
#[derive(Clone, Default)]
struct BufferLog(Arc<Mutex<Vec<u8>>>);

impl Write for BufferLog {
    fn write(&mut self, dati: &[u8]) -> std::io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(dati);
        Ok(dati.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

type Header = Vec<(String, String)>;

/// Server che risponde nell'ordine e invia gli header ricevuti.
fn server(risposte: Vec<(u16, String)>) -> (u16, mpsc::Receiver<Header>) {
    let server = tiny_http::Server::http("127.0.0.1:0").unwrap();
    let porta = server.server_addr().to_ip().unwrap().port();
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        for (stato, corpo) in risposte {
            let mut r = server.recv().unwrap();
            let mut testo = String::new();
            r.as_reader().read_to_string(&mut testo).unwrap();
            let header = r
                .headers()
                .iter()
                .map(|h| {
                    (
                        h.field.as_str().as_str().to_lowercase(),
                        h.value.as_str().to_string(),
                    )
                })
                .collect();
            tx.send(header).unwrap();
            let _ = r.respond(tiny_http::Response::from_string(corpo).with_status_code(stato));
        }
    });
    (porta, rx)
}

fn valore<'a>(h: &'a Header, nome: &str) -> Option<&'a str> {
    h.iter().find(|(n, _)| n == nome).map(|(_, v)| v.as_str())
}

fn richiesta() -> RichiestaAi {
    RichiestaAi {
        messaggi: vec![Messaggio::sistema("regole"), Messaggio::utente("azione")],
        contesto: Default::default(),
    }
}

fn con_chiave(fornitore: Fornitore, base_url: String) -> ConfigAi {
    let mut cfg = ConfigAi {
        fornitore,
        base_url,
        modello: "m".into(),
        api_key: Some(CHIAVE.into()),
        ..ConfigAi::default()
    };
    cfg.chiave = Some(CHIAVE.into());
    cfg
}

#[test]
fn chiave_inviata_ma_mai_nei_log_ne_negli_errori() {
    let log = BufferLog::default();
    let scrittore = log.clone();
    tracing_subscriber::fmt()
        .with_writer(move || scrittore.clone())
        .with_max_level(tracing::Level::DEBUG)
        .with_ansi(false)
        .init();

    // OpenAI: Authorization: Bearer, anche verso un server locale se la chiave c'è.
    let ok = r#"{"choices":[{"message":{"content":"{\"narrazione\":\"ok\"}"}}],"usage":{"prompt_tokens":3}}"#;
    let (porta, rx) = server(vec![
        (200, ok.to_string()),
        (401, r#"{"error":"invalid api key"}"#.to_string()),
    ]);
    let mut cfg = con_chiave(Fornitore::Openai, format!("http://127.0.0.1:{porta}/v1"));
    cfg.header_extra.insert("X-Prova".into(), "valore".into());
    cfg.header_extra.insert("api-key".into(), CHIAVE.into());
    let client = ClientOpenAi::nuovo(&cfg).unwrap();
    tracing::info!(config = ?cfg, client = ?client, "configurazione");
    client.turno(&richiesta()).unwrap();
    let errore = client.turno(&richiesta()).unwrap_err();
    let h = rx.recv().unwrap();
    assert_eq!(
        valore(&h, "authorization"),
        Some(format!("Bearer {CHIAVE}").as_str()),
        "con una chiave presente l'header viene inviato anche verso un server locale"
    );
    assert_eq!(valore(&h, "x-prova"), Some("valore"));
    assert!(matches!(errore, ErroreAi::Autenticazione { stato: 401 }));
    assert!(!errore.to_string().contains("SEGRET"));
    assert!(!format!("{cfg:?}").contains("SEGRET"));

    // Anthropic: x-api-key + anthropic-version, nessun Authorization.
    let ok = r#"{"content":[{"type":"text","text":"{\"narrazione\":\"ok\"}"}],"usage":{"input_tokens":3}}"#;
    let (porta, rx) = server(vec![(200, ok.to_string())]);
    let cfg = con_chiave(Fornitore::Anthropic, format!("http://127.0.0.1:{porta}"));
    let client = ClientAnthropic::nuovo(&cfg).unwrap();
    tracing::info!(client = ?client, "client anthropic");
    client.turno(&richiesta()).unwrap();
    let h = rx.recv().unwrap();
    assert_eq!(valore(&h, "x-api-key"), Some(CHIAVE));
    assert_eq!(valore(&h, "anthropic-version"), Some("2023-06-01"));
    assert!(valore(&h, "authorization").is_none());

    // Gemini: x-goog-api-key, la chiave non va mai nell'URL.
    let ok = r#"{"candidates":[{"content":{"parts":[{"text":"{\"narrazione\":\"ok\"}"}]},"finishReason":"STOP"}]}"#;
    let (porta, rx) = server(vec![(200, ok.to_string())]);
    let cfg = con_chiave(
        Fornitore::Gemini,
        format!("http://127.0.0.1:{porta}/v1beta"),
    );
    let client = ClientGemini::nuovo(&cfg).unwrap();
    client.turno(&richiesta()).unwrap();
    let h = rx.recv().unwrap();
    assert_eq!(valore(&h, "x-goog-api-key"), Some(CHIAVE));
    assert!(valore(&h, "authorization").is_none());

    let testo_log = String::from_utf8(log.0.lock().unwrap().clone()).unwrap();
    assert!(testo_log.contains("risposta grezza"), "{testo_log}");
    assert!(testo_log.contains("corpo della richiesta"), "{testo_log}");
    assert!(
        !testo_log.contains("SEGRET"),
        "chiave nel log:\n{testo_log}"
    );
}
