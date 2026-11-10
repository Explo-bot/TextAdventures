//! Bot Telegram (§7.5, §10) contro un server HTTP di prova che imita l'API Bot:
//! registra le richieste e restituisce aggiornamenti scriptati. L'AI è quella
//! simulata (`finto`), eventualmente rallentata o difettosa.
//!
//! È un binario di test separato perché installa un log `tracing` globale, usato
//! per verificare che il token non compaia mai nel log.

mod comune;

use std::collections::VecDeque;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use avventura::ai::finto::NarratoreFinto;
use avventura::ai::{ErroreAi, Narratore, RichiestaAi, RispostaGrezza};
use avventura::config::Config;
use avventura::telegram::api::{Api, lunghezza_telegram};
use avventura::telegram::bot::{self, DatiBot, OpzioniBot};
use serde_json::{Value, json};

const TOKEN: &str = "123456789:TOKEN-SEGRETISSIMO-di-prova";

// ---------------------------------------------------------------------------
// Log globale (per il controllo del token)
// ---------------------------------------------------------------------------

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

fn log_globale() -> &'static BufferLog {
    static LOG: OnceLock<BufferLog> = OnceLock::new();
    LOG.get_or_init(|| {
        let buffer = BufferLog::default();
        let b = buffer.clone();
        tracing_subscriber::fmt()
            .with_writer(move || b.clone())
            .with_ansi(false)
            .with_max_level(tracing::Level::DEBUG)
            .init();
        buffer
    })
}

// ---------------------------------------------------------------------------
// Server di prova dell'API Bot
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
struct Richiesta {
    metodo: String,
    corpo: Value,
    risposta: Value,
    quando: Instant,
}

impl Richiesta {
    fn chat(&self) -> i64 {
        self.corpo["chat_id"].as_i64().unwrap_or_default()
    }
    fn testo(&self) -> String {
        self.corpo["text"].as_str().unwrap_or_default().to_string()
    }
    fn message_id(&self) -> i64 {
        self.risposta["message_id"].as_i64().unwrap_or_default()
    }
    /// Pulsanti della tastiera inline: (testo, callback_data).
    fn pulsanti(&self) -> Vec<(String, String)> {
        self.corpo["reply_markup"]["inline_keyboard"]
            .as_array()
            .map(|righe| {
                righe
                    .iter()
                    .flat_map(|r| r.as_array().cloned().unwrap_or_default())
                    .map(|p| {
                        (
                            p["text"].as_str().unwrap_or_default().to_string(),
                            p["callback_data"].as_str().unwrap_or_default().to_string(),
                        )
                    })
                    .collect()
            })
            .unwrap_or_default()
    }
}

#[derive(Default)]
struct StatoFinto {
    aggiornamenti: Vec<Value>,
    prossimo_update: i64,
    prossimo_messaggio: i64,
    richieste: Vec<Richiesta>,
    offset_polling: Vec<(Instant, i64)>,
    errori: VecDeque<(String, u16, Value)>,
}

#[derive(Clone)]
struct ServerFinto {
    porta: u16,
    stato: Arc<Mutex<StatoFinto>>,
}

impl ServerFinto {
    fn avvia() -> Self {
        let server = Arc::new(tiny_http::Server::http("127.0.0.1:0").unwrap());
        let porta = server.server_addr().to_ip().unwrap().port();
        let stato = Arc::new(Mutex::new(StatoFinto {
            prossimo_update: 1,
            prossimo_messaggio: 1000,
            ..Default::default()
        }));
        let s = Arc::clone(&stato);
        thread::spawn(move || {
            for r in server.incoming_requests() {
                let s = Arc::clone(&s);
                thread::spawn(move || rispondi(r, &s));
            }
        });
        ServerFinto { porta, stato }
    }

    fn base_url(&self) -> String {
        format!("http://127.0.0.1:{}", self.porta)
    }

    fn aggiungi(&self, mut aggiornamento: Value) -> i64 {
        let mut s = self.stato.lock().unwrap();
        let id = s.prossimo_update;
        s.prossimo_update += 1;
        aggiornamento["update_id"] = json!(id);
        s.aggiornamenti.push(aggiornamento);
        id
    }

    fn testo(&self, utente: i64, testo: &str) -> i64 {
        self.aggiungi(json!({ "message": {
            "message_id": 1, "date": 0, "text": testo,
            "from": { "id": utente, "is_bot": false, "first_name": "Giocatore" },
            "chat": { "id": utente, "type": "private" } } }))
    }

    /// Messaggio di un utente con l'app Telegram nella lingua `codice`.
    fn testo_con_lingua(&self, utente: i64, testo: &str, codice: &str) -> i64 {
        self.aggiungi(json!({ "message": {
            "message_id": 1, "date": 0, "text": testo,
            "from": { "id": utente, "is_bot": false, "first_name": "Player",
                      "language_code": codice },
            "chat": { "id": utente, "type": "private" } } }))
    }

    fn testo_gruppo(&self, utente: i64, testo: &str) -> i64 {
        self.aggiungi(json!({ "message": {
            "message_id": 1, "date": 0, "text": testo,
            "from": { "id": utente, "is_bot": false, "first_name": "Giocatore" },
            "chat": { "id": -4242, "type": "group", "title": "Gruppo" } } }))
    }

