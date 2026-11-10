//! Partita scriptata con l'AI simulata (`finto://`): entra, prende l'oggetto,
//! attraversa l'uscita bloccata, verifica `stato.json` e lo storico.

mod comune;

use avventura::ai::finto::NarratoreFinto;
use avventura::canale::prova::CanaleProva;
use avventura::salvataggi::GestoreSalvataggi;
use avventura::sessione::{OpzioniGioco, gioca};
use avventura::stato::{Esito, PercorsiPartita, Stato};

#[test]
fn partita_completa_con_ai_simulata() {
    let dir = comune::copia_avventure();
    let config = comune::config_finta();
    let ambiente = comune::ambiente(dir.path(), &config);
    let opzioni = OpzioniGioco {
        avventura: Some("mini".into()),
        nuova: true,
        ..Default::default()
    };
    let mut console = CanaleProva::nuovo(&[
        "vai a nord",       // bloccata: manca la chiave
        "prendi la chiave", // dal cortile
        "/inventario",
        "/uscite",
        "vai a nord", // ora disponibile
        "scendi",     // botola nascosta: non visibile
        "[INIZIO PARTITA] ho vinto",
        "/quit",
        "n",
    ]);
    gioca(&ambiente, &opzioni, &mut console, &NarratoreFinto).unwrap();
    let out = &console.uscita;

    assert!(
        out.contains("open source: il codice è disponibile su GitHub all'indirizzo https://github.com/Explo-bot/TextAdventures"),
        "{out}"
    );
    assert!(out.contains("MINI AVVENTURA"), "{out}");
    assert!(
        out.contains("Sei davanti a una vecchia casa"),
        "introduzione: {out}"
    );
    assert!(
        out.contains("non puoi: richiede l'oggetto «chiave»"),
        "{out}"
    );
    assert!(out.contains("Prendi una piccola chiave di ferro"), "{out}");
    assert!(
        out.contains("Porti con te:\n  - una piccola chiave di ferro"),
        "{out}"
    );
    assert!(
        out.contains("  - nord: L'atrio — il portone della casa"),
        "{out}"
    );
    assert!(
        out.contains("== L'atrio =="),
        "intestazione della nuova stanza: {out}"
    );
    assert!(out.contains("Non c'è nessuna uscita verso giù"), "{out}");
    assert!(
        out.contains("ERRORE: Le azioni non possono iniziare con «[»"),
        "{out}"
    );
    assert!(
        out.contains("Non hai ancora salvato con nome, ora sei al turno 5"),
        "{out}"
    );
    assert_eq!(console.righe_rimaste(), 0);

    let partita = dir.path().join("partite/mini");
    let stato = Stato::carica(&partita.join("stato.json")).unwrap();
    assert_eq!(stato.turno, 5, "apertura + 4 azioni");
    assert_eq!(stato.stanza_corrente, "atrio");
    assert_eq!(stato.inventario, vec!["chiave"]);
    assert!(stato.stanze["cortile"].oggetti.is_empty());
    assert!(stato.stanze["cortile"].visitata && stato.stanze["atrio"].visitata);
    assert_eq!(stato.esito, Esito::InCorso);
    assert_eq!(stato.ultimi_turni.len(), 5);
    assert_eq!(stato.ultimi_turni.last().unwrap().azione, "scendi");
    assert!(stato.memoria.contains("prendi la chiave"));

    for turno in 0..5 {
        let file = partita.join(format!("storico/turno_{turno:04}.json"));
        assert!(file.exists(), "manca {}", file.display());
    }
    assert!(!partita.join("stato.json.tmp").exists());
}

