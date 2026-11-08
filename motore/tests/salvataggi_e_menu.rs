//! Salvataggi con nome, `/undo`, menu di avvio e isolamento tra avventure,
//! con input simulato da una sequenza di righe e l'AI simulata.

mod comune;

use avventura::ai::finto::NarratoreFinto;
use avventura::canale::prova::CanaleProva;
use avventura::salvataggi::GestoreSalvataggi;
use avventura::sessione::{OpzioniGioco, gioca};
use avventura::stato::{PercorsiPartita, Stato};

fn opzioni(id: &str, nuova: bool) -> OpzioniGioco {
    OpzioniGioco {
        avventura: Some(id.into()),
        nuova,
        ..Default::default()
    }
}

fn gestore(dir: &std::path::Path, id: &str) -> GestoreSalvataggi {
    GestoreSalvataggi::nuovo(PercorsiPartita::nuovi(&dir.join("partite"), id), id)
}

fn stato(dir: &std::path::Path, id: &str) -> Stato {
    Stato::carica(&dir.join(format!("partite/{id}/stato.json"))).unwrap()
}

/// Esegue una sessione e restituisce l'output.
fn sessione(dir: &std::path::Path, opz: &OpzioniGioco, righe: &[&str]) -> String {
    let config = comune::config_finta();
    let ambiente = comune::ambiente(dir, &config);
    let mut console = CanaleProva::nuovo(righe);
    gioca(&ambiente, opz, &mut console, &NarratoreFinto).unwrap();
    assert_eq!(
        console.righe_rimaste(),
        0,
        "righe non consumate:\n{}",
        console.uscita
    );
    console.uscita
}

#[test]
fn menu_di_save_nuovo_corrente_esistente_e_nome_non_valido() {
    let dir = comune::copia_avventure();
    let d = dir.path();
    let out = sessione(
        d,
        &opzioni("mini", true),
        &[
            // Nuovo salvataggio con nome predefinito (Invio).
            "/save",
            "1",
            "",
            "guarda",
            // Ora c'è il corrente: 1) corrente 2) esistente 3) nuovo.
            "/save",
            "1",
            // Nuovo con nome non valido, poi valido.
            "/save",
            "3",
            "con:due punti",
            "CON",
            "secondo salvo",
            "guarda",
            // Sovrascrivi un esistente scelto per numero, rifiutando e poi confermando.
            "/save",
            "2",
            "2",
            "n",
            "2",
            "2",
            "s",
            "/quit",
        ],
    );
    assert!(out.contains("SALVA PARTITA (turno 1, Il cortile)"), "{out}");
    assert!(
        out.contains("Nome del salvataggio [cortile-turno-1]: "),
        "{out}"
    );
    assert!(
        out.contains("Partita salvata in «cortile-turno-1» (turno 1)."),
        "{out}"
    );
    assert!(
        out.contains("1) Salva in «cortile-turno-1» (salvataggio corrente)"),
        "{out}"
    );
    assert!(
        out.contains("Partita salvata in «cortile-turno-1» (turno 2)."),
        "{out}"
    );
    assert!(
        out.contains("Nome non valido: il carattere «:» non è ammesso"),
        "{out}"
    );
    assert!(
        out.contains("Nome non valido: «CON» è un nome riservato di Windows"),
        "{out}"
    );
    assert!(
        out.contains("Partita salvata in «secondo salvo» (turno 2)."),
        "{out}"
    );
    assert!(
        out.contains("Sovrascrivere «cortile-turno-1» (turno 2)? (s/n)"),
        "{out}"
    );
    assert!(
        out.contains("Partita salvata in «cortile-turno-1» (turno 3)."),
        "{out}"
    );

    let g = gestore(d, "mini");
    let elenco = g.elenca();
    assert_eq!(elenco.len(), 2);
    assert_eq!(
        elenco[0].nome, "cortile-turno-1",
        "il più recente per primo"
    );
    assert!(
        g.percorsi()
            .salvataggi()
            .join("secondo-salvo.json")
            .exists()
    );
    let s = stato(d, "mini");
    assert_eq!(s.salvataggio_corrente.as_deref(), Some("cortile-turno-1"));
    assert_eq!(s.turno_ultimo_salvataggio, 3);
}