    fn foto(&self, utente: i64) -> i64 {
        self.aggiungi(json!({ "message": {
            "message_id": 1, "date": 0, "photo": [],
            "from": { "id": utente, "is_bot": false, "first_name": "Giocatore" },
            "chat": { "id": utente, "type": "private" } } }))
    }

    fn premi(&self, utente: i64, message_id: i64, dati: &str) -> i64 {
        self.aggiungi(json!({ "callback_query": {
            "id": format!("cb-{utente}-{message_id}-{dati}"), "chat_instance": "x", "data": dati,
            "from": { "id": utente, "is_bot": false, "first_name": "Giocatore" },
            "message": { "message_id": message_id, "date": 0,
                         "chat": { "id": utente, "type": "private" } } } }))
    }

    /// Prossima risposta di `metodo` forzata (stato HTTP e corpo).
    fn errore(&self, metodo: &str, stato: u16, corpo: Value) {
        self.stato
            .lock()
            .unwrap()
            .errori
            .push_back((metodo.into(), stato, corpo));
    }

    fn segno(&self) -> usize {
        self.stato.lock().unwrap().richieste.len()
    }

    fn richieste(&self) -> Vec<Richiesta> {
        self.stato.lock().unwrap().richieste.clone()
    }

    /// Attende una richiesta (registrata dopo `da`) che soddisfa `pred`.
    fn attendi(&self, da: usize, cosa: &str, pred: impl Fn(&Richiesta) -> bool) -> Richiesta {
        let fine = Instant::now() + Duration::from_secs(15);
        while Instant::now() < fine {
            if let Some(r) = self.richieste().into_iter().skip(da).find(|r| pred(r)) {
                return r;
            }
            thread::sleep(Duration::from_millis(20));
        }
        let riassunto: Vec<String> = self
            .richieste()
            .iter()
            .map(|r| format!("{} → {}", r.metodo, r.corpo))
            .collect();
        panic!("atteso: {cosa}\nrichieste:\n{}", riassunto.join("\n"));
    }

    /// Attende un `sendMessage` a `chat` il cui testo contiene `parte`.
    fn messaggio(&self, da: usize, chat: i64, parte: &str) -> Richiesta {
        self.attendi(da, &format!("sendMessage a {chat} con «{parte}»"), |r| {
            r.metodo == "sendMessage" && r.chat() == chat && r.testo().contains(parte)
        })
    }

    /// Premi il pulsante il cui testo contiene `etichetta` nel messaggio `m`.
    fn premi_pulsante(&self, utente: i64, m: &Richiesta, etichetta: &str) {
        let (_, dati) = m
            .pulsanti()
            .into_iter()
            .find(|(t, _)| t.contains(etichetta))
            .unwrap_or_else(|| panic!("pulsante «{etichetta}» assente: {:?}", m.pulsanti()));
        self.premi(utente, m.message_id(), &dati);
    }

    fn messaggi_a(&self, chat: i64) -> Vec<String> {
        self.richieste()
            .iter()
            .filter(|r| r.metodo == "sendMessage" && r.chat() == chat)
            .map(Richiesta::testo)
            .collect()
    }
}

fn rispondi(mut r: tiny_http::Request, stato: &Mutex<StatoFinto>) {
    let mut testo = String::new();
    let _ = r.as_reader().read_to_string(&mut testo);
    let corpo: Value = serde_json::from_str(&testo).unwrap_or(Value::Null);
    let metodo = r.url().rsplit('/').next().unwrap_or_default().to_string();
    assert!(
        r.url().starts_with(&format!("/bot{TOKEN}/")),
        "URL inatteso: {}",
        r.url()
    );

    let forzato = {
        let mut s = stato.lock().unwrap();
        let i = s.errori.iter().position(|(m, _, _)| *m == metodo);
        i.and_then(|i| s.errori.remove(i))
    };
    if let Some((_, codice, risposta)) = forzato {
        stato.lock().unwrap().richieste.push(Richiesta {
            metodo: format!("{metodo} (forzato {codice})"),
            corpo,
            risposta: risposta.clone(),
            quando: Instant::now(),
        });
        let _ = r.respond(
            tiny_http::Response::from_string(risposta.to_string()).with_status_code(codice),
        );
        return;
    }

    let risultato = if metodo == "getUpdates" {
        let offset = corpo["offset"].as_i64().unwrap_or(0);
        let pronti: Vec<Value> = {
            let mut s = stato.lock().unwrap();
            s.offset_polling.push((Instant::now(), offset));
            s.aggiornamenti
                .retain(|u| u["update_id"].as_i64().unwrap_or(0) >= offset);
            s.aggiornamenti.clone()
        };
        if pronti.is_empty() {
            thread::sleep(Duration::from_millis(30));
        }
        json!(pronti)
    } else {
        let mut s = stato.lock().unwrap();
        let risultato = match metodo.as_str() {
            "getMe" => json!({ "id": 1, "is_bot": true, "first_name": "Prova",
                               "username": "prova_bot" }),
            "sendMessage" | "editMessageText" => {
                let id = if metodo == "sendMessage" {
                    s.prossimo_messaggio += 1;
                    s.prossimo_messaggio
                } else {
                    corpo["message_id"].as_i64().unwrap_or(0)
                };
                json!({ "message_id": id, "date": 0, "text": corpo["text"],
                        "chat": { "id": corpo["chat_id"], "type": "private" } })
            }
            _ => json!(true),
        };
        s.richieste.push(Richiesta {
            metodo: metodo.clone(),
            corpo,
            risposta: risultato.clone(),
            quando: Instant::now(),
        });
        risultato
    };
    let _ = r.respond(tiny_http::Response::from_string(
        json!({ "ok": true, "result": risultato }).to_string(),
    ));
}

