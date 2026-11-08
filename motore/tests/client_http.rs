//! Tutti i fornitori HTTP (`openai`, `lmstudio`, `anthropic`, `gemini`,
//! `ollama`) contro un server HTTP locale di prova (`tiny_http`, §10): URL,
//! header di autenticazione (e loro assenza senza chiave), corpo della
//! richiesta (messaggi, JSON mode/schema, ragionamento), lettura del testo e dei
//! token di prompt, errori (401, 404, 429, 529, 5xx, 400 su parametro non
//! supportato → nuovo tentativo senza), troncamento, timeout di lettura senza
//! nuovi tentativi, connessione rifiutata, risposte in streaming (SSE e JSON a
//! righe) con il testo progressivo. Nessun test usa la rete esterna o chiavi
//! reali.

mod comune;

use std::net::TcpListener;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use avventura::ai::anthropic::ClientAnthropic;
use avventura::ai::gemini::ClientGemini;
use avventura::ai::ollama::ClientOllama;
use avventura::ai::openai::ClientOpenAi;
use avventura::ai::ragionamento::Ragionamento;
use avventura::ai::{ErroreAi, Messaggio, Narratore, RichiestaAi};
use avventura::avventura::Avventura;
use avventura::canale::prova::CanaleProva;
use avventura::config::{Config, ConfigAi, Fornitore};
use avventura::motore::{EsitoTurno, Partita};
use avventura::prompt::AzioneTurno;
use avventura::risposta;
use serde_json::{Value, json};

/// Una richiesta ricevuta dal server di prova.
#[derive(Debug)]
struct Richiesta {
    percorso: String,
    header: Vec<(String, String)>,
    corpo: Value,
}

impl Richiesta {
    fn header(&self, nome: &str) -> Option<&str> {
        self.header
            .iter()
            .find(|(n, _)| n.eq_ignore_ascii_case(nome))
            .map(|(_, v)| v.as_str())
    }
}

/// Server che risponde con le risposte date, nell'ordine; registra le richieste.
/// Dopo l'ultima risposta resta in ascolto `attesa_finale` per contare eventuali
/// richieste in più (che vengono registrate ma non ricevono risposta).
fn server(
    risposte: Vec<(u16, String)>,
    attesa_finale: Duration,
) -> (String, mpsc::Receiver<Richiesta>, thread::JoinHandle<()>) {
    let server = tiny_http::Server::http("127.0.0.1:0").unwrap();
    let porta = server.server_addr().to_ip().unwrap().port();
    let (tx, rx) = mpsc::channel();
    let gestore = thread::spawn(move || {
        let registra = |richiesta: &mut tiny_http::Request| {
            let mut testo = String::new();
            richiesta.as_reader().read_to_string(&mut testo).unwrap();
            Richiesta {
                percorso: richiesta.url().to_string(),
                header: richiesta
                    .headers()
                    .iter()
                    .map(|h| (h.field.as_str().to_string(), h.value.as_str().to_string()))
                    .collect(),
                corpo: serde_json::from_str(&testo).unwrap_or(Value::Null),
            }
        };
        for (stato, corpo) in risposte {
            let Ok(mut richiesta) = server.recv() else {
                return;
            };
            let _ = tx.send(registra(&mut richiesta));
            let risposta = tiny_http::Response::from_string(corpo)
                .with_status_code(stato)
                .with_header(
                    tiny_http::Header::from_bytes("Content-Type", "application/json").unwrap(),
                );
            let _ = richiesta.respond(risposta);
        }
        let fine = Instant::now() + attesa_finale;
        let mut in_sospeso = Vec::new();
        while let Some(resto) = fine.checked_duration_since(Instant::now()) {
            match server.recv_timeout(resto) {
                Ok(Some(mut r)) => {
                    let _ = tx.send(registra(&mut r));
                    in_sospeso.push(r);
                }
                _ => break,
            }
        }
    });
    (format!("http://127.0.0.1:{porta}"), rx, gestore)
}

fn risposta_openai(contenuto: &str, prompt_tokens: u64) -> String {
    json!({
        "choices": [{ "message": { "role": "assistant", "content": contenuto }, "finish_reason": "stop" }],
        "usage": { "prompt_tokens": prompt_tokens, "completion_tokens": 10 }
    })
    .to_string()
}

fn risposta_anthropic(testo: &str, input_tokens: u64) -> String {
    json!({
        "type": "message",
        "role": "assistant",
        "content": [
            { "type": "thinking", "thinking": "" },
            { "type": "text", "text": testo }
        ],
        "stop_reason": "end_turn",
        "usage": { "input_tokens": input_tokens, "output_tokens": 20 }
    })
    .to_string()
}