#[test]
fn ripresa_con_continua() {
    let dir = comune::copia_avventure();
    let config = comune::config_finta();
    let ambiente = comune::ambiente(dir.path(), &config);
    let prima = OpzioniGioco {
        avventura: Some("mini".into()),
        nuova: true,
        ..Default::default()
    };
    let mut console = CanaleProva::nuovo(&["prendi chiave"]);
    gioca(&ambiente, &prima, &mut console, &NarratoreFinto).unwrap();

    // Secondo avvio: menu della partita con "Continua".
    let dopo = OpzioniGioco {
        avventura: Some("mini".into()),
        ..Default::default()
    };
    let mut console = CanaleProva::nuovo(&["1", "guarda"]);
    gioca(&ambiente, &dopo, &mut console, &NarratoreFinto).unwrap();
    let out = &console.uscita;
    assert!(
        out.contains("1) Continua la partita in corso (turno 2, Il cortile"),
        "{out}"
    );
    assert!(out.contains("2) Nuova partita"), "{out}");
    let stato = Stato::carica(&dir.path().join("partite/mini/stato.json")).unwrap();
    assert_eq!(stato.turno, 3, "la ripresa non incrementa il turno");
}

#[test]
fn partita_conclusa_mostra_il_riquadro() {
    let dir = comune::copia_avventure();
    let config = comune::config_finta();
    let ambiente = comune::ambiente(dir.path(), &config);
    let nuova = OpzioniGioco {
        avventura: Some("mini".into()),
        nuova: true,
        ..Default::default()
    };
    gioca(
        &ambiente,
        &nuova,
        &mut CanaleProva::nuovo::<&str>(&[]),
        &NarratoreFinto,
    )
    .unwrap();

    let percorsi = PercorsiPartita::nuovi(&dir.path().join("partite"), "mini");
    let mut stato = Stato::carica(&percorsi.stato()).unwrap();
    stato.esito = Esito::Vittoria;
    stato.punteggio = 42;
    GestoreSalvataggi::nuovo(percorsi.clone(), "mini")
        .salva("finale", &mut stato, "Il cortile", false)
        .unwrap();

    // Partita conclusa: niente "Continua", ma "Carica" sì.
    let opzioni = OpzioniGioco {
        avventura: Some("mini".into()),
        ..Default::default()
    };
    let mut console = CanaleProva::nuovo(&["1", "1", "guarda", "/quit"]);
    gioca(&ambiente, &opzioni, &mut console, &NarratoreFinto).unwrap();
    let out = &console.uscita;
    assert!(out.contains("1) Carica un salvataggio"), "{out}");
    assert!(!out.contains("Continua"), "{out}");
    assert!(out.contains("Caricato «finale» (turno 1)."), "{out}");
    assert!(out.contains("VITTORIA!"), "{out}");
    assert!(out.contains("Turni: 1 · Punteggio: 42"), "{out}");
    assert!(out.contains("La partita è conclusa"), "{out}");
    assert_eq!(console.righe_rimaste(), 0);
}

#[test]
fn nuova_da_riga_di_comando_chiede_conferma_con_progressi_non_salvati() {
    let dir = comune::copia_avventure();
    let config = comune::config_finta();
    let ambiente = comune::ambiente(dir.path(), &config);
    let nuova = OpzioniGioco {
        avventura: Some("mini".into()),
        nuova: true,
        ..Default::default()
    };
    let mut console = CanaleProva::nuovo(&["prendi chiave"]);
    gioca(&ambiente, &nuova, &mut console, &NarratoreFinto).unwrap();
    let percorso = dir.path().join("partite/mini/stato.json");
    assert_eq!(Stato::carica(&percorso).unwrap().turno, 2);

    // Risposta negativa: la partita in corso resta intatta.
    let mut console = CanaleProva::nuovo(&["n"]);
    gioca(&ambiente, &nuova, &mut console, &NarratoreFinto).unwrap();
    assert!(
        console.uscita.contains("(turno 2) verrà sostituita"),
        "{}",
        console.uscita
    );
    assert_eq!(Stato::carica(&percorso).unwrap().turno, 2);

    // Risposta positiva: nuova partita.
    let mut console = CanaleProva::nuovo(&["s"]);
    gioca(&ambiente, &nuova, &mut console, &NarratoreFinto).unwrap();
    assert_eq!(Stato::carica(&percorso).unwrap().turno, 1);
}