// ---------------------------------------------------------------------------
// Narratori di prova
// ---------------------------------------------------------------------------

/// AI simulata rallentata.
struct NarratoreLento(Duration);

impl Narratore for NarratoreLento {
    fn turno(&self, r: &RichiestaAi) -> Result<RispostaGrezza, ErroreAi> {
        thread::sleep(self.0);
        NarratoreFinto.turno(r)
    }
    fn descrizione(&self) -> String {
        "finto lento".into()
    }
}

/// AI che va in panic al primo turno, poi funziona.
struct NarratoreDifettoso(AtomicBool);

impl Narratore for NarratoreDifettoso {
    fn turno(&self, r: &RichiestaAi) -> Result<RispostaGrezza, ErroreAi> {
        if !self.0.swap(true, Ordering::SeqCst) {
            panic!("guasto simulato");
        }
        NarratoreFinto.turno(r)
    }
    fn descrizione(&self) -> String {
        "finto difettoso".into()
    }
}

// ---------------------------------------------------------------------------
// Avvio del bot nei test
// ---------------------------------------------------------------------------

struct BotInProva {
    ferma: Arc<AtomicBool>,
    thread: Option<JoinHandle<anyhow::Result<()>>>,
}

impl BotInProva {
    fn ferma(mut self) {
        self.ferma.store(true, Ordering::SeqCst);
        if let Some(t) = self.thread.take() {
            t.join().unwrap().unwrap();
        }
    }
}

impl Drop for BotInProva {
    fn drop(&mut self) {
        self.ferma.store(true, Ordering::SeqCst);
    }
}

fn config_bot() -> Config {
    let mut c = comune::config_finta();
    c.telegram.utenti_ammessi = vec![100, 200];
    c.telegram.amministratori = vec![900];
    c.telegram.max_turni_paralleli = 1;
    c
}

fn avvia_bot(
    dir: &Path,
    server: &ServerFinto,
    config: Config,
    narratore: Arc<dyn Narratore>,
    scadenza: Duration,
) -> BotInProva {
    log_globale();
    let ferma = Arc::new(AtomicBool::new(false));
    let opzioni = OpzioniBot {
        timeout_polling: 0,
        scadenza_domande: scadenza,
        chiusura_inattive: None,
        attesa_chiusura: Duration::from_secs(10),
        backoff_max: Duration::from_secs(1),
        ferma: Arc::clone(&ferma),
    };
    let dati = DatiBot {
        config,
        cartella_avventure: dir.join("avventure"),
        cartella_partite: cartella_tg(dir),
        narratore,
        api: Api::con_base_url(TOKEN, &server.base_url()).unwrap(),
    };
    let thread = thread::spawn(move || bot::esegui(dati, opzioni));
    BotInProva {
        ferma,
        thread: Some(thread),
    }
}

fn cartella_tg(dir: &Path) -> PathBuf {
    dir.join("partite").join("telegram")
}

fn avvia_standard(dir: &Path, server: &ServerFinto) -> BotInProva {
    avvia_bot(
        dir,
        server,
        config_bot(),
        Arc::new(NarratoreFinto),
        Duration::from_secs(30),
    )
}

/// `/start` → menu delle avventure → «Mini avventura» → nuova partita.
fn entra_in_mini(server: &ServerFinto, utente: i64) {
    let s = server.segno();
    server.testo(utente, "/start");
    let menu = server.messaggio(s, utente, "AVVENTURE DISPONIBILI");
    server.premi_pulsante(utente, &menu, "Mini avventura");
}

fn token_assente_dappertutto(server: &ServerFinto) {
    let log = String::from_utf8_lossy(&log_globale().0.lock().unwrap()).to_string();
    assert!(
        !log.contains(TOKEN) && !log.contains("SEGRETISSIMO"),
        "token nel log"
    );
    for r in server.richieste() {
        assert!(!r.corpo.to_string().contains("SEGRETISSIMO"), "{r:?}");
    }
}

// ---------------------------------------------------------------------------
// Test
// ---------------------------------------------------------------------------

/// 1. Un utente non ammesso riceve solo il messaggio con il suo ID.
#[test]
fn utente_non_ammesso() {
    let dir = comune::copia_avventure();
    let server = ServerFinto::avvia();
    let bot = avvia_standard(dir.path(), &server);
    server.testo(555, "ciao");
    server.messaggio(0, 555, "Il tuo ID Telegram è 555");
    server.foto(555);
    server.testo(555, "/start");
    let s = server.segno();
    server.testo(100, "/help");
    server.messaggio(s, 100, "Benvenuto");
    server.messaggio(s, 100, "https://github.com/Explo-bot/TextAdventures");
    server.messaggio(s, 100, "è disponibile gratuitamente su https://gronk.it");
    thread::sleep(Duration::from_millis(200));
    let a_555 = server.messaggi_a(555);
    assert_eq!(a_555.len(), 3, "{a_555:?}");
    for m in &a_555 {
        assert_eq!(
            m,
            "Questo bot è privato. Il tuo ID Telegram è 555: chiedi al proprietario di aggiungerlo."
        );
    }
    bot.ferma();
    token_assente_dappertutto(&server);
}

