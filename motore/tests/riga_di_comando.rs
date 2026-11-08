//! Test del binario `avventura`: `valida`, `elenca`, `salvataggi`, `nuova`,
//! `profili`, `prova-ai` (anche `--solo-config`), `telegram`, partita da file
//! (stdin non terminale) con l'AI simulata e con `--profilo prova`, chiave
//! mancante per i fornitori cloud e avvio senza chiave verso i server locali.

mod comune;

use std::io::Write;
use std::path::Path;
use std::process::{Command, Output, Stdio};

/// Prepara il comando senza variabili d'ambiente che potrebbero influenzarlo.
fn avventura(cartella_lavoro: &Path) -> Command {
    let mut c = Command::new(env!("CARGO_BIN_EXE_avventura"));
    c.current_dir(cartella_lavoro);
    for v in [
        "AVVENTURA_BASE_URL",
        "AVVENTURA_MODELLO",
        "AVVENTURA_API_KEY",
        "AVVENTURA_PROTOCOLLO",
        "AVVENTURA_PROFILO",
        "AVVENTURA_FORNITORE",
        "AVVENTURA_MODELLO_GGUF",
        "ANTHROPIC_API_KEY",
        "GEMINI_API_KEY",
        "AVVENTURA_CONTESTO",
        "AVVENTURA_TIMEOUT",
        "OPENAI_API_KEY",
    ] {
        c.env_remove(v);
    }
    c.env("NO_COLOR", "1");
    // I test controllano i testi italiani (§7.6).
    c.env("AVVENTURA_LINGUA", "it");
    c
}

fn esegui_con_input(mut comando: Command, input: &str) -> Output {
    let mut figlio = comando
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    figlio
        .stdin
        .take()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    figlio.wait_with_output().unwrap()
}

fn testo(o: &[u8]) -> String {
    String::from_utf8_lossy(o).to_string()
}

#[test]
fn valida_mini_ok_e_rotta_con_errori() {
    let dir = comune::copia_avventure();
    let o = avventura(dir.path())
        .args(["valida", "--avventura", "avventure/mini"])
        .output()
        .unwrap();
    let out = testo(&o.stdout);
    assert!(o.status.success(), "{out}{}", testo(&o.stderr));
    assert!(out.contains("0 errori, 0 avvisi."), "{out}");
    assert!(
        out.contains("Contesto consigliato per questa avventura: 8.192 token"),
        "{out}"
    );

    let o = avventura(dir.path())
        .args([
            "valida",
            "--avventura",
            "rotta",
            "--cartella-avventure",
            "avventure",
        ])
        .output()
        .unwrap();
    let out = testo(&o.stdout);
    assert_eq!(o.status.code(), Some(1), "{out}");
    for atteso in [
        "stanze/tre.md (front matter): TOML non valido",
        "stanze/uno.md (id): l'id «due» è diverso dal nome del file («uno»)",
        "stanze/uno.md (## Descrizione): sezione «Descrizione» mancante",
        "gioco.md (stanza_iniziale): la stanza «inesistente» non esiste",
        "stanze/uno.md (uscite.nord.verso): la stanza di destinazione «nessuna» non esiste",
        "stanze/uno.md (uscite.nord.richiede_oggetto): l'oggetto «spada»",
    ] {
        assert!(out.contains(atteso), "manca «{atteso}» in:\n{out}");
    }

    // Senza --avventura valida tutte: una è rotta → codice 1.
    let o = avventura(dir.path()).arg("valida").output().unwrap();
    assert_eq!(o.status.code(), Some(1));
    let out = testo(&o.stdout);
    assert!(out.contains("mini2") && out.contains("rotta"), "{out}");
}

#[test]
fn elenca_mostra_valide_e_non_valide() {
    let dir = comune::copia_avventure();
    let o = avventura(dir.path())
        .args(["elenca", "--cartella-avventure", "avventure"])
        .output()
        .unwrap();
    let out = testo(&o.stdout);
    assert!(o.status.success(), "{out}");
    let altra = out.find("mini2 — Altra avventura  [nuova]").expect(&out);
    let mini = out.find("mini — Mini avventura  [nuova]").expect(&out);
    let rotta = out.find("rotta — NON VALIDA:").expect(&out);
    assert!(altra < mini && mini < rotta, "{out}");
    assert!(out.contains("3 stanze"), "{out}");
}