fn risposta_gemini(testo: &str, prompt_tokens: u64) -> String {
    json!({
        "candidates": [{
            "content": { "role": "model", "parts": [
                { "text": "ragiono…", "thought": true },
                { "text": testo }
            ]},
            "finishReason": "STOP"
        }],
        "usageMetadata": { "promptTokenCount": prompt_tokens, "candidatesTokenCount": 9 }
    })
    .to_string()
}

fn config_ai(fornitore: Fornitore, base_url: &str) -> ConfigAi {
    ConfigAi {
        fornitore,
        base_url: base_url.to_string(),
        modello: "modello-prova".into(),
        ragionamento: fornitore.ragionamento_predefinito(),
        ..ConfigAi::default()
    }
}

fn richiesta() -> RichiestaAi {
    RichiestaAi {
        messaggi: vec![Messaggio::sistema("regole"), Messaggio::utente("azione")],
        contesto: Default::default(),
    }
}

/// Richiesta con un tentativo di correzione (utente, assistente, utente).
fn richiesta_con_correzione() -> RichiestaAi {
    RichiestaAi {
        messaggi: vec![
            Messaggio::sistema("regole"),
            Messaggio::utente("azione"),
            Messaggio::assistente("risposta sbagliata"),
            Messaggio::utente("correggi"),
        ],
        contesto: Default::default(),
    }
}

const VALIDA: &str = r#"{"narrazione":"Apri la porta."}"#;

// ---------------------------------------------------------------- openai

#[test]
fn openai_risposta_valida_senza_chiave_verso_localhost() {
    let (url, rx, _s) = server(vec![(200, risposta_openai(VALIDA, 321))], Duration::ZERO);
    let client = ClientOpenAi::nuovo(&config_ai(Fornitore::Openai, &format!("{url}/v1/"))).unwrap();
    let r = client.turno(&richiesta()).unwrap();
    assert_eq!(r.testo, VALIDA);
    assert_eq!(r.token_prompt, Some(321));
    let richiesta = rx.recv().unwrap();
    assert_eq!(richiesta.percorso, "/v1/chat/completions");
    assert!(
        richiesta.header("Authorization").is_none(),
        "nessuna chiave verso un server locale"
    );
    assert_eq!(richiesta.corpo["model"], "modello-prova");
    assert_eq!(richiesta.corpo["max_tokens"], 1200);
    assert_eq!(richiesta.corpo["response_format"]["type"], "json_object");
    assert!(
        richiesta.corpo.get("reasoning_effort").is_none(),
        "predefinito"
    );
    assert_eq!(richiesta.corpo["messages"][0]["role"], "system");
    assert_eq!(richiesta.corpo["messages"][1]["content"], "azione");
}

#[test]
fn openai_risposta_in_code_fence_e_con_ragionamento() {
    let contenuto = format!("<think>uhm {{x}}</think>Ecco la risposta:\n```json\n{VALIDA}\n```");
    let (url, _rx, _s) = server(vec![(200, risposta_openai(&contenuto, 1))], Duration::ZERO);
    let client = ClientOpenAi::nuovo(&config_ai(Fornitore::Openai, &url)).unwrap();
    let r = client.turno(&richiesta()).unwrap();
    let analizzata = risposta::analizza(&r.testo).unwrap();
    assert_eq!(analizzata.narrazione, "Apri la porta.");
}

