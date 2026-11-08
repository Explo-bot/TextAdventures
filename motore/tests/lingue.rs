//! Lingue dell'interfaccia (§7.6): domanda al primo utilizzo, preferenza
//! salvata, `/language`, lingua rilevata con un canale non interattivo e scelta
//! della versione di una storia nella lingua dell'interfaccia.

mod comune;

use std::fs;
use std::path::Path;

use avventura::ai::finto::NarratoreFinto;
use avventura::canale::prova::CanaleProva;
use avventura::config::Config;
use avventura::lingua::{self, Lingua};
use avventura::sessione::{OpzioniGioco, gioca};

/// Crea `avventure/mini-en`: copia di `mini` in inglese, versione della storia `mini`.
fn aggiungi_mini_inglese(dir: &Path) {
    let da = dir.join("avventure/mini");
    let a = dir.join("avventure/mini-en");
    fs::create_dir_all(a.join("stanze")).unwrap();
    for voce in fs::read_dir(da.join("stanze")).unwrap() {
        let voce = voce.unwrap();
        fs::copy(voce.path(), a.join("stanze").join(voce.file_name())).unwrap();
    }
    let gioco = fs::read_to_string(da.join("gioco.md"))
        .unwrap()
        .replace("titolo = \"Mini avventura\"", "titolo = \"Mini adventure\"")
        .replace("lingua = \"it\"", "lingua = \"en\"\nstoria = \"mini\"")
        .replace("## Introduzione", "## Introduction");
    fs::write(a.join("gioco.md"), gioco).unwrap();
}

fn config(lingua: &str) -> Config {
    let mut c = comune::config_finta();
    c.interfaccia.lingua = lingua.into();
    c
}

fn esegui(dir: &Path, config: &Config, opzioni: &OpzioniGioco, canale: &mut CanaleProva) {
    let ambiente = comune::ambiente(dir, config);
    gioca(&ambiente, opzioni, canale, &NarratoreFinto).unwrap();
    // Ogni test riparte dalla lingua iniziale del thread.
    lingua::imposta(Lingua::It);
}

#[test]
fn versione_inglese_di_una_storia_nel_menu() {
    let dir = comune::copia_avventure();
    aggiungi_mini_inglese(dir.path());
    let mut c = CanaleProva::nuovo(&["2", "look", "/help"]);
    esegui(dir.path(), &config("en"), &OpzioniGioco::default(), &mut c);
    let out = &c.uscita;
    assert!(out.contains("AVAILABLE ADVENTURES"), "{out}");
    // La lingua in cui è scritta un'avventura non si mostra: il narratore usa
    // comunque quella dell'interfaccia.
    assert!(out.contains("1) Altra avventura "), "{out}");
    assert!(!out.contains("(Italiano)"), "{out}");
    assert!(out.contains("2) Mini adventure"), "{out}");
    assert!(
        !out.contains("Mini avventura"),
        "una sola versione per storia: {out}"
    );
    assert!(out.contains("3) 🌐 Language: English"), "{out}");
    assert!(
        out.contains("== Introduction") || out.contains("MINI ADVENTURE"),
        "{out}"
    );
    assert!(out.contains("Type what you want to do"), "{out}");
    assert!(
        out.contains("Exits:"),
        "narrazione del finto in inglese: {out}"
    );
    assert!(out.contains("COMMANDS"), "{out}");
    assert!(out.contains("Goodbye!"), "{out}");
    assert!(dir.path().join("partite/mini-en/stato.json").is_file());

    // In italiano la stessa storia si gioca nella versione italiana.
    let mut c = CanaleProva::nuovo(&["q"]);
    esegui(dir.path(), &config("it"), &OpzioniGioco::default(), &mut c);
    assert!(c.uscita.contains("2) Mini avventura"), "{}", c.uscita);
    assert!(!c.uscita.contains("Mini adventure"), "{}", c.uscita);
}

#[test]
fn avventura_in_altra_lingua_con_traduzione() {
    let dir = comune::copia_avventure();
    let t = dir.path().join("avventure/mini/traduzioni");
    fs::create_dir_all(&t).unwrap();
    fs::write(
        t.join("en.toml"),
        "titolo = \"Mini adventure\"\n\
         descrizione_breve = \"Three rooms, a key and a trapdoor.\"\n\
         introduzione = \"You stand before an old house.\"\n\
         [stanze]\ncortile = \"The courtyard\"\natrio = \"The hall\"\n\
         [oggetti]\nchiave = \"a small iron key\"\n\
         [uscite]\n\"cortile.nord\" = \"the front door\"\n",
    )
    .unwrap();

    // Menu: titolo e descrizione tradotti.
    let mut c = CanaleProva::nuovo(&["q"]);
    esegui(dir.path(), &config("en"), &OpzioniGioco::default(), &mut c);
    assert!(c.uscita.contains("Mini adventure"), "{}", c.uscita);
    assert!(
        c.uscita.contains("Three rooms, a key and a trapdoor."),
        "{}",
        c.uscita
    );
    assert!(!c.uscita.contains("Mini avventura"), "{}", c.uscita);

    // Partita: introduzione, nomi di stanze, oggetti, direzioni e uscite tradotti.
    let opzioni = OpzioniGioco {
        avventura: Some("mini".into()),
        nuova: true,
        ..Default::default()
    };
    let mut c = CanaleProva::nuovo(&["take the key", "/inventory", "/exits"]);
    esegui(dir.path(), &config("en"), &opzioni, &mut c);
    let out = &c.uscita;
    assert!(out.contains("MINI ADVENTURE"), "{out}");
    assert!(out.contains("You stand before an old house."), "{out}");
    assert!(!out.contains("Sei davanti a una vecchia casa"), "{out}");
    assert!(out.contains("The courtyard"), "{out}");
    assert!(out.contains("Exits: north"), "narrazione del finto: {out}");
    assert!(out.contains("You take a small iron key"), "{out}");
    assert!(out.contains("- a small iron key"), "{out}");
    assert!(out.contains("- north: The hall — the front door"), "{out}");

    // In italiano restano i testi originali.
    let mut c = CanaleProva::nuovo(&["q"]);
    esegui(dir.path(), &config("it"), &OpzioniGioco::default(), &mut c);
    assert!(c.uscita.contains("Mini avventura"), "{}", c.uscita);
}