#[test]
fn partita_da_file_con_ai_simulata() {
    let dir = comune::copia_avventure();
    let mut c = avventura(dir.path());
    c.args([
        "gioca",
        "--avventura",
        "mini",
        "--nuova",
        "--trascrizione",
        "partita.md",
    ])
    .env("AVVENTURA_BASE_URL", "finto://");
    let o = esegui_con_input(c, "prendi la chiave\nvai a nord\n/save fine\n");
    let out = testo(&o.stdout);
    assert!(o.status.success(), "{out}\n{}", testo(&o.stderr));
    assert!(out.contains("> prendi la chiave"), "eco dell'input: {out}");
    assert!(out.contains("L'atrio"), "{out}");
    assert!(
        out.contains("Partita salvata in «fine» (turno 3)."),
        "{out}"
    );
    assert!(
        out.contains("Arrivederci!"),
        "EOF = /quit senza domande: {out}"
    );
    let stato = std::fs::read_to_string(dir.path().join("partite/mini/stato.json")).unwrap();
    assert!(stato.contains("\"stanza_corrente\": \"atrio\""), "{stato}");
    let trascrizione = std::fs::read_to_string(dir.path().join("partita.md")).unwrap();
    assert!(trascrizione.contains("## L'atrio"), "{trascrizione}");
    assert!(dir.path().join("partite/avventura.log").exists());

    let o = avventura(dir.path())
        .args(["salvataggi", "--avventura", "mini"])
        .output()
        .unwrap();
    assert!(testo(&o.stdout).contains("fine  *"), "{}", testo(&o.stdout));

    let o = avventura(dir.path())
        .args(["nuova", "--avventura", "mini"])
        .output()
        .unwrap();
    assert!(
        testo(&o.stdout).contains("azzerata"),
        "{}",
        testo(&o.stdout)
    );
    assert!(!dir.path().join("partite/mini/stato.json").exists());
    assert!(
        dir.path()
            .join("partite/mini/salvataggi/fine.json")
            .exists()
    );
}

#[test]
fn server_remoto_senza_chiave_errore_chiaro() {
    let dir = comune::copia_avventure();
    let o = avventura(dir.path())
        .args(["gioca", "--avventura", "mini"])
        .env("AVVENTURA_BASE_URL", "https://api.example.com/v1")
        .stdin(Stdio::null())
        .output()
        .unwrap();
    let err = testo(&o.stderr);
    assert_eq!(o.status.code(), Some(1));
    assert!(err.starts_with("Errore: manca la chiave API"), "{err}");
    assert!(err.contains("OPENAI_API_KEY"), "{err}");
    assert!(!err.contains("stack backtrace"), "{err}");
    assert!(
        !dir.path().join("partite").exists(),
        "errore prima del menu"
    );
}

#[test]
fn server_locale_senza_chiave_parte_senza_errori() {
    let dir = comune::copia_avventure();
    let porta = {
        let l = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        l.local_addr().unwrap().port()
    };
    let mut c = avventura(dir.path());
    c.args(["gioca", "--avventura", "mini", "--nuova"])
        .env("AVVENTURA_BASE_URL", format!("http://127.0.0.1:{porta}/v1"));
    let o = esegui_con_input(c, "/quit\n");
    let out = testo(&o.stdout);
    assert!(o.status.success(), "{out}\n{}", testo(&o.stderr));
    assert!(
        out.contains(&format!(
            "il server locale non risponde su 127.0.0.1:{porta}"
        )),
        "{out}"
    );
    assert!(
        out.contains("Un cortile d'erba alta"),
        "descrizione di riserva: {out}"
    );
}