/// 2–4. `/start` → tastiera → scelta → nuova partita; azioni; `/save` (nome
/// digitato e nome proposto) e `/load` via pulsanti.
#[test]
fn partita_con_pulsanti_save_e_load() {
    let dir = comune::copia_avventure();
    let server = ServerFinto::avvia();
    let bot = avvia_standard(dir.path(), &server);

    let s = server.segno();
    server.testo(100, "/start");
    let menu = server.messaggio(s, 100, "AVVENTURE DISPONIBILI");
    let etichette: Vec<String> = menu.pulsanti().into_iter().map(|p| p.0).collect();
    assert!(
        etichette.iter().any(|e| e.contains("Mini avventura")),
        "{etichette:?}"
    );
    assert!(
        etichette.iter().any(|e| e.contains("Altra avventura")),
        "{etichette:?}"
    );
    assert!(etichette.iter().any(|e| e.contains("Annulla")));
    assert!(
        !etichette.iter().any(|e| e.contains("rotta")),
        "le non valide non sono pulsanti"
    );
    assert!(
        menu.testo()
            .contains("Tre stanze, una chiave e una botola.")
    );
    assert!(menu.testo().contains("<s>rotta"), "{}", menu.testo());

    let s = server.segno();
    server.premi_pulsante(100, &menu, "Mini avventura");
    server.attendi(s, "answerCallbackQuery", |r| {
        r.metodo == "answerCallbackQuery"
    });
    let modifica = server.attendi(s, "editMessageText del menu", |r| {
        r.metodo == "editMessageText" && r.corpo["message_id"] == json!(menu.message_id())
    });
    assert!(
        modifica.testo().contains("➜ <i>Mini avventura"),
        "{}",
        modifica.testo()
    );
    assert!(
        modifica.corpo.get("reply_markup").is_none(),
        "pulsanti rimossi"
    );
    server.messaggio(s, 100, "<b>MINI AVVENTURA</b>");
    server.messaggio(s, 100, "Sei davanti a una vecchia casa");
    server.messaggio(s, 100, "<i>[Turno 1");

    // Azioni.
    let s = server.segno();
    server.testo(100, "prendi la chiave");
    server.messaggio(s, 100, "Prendi una piccola chiave di ferro");
    let s = server.segno();
    server.testo(100, "/inventario");
    server.messaggio(s, 100, "<pre>Porti con te:");

    // /save → «Nuovo salvataggio» → nome digitato.
    let s = server.segno();
    server.testo(100, "/save");
    let m = server.messaggio(s, 100, "SALVA PARTITA");
    server.premi_pulsante(100, &m, "Nuovo salvataggio");
    let domanda = server.messaggio(s, 100, "Nome del salvataggio");
    assert!(domanda.pulsanti().iter().any(|p| p.0.starts_with('✔')));
    let s = server.segno();
    server.testo(100, "primo");
    server.messaggio(s, 100, "Partita salvata in «primo» (turno 2)");

    // Un'altra azione, poi /save → «Nuovo salvataggio» → nome proposto.
    let s = server.segno();
    server.testo(100, "vai a nord");
    server.messaggio(s, 100, "<b>L'atrio</b>");
    let s2 = server.segno();
    server.testo(100, "/save");
    let m = server.messaggio(s2, 100, "SALVA PARTITA");
    let etichette: Vec<String> = m.pulsanti().into_iter().map(|p| p.0).collect();
    assert!(
        etichette.iter().any(|e| e.contains("Salva in «primo»")),
        "{etichette:?}"
    );
    assert!(
        etichette.iter().any(|e| e.contains("Sovrascrivi")),
        "{etichette:?}"
    );
    server.premi_pulsante(100, &m, "Nuovo salvataggio");
    let domanda = server.messaggio(s2, 100, "Nome del salvataggio");
    server.premi_pulsante(100, &domanda, "✔");
    let fatto = server.messaggio(s2, 100, "Partita salvata in «");
    assert!(fatto.testo().contains("turno 3"), "{}", fatto.testo());

    // /load → lista come pulsanti (★ sul corrente) → «primo» → conferma? no: niente progressi.
    let s = server.segno();
    server.testo(100, "/load");
    let lista = server.messaggio(s, 100, "SALVATAGGI DISPONIBILI");
    let etichette: Vec<String> = lista.pulsanti().into_iter().map(|p| p.0).collect();
    assert!(
        etichette.iter().any(|e| e.starts_with("★ ")),
        "{etichette:?}"
    );
    assert!(
        etichette
            .iter()
            .any(|e| e.starts_with("primo · turno 2 · ")),
        "{etichette:?}"
    );
    server.premi_pulsante(100, &lista, "primo ·");
    server.messaggio(s, 100, "Caricato «primo» (turno 2)");

    let salvataggi = cartella_tg(dir.path())
        .join("100")
        .join("mini")
        .join("salvataggi");
    assert!(salvataggi.join("primo.json").is_file());
    assert_eq!(fs::read_dir(&salvataggi).unwrap().count(), 2);
    bot.ferma();
    token_assente_dappertutto(&server);
}