#[test]
fn save_con_nome_e_conferma_di_sovrascrittura() {
    let dir = comune::copia_avventure();
    let d = dir.path();
    let out = sessione(
        d,
        &opzioni("mini", true),
        &[
            "/salva Primo",
            "guarda",
            "/salva primo", // è il corrente: nessuna conferma
            "/save altro",
            "/save PRIMO", // esiste e non è il corrente: conferma
            "n",
            "/save a/b",
            "/quit",
        ],
    );
    assert!(
        out.contains("Partita salvata in «Primo» (turno 1)."),
        "{out}"
    );
    assert!(
        out.contains("Partita salvata in «Primo» (turno 2)."),
        "{out}"
    );
    assert!(
        out.contains("Sovrascrivere «Primo» (turno 2)? (s/n)"),
        "{out}"
    );
    assert!(out.contains("Salvataggio annullato."), "{out}");
    assert!(
        out.contains("Nome non valido: il carattere «/» non è ammesso"),
        "{out}"
    );
    assert_eq!(gestore(d, "mini").elenca().len(), 2);
}

#[test]
fn load_per_numero_e_per_nome_con_conferma_dei_progressi() {
    let dir = comune::copia_avventure();
    let d = dir.path();
    let out = sessione(
        d,
        &opzioni("mini", true),
        &[
            "/save inizio",
            "prendi la chiave",
            "vai a nord",
            "/save nell'atrio", // apostrofo non ammesso
            "/save atrio",
            "guarda",
            "guarda",
            // Lista: 1 atrio, 2 inizio. Due turni non salvati.
            "/load",
            "2",
            "n",
            "/load",
            "inizio",
            "s",
            "/inventario",
            "/carica ATRIO",
            "/inventario",
            "/load nessuno",
            "q",
            "/quit",
        ],
    );
    assert!(
        out.contains("SALVATAGGI DISPONIBILI — Mini avventura"),
        "{out}"
    );
    assert!(out.contains("* = salvataggio corrente."), "{out}");
    assert!(
        out.contains("Hai 2 turni non salvati che andranno persi. Caricare comunque? (s/n)"),
        "{out}"
    );
    assert!(out.contains("Caricamento annullato."), "{out}");
    assert!(out.contains("Caricato «inizio» (turno 1)."), "{out}");
    assert!(out.contains("Non porti niente con te."), "{out}");
    assert!(out.contains("Caricato «atrio» (turno 3)."), "{out}");
    assert!(
        out.contains("Porti con te:\n  - una piccola chiave di ferro"),
        "{out}"
    );
    assert!(
        out.contains("Non esiste un salvataggio «nessuno»."),
        "{out}"
    );
    let s = stato(d, "mini");
    assert_eq!(s.turno, 3);
    assert_eq!(s.stanza_corrente, "atrio");
    assert_eq!(s.salvataggio_corrente.as_deref(), Some("atrio"));
}

#[test]
fn elenco_e_eliminazione_con_file_danneggiato() {
    let dir = comune::copia_avventure();
    let d = dir.path();
    sessione(d, &opzioni("mini", true), &["/save buono", "/quit"]);
    let g = gestore(d, "mini");
    std::fs::write(g.percorsi().salvataggi().join("vecchio.json"), "{rotto").unwrap();
    let out = sessione(
        d,
        &opzioni("mini", false),
        &[
            "1", // Continua
            "/saves",
            "/load vecchio",
            "/delete",
            "/delete buono",
            "n",
            "/elimina buono",
            "s",
            "/salvataggi",
            "/quit",
        ],
    );
    assert!(
        out.lines()
            .any(|r| r.contains("vecchio") && r.contains("(danneggiato)")),
        "{out}"
    );
    assert!(
        out.contains("è danneggiato e non può essere caricato"),
        "{out}"
    );
    assert!(out.contains("Uso: /delete NOME"), "{out}");
    assert!(out.contains("Eliminazione annullata."), "{out}");
    assert!(out.contains("Salvataggio «buono» eliminato."), "{out}");
    assert_eq!(g.elenca().len(), 1);
}

#[test]
fn undo_torna_indietro_anche_dopo_un_load() {
    let dir = comune::copia_avventure();
    let d = dir.path();
    let out = sessione(
        d,
        &opzioni("mini", true),
        &[
            "prendi la chiave", // turno 2
            "/save presa",
            "vai a nord", // turno 3, atrio
            "/undo",      // torna al turno 2, cortile
            "/inventario",
            "vai a nord",
            "guarda", // turno 4
            "/load presa",
            "s",        // torna al turno 2
            "/annulla", // annulla il load: turno 4
            "/annulla",
            "/annulla",
            "/annulla",
            "/annulla",
            "/undo",
            "/quit",
        ],
    );
    assert!(
        out.contains("Annullato: sei tornato al turno 2 (Il cortile)."),
        "{out}"
    );
    assert!(
        out.contains("Annullato: sei tornato al turno 4 (L'atrio)."),
        "{out}"
    );
    assert!(
        out.contains("Annullato: sei tornato al turno 3 (L'atrio)."),
        "{out}"
    );
    assert!(
        out.contains("Annullato: sei tornato al turno 0 (Il cortile)."),
        "{out}"
    );
    assert!(out.contains("Non c'è niente da annullare."), "{out}");
    assert_eq!(stato(d, "mini").turno, 0);
}