#[test]
fn partita_in_tedesco_francese_e_spagnolo() {
    let dir = comune::copia_avventure();
    let t = dir.path().join("avventure/mini/traduzioni");
    fs::create_dir_all(&t).unwrap();
    fs::write(
        t.join("de.toml"),
        "titolo = \"Mini-Abenteuer\"\n\
         introduzione = \"Du stehst vor einem alten Haus.\"\n\
         [stanze]\ncortile = \"Der Hof\"\natrio = \"Die Halle\"\n\
         [oggetti]\nchiave = \"ein kleiner Eisenschlüssel\"\n\
         [uscite]\n\"cortile.nord\" = \"die Haustür\"\n",
    )
    .unwrap();
    let opzioni = OpzioniGioco {
        avventura: Some("mini".into()),
        nuova: true,
        ..Default::default()
    };
    let mut c = CanaleProva::nuovo(&["nimm den schlüssel", "/inventar", "/ausgänge", "/hilfe"]);
    esegui(dir.path(), &config("de"), &opzioni, &mut c);
    let out = &c.uscita;
    assert!(out.contains("MINI-ABENTEUER"), "{out}");
    assert!(out.contains("Du stehst vor einem alten Haus."), "{out}");
    assert!(out.contains("Der Hof"), "{out}");
    assert!(
        out.contains("Ausgänge: Norden"),
        "narrazione del finto: {out}"
    );
    assert!(
        out.contains("Du nimmst und steckst zu deinen Sachen: ein kleiner Eisenschlüssel"),
        "{out}"
    );
    assert!(
        out.contains("Du trägst:\n  - ein kleiner Eisenschlüssel"),
        "{out}"
    );
    assert!(out.contains("- Norden: Die Halle — die Haustür"), "{out}");
    assert!(out.contains("BEFEHLE"), "{out}");
    assert!(out.contains("[Zug 1 · Punkte 0]"), "{out}");

    // Senza traduzione francese: testi originali, ma interfaccia e AI simulata
    // in francese.
    let mut c = CanaleProva::nuovo(&["oui", "nord", "/sorties"]);
    esegui(dir.path(), &config("fr"), &opzioni, &mut c);
    let out = &c.uscita;
    assert!(out.contains("MINI AVVENTURA"), "{out}");
    assert!(out.contains("Vous essayez d'aller vers nord"), "{out}");
    assert!(out.contains("Aucune sortie disponible"), "{out}");
    assert!(out.contains("[Tour 2 · Score 0]"), "{out}");

    // Spagnolo: il menu e la conferma all'uscita.
    let mut c = CanaleProva::nuovo(&["q"]);
    esegui(dir.path(), &config("es"), &OpzioniGioco::default(), &mut c);
    assert!(c.uscita.contains("AVENTURAS DISPONIBLES"), "{}", c.uscita);
    assert!(c.uscita.contains("Idioma: Español"), "{}", c.uscita);
}

#[test]
fn partita_di_una_versione_precedente() {
    let dir = comune::copia_avventure();
    let opzioni = OpzioniGioco {
        avventura: Some("mini".into()),
        nuova: true,
        ..Default::default()
    };
    let mut c = CanaleProva::nuovo(&["/save vecchio"]);
    esegui(dir.path(), &config("it"), &opzioni, &mut c);
    // Simula gli id di una versione precedente dell'avventura.
    let partite = dir.path().join("partite/mini");
    for file in [
        partite.join("stato.json"),
        partite.join("salvataggi/vecchio.json"),
    ] {
        let testo = fs::read_to_string(&file).unwrap();
        fs::write(&file, testo.replace("\"cortile\"", "\"scogliera\"")).unwrap();
    }

    // La partita in corso non si può continuare: si propone una nuova partita.
    let solo_menu = OpzioniGioco {
        avventura: Some("mini".into()),
        ..Default::default()
    };
    let mut c = CanaleProva::nuovo(&["q"]);
    esegui(dir.path(), &config("it"), &solo_menu, &mut c);
    assert!(
        c.uscita
            .contains("versione precedente dell'avventura (la stanza «scogliera» non esiste più)"),
        "{}",
        c.uscita
    );
    assert!(!c.uscita.contains("Continua"), "{}", c.uscita);

    // Il vecchio salvataggio non si carica.
    let carica = OpzioniGioco {
        avventura: Some("mini".into()),
        carica: Some("vecchio".into()),
        ..Default::default()
    };
    let config = config("it");
    let ambiente = comune::ambiente(dir.path(), &config);
    let mut c = CanaleProva::nuovo::<&str>(&[]);
    let errore = gioca(&ambiente, &carica, &mut c, &NarratoreFinto).unwrap_err();
    lingua::imposta(Lingua::It);
    assert!(
        format!("{errore:#}").contains("il salvataggio «vecchio» è di una versione precedente"),
        "{errore:#}"
    );
}