/// 5. Due utenti in parallelo: cartelle separate, limitatore a 1 (uno è in coda).
#[test]
fn due_utenti_in_parallelo() {
    let dir = comune::copia_avventure();
    let server = ServerFinto::avvia();
    let bot = avvia_bot(
        dir.path(),
        &server,
        config_bot(),
        Arc::new(NarratoreLento(Duration::from_millis(1500))),
        Duration::from_secs(30),
    );
    let s = server.segno();
    server.testo(100, "/start");
    server.testo(200, "/start");
    let m100 = server.messaggio(s, 100, "AVVENTURE DISPONIBILI");
    let m200 = server.messaggio(s, 200, "AVVENTURE DISPONIBILI");
    server.premi_pulsante(100, &m100, "Mini avventura");
    server.premi_pulsante(200, &m200, "Mini avventura");
    let coda = server.attendi(s, "avviso di coda", |r| {
        r.metodo == "sendMessage" && r.testo().contains("sei in coda")
    });
    let in_coda = coda.chat();
    assert!(in_coda == 100 || in_coda == 200);
    server.messaggio(s, 100, "<i>[Turno 1");
    server.messaggio(s, 200, "<i>[Turno 1");
    assert_eq!(
        server
            .richieste()
            .iter()
            .filter(|r| r.testo().contains("sei in coda"))
            .count(),
        1
    );
    let tg = cartella_tg(dir.path());
    assert!(tg.join("100").join("mini").join("stato.json").is_file());
    assert!(tg.join("200").join("mini").join("stato.json").is_file());
    assert!(
        !dir.path().join("partite").join("mini").exists(),
        "niente partite del terminale"
    );
    // Il limitatore serializza i turni: il secondo inizia dopo il primo.
    assert!(
        server
            .richieste()
            .iter()
            .any(|r| r.metodo == "sendChatAction" && r.corpo["action"] == "typing")
    );
    // Messaggio «sta scrivendo…» per ogni turno, poi cancellato.
    let richieste = server.richieste();
    let segnaposti: Vec<i64> = richieste
        .iter()
        .filter(|r| r.metodo == "sendMessage" && r.testo().contains("Il narratore sta scrivendo"))
        .map(|r| r.risposta["message_id"].as_i64().unwrap())
        .collect();
    assert!(segnaposti.len() >= 2, "uno per giocatore: {segnaposti:?}");
    for id in segnaposti {
        assert!(
            richieste
                .iter()
                .any(|r| r.metodo == "deleteMessage" && r.corpo["message_id"] == id),
            "messaggio {id} non cancellato"
        );
    }
    bot.ferma();
}

/// 6–7. Messaggio lungo diviso in più parti ≤ 4.096 e escape HTML.
#[test]
fn messaggio_lungo_e_escape_html() {
    let dir = comune::copia_avventure();
    let lunga = dir.path().join("avventure").join("lunga");
    fs::create_dir_all(lunga.join("stanze")).unwrap();
    let paragrafo = "Il cartello dice: a < b && c > d, \"attenzione\". ".repeat(150);
    fs::write(
        lunga.join("gioco.md"),
        format!(
            "+++\ntitolo = \"Lunga <prova>\"\ndescrizione_breve = \"x & y\"\nstanza_iniziale = \"s\"\n+++\n\n\
             ## Introduzione\n{paragrafo}\n"
        ),
    )
    .unwrap();
    fs::write(
        lunga.join("stanze").join("s.md"),
        "+++\nid = \"s\"\nnome = \"Sala <grande>\"\n+++\n\n## Descrizione\nUna sala.\n",
    )
    .unwrap();
    let server = ServerFinto::avvia();
    let bot = avvia_standard(dir.path(), &server);
    let s = server.segno();
    server.testo(100, "/start");
    let menu = server.messaggio(s, 100, "AVVENTURE DISPONIBILI");
    assert!(
        menu.testo().contains("Lunga &lt;prova&gt;"),
        "{}",
        menu.testo()
    );
    assert!(menu.testo().contains("x &amp; y"));
    server.premi_pulsante(100, &menu, "Lunga <prova>");
    server.messaggio(s, 100, "<b>Sala &lt;grande&gt;</b>");
    let parti: Vec<String> = server
        .messaggi_a(100)
        .into_iter()
        .filter(|t| t.contains("Il cartello dice"))
        .collect();
    assert!(parti.len() >= 2, "{} parti", parti.len());
    for p in &parti {
        assert!(lunghezza_telegram(p) <= 4096, "{}", lunghezza_telegram(p));
        assert!(p.contains("a &lt; b &amp;&amp; c &gt; d"));
        assert!(!p.contains(" < ") && !p.contains(" && "));
        // Nessuna parola spezzata: ogni parte inizia e finisce con parole intere.
        let parole = [
            "Il",
            "cartello",
            "dice:",
            "a",
            "&lt;",
            "b",
            "&amp;&amp;",
            "c",
            "&gt;",
            "d,",
            "&quot;attenzione&quot;.",
        ];
        for w in p.split_whitespace() {
            assert!(parole.contains(&w), "parola spezzata: «{w}»");
        }
    }
    let totale: usize = parti
        .iter()
        .map(|p| p.matches("Il cartello dice").count())
        .sum();
    assert_eq!(totale, 150);
    bot.ferma();
}