#[test]
fn timeout_non_valido_e_prova_ai_simulata() {
    let dir = comune::copia_avventure();
    let o = avventura(dir.path())
        .args(["gioca", "--timeout", "0"])
        .output()
        .unwrap();
    assert_eq!(o.status.code(), Some(1));
    assert!(
        testo(&o.stderr).contains("timeout_secondi"),
        "{}",
        testo(&o.stderr)
    );

    let o = avventura(dir.path())
        .args(["prova-ai", "--avventura", "mini", "--timeout", "120"])
        .env("AVVENTURA_BASE_URL", "finto://")
        .output()
        .unwrap();
    let out = testo(&o.stdout);
    assert!(o.status.success(), "{out}{}", testo(&o.stderr));
    assert!(out.contains("Server locale:  sì"), "{out}");
    assert!(out.contains("Timeout:        risposta 120 s"), "{out}");
    assert!(out.contains("Risposta JSON valida: sì"), "{out}");
    assert!(
        out.contains("Contesto consigliato: contesto_token = 8192"),
        "{out}"
    );
}

/// Copia il file di configurazione di esempio nella cartella di lavoro.
fn con_config_di_esempio(dir: &Path) {
    let esempio = Path::new(env!("CARGO_MANIFEST_DIR")).join("../config/config.esempio.toml");
    std::fs::create_dir_all(dir.join("config")).unwrap();
    std::fs::copy(esempio, dir.join("config/config.toml")).unwrap();
}

#[test]
fn profili_elencati_e_profilo_prova_senza_rete() {
    let dir = comune::copia_avventure();
    con_config_di_esempio(dir.path());
    let o = avventura(dir.path()).arg("profili").output().unwrap();
    let out = testo(&o.stdout);
    assert!(o.status.success(), "{out}{}", testo(&o.stderr));
    for atteso in [
        "* [ai] (predefinito)",
        "claude",
        "fornitore anthropic · modello claude-sonnet-5-5",
        "ollama-rete",
        "chiave: non richiesta",
        "gguf",
        "prova                fornitore finto",
    ] {
        assert!(out.contains(atteso), "manca «{atteso}» in:\n{out}");
    }
    assert!(!out.contains("NON VALIDO"), "{out}");

    let o = avventura(dir.path())
        .args(["profili", "--profilo", "ollama"])
        .output()
        .unwrap();
    assert!(
        testo(&o.stdout).contains("* ollama "),
        "{}",
        testo(&o.stdout)
    );

    // --profilo cambia fornitore senza modificare il file: partita offline.
    let mut c = avventura(dir.path());
    c.args([
        "gioca",
        "--profilo",
        "prova",
        "--avventura",
        "mini",
        "--nuova",
    ]);
    let o = esegui_con_input(c, "prendi la chiave\n/inventario\n");
    let out = testo(&o.stdout);
    assert!(o.status.success(), "{out}\n{}", testo(&o.stderr));
    assert!(out.contains("una piccola chiave di ferro"), "{out}");

    // Lo stesso con la variabile d'ambiente.
    let mut c = avventura(dir.path());
    c.args(["gioca", "--avventura", "mini", "--nuova"])
        .env("AVVENTURA_PROFILO", "prova");
    let o = esegui_con_input(c, "/quit\n");
    assert!(o.status.success(), "{}", testo(&o.stderr));

    let o = avventura(dir.path())
        .args(["gioca", "--profilo", "inesistente"])
        .output()
        .unwrap();
    assert_eq!(o.status.code(), Some(1));
    assert!(
        testo(&o.stderr).contains("il profilo «inesistente» non esiste"),
        "{}",
        testo(&o.stderr)
    );
}