#[test]
fn conferme_su_restart_menu_e_quit() {
    let dir = comune::copia_avventure();
    let d = dir.path();
    let out = sessione(
        d,
        &opzioni("mini", true),
        &[
            "guarda", "/restart", "n", "/menu", "n", //
            "/quit", "q", // annulla: si continua
            "/quit", "1", // sì, salva
            "1", // unica voce del menu di /save: nuovo salvataggio
            "fine",
        ],
    );
    assert!(
        out.contains("Hai 2 turni non salvati con nome. Ricominciare comunque? (s/n)"),
        "{out}"
    );
    assert!(out.contains("Tornare al menu? (s/n)"), "{out}");
    assert!(
        out.contains("Non hai ancora salvato con nome, ora sei al turno 2: vuoi salvare?"),
        "{out}"
    );
    assert!(out.contains("1) Sì, salva"), "{out}");
    assert!(out.contains("2) No, esci senza salvare"), "{out}");
    assert!(out.contains("Ok, si continua."), "{out}");
    assert!(
        out.contains("Partita salvata in «fine» (turno 2)."),
        "{out}"
    );
    assert!(out.contains("Arrivederci!"), "{out}");

    let out = sessione(d, &opzioni("mini", false), &["1", "guarda", "/quit", "2"]);
    assert!(
        out.contains("Ultimo /save al turno 2, ora sei al turno 3: vuoi salvare?"),
        "{out}"
    );
}

/// Nel promemoria di `/quit` si può rispondere anche con `s` / `n` (§7.2: «s/n/annulla»).
#[test]
fn quit_accetta_s_e_n_oltre_ai_numeri() {
    let dir = comune::copia_avventure();
    let d = dir.path();
    // `n` = esci senza salvare: non resta alcun salvataggio e il programma termina.
    let out = sessione(d, &opzioni("mini", true), &["guarda", "/quit", "n"]);
    assert!(out.contains("vuoi salvare?"), "{out}");
    assert!(!out.contains("Scelta non valida"), "{out}");
    assert!(!out.contains("Ok, si continua."), "{out}");
    assert!(out.contains("Arrivederci!"), "{out}");
    // `s` apre il menu di /save (partita nuova in una cartella pulita).
    let dir = comune::copia_avventure();
    let d = dir.path();
    let out = sessione(
        d,
        &opzioni("mini", true),
        &["guarda", "/quit", "s", "1", "fine"],
    );
    assert!(!out.contains("Scelta non valida"), "{out}");
    assert!(out.contains("Partita salvata in «fine»"), "{out}");
}

#[test]
fn comandi_vari() {
    let dir = comune::copia_avventure();
    let out = sessione(
        dir.path(),
        &opzioni("mini", true),
        &[
            "/help",
            "/volo",
            "/stato",
            &"x".repeat(501),
            "/uscite",
            "/quit",
        ],
    );
    assert!(out.contains("/inventory, /inventario"), "{out}");
    assert!(out.contains("Comando sconosciuto, scrivi /help"), "{out}");
    assert!(out.contains("solo avviando il gioco con --debug"), "{out}");
    assert!(out.contains("Azione troppo lunga (501 caratteri"), "{out}");
    assert!(
        out.contains("Nessuna uscita disponibile al momento."),
        "{out}"
    );

    let config = comune::config_finta();
    let ambiente = comune::ambiente(dir.path(), &config);
    let mut console = CanaleProva::nuovo(&["1", "/stato", "/grezzo", "/quit"]);
    let debug = OpzioniGioco {
        debug: true,
        ..opzioni("mini", false)
    };
    gioca(&ambiente, &debug, &mut console, &NarratoreFinto).unwrap();
    assert!(
        console.uscita.contains("\"versione_formato\": 1"),
        "{}",
        console.uscita
    );
    assert!(
        console.uscita.contains("\"narrazione\""),
        "{}",
        console.uscita
    );
}