/// 8. Una risposta 429 con `retry_after` viene rispettata.
#[test]
fn rispetto_di_retry_after() {
    let dir = comune::copia_avventure();
    let server = ServerFinto::avvia();
    server.errore(
        "sendMessage",
        429,
        json!({ "ok": false, "error_code": 429,
                "description": "Too Many Requests: retry after 1",
                "parameters": { "retry_after": 1 } }),
    );
    let bot = avvia_standard(dir.path(), &server);
    server.testo(100, "/start");
    let rifiutata = server.attendi(0, "429", |r| r.metodo.starts_with("sendMessage (forzato"));
    let riuscita = server.attendi(0, "sendMessage dopo il 429", |r| {
        r.metodo == "sendMessage" && r.quando > rifiutata.quando
    });
    let attesa = riuscita.quando - rifiutata.quando;
    assert!(attesa >= Duration::from_millis(950), "{attesa:?}");
    assert_eq!(riuscita.testo(), rifiutata.corpo["text"].as_str().unwrap());
    bot.ferma();
}

/// 9. L'offset è salvato e un riavvio non rielabora i messaggi.
#[test]
fn persistenza_dell_offset() {
    let dir = comune::copia_avventure();
    let server = ServerFinto::avvia();
    let bot = avvia_standard(dir.path(), &server);
    let id = server.testo(100, "/start");
    server.messaggio(0, 100, "AVVENTURE DISPONIBILI");
    bot.ferma();
    let offset = bot::carica_offset(&cartella_tg(dir.path()).join("offset.json"));
    assert_eq!(offset, id + 1);

    let riavvio = Instant::now();
    let bot = avvia_standard(dir.path(), &server);
    let fine = Instant::now() + Duration::from_secs(10);
    loop {
        let offsets: Vec<i64> = server
            .stato
            .lock()
            .unwrap()
            .offset_polling
            .iter()
            .filter(|(t, _)| *t > riavvio)
            .map(|(_, o)| *o)
            .collect();
        if offsets.len() >= 3 {
            assert!(offsets.iter().all(|&o| o == id + 1), "{offsets:?}");
            break;
        }
        assert!(Instant::now() < fine, "nessun polling dopo il riavvio");
        thread::sleep(Duration::from_millis(20));
    }
    thread::sleep(Duration::from_millis(300));
    let menu = server
        .messaggi_a(100)
        .iter()
        .filter(|t| t.contains("AVVENTURE DISPONIBILI"))
        .count();
    assert_eq!(menu, 1, "il messaggio non deve essere rielaborato");
    bot.ferma();
}

/// 10. Messaggi da un gruppo: ignorati con una spiegazione; non testuali.
#[test]
fn gruppo_e_messaggi_non_testuali() {
    let dir = comune::copia_avventure();
    let server = ServerFinto::avvia();
    let bot = avvia_standard(dir.path(), &server);
    server.testo_gruppo(100, "/start");
    server.messaggio(0, -4242, "solo nelle chat private");
    server.foto(100);
    server.messaggio(0, 100, "Scrivi la tua azione come testo.");
    thread::sleep(Duration::from_millis(200));
    assert!(
        !server
            .messaggi_a(100)
            .iter()
            .chain(server.messaggi_a(-4242).iter())
            .any(|t| t.contains("AVVENTURE")),
        "nessuna sessione per il gruppo"
    );
    bot.ferma();
}

/// 11. Un messaggio mentre l'AI sta scrivendo riceve subito una risposta.
#[test]
fn messaggio_durante_un_turno() {
    let dir = comune::copia_avventure();
    let server = ServerFinto::avvia();
    let bot = avvia_bot(
        dir.path(),
        &server,
        config_bot(),
        Arc::new(NarratoreLento(Duration::from_millis(2500))),
        Duration::from_secs(30),
    );
    let s = server.segno();
    entra_in_mini(&server, 100);
    server.attendi(s, "sendChatAction", |r| r.metodo == "sendChatAction");
    server.testo(100, "guardo intorno");
    let risposta = server.messaggio(s, 100, "Sto ancora scrivendo la risposta precedente");
    let fine_turno = server.messaggio(s, 100, "<i>[Turno 1");
    assert!(risposta.quando < fine_turno.quando);
    // L'azione non è stata accodata: nessun turno 2.
    thread::sleep(Duration::from_millis(500));
    assert!(
        !server
            .messaggi_a(100)
            .iter()
            .any(|t| t.contains("[Turno 2"))
    );
    bot.ferma();
}