#[test]
fn fornitori_cloud_senza_chiave_e_locali_senza_errori() {
    let dir = comune::copia_avventure();
    for (fornitore, variabile) in [
        ("anthropic", "ANTHROPIC_API_KEY"),
        ("gemini", "GEMINI_API_KEY"),
        ("openai", "OPENAI_API_KEY"),
    ] {
        let o = avventura(dir.path())
            .args(["gioca", "--avventura", "mini"])
            .env("AVVENTURA_FORNITORE", fornitore)
            .stdin(Stdio::null())
            .output()
            .unwrap();
        let err = testo(&o.stderr);
        assert_eq!(o.status.code(), Some(1), "{fornitore}");
        assert!(err.contains(variabile), "{fornitore}: {err}");
    }
    let porta = {
        let l = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        l.local_addr().unwrap().port()
    };
    for fornitore in ["ollama", "lmstudio"] {
        let mut c = avventura(dir.path());
        c.args(["gioca", "--avventura", "mini", "--nuova"])
            .env("AVVENTURA_FORNITORE", fornitore)
            .env("AVVENTURA_BASE_URL", format!("http://127.0.0.1:{porta}"));
        let o = esegui_con_input(c, "/quit\n");
        let out = testo(&o.stdout);
        assert!(
            o.status.success(),
            "{fornitore}: {out}\n{}",
            testo(&o.stderr)
        );
        assert!(out.contains("il server locale non risponde"), "{out}");
    }
}

#[test]
fn prova_ai_solo_config_telegram_e_locale() {
    let dir = comune::copia_avventure();
    let o = avventura(dir.path())
        .args(["prova-ai", "--solo-config"])
        .env("AVVENTURA_FORNITORE", "finto")
        .output()
        .unwrap();
    assert!(o.status.success(), "{}", testo(&o.stderr));
    assert!(testo(&o.stdout).contains("Fornitore:      finto"));

    let porta = {
        let l = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        l.local_addr().unwrap().port()
    };
    let o = avventura(dir.path())
        .args(["prova-ai", "--solo-config"])
        .env("AVVENTURA_FORNITORE", "ollama")
        .env("AVVENTURA_BASE_URL", format!("http://127.0.0.1:{porta}"))
        .output()
        .unwrap();
    assert_eq!(o.status.code(), Some(1));
    assert!(
        testo(&o.stdout).contains("non è raggiungibile"),
        "{}",
        testo(&o.stdout)
    );

    // Bot Telegram senza token: errore chiaro con le istruzioni (anche con --verifica).
    for argomenti in [&["telegram"][..], &["telegram", "--verifica"][..]] {
        let o = avventura(dir.path())
            .args(argomenti)
            .env_remove("TELEGRAM_BOT_TOKEN")
            .env("AVVENTURA_FORNITORE", "finto")
            .output()
            .unwrap();
        assert_eq!(o.status.code(), Some(1));
        let err = testo(&o.stderr);
        assert!(
            err.contains("TELEGRAM_BOT_TOKEN") && err.contains("@BotFather"),
            "{err}"
        );
        assert!(err.contains("MANUALE"), "{err}");
    }

    let o = avventura(dir.path())
        .args(["gioca", "--avventura", "mini"])
        .env("AVVENTURA_FORNITORE", "locale")
        .env("AVVENTURA_MODELLO_GGUF", "modelli/prova.gguf")
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert_eq!(o.status.code(), Some(1));
    // Senza la feature: lo stub spiega come ottenere il supporto; con la
    // feature: il file indicato non esiste.
    let atteso = if cfg!(feature = "locale") {
        "non esiste"
    } else {
        "--features locale"
    };
    assert!(testo(&o.stderr).contains(atteso), "{}", testo(&o.stderr));
}

#[test]
fn timeout_da_variabile_d_ambiente() {
    let dir = comune::copia_avventure();
    let o = avventura(dir.path())
        .args(["prova-ai", "--solo-config"])
        .env("AVVENTURA_FORNITORE", "finto")
        .env("AVVENTURA_TIMEOUT", "77")
        .output()
        .unwrap();
    assert!(
        testo(&o.stdout).contains("risposta 77 s"),
        "{}",
        testo(&o.stdout)
    );
    let o = avventura(dir.path())
        .args(["prova-ai", "--solo-config", "--timeout", "33"])
        .env("AVVENTURA_FORNITORE", "finto")
        .env("AVVENTURA_TIMEOUT", "77")
        .output()
        .unwrap();
    assert!(
        testo(&o.stdout).contains("risposta 33 s"),
        "{}",
        testo(&o.stdout)
    );
    let o = avventura(dir.path())
        .args(["prova-ai", "--solo-config"])
        .env("AVVENTURA_TIMEOUT", "100000")
        .output()
        .unwrap();
    assert_eq!(o.status.code(), Some(1));
}