#[test]
fn menu_di_avvio_con_avventura_non_valida() {
    let dir = comune::copia_avventure();
    let d = dir.path();
    // Prima partita su mini2 con un salvataggio.
    sessione(d, &opzioni("mini2", true), &["/save uno", "/quit"]);

    let tutte = OpzioniGioco::default();
    let out = sessione(
        d,
        &tutte,
        &[
            "x", // non valido
            "5", // fuori intervallo (rotta non è selezionabile; 3 = lingua)
            "1", // "Altra avventura" (ordine alfabetico per titolo)
            "9", // scelta non valida
            "4", // Torna all'elenco
            "2", // Mini avventura: nessuna partita né salvataggi → nuova partita
            "/menu", "q",
        ],
    );
    assert!(out.contains("AVVENTURE DISPONIBILI"), "{out}");
    let altra = out
        .find("1) Altra avventura")
        .expect("Altra avventura per prima");
    let mini = out
        .find("2) Mini avventura")
        .expect("Mini avventura seconda");
    assert!(altra < mini, "{out}");
    assert!(
        out.contains("[partita in corso: turno 1 · 1 salvataggio]"),
        "{out}"
    );
    assert!(out.contains("[nuova]"), "{out}");
    assert!(out.contains("Due stanze soltanto."), "{out}");
    assert!(out.contains("rotta: non valida —"), "{out}");
    assert!(!out.contains("3) Avventura rotta"), "{out}");
    assert!(out.contains("3) 🌐 Lingua: Italiano"), "{out}");
    assert!(
        out.contains("Scelta non valida: scrivi un numero da 1 a 3 oppure q."),
        "{out}"
    );
    assert!(
        out.contains("ALTRA AVVENTURA\n\n  1) Continua la partita in corso (turno 1, La sala"),
        "{out}"
    );
    assert!(out.contains("2) Carica un salvataggio"), "{out}");
    assert!(out.contains("3) Nuova partita"), "{out}");
    assert!(out.contains("4) Torna all'elenco delle avventure"), "{out}");
    assert!(
        out.contains("Scelta non valida: scrivi un numero da 1 a 4 oppure q."),
        "{out}"
    );
    assert!(out.contains("MINI AVVENTURA"), "{out}");
}

#[test]
fn menu_di_avvio_carica_e_nuova_con_conferma() {
    let dir = comune::copia_avventure();
    let d = dir.path();
    sessione(
        d,
        &opzioni("mini", true),
        &["/save base", "guarda", "guarda", "/quit", "n"],
    );
    let out = sessione(
        d,
        &opzioni("mini", false),
        &[
            "3", // Nuova partita: 2 turni non salvati → conferma
            "n", "2", // Carica un salvataggio
            "base", "n", // non perdere i progressi
            "2", "1", "s", "/quit",
        ],
    );
    assert!(out.contains("La partita in corso (turno 3) verrà sostituita. I salvataggi con nome restano. Continuare? (s/n)"), "{out}");
    assert!(
        out.contains("ha 2 turni non salvati che andranno persi. Caricare comunque? (s/n)"),
        "{out}"
    );
    assert!(out.contains("Caricato «base» (turno 1)."), "{out}");
    assert_eq!(stato(d, "mini").turno, 1);

    let out = sessione(d, &opzioni("mini", false), &["3", "/quit"]);
    assert!(
        out.contains("MINI AVVENTURA"),
        "nuova partita senza conferma: {out}"
    );
    assert_eq!(stato(d, "mini").turno, 1, "solo il turno di apertura");
}

#[test]
fn isolamento_tra_avventure() {
    let dir = comune::copia_avventure();
    let d = dir.path();
    sessione(
        d,
        &opzioni("mini", true),
        &["prendi la chiave", "/save comune", "/quit"],
    );
    sessione(
        d,
        &opzioni("mini2", true),
        &["prendi libro", "vai a est", "/save comune", "/quit"],
    );

    let mini = stato(d, "mini");
    let mini2 = stato(d, "mini2");
    assert_eq!(mini.avventura, "mini");
    assert_eq!(mini2.avventura, "mini2");
    assert_eq!(mini.stanza_corrente, "cortile");
    assert_eq!(mini2.stanza_corrente, "studio");
    assert_eq!(mini.inventario, vec!["chiave"]);
    assert_eq!(mini2.inventario, vec!["libro"]);

    let g1 = gestore(d, "mini");
    let g2 = gestore(d, "mini2");
    assert_eq!(g1.elenca().len(), 1);
    assert_eq!(g2.elenca().len(), 1);
    assert_eq!(g1.carica("comune").unwrap().stanza_corrente, "cortile");
    assert_eq!(g2.carica("comune").unwrap().stanza_corrente, "studio");

    // Un salvataggio di mini copiato tra quelli di mini2 viene rifiutato.
    std::fs::copy(
        g1.percorsi().salvataggi().join("comune.json"),
        g2.percorsi().salvataggi().join("intruso.json"),
    )
    .unwrap();
    let errore = g2
        .carica("comune")
        .map(|_| ())
        .and_then(|()| g2.carica("intruso").map(|_| ()));
    assert!(
        errore
            .unwrap_err()
            .to_string()
            .contains("appartiene all'avventura «mini»")
    );
    // /undo di mini2 non tocca mini.
    avventura::stato::annulla(g2.percorsi()).unwrap();
    assert_eq!(stato(d, "mini"), mini);
}