#[test]
fn openai_429_poi_successo() {
    let (url, rx, _s) = server(
        vec![
            (429, r#"{"error":"rate limit"}"#.into()),
            (503, "occupato".into()),
            (200, risposta_openai(VALIDA, 5)),
        ],
        Duration::ZERO,
    );
    let client = ClientOpenAi::nuovo(&config_ai(Fornitore::Openai, &url))
        .unwrap()
        .con_backoff(Duration::from_millis(10));
    let r = client.turno(&richiesta()).unwrap();
    assert_eq!(r.testo, VALIDA);
    assert_eq!(rx.try_iter().count(), 3);
}

#[test]
fn openai_errori_5xx_oltre_i_tentativi() {
    let (url, rx, _s) = server(
        vec![(500, "a".into()), (500, "b".into()), (500, "boom".into())],
        Duration::from_millis(300),
    );
    let client = ClientOpenAi::nuovo(&config_ai(Fornitore::Openai, &url))
        .unwrap()
        .con_backoff(Duration::from_millis(10));
    let errore = client.turno(&richiesta()).unwrap_err();
    assert!(
        matches!(errore, ErroreAi::Http { stato: 500, .. }),
        "{errore:?}"
    );
    assert!(errore.to_string().contains("boom"));
    thread::sleep(Duration::from_millis(400));
    assert_eq!(rx.try_iter().count(), 3, "max_tentativi = 3");
}

#[test]
fn openai_400_su_response_format_riprova_senza() {
    let (url, rx, _s) = server(
        vec![
            (
                400,
                r#"{"error":{"message":"Unsupported parameter: 'response_format'"}}"#.into(),
            ),
            (200, risposta_openai(VALIDA, 5)),
            (200, risposta_openai(VALIDA, 5)),
        ],
        Duration::ZERO,
    );
    let client = ClientOpenAi::nuovo(&config_ai(Fornitore::Openai, &url)).unwrap();
    let r = client.turno(&richiesta()).unwrap();
    assert_eq!(r.avvisi.len(), 1);
    assert!(r.avvisi[0].contains("response_format"));
    assert_eq!(client.json_mode_attivo(), Some(false));
    let prima = rx.recv().unwrap();
    let seconda = rx.recv().unwrap();
    assert!(prima.corpo.get("response_format").is_some());
    assert!(seconda.corpo.get("response_format").is_none());
    // Il turno successivo non riprova più response_format e non ripete l'avviso.
    let r = client.turno(&richiesta()).unwrap();
    assert!(r.avvisi.is_empty());
    assert!(rx.recv().unwrap().corpo.get("response_format").is_none());
}

#[test]
fn openai_ragionamento_e_max_completion_tokens() {
    let (url, rx, _s) = server(
        vec![
            (
                400,
                r#"{"error":{"message":"Unsupported parameter: 'max_tokens' is not supported with this model. Use 'max_completion_tokens' instead."}}"#.into(),
            ),
            (
                400,
                r#"{"error":{"message":"Unrecognized request argument supplied: reasoning_effort"}}"#.into(),
            ),
            (200, risposta_openai(VALIDA, 5)),
        ],
        Duration::ZERO,
    );
    let cfg = ConfigAi {
        ragionamento: Ragionamento::Spento,
        ..config_ai(Fornitore::Openai, &url)
    };
    let r = ClientOpenAi::nuovo(&cfg)
        .unwrap()
        .turno(&richiesta())
        .unwrap();
    assert_eq!(r.testo, VALIDA);
    let r1 = rx.recv().unwrap();
    let r2 = rx.recv().unwrap();
    let r3 = rx.recv().unwrap();
    assert_eq!(r1.corpo["reasoning_effort"], "minimal");
    assert_eq!(r1.corpo["max_tokens"], 1200);
    assert!(r2.corpo.get("max_tokens").is_none());
    assert_eq!(r2.corpo["max_completion_tokens"], 1200);
    assert!(r3.corpo.get("reasoning_effort").is_none());
    assert_eq!(r3.corpo["max_completion_tokens"], 1200);
    assert_eq!(r.avvisi.len(), 1, "un solo avviso: {:?}", r.avvisi);
}

#[test]
fn openai_401_e_404() {
    let (url, _rx, _s) = server(
        vec![
            (401, "{}".into()),
            (404, r#"{"error":"model not found"}"#.into()),
        ],
        Duration::ZERO,
    );
    let client = ClientOpenAi::nuovo(&config_ai(Fornitore::Openai, &url)).unwrap();
    assert!(matches!(
        client.turno(&richiesta()).unwrap_err(),
        ErroreAi::Autenticazione { stato: 401 }
    ));
    let e = client.turno(&richiesta()).unwrap_err();
    assert!(matches!(e, ErroreAi::NonTrovato(_)), "{e:?}");
    assert!(e.to_string().contains("modello-prova"), "{e}");
}

// ---------------------------------------------------------------- lmstudio

#[test]
fn lmstudio_con_schema_e_senza_chiave() {
    let (url, rx, _s) = server(vec![(200, risposta_openai(VALIDA, 7))], Duration::ZERO);
    let client =
        ClientOpenAi::nuovo(&config_ai(Fornitore::Lmstudio, &format!("{url}/v1"))).unwrap();
    let r = client.turno(&richiesta()).unwrap();
    assert_eq!(r.token_prompt, Some(7));
    let richiesta = rx.recv().unwrap();
    assert_eq!(richiesta.percorso, "/v1/chat/completions");
    assert!(richiesta.header("Authorization").is_none());
    assert_eq!(richiesta.corpo["response_format"]["type"], "json_schema");
    assert_eq!(
        richiesta.corpo["response_format"]["json_schema"]["schema"]["properties"]["narrazione"]["type"],
        "string"
    );
    // ragionamento predefinito per lmstudio = spento.
    assert_eq!(richiesta.corpo["reasoning_effort"], "minimal");
}

// ---------------------------------------------------------------- anthropic

#[test]
fn anthropic_richiesta_e_risposta() {
    let (url, rx, _s) = server(
        vec![(200, risposta_anthropic(VALIDA, 4321))],
        Duration::ZERO,
    );
    let client = ClientAnthropic::nuovo(&config_ai(Fornitore::Anthropic, &url)).unwrap();
    let r = client.turno(&richiesta_con_correzione()).unwrap();
    assert_eq!(r.testo, VALIDA, "solo i blocchi di testo");
    assert_eq!(r.token_prompt, Some(4321));
    let richiesta = rx.recv().unwrap();
    assert_eq!(richiesta.percorso, "/v1/messages");
    assert_eq!(richiesta.header("anthropic-version"), Some("2023-06-01"));
    assert!(richiesta.header("x-api-key").is_none(), "nessuna chiave");
    assert!(richiesta.header("Authorization").is_none());
    let c = &richiesta.corpo;
    assert_eq!(c["system"], "regole");
    assert_eq!(c["max_tokens"], 1200);
    let messaggi = c["messages"].as_array().unwrap();
    assert_eq!(messaggi.len(), 3, "il sistema non è tra i messaggi");
    assert_eq!(messaggi[0]["role"], "user");
    assert_eq!(messaggi[1]["role"], "assistant");
    assert_eq!(messaggi[2]["role"], "user");
    assert_eq!(c["output_config"]["format"]["type"], "json_schema");
    assert_eq!(
        c["output_config"]["format"]["schema"]["additionalProperties"],
        false
    );
    assert!(c.get("thinking").is_none());
}

#[test]
fn anthropic_ragionamento_529_e_temperatura_rifiutata() {
    let (url, rx, _s) = server(
        vec![
            (
                529,
                r#"{"type":"error","error":{"type":"overloaded_error"}}"#.into(),
            ),
            (200, risposta_anthropic(VALIDA, 10)),
        ],
        Duration::ZERO,
    );
    let cfg = ConfigAi {
        ragionamento: Ragionamento::Alto,
        ..config_ai(Fornitore::Anthropic, &url)
    };
    let client = ClientAnthropic::nuovo(&cfg)
        .unwrap()
        .con_backoff(Duration::from_millis(10));
    client.turno(&richiesta()).unwrap();
    let prima = rx.recv().unwrap();
    rx.recv().unwrap();
    assert_eq!(prima.corpo["thinking"]["type"], "adaptive");
    assert_eq!(prima.corpo["output_config"]["effort"], "high");
    assert!(
        prima.corpo.get("temperature").is_none(),
        "con il ragionamento la temperatura non si invia"
    );

    let (url, rx, _s) = server(
        vec![
            (
                400,
                r#"{"type":"error","error":{"type":"invalid_request_error","message":"temperature is not supported for this model"}}"#.into(),
            ),
            (200, risposta_anthropic(VALIDA, 10)),
        ],
        Duration::ZERO,
    );
    let client = ClientAnthropic::nuovo(&config_ai(Fornitore::Anthropic, &url)).unwrap();
    let r = client.turno(&richiesta()).unwrap();
    assert!(r.avvisi.is_empty(), "temperatura rifiutata: solo nel log");
    assert!(rx.recv().unwrap().corpo.get("temperature").is_some());
    assert!(rx.recv().unwrap().corpo.get("temperature").is_none());
}

#[test]
fn anthropic_rifiuto_e_errori() {
    let rifiuto = json!({
        "content": [], "stop_reason": "refusal", "usage": { "input_tokens": 3 }
    })
    .to_string();
    let (url, _rx, _s) = server(
        vec![(200, rifiuto), (401, "{}".into()), (404, "{}".into())],
        Duration::ZERO,
    );
    let client = ClientAnthropic::nuovo(&config_ai(Fornitore::Anthropic, &url)).unwrap();
    assert!(
        client
            .turno(&richiesta())
            .unwrap_err()
            .to_string()
            .contains("rifiutato")
    );
    assert!(matches!(
        client.turno(&richiesta()).unwrap_err(),
        ErroreAi::Autenticazione { stato: 401 }
    ));
    assert!(matches!(
        client.turno(&richiesta()).unwrap_err(),
        ErroreAi::NonTrovato(_)
    ));
}

// ---------------------------------------------------------------- gemini

#[test]
fn gemini_richiesta_e_risposta() {
    let (url, rx, _s) = server(vec![(200, risposta_gemini(VALIDA, 999))], Duration::ZERO);
    let cfg = ConfigAi {
        ragionamento: Ragionamento::Basso,
        ..config_ai(Fornitore::Gemini, &format!("{url}/v1beta"))
    };
    let client = ClientGemini::nuovo(&cfg).unwrap();
    let r = client.turno(&richiesta_con_correzione()).unwrap();
    assert_eq!(r.testo, VALIDA, "le parti di ragionamento sono ignorate");
    assert_eq!(r.token_prompt, Some(999));
    let richiesta = rx.recv().unwrap();
    assert_eq!(
        richiesta.percorso,
        "/v1beta/models/modello-prova:generateContent"
    );
    assert!(richiesta.header("x-goog-api-key").is_none());
    let c = &richiesta.corpo;
    assert_eq!(c["systemInstruction"]["parts"][0]["text"], "regole");
    assert_eq!(c["contents"].as_array().unwrap().len(), 3);
    assert_eq!(c["contents"][1]["role"], "model");
    assert_eq!(
        c["generationConfig"]["responseMimeType"],
        "application/json"
    );
    assert_eq!(c["generationConfig"]["maxOutputTokens"], 1200);
    assert_eq!(
        c["generationConfig"]["thinkingConfig"]["thinkingLevel"],
        "low"
    );
}

#[test]
fn gemini_livello_rifiutato_e_blocco_di_sicurezza() {
    let bloccata = json!({
        "candidates": [{ "content": { "parts": [] }, "finishReason": "SAFETY" }]
    })
    .to_string();
    let (url, rx, _s) = server(
        vec![
            (
                400,
                r#"{"error":{"code":400,"message":"thinking_level is not supported by this model","status":"INVALID_ARGUMENT"}}"#.into(),
            ),
            (200, risposta_gemini(VALIDA, 1)),
            (200, bloccata),
        ],
        Duration::ZERO,
    );
    let cfg = ConfigAi {
        ragionamento: Ragionamento::Spento,
        ..config_ai(Fornitore::Gemini, &url)
    };
    let client = ClientGemini::nuovo(&cfg).unwrap();
    let r = client.turno(&richiesta()).unwrap();
    assert_eq!(r.avvisi.len(), 1);
    assert!(rx.recv().unwrap().corpo["generationConfig"]["thinkingConfig"].is_object());
    assert!(
        rx.recv().unwrap().corpo["generationConfig"]
            .get("thinkingConfig")
            .is_none()
    );
    let e = client.turno(&richiesta()).unwrap_err();
    assert!(e.to_string().contains("sicurezza"), "{e}");
}

// ---------------------------------------------------------------- ollama

#[test]
fn ollama_nativo_con_num_ctx_e_think() {
    let corpo = json!({
        "model": "gemma4:12b",
        "message": { "role": "assistant", "content": VALIDA, "thinking": "ignorato" },
        "done": true,
        "prompt_eval_count": 1234
    })
    .to_string();
    let (url, rx, _s) = server(vec![(200, corpo)], Duration::ZERO);
    let cfg = ConfigAi {
        contesto_token: 16384,
        ..config_ai(Fornitore::Ollama, &url)
    };
    let client = ClientOllama::nuovo(&cfg).unwrap();
    let r = client.turno(&richiesta()).unwrap();
    assert_eq!(r.testo, VALIDA);
    assert_eq!(r.token_prompt, Some(1234));
    let richiesta = rx.recv().unwrap();
    assert_eq!(richiesta.percorso, "/api/chat");
    assert!(richiesta.header("Authorization").is_none());
    assert_eq!(richiesta.corpo["stream"], false);
    assert_eq!(richiesta.corpo["format"], "json");
    assert_eq!(richiesta.corpo["think"], false, "spento per ollama");
    assert_eq!(richiesta.corpo["options"]["num_ctx"], 16384);
    assert_eq!(richiesta.corpo["options"]["num_predict"], 1200);
    assert_eq!(richiesta.corpo["messages"][1]["role"], "user");
}

#[test]
fn ollama_think_non_supportato_riprova_senza() {
    let ok = json!({ "message": { "content": VALIDA }, "prompt_eval_count": 5 }).to_string();
    let (url, rx, _s) = server(
        vec![
            (
                400,
                r#"{"error":"\"gemma3:12b\" does not support thinking"}"#.into(),
            ),
            (200, ok),
        ],
        Duration::ZERO,
    );
    let client = ClientOllama::nuovo(&config_ai(Fornitore::Ollama, &url)).unwrap();
    let r = client.turno(&richiesta()).unwrap();
    assert_eq!(r.avvisi.len(), 1);
    assert!(rx.recv().unwrap().corpo.get("think").is_some());
    assert!(rx.recv().unwrap().corpo.get("think").is_none());
}

#[test]
fn ollama_modello_non_scaricato() {
    let (url, _rx, _s) = server(
        vec![(
            404,
            r#"{"error":"model \"llama9\" not found, try pulling it first"}"#.into(),
        )],
        Duration::ZERO,
    );
    let cfg = ConfigAi {
        modello: "llama9".into(),
        ..config_ai(Fornitore::Ollama, &url)
    };
    let errore = ClientOllama::nuovo(&cfg)
        .unwrap()
        .turno(&richiesta())
        .unwrap_err();
    assert!(
        errore.to_string().contains("ollama pull llama9"),
        "{errore}"
    );
}

#[test]
fn avviso_di_troncamento_con_ollama() {
    let dir = comune::copia_avventure();
    let avventura = Avventura::carica(&dir.path().join("avventure/mini")).unwrap();
    let corpo = json!({
        "message": { "content": VALIDA },
        "prompt_eval_count": 50
    })
    .to_string();
    let (url, _rx, _s) = server(vec![(200, corpo)], Duration::ZERO);
    let config = Config {
        ai: ConfigAi {
            contesto_token: 2048,
            ..config_ai(Fornitore::Ollama, &url)
        },
        ..Config::default()
    };
    let client = ClientOllama::nuovo(&config.ai).unwrap();
    let mut partita = Partita::nuova(&avventura, &config, &dir.path().join("partite")).unwrap();
    let mut canale = CanaleProva::default();
    let esito = partita
        .esegui_turno(
            &client,
            AzioneTurno::Giocatore("guardo".into()),
            &mut canale,
        )
        .unwrap();
    assert!(matches!(esito, EsitoTurno::Applicato { .. }), "{esito:?}");
    assert_eq!(canale.avvisi.len(), 2, "{:?}", canale.avvisi);
    assert!(canale.avvisi[0].contains("supera il contesto configurato (2.048)"));
    assert!(canale.avvisi[1].contains("ha elaborato solo ≈ 50 token di prompt"));
}

// ---------------------------------------------------------------- comuni

#[test]
fn timeout_di_lettura_senza_nuovi_tentativi() {
    // Il server accetta la richiesta ma non risponde mai entro il timeout.
    let (url, rx, _s) = server(vec![], Duration::from_millis(3500));
    let cfg = ConfigAi {
        timeout_secondi: 1,
        ..config_ai(Fornitore::Openai, &url)
    };
    let client = ClientOpenAi::nuovo(&cfg)
        .unwrap()
        .con_backoff(Duration::from_millis(10));
    let inizio = Instant::now();
    let errore = client.turno(&richiesta()).unwrap_err();
    let durata = inizio.elapsed();
    assert!(
        matches!(errore, ErroreAi::TimeoutLettura { secondi: 1 }),
        "{errore:?}"
    );
    assert!(durata < Duration::from_secs(3), "{durata:?}");
    thread::sleep(Duration::from_millis(1200));
    assert_eq!(
        rx.try_iter().count(),
        1,
        "nessun nuovo tentativo automatico"
    );
}

#[test]
fn timeout_nel_turno_chiede_al_giocatore() {
    let dir = comune::copia_avventure();
    let avventura = Avventura::carica(&dir.path().join("avventure/mini")).unwrap();
    let (url, rx, _s) = server(vec![], Duration::from_millis(3500));
    let config = Config {
        ai: ConfigAi {
            timeout_secondi: 1,
            ..config_ai(Fornitore::Ollama, &url)
        },
        ..Config::default()
    };
    let client = ClientOllama::nuovo(&config.ai).unwrap();
    let mut partita = Partita::nuova(&avventura, &config, &dir.path().join("partite")).unwrap();
    let mut canale = CanaleProva::nuovo(&["n"]);
    let esito = partita
        .esegui_turno(
            &client,
            AzioneTurno::Giocatore("aspetto".into()),
            &mut canale,
        )
        .unwrap();
    assert_eq!(esito, EsitoTurno::Annullato);
    assert!(
        canale
            .uscita
            .contains("Nessuna risposta entro 1 s. Riprovare? (s/n) n"),
        "{}",
        canale.uscita
    );
    assert!(canale.attese >= 1, "indicatore di attesa chiamato");
    thread::sleep(Duration::from_millis(500));
    assert_eq!(rx.try_iter().count(), 1);
    assert_eq!(partita.stato.turno, 0, "stato invariato");
}

// ---------------------------------------------------------------- streaming

/// `VALIDA` divisa in pezzi da 7 caratteri, come arriverebbe in streaming.
fn pezzi_validi() -> Vec<String> {
    VALIDA
        .chars()
        .collect::<Vec<_>>()
        .chunks(7)
        .map(|c| c.iter().collect())
        .collect()
}

/// Corpo Server-Sent Events con un evento `data:` per ogni valore.
fn sse(eventi: &[Value]) -> String {
    let mut corpo: String = eventi.iter().map(|e| format!("data: {e}\n\n")).collect();
    corpo.push_str("data: [DONE]\n\n");
    corpo
}

/// Esegue un turno in streaming e restituisce risposta e pezzi ricevuti.
fn turno_in_flusso(client: &dyn Narratore) -> (avventura::ai::RispostaGrezza, Vec<String>) {
    let mut ricevuti = Vec::new();
    let r = client
        .turno_in_flusso(&richiesta(), &mut |p| ricevuti.push(p.to_string()))
        .unwrap();
    (r, ricevuti)
}

#[test]
fn openai_e_lmstudio_in_streaming() {
    let mut eventi: Vec<Value> = pezzi_validi()
        .iter()
        .map(|p| json!({ "choices": [{ "delta": { "content": p }, "finish_reason": null }] }))
        .collect();
    eventi.insert(
        0,
        json!({ "choices": [{ "delta": { "reasoning_content": "penso" } }] }),
    );
    eventi.push(json!({ "choices": [{ "delta": {}, "finish_reason": "stop" }] }));
    eventi.push(json!({ "choices": [], "usage": { "prompt_tokens": 77, "completion_tokens": 9 } }));
    for fornitore in [Fornitore::Openai, Fornitore::Lmstudio] {
        let (url, rx, _s) = server(vec![(200, sse(&eventi))], Duration::ZERO);
        let client = ClientOpenAi::nuovo(&config_ai(fornitore, &url)).unwrap();
        let (r, pezzi) = turno_in_flusso(&client);
        assert_eq!(r.testo, VALIDA, "{fornitore}");
        assert_eq!(
            pezzi.concat(),
            VALIDA,
            "il ragionamento non arriva al canale"
        );
        assert_eq!((r.token_prompt, r.token_generati), (Some(77), Some(9)));
        let inviata = rx.recv().unwrap();
        assert_eq!(inviata.corpo["stream"], true);
        assert_eq!(inviata.corpo["stream_options"]["include_usage"], true);
    }
}

#[test]
fn openai_stream_options_rifiutato_riprova_senza() {
    let eventi = [json!({ "choices": [{ "delta": { "content": VALIDA } }] })];
    let (url, rx, _s) = server(
        vec![
            (400, r#"{"error":"unknown field: stream_options"}"#.into()),
            (200, sse(&eventi)),
        ],
        Duration::ZERO,
    );
    let client = ClientOpenAi::nuovo(&config_ai(Fornitore::Lmstudio, &url)).unwrap();
    let (r, pezzi) = turno_in_flusso(&client);
    assert_eq!(r.testo, VALIDA);
    assert_eq!(
        pezzi,
        vec![VALIDA.to_string()],
        "nessun pezzo dalla risposta 400"
    );
    assert_eq!(r.avvisi.len(), 1);
    assert!(rx.recv().unwrap().corpo.get("stream_options").is_some());
    assert!(rx.recv().unwrap().corpo.get("stream_options").is_none());
}

#[test]
fn anthropic_in_streaming() {
    let mut corpo = String::from(
        "event: message_start\ndata: {\"type\":\"message_start\",\"message\":{\"usage\":{\"input_tokens\":55,\"output_tokens\":1}}}\n\n\
         event: content_block_delta\ndata: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"thinking_delta\",\"thinking\":\"hmm\"}}\n\n",
    );
    for p in pezzi_validi() {
        let evento = json!({ "type": "content_block_delta", "index": 1, "delta": { "type": "text_delta", "text": p } });
        corpo.push_str(&format!("event: content_block_delta\ndata: {evento}\n\n"));
    }
    corpo.push_str(
        "event: message_delta\ndata: {\"type\":\"message_delta\",\"delta\":{\"stop_reason\":\"end_turn\"},\"usage\":{\"output_tokens\":12}}\n\n\
         event: message_stop\ndata: {\"type\":\"message_stop\"}\n\n",
    );
    let (url, rx, _s) = server(vec![(200, corpo)], Duration::ZERO);
    let client = ClientAnthropic::nuovo(&config_ai(Fornitore::Anthropic, &url)).unwrap();
    let (r, pezzi) = turno_in_flusso(&client);
    assert_eq!(r.testo, VALIDA);
    assert_eq!(pezzi.concat(), VALIDA);
    assert_eq!((r.token_prompt, r.token_generati), (Some(55), Some(12)));
    assert_eq!(rx.recv().unwrap().corpo["stream"], true);

    // Errore a metà dello streaming.
    let corpo = "event: error\ndata: {\"type\":\"error\",\"error\":{\"type\":\"overloaded_error\",\"message\":\"Overloaded\"}}\n\n";
    let (url, _rx, _s) = server(vec![(200, corpo.into())], Duration::ZERO);
    let client = ClientAnthropic::nuovo(&config_ai(Fornitore::Anthropic, &url)).unwrap();
    let errore = client
        .turno_in_flusso(&richiesta(), &mut |_| {})
        .unwrap_err();
    assert!(errore.to_string().contains("Overloaded"), "{errore}");
}

#[test]
fn gemini_in_streaming() {
    let mut eventi: Vec<Value> = vec![json!({
        "candidates": [{ "content": { "role": "model", "parts": [{ "text": "ragiono", "thought": true }] } }]
    })];
    for p in pezzi_validi() {
        eventi.push(
            json!({ "candidates": [{ "content": { "role": "model", "parts": [{ "text": p }] } }] }),
        );
    }
    eventi.push(json!({
        "candidates": [{ "content": { "role": "model", "parts": [{ "text": "" }] }, "finishReason": "STOP" }],
        "usageMetadata": { "promptTokenCount": 44, "candidatesTokenCount": 8 }
    }));
    let mut corpo = sse(&eventi);
    corpo.truncate(corpo.len() - "data: [DONE]\n\n".len()); // Gemini non lo invia
    let (url, rx, _s) = server(vec![(200, corpo)], Duration::ZERO);
    let client = ClientGemini::nuovo(&config_ai(Fornitore::Gemini, &url)).unwrap();
    let (r, pezzi) = turno_in_flusso(&client);
    assert_eq!(r.testo, VALIDA);
    assert_eq!(pezzi.concat(), VALIDA);
    assert_eq!((r.token_prompt, r.token_generati), (Some(44), Some(8)));
    assert_eq!(
        rx.recv().unwrap().percorso,
        "/models/modello-prova:streamGenerateContent?alt=sse"
    );
}

#[test]
fn ollama_in_streaming() {
    let mut corpo = String::from(
        "{\"message\":{\"role\":\"assistant\",\"content\":\"\",\"thinking\":\"hmm\"},\"done\":false}\n",
    );
    for p in pezzi_validi() {
        corpo.push_str(&format!(
            "{}\n",
            json!({ "message": { "role": "assistant", "content": p }, "done": false })
        ));
    }
    corpo.push_str("{\"message\":{\"role\":\"assistant\",\"content\":\"\"},\"done\":true,\"done_reason\":\"stop\",\"prompt_eval_count\":66,\"eval_count\":7}\n");
    let (url, rx, _s) = server(vec![(200, corpo)], Duration::ZERO);
    let client = ClientOllama::nuovo(&config_ai(Fornitore::Ollama, &url)).unwrap();
    let (r, pezzi) = turno_in_flusso(&client);
    assert_eq!(r.testo, VALIDA);
    assert_eq!(pezzi.concat(), VALIDA);
    assert_eq!((r.token_prompt, r.token_generati), (Some(66), Some(7)));
    assert_eq!(rx.recv().unwrap().corpo["stream"], true);
}

#[test]
fn turno_progressivo_mostra_la_narrazione_a_pezzi() {
    let eventi: Vec<Value> = pezzi_validi()
        .iter()
        .map(|p| json!({ "choices": [{ "delta": { "content": p } }] }))
        .collect();
    let (url, _rx, _s) = server(vec![(200, sse(&eventi))], Duration::ZERO);
    let dir = comune::copia_avventure();
    let avventura = Avventura::carica(&dir.path().join("avventure/mini")).unwrap();
    let config = Config {
        ai: config_ai(Fornitore::Openai, &url),
        ..Config::default()
    };
    let client = ClientOpenAi::nuovo(&config.ai).unwrap();
    let mut partita = Partita::nuova(&avventura, &config, &dir.path().join("partite")).unwrap();
    let mut canale = CanaleProva::default();
    canale.progressivo = true;
    let esito = partita
        .esegui_turno(
            &client,
            AzioneTurno::Giocatore("apro la porta".into()),
            &mut canale,
        )
        .unwrap();
    assert!(
        matches!(&esito, EsitoTurno::Applicato { narrazione, .. } if narrazione == "Apri la porta."),
        "{esito:?}"
    );
    assert_eq!(canale.frammenti.concat(), "Apri la porta.");
}

#[test]
fn connessione_rifiutata_errore_rapido() {
    let porta = {
        let l = TcpListener::bind("127.0.0.1:0").unwrap();
        l.local_addr().unwrap().port()
    };
    for fornitore in [Fornitore::Openai, Fornitore::Ollama, Fornitore::Lmstudio] {
        let cfg = config_ai(fornitore, &format!("http://127.0.0.1:{porta}/v1"));
        let client = avventura::ai::crea_narratore(&cfg).unwrap();
        let inizio = Instant::now();
        let errore = client.turno(&richiesta()).unwrap_err();
        assert!(
            inizio.elapsed() < Duration::from_secs(4),
            "nessuna attesa di backoff: {:?}",
            inizio.elapsed()
        );
        assert!(
            errore.to_string().contains(&format!(
                "il server locale non risponde su 127.0.0.1:{porta}"
            )),
            "{fornitore}: {errore}"
        );
    }
}