/// 12. Una domanda in sospeso scade (= annulla).
#[test]
fn scadenza_di_una_domanda() {
    let dir = comune::copia_avventure();
    let server = ServerFinto::avvia();
    let bot = avvia_bot(
        dir.path(),
        &server,
        config_bot(),
        Arc::new(NarratoreFinto),
        Duration::from_millis(400),
    );
    let s = server.segno();
    server.testo(100, "/start");
    let menu = server.messaggio(s, 100, "AVVENTURE DISPONIBILI");
    let modifica = server.attendi(s, "domanda scaduta", |r| {
        r.metodo == "editMessageText" && r.testo().contains("Domanda scaduta")
    });
    assert_eq!(modifica.corpo["message_id"], json!(menu.message_id()));
    server.messaggio(s, 100, "Tempo scaduto");
    server.messaggio(s, 100, "Partita chiusa");
    // Scrivendo di nuovo si riparte.
    let s = server.segno();
    server.testo(100, "ciao");
    server.messaggio(s, 100, "AVVENTURE DISPONIBILI");
    bot.ferma();
}

/// 13. La callback di una domanda vecchia: «non più valida».
#[test]
fn callback_di_una_domanda_vecchia() {
    let dir = comune::copia_avventure();
    let server = ServerFinto::avvia();
    let bot = avvia_standard(dir.path(), &server);
    let s = server.segno();
    server.testo(100, "/start");
    let menu = server.messaggio(s, 100, "AVVENTURE DISPONIBILI");
    server.premi_pulsante(100, &menu, "Mini avventura");
    server.messaggio(s, 100, "<i>[Turno 1");
    let s = server.segno();
    server.premi_pulsante(100, &menu, "Mini avventura");
    let r = server.attendi(s, "risposta alla callback vecchia", |r| {
        r.metodo == "answerCallbackQuery"
    });
    assert_eq!(r.corpo["text"], "Questa scelta non è più valida.");
    server.attendi(s, "pulsanti rimossi", |r| {
        r.metodo == "editMessageReplyMarkup"
    });
    // La partita continua normalmente.
    let s = server.segno();
    server.testo(100, "/inventario");
    server.messaggio(s, 100, "Non porti niente con te.");
    bot.ferma();
}

/// 14. `codice_invito` e comandi amministratore.
#[test]
fn codice_invito_e_amministratori() {
    let dir = comune::copia_avventure();
    let server = ServerFinto::avvia();
    let mut config = config_bot();
    config.telegram.codice_invito = Some("apriti".into());
    let bot = avvia_bot(
        dir.path(),
        &server,
        config,
        Arc::new(NarratoreFinto),
        Duration::from_secs(30),
    );
    let tg = cartella_tg(dir.path());

    server.testo(777, "/start sbagliato");
    server.messaggio(0, 777, "Il tuo ID Telegram è 777");
    let s = server.segno();
    server.testo(777, "/start apriti");
    server.messaggio(s, 777, "Codice d'invito accettato");
    server.messaggio(s, 777, "AVVENTURE DISPONIBILI");
    let utenti: Value =
        serde_json::from_str(&fs::read_to_string(tg.join("utenti.json")).unwrap()).unwrap();
    assert_eq!(utenti["ammessi"], json!([777]));

    // Comandi amministratore.
    let s = server.segno();
    server.testo(900, "/utenti");
    let elenco = server.messaggio(s, 900, "Utenti ammessi");
    for id in ["100", "200", "777", "900 (amministratore)"] {
        assert!(elenco.testo().contains(id), "{}", elenco.testo());
    }
    let s = server.segno();
    server.testo(900, "/revoca 777");
    server.messaggio(s, 900, "Accesso revocato all'utente 777");
    let s = server.segno();
    server.testo(777, "ciao");
    server.messaggio(s, 777, "Questo bot è privato");
    // Il codice d'invito non riammette un utente revocato.
    let s = server.segno();
    server.testo(777, "/start apriti");
    server.messaggio(s, 777, "Questo bot è privato");
    let utenti: Value =
        serde_json::from_str(&fs::read_to_string(tg.join("utenti.json")).unwrap()).unwrap();
    assert_eq!(utenti["revocati"], json!([777]));
    // Troppi codici sbagliati: bloccato, anche con il codice giusto.
    for _ in 0..5 {
        let s = server.segno();
        server.testo(779, "/start indovino");
        server.messaggio(s, 779, "Questo bot è privato");
    }
    let s = server.segno();
    server.testo(779, "/start apriti");
    server.messaggio(s, 779, "Questo bot è privato");
    let s = server.segno();
    server.testo(900, "/ammetti 778");
    server.messaggio(s, 900, "Utente 778 ammesso");
    let s = server.segno();
    server.testo(778, "/start");
    server.messaggio(s, 778, "AVVENTURE DISPONIBILI");
    let s = server.segno();
    server.testo(100, "/utenti");
    // Per chi non è amministratore /utenti non esiste: va alla sessione (menu).
    server.messaggio(s, 100, "AVVENTURE DISPONIBILI");
    let s = server.segno();
    server.testo(100, "/state");
    server.messaggio(s, 100, "Comando riservato agli amministratori");

    // /state di un amministratore sulla propria sessione.
    entra_in_mini(&server, 900);
    server.messaggio(0, 900, "<i>[Turno 1");
    let s = server.segno();
    server.testo(900, "/state");
    server.messaggio(s, 900, "&quot;versione_formato&quot;");
    bot.ferma();
}