#[test]
fn domanda_al_primo_utilizzo_e_preferenza_salvata() {
    let dir = comune::copia_avventure();
    let opzioni = OpzioniGioco {
        lingua_rilevata: Some(Lingua::En),
        ..Default::default()
    };
    let mut c = CanaleProva::nuovo(&["2", "q"]);
    c.interattivo = true;
    esegui(dir.path(), &config("auto"), &opzioni, &mut c);
    let out = &c.uscita;
    assert!(out.contains("LANGUAGE / LINGUA"), "{out}");
    let en = out
        .find("1) English")
        .expect("la lingua rilevata è la prima voce");
    let it = out.find("2) Italiano").expect("seconda voce");
    assert!(en < it);
    assert!(out.contains("AVVENTURE DISPONIBILI"), "{out}");
    let pref = fs::read_to_string(dir.path().join("partite/preferenze.json")).unwrap();
    assert!(pref.contains("\"it\""), "{pref}");

    // Al secondo avvio non si chiede più.
    let mut c = CanaleProva::nuovo(&["q"]);
    c.interattivo = true;
    esegui(dir.path(), &config("auto"), &opzioni, &mut c);
    assert!(!c.uscita.contains("LANGUAGE / LINGUA"), "{}", c.uscita);
    assert!(c.uscita.contains("AVVENTURE DISPONIBILI"), "{}", c.uscita);

    // `--lingua` vale solo per la sessione e vince sulla preferenza.
    let forzata = OpzioniGioco {
        lingua: Some(Lingua::En),
        ..opzioni.clone()
    };
    let mut c = CanaleProva::nuovo(&["q"]);
    esegui(dir.path(), &config("auto"), &forzata, &mut c);
    assert!(c.uscita.contains("AVAILABLE ADVENTURES"), "{}", c.uscita);
    let pref = fs::read_to_string(dir.path().join("partite/preferenze.json")).unwrap();
    assert!(pref.contains("\"it\""), "{pref}");
}

#[test]
fn senza_interazione_vale_la_lingua_rilevata() {
    let dir = comune::copia_avventure();
    let opzioni = OpzioniGioco {
        lingua_rilevata: Some(Lingua::En),
        ..Default::default()
    };
    let mut c = CanaleProva::nuovo(&["q"]);
    esegui(dir.path(), &config("auto"), &opzioni, &mut c);
    assert!(!c.uscita.contains("LANGUAGE / LINGUA"), "{}", c.uscita);
    assert!(c.uscita.contains("AVAILABLE ADVENTURES"), "{}", c.uscita);
    assert!(!dir.path().join("partite/preferenze.json").exists());

    // Nessuna lingua rilevata: inglese.
    let mut c = CanaleProva::nuovo(&["q"]);
    esegui(
        dir.path(),
        &config("auto"),
        &OpzioniGioco::default(),
        &mut c,
    );
    assert!(c.uscita.contains("AVAILABLE ADVENTURES"), "{}", c.uscita);
}

#[test]
fn comando_language_durante_la_partita() {
    let dir = comune::copia_avventure();
    let opzioni = OpzioniGioco {
        avventura: Some("mini".into()),
        nuova: true,
        ..Default::default()
    };
    let mut c = CanaleProva::nuovo(&["/language en", "/inventory", "/lingua", "2", "/aiuto"]);
    esegui(dir.path(), &config("it"), &opzioni, &mut c);
    let out = &c.uscita;
    assert!(out.contains("Language set to English."), "{out}");
    assert!(
        out.contains("The narrator will answer in English from the next turn."),
        "{out}"
    );
    assert!(
        out.contains("Il narratore risponderà in italiano dal prossimo turno."),
        "{out}"
    );
    assert!(out.contains("You are not carrying anything."), "{out}");
    // `/lingua` senza argomento: menu con la lingua attuale per prima.
    assert!(
        out.contains("1) English") && out.contains("2) Italiano"),
        "{out}"
    );
    assert!(out.contains("Lingua impostata: Italiano."), "{out}");
    assert!(out.contains("COMANDI"), "{out}");
    let pref = fs::read_to_string(dir.path().join("partite/preferenze.json")).unwrap();
    assert!(pref.contains("\"it\""), "{pref}");
}