/// 15. Il token non compare in log, errori, `Debug` o richieste.
#[test]
fn token_mai_visibile() {
    log_globale();
    let porta_chiusa = {
        let l = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        l.local_addr().unwrap().port()
    };
    let api = Api::con_base_url(TOKEN, &format!("http://127.0.0.1:{porta_chiusa}")).unwrap();
    let errore = api.get_me().unwrap_err();
    let testo = format!("{errore} {errore:?} {api:?}");
    assert!(!testo.contains("SEGRETISSIMO"), "{testo}");
    assert!(testo.contains("getMe"), "{testo}");

    let server = ServerFinto::avvia();
    server.errore(
        "getMe",
        401,
        json!({ "ok": false, "error_code": 401,
                "description": format!("Unauthorized bot{TOKEN}") }),
    );
    let api = Api::con_base_url(TOKEN, &server.base_url()).unwrap();
    let errore = bot::prepara(&api).unwrap_err();
    let testo = format!("{errore:#} {errore:?}");
    assert!(!testo.contains("SEGRETISSIMO"), "{testo}");
    assert!(testo.contains("bot***"), "{testo}");

    // Una partita completa con log a livello debug.
    let dir = comune::copia_avventure();
    let bot = avvia_standard(dir.path(), &server);
    entra_in_mini(&server, 100);
    server.messaggio(0, 100, "<i>[Turno 1");
    bot.ferma();
    token_assente_dappertutto(&server);
}

/// Un panic in una sessione non ferma il bot: messaggio d'errore e sessione ricreata.
#[test]
fn errore_in_una_sessione() {
    let dir = comune::copia_avventure();
    let server = ServerFinto::avvia();
    let bot = avvia_bot(
        dir.path(),
        &server,
        config_bot(),
        Arc::new(NarratoreDifettoso(AtomicBool::new(false))),
        Duration::from_secs(30),
    );
    entra_in_mini(&server, 100);
    server.messaggio(0, 100, "Si è verificato un errore");
    // Un altro utente non ne risente; lo stesso utente riparte.
    let s = server.segno();
    server.testo(200, "/start");
    server.messaggio(s, 200, "AVVENTURE DISPONIBILI");
    let s = server.segno();
    server.testo(100, "/start");
    let menu = server.messaggio(s, 100, "AVVENTURE DISPONIBILI");
    server.premi_pulsante(100, &menu, "Mini avventura");
    let partita = server.messaggio(s, 100, "MINI AVVENTURA");
    assert!(partita.quando > menu.quando);
    bot.ferma();
}

/// §7.6: con `interfaccia.lingua = "auto"` il primo messaggio chiede la lingua
/// proponendo quella dell'app; la scelta resta salvata. I comandi del bot
/// sono registrati in inglese e, per l'app in italiano, in italiano.
#[test]
fn lingua_dall_app_telegram() {
    let dir = comune::copia_avventure();
    let server = ServerFinto::avvia();
    let mut config = config_bot();
    config.interfaccia.lingua = "auto".into();
    let bot = avvia_bot(
        dir.path(),
        &server,
        config,
        Arc::new(NarratoreFinto),
        Duration::from_secs(30),
    );
    // Chi non è ammesso riceve il messaggio nella lingua dell'app.
    server.testo_con_lingua(555, "hi", "en-GB");
    server.messaggio(0, 555, "This bot is private. Your Telegram ID is 555");

    let s = server.segno();
    server.testo_con_lingua(100, "/start", "en");
    let domanda = server.messaggio(s, 100, "LANGUAGE / LINGUA");
    let etichette: Vec<String> = domanda.pulsanti().into_iter().map(|p| p.0).collect();
    assert_eq!(etichette[0], "English", "{etichette:?}");
    assert!(
        etichette.iter().any(|e| e.contains("Cancel")),
        "{etichette:?}"
    );
    server.premi_pulsante(100, &domanda, "English");
    server.messaggio(s, 100, "Welcome!");
    let menu = server.messaggio(s, 100, "AVAILABLE ADVENTURES");
    assert!(
        menu.pulsanti()
            .iter()
            .any(|p| p.0.contains("Language: English")),
        "{:?}",
        menu.pulsanti()
    );
    let pref =
        std::fs::read_to_string(cartella_tg(dir.path()).join("100/preferenze.json")).unwrap();
    assert!(pref.contains("\"en\""), "{pref}");

    // Comandi registrati: inglese per tutti, poi italiano, francese, spagnolo e
    // tedesco per l'app in quelle lingue.
    let comandi: Vec<Richiesta> = server
        .richieste()
        .into_iter()
        .filter(|r| r.metodo == "setMyCommands")
        .collect();
    assert_eq!(comandi.len(), 5, "{comandi:?}");
    assert!(comandi[0].corpo.get("language_code").is_none());
    assert!(comandi[0].corpo.to_string().contains("Change language"));
    assert_eq!(comandi[1].corpo["language_code"], "it");
    assert!(comandi[1].corpo.to_string().contains("Scegli un'avventura"));
    assert_eq!(comandi[2].corpo["language_code"], "fr");
    assert_eq!(comandi[3].corpo["language_code"], "es");
    assert_eq!(comandi[4].corpo["language_code"], "de");
    assert!(
        comandi[4]
            .corpo
            .to_string()
            .contains("Ein Abenteuer wählen")
    );
    bot.ferma();
}
