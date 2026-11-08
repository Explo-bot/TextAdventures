//! Menu interattivi: scelta della lingua (§7.6), scelta dell'avventura e della
//! partita all'avvio (§7.1), menu di `/save` e lista di `/load` (§7.3), conferme.
//!
//! Tutto è scritto solo in termini di [`Canale`] (`scegli`, `chiedi_testo`,
//! `conferma`, `mostra`): sul terminale i menu sono elenchi numerati, nel bot
//! Telegram diventano pulsanti. In ogni menu «annulla» (q, Invio dove
//! previsto, fine dell'input, scadenza) torna indietro senza effetti.
//! I testi sono nella lingua dell'interfaccia del thread ([`crate::lingua`]).

use std::collections::HashMap;

use anyhow::Result;

use crate::canale::{Canale, Stile, Voce};
use crate::catalogo_avventure::{InfoPartita, VoceAvventura};
use crate::lingua::{self, Lingua};
use crate::salvataggi::{
    GestoreSalvataggi, InfoSalvataggio, nome_predefinito, progressi_non_salvati, valida_nome,
};
use crate::stato::{Esito, Stato};
use crate::tr;

/// Messaggio quando un'avventura non ha salvataggi.
pub fn nessun_salvataggio() -> &'static str {
    tr!(
        "Nessun salvataggio per questa avventura. Usa /save per crearne uno.",
        "No saved games for this adventure. Use /save to create one.",
        "Aucune sauvegarde pour cette aventure. Utilisez /save pour en créer une.",
        "No hay partidas guardadas de esta aventura. Usa /save para crear una.",
        "Keine Spielstände für dieses Abenteuer. Mit /save legst du einen an."
    )
}

/// Formato breve di data e ora nei menu (giorno/mese per l'italiano, mese/giorno per l'inglese).
fn formato_data() -> &'static str {
    tr!(
        "%d/%m %H:%M",
        "%b %d %H:%M",
        "%d/%m %H:%M",
        "%d/%m %H:%M",
        "%d.%m. %H:%M"
    )
}

/// Risposta a una domanda «salvare prima di uscire?».
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Risposta3 {
    /// Sì.
    Si,
    /// No.
    No,
    /// Annulla.
    Annulla,
}

/// Domanda a tre vie: sì / no / annulla. Oltre al numero si può scrivere `s`
/// (`y` in inglese) o `n` (§7.2).
pub fn domanda_tre_vie(
    canale: &mut dyn Canale,
    domanda: &str,
    si: &str,
    no: &str,
) -> Result<Risposta3> {
    Ok(
        match canale.scegli(
            domanda,
            &[
                Voce::nuova(si).con_chiave(tr!("s", "y", "o", "s", "j")),
                Voce::nuova(no).con_chiave("n"),
            ],
        )? {
            Some(0) => Risposta3::Si,
            Some(_) => Risposta3::No,
            None => Risposta3::Annulla,
        },
    )
}

/// Menu della lingua (§7.6): `proposta` è la prima voce. `None` = annullato.
pub fn scegli_lingua(canale: &mut dyn Canale, proposta: Lingua) -> Result<Option<Lingua>> {
    let mut ordine = vec![proposta];
    ordine.extend(lingua::TUTTE.iter().copied().filter(|l| *l != proposta));
    let voci: Vec<Voce> = ordine
        .iter()
        .map(|l| Voce::nuova(l.nome()).con_chiave(l.codice()))
        .collect();
    Ok(canale
        .scegli("LANGUAGE / LINGUA / LANGUE / IDIOMA / SPRACHE", &voci)?
        .map(|i| ordine[i]))
}

/// Scelta nel menu delle avventure.
#[derive(Debug, Clone, PartialEq)]
pub enum SceltaAvventura {
    /// Indice nell'elenco delle voci.
    Avventura(usize),
    /// Cambio della lingua dell'interfaccia.
    Lingua,
    /// Uscita dal programma.
    Esci,
}

/// Larghezza della colonna del titolo nel menu delle avventure.
const LARGHEZZA_TITOLO: usize = 44;

/// Indici delle voci da mostrare: per ogni storia (campo `storia`) solo la
/// versione nella lingua dell'interfaccia, altrimenti quella inglese, altrimenti
/// la prima; le avventure non valide sono sempre mostrate (§7.6).
pub fn versioni_da_mostrare(voci: &[VoceAvventura], lingua: Lingua) -> Vec<usize> {
    let punteggio = |l: Lingua| {
        if l == lingua {
            0
        } else if l == Lingua::En {
            1
        } else {
            2
        }
    };
    let mut migliore: HashMap<&str, usize> = HashMap::new();
    for (i, v) in voci.iter().enumerate() {
        let Ok(r) = &v.esito else { continue };
        match migliore.get(r.storia.as_str()) {
            Some(&j) => {
                let altra = voci[j]
                    .esito
                    .as_ref()
                    .map(|x| x.lingua)
                    .unwrap_or(Lingua::En);
                if punteggio(r.lingua) < punteggio(altra) {
                    migliore.insert(&r.storia, i);
                }
            }
            None => {
                migliore.insert(&r.storia, i);
            }
        }
    }
    (0..voci.len())
        .filter(|&i| match &voci[i].esito {
            Ok(r) => migliore.get(r.storia.as_str()) == Some(&i),
            Err(_) => true,
        })
        .collect()
}

/// Passo 1 del §7.1: menu numerato delle avventure (le non valide in fondo,
/// non selezionabili, con il motivo) e, per ultima, la voce della lingua.
pub fn scegli_avventura(
    canale: &mut dyn Canale,
    voci: &[VoceAvventura],
    cartella: &std::path::Path,
) -> Result<SceltaAvventura> {
    if voci.is_empty() {
        canale.mostra(
            &tr!(
format!(
                    "Nessuna avventura trovata in «{}». Ogni avventura è una sottocartella con \
                     un file gioco.md: vedi il manuale (docs/MANUALE.md).",
                    cartella.display()
                ),
format!(
                    "No adventures found in \"{}\". Each adventure is a subfolder with a \
                     gioco.md file: see the manual (docs/MANUALE.md).",
                    cartella.display()
                ),
format!("Aucune aventure trouvée dans « {} ». Chaque aventure est un sous-dossier contenant un fichier gioco.md : voir le manuel (docs/MANUALE.md).", cartella.display()),
format!("No se encontró ninguna aventura en «{}». Cada aventura es una subcarpeta con un archivo gioco.md: consulta el manual (docs/MANUALE.md).", cartella.display()),
format!("Keine Abenteuer in „{}“ gefunden. Jedes Abenteuer ist ein Unterordner mit einer Datei gioco.md: siehe Handbuch (docs/MANUALE.md).", cartella.display())
),
            Stile::Errore,
        )?;
        return Ok(SceltaAvventura::Esci);
    }
    let lingua = lingua::attuale();
    let visibili = versioni_da_mostrare(voci, lingua);
    let mut ordine: Vec<usize> = visibili
        .iter()
        .copied()
        .filter(|&i| voci[i].valida())
        .collect();
    ordine.extend(visibili.iter().copied().filter(|&i| !voci[i].valida()));
    if !voci.iter().any(VoceAvventura::valida) {
        canale.mostra(
            tr!(
                "Nessuna avventura valida da giocare.",
                "No valid adventure to play.",
                "Aucune aventure valide à jouer.",
                "No hay ninguna aventura válida para jugar.",
                "Kein gültiges Abenteuer zum Spielen."
            ),
            Stile::Errore,
        )?;
    }
    let mut menu: Vec<Voce> = ordine
        .iter()
        .map(|&i| match &voci[i].esito {
            Ok(r) => {
                // Il narratore usa comunque la lingua dell'interfaccia: la
                // lingua in cui è scritta l'avventura non si mostra.
                let titolo = r.titolo.clone();
                Voce::nuova(format!(
                    "{}  {}",
                    riempi(&titolo, LARGHEZZA_TITOLO),
                    r.etichetta()
                ))
                .con_dettaglio(r.descrizione_breve.clone())
                .con_breve(format!("{titolo} {}", r.etichetta()))
            }
            Err(motivo) => Voce::nuova(tr!(
                format!("{}: non valida — {motivo}", voci[i].id),
                format!("{}: not valid — {motivo}", voci[i].id),
                format!("{} : non valide — {motivo}", voci[i].id),
                format!("{}: no válida — {motivo}", voci[i].id),
                format!("{}: ungültig — {motivo}", voci[i].id)
            ))
            .non_attiva(),
        })
        .collect();
    menu.push(Voce::nuova(tr!(
        "🌐 Lingua: Italiano (cambia / change language)",
        "🌐 Language: English (change / cambia lingua)",
        "🌐 Langue : Français (changer / change language)",
        "🌐 Idioma: Español (cambiar / change language)",
        "🌐 Sprache: Deutsch (ändern / change language)"
    )));
    let titolo = tr!(
        "AVVENTURE DISPONIBILI",
        "AVAILABLE ADVENTURES",
        "AVENTURES DISPONIBLES",
        "AVENTURAS DISPONIBLES",
        "VERFÜGBARE ABENTEUER"
    );
    Ok(match canale.scegli(titolo, &menu)? {
        Some(i) if i == ordine.len() => SceltaAvventura::Lingua,
        Some(i) => SceltaAvventura::Avventura(ordine[i]),
        None => SceltaAvventura::Esci,
    })
}

/// Scelta nel menu della partita.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SceltaPartita {
    /// Continua `stato.json`.
    Continua,
    /// Carica il salvataggio con questo nome.
    Carica(String),
    /// Nuova partita.
    Nuova,
    /// Torna all'elenco delle avventure.
    Indietro,
}

#[derive(Clone, Copy)]
enum VoceMenuPartita {
    Continua,
    Carica,
    Nuova,
    Indietro,
}

/// Passo 2 del §7.1: Continua / Carica / Nuova / Indietro.
pub fn scegli_partita(
    canale: &mut dyn Canale,
    titolo_avventura: &str,
    in_corso: Option<(&InfoPartita, &Stato)>,
    salvataggi: &[InfoSalvataggio],
) -> Result<SceltaPartita> {
    let in_corso = in_corso.filter(|(p, _)| p.esito == Esito::InCorso);
    if in_corso.is_none() && salvataggi.is_empty() {
        return Ok(SceltaPartita::Nuova);
    }
    let mut voci = Vec::new();
    if in_corso.is_some() {
        voci.push(VoceMenuPartita::Continua);
    }
    if !salvataggi.is_empty() {
        voci.push(VoceMenuPartita::Carica);
    }
    voci.push(VoceMenuPartita::Nuova);
    voci.push(VoceMenuPartita::Indietro);
    let corrente = in_corso.and_then(|(_, s)| s.salvataggio_corrente.clone());
    let etichette: Vec<Voce> = voci
        .iter()
        .map(|v| {
            Voce::nuova(match v {
                VoceMenuPartita::Continua => match in_corso {
                    Some((p, _)) => tr!(
                        format!(
                            "Continua la partita in corso (turno {}, {}, {})",
                            p.turno,
                            p.nome_stanza,
                            p.aggiornato_il.format(formato_data())
                        ),
                        format!(
                            "Continue the game in progress (turn {}, {}, {})",
                            p.turno,
                            p.nome_stanza,
                            p.aggiornato_il.format(formato_data())
                        ),
                        format!(
                            "Continuer la partie en cours (tour {}, {}, {})",
                            p.turno,
                            p.nome_stanza,
                            p.aggiornato_il.format(formato_data())
                        ),
                        format!(
                            "Continuar la partida en curso (turno {}, {}, {})",
                            p.turno,
                            p.nome_stanza,
                            p.aggiornato_il.format(formato_data())
                        ),
                        format!(
                            "Laufendes Spiel fortsetzen (Zug {}, {}, {})",
                            p.turno,
                            p.nome_stanza,
                            p.aggiornato_il.format(formato_data())
                        )
                    ),
                    None => tr!(
                        "Continua la partita in corso",
                        "Continue the game in progress",
                        "Continuer la partie en cours",
                        "Continuar la partida en curso",
                        "Laufendes Spiel fortsetzen"
                    )
                    .to_string(),
                },
                VoceMenuPartita::Carica => tr!(
                    "Carica un salvataggio",
                    "Load a saved game",
                    "Charger une sauvegarde",
                    "Cargar una partida guardada",
                    "Spielstand laden"
                )
                .to_string(),
                VoceMenuPartita::Nuova => tr!(
                    "Nuova partita",
                    "New game",
                    "Nouvelle partie",
                    "Nueva partida",
                    "Neues Spiel"
                )
                .to_string(),
                VoceMenuPartita::Indietro => tr!(
                    "Torna all'elenco delle avventure",
                    "Back to the list of adventures",
                    "Retour à la liste des aventures",
                    "Volver a la lista de aventuras",
                    "Zurück zur Liste der Abenteuer"
                )
                .to_string(),
            })
        })
        .collect();

    loop {
        let Some(i) = canale.scegli(&titolo_avventura.to_uppercase(), &etichette)? else {
            return Ok(SceltaPartita::Indietro);
        };
        match voci[i] {
            VoceMenuPartita::Continua => return Ok(SceltaPartita::Continua),
            VoceMenuPartita::Indietro => return Ok(SceltaPartita::Indietro),
            VoceMenuPartita::Carica => {
                let Some(nome) =
                    scegli_salvataggio(canale, titolo_avventura, salvataggi, corrente.as_deref())?
                else {
                    continue;
                };
                if let Some((_, stato)) = in_corso
                    && progressi_non_salvati(stato)
                    && !canale.conferma(&tr!(
                        format!(
                            "La partita in corso (turno {}) ha {}. Caricare comunque?",
                            stato.turno,
                            turni_che_andranno_persi(stato)
                        ),
                        format!(
                            "The game in progress (turn {}) has {}. Load anyway?",
                            stato.turno,
                            turni_che_andranno_persi(stato)
                        ),
                        format!(
                            "La partie en cours (tour {}) a {}. Charger quand même ?",
                            stato.turno,
                            turni_che_andranno_persi(stato)
                        ),
                        format!(
                            "La partida en curso (turno {}) tiene {}. ¿Cargar de todos modos?",
                            stato.turno,
                            turni_che_andranno_persi(stato)
                        ),
                        format!(
                            "Das laufende Spiel (Zug {}) hat {}. Trotzdem laden?",
                            stato.turno,
                            turni_che_andranno_persi(stato)
                        )
                    ))?
                {
                    continue;
                }
                return Ok(SceltaPartita::Carica(nome));
            }
            VoceMenuPartita::Nuova => {
                if let Some((_, stato)) = in_corso
                    && progressi_non_salvati(stato)
                    && !canale.conferma(&domanda_sostituzione(stato))?
                {
                    continue;
                }
                return Ok(SceltaPartita::Nuova);
            }
        }
    }
}

/// Domanda prima di sostituire una partita in corso con progressi non salvati.
pub fn domanda_sostituzione(stato: &Stato) -> String {
    tr!(
        format!(
            "La partita in corso (turno {}) verrà sostituita. I salvataggi con nome restano. \
             Continuare?",
            stato.turno
        ),
        format!(
            "The game in progress (turn {}) will be replaced. Named saves are kept. Continue?",
            stato.turno
        ),
        format!(
            "La partie en cours (tour {}) sera remplacée. Les sauvegardes nommées sont conservées. Continuer ?",
            stato.turno
        ),
        format!(
            "La partida en curso (turno {}) será reemplazada. Las partidas guardadas con nombre se conservan. ¿Continuar?",
            stato.turno
        ),
        format!(
            "Das laufende Spiel (Zug {}) wird ersetzt. Benannte Spielstände bleiben erhalten. Fortfahren?",
            stato.turno
        )
    )
}

/// «1 turno non salvato che andrà perso» / «N turni non salvati che andranno persi».
pub fn turni_che_andranno_persi(stato: &Stato) -> String {
    match turni_non_salvati(stato) {
        1 => tr!(
            "1 turno non salvato che andrà perso",
            "1 unsaved turn that will be lost",
            "1 tour non sauvegardé qui sera perdu",
            "1 turno sin guardar que se perderá",
            "1 nicht gespeicherten Zug, der verloren geht"
        )
        .to_string(),
        n => tr!(
            format!("{n} turni non salvati che andranno persi"),
            format!("{n} unsaved turns that will be lost"),
            format!("{n} tours non sauvegardés qui seront perdus"),
            format!("{n} turnos sin guardar que se perderán"),
            format!("{n} nicht gespeicherte Züge, die verloren gehen")
        ),
    }
}

/// «1 turno non salvato» / «N turni non salvati».
pub fn testo_turni_non_salvati(stato: &Stato) -> String {
    match turni_non_salvati(stato) {
        1 => tr!(
            "1 turno non salvato",
            "1 unsaved turn",
            "1 tour non sauvegardé",
            "1 turno sin guardar",
            "1 nicht gespeicherter Zug"
        )
        .to_string(),
        n => tr!(
            format!("{n} turni non salvati"),
            format!("{n} unsaved turns"),
            format!("{n} tours non sauvegardés"),
            format!("{n} turnos sin guardar"),
            format!("{n} nicht gespeicherte Züge")
        ),
    }
}

/// Numero di turni giocati dopo l'ultimo `/save` o `/load`.
pub fn turni_non_salvati(stato: &Stato) -> u32 {
    stato.turno.saturating_sub(stato.turno_ultimo_salvataggio)
}

/// Tronca o riempie con spazi fino a `larghezza` caratteri.
fn riempi(testo: &str, larghezza: usize) -> String {
    let n = testo.chars().count();
    if n > larghezza {
        let mut t: String = testo.chars().take(larghezza.saturating_sub(1)).collect();
        t.push('…');
        t
    } else {
        format!("{testo}{}", " ".repeat(larghezza - n))
    }
}

fn e_corrente(corrente: Option<&str>, nome: &str) -> bool {
    corrente.is_some_and(|c| c.to_lowercase() == nome.to_lowercase())
}

/// Riga di un salvataggio (senza numero), nel formato della tabella di `/load`.
fn riga_salvataggio(s: &InfoSalvataggio, corrente: Option<&str>) -> String {
    let nome = if e_corrente(corrente, &s.nome) {
        format!("{}  *", riempi(&s.nome, 19).trim_end())
    } else {
        s.nome.clone()
    };
    match (&s.riepilogo, &s.aggiornato_il) {
        (Some(r), Some(data)) => {
            let esito = match r.esito {
                Esito::InCorso => "",
                Esito::Vittoria => tr!(
                    " (vinta)",
                    " (won)",
                    " (gagnée)",
                    " (ganada)",
                    " (gewonnen)"
                ),
                Esito::Sconfitta => tr!(
                    " (persa)",
                    " (lost)",
                    " (perdue)",
                    " (perdida)",
                    " (verloren)"
                ),
            };
            format!(
                "{}  {:>5}  {}  {:>5}  {}{esito}",
                riempi(&nome, 22),
                r.turno,
                riempi(&r.nome_stanza, 28),
                r.punteggio,
                data.format(formato_data())
            )
        }
        _ => format!(
            "{}  {}",
            riempi(&nome, 22),
            tr!(
                "(danneggiato)",
                "(damaged)",
                "(endommagée)",
                "(dañada)",
                "(beschädigt)"
            )
        ),
    }
}

/// Riga compatta di un salvataggio per i pulsanti (§7.5):
/// `★ nome · turno N · stanza · data`.
fn riga_breve_salvataggio(s: &InfoSalvataggio, corrente: Option<&str>) -> String {
    let stella = if e_corrente(corrente, &s.nome) {
        "★ "
    } else {
        ""
    };
    match (&s.riepilogo, &s.aggiornato_il) {
        (Some(r), Some(data)) => format!(
            "{stella}{} · {} {} · {} · {}",
            s.nome,
            tr!("turno", "turn", "tour", "turno", "Zug"),
            r.turno,
            r.nome_stanza,
            data.format(formato_data())
        ),
        _ => format!(
            "{stella}{} {}",
            s.nome,
            tr!(
                "(danneggiato)",
                "(damaged)",
                "(endommagée)",
                "(dañada)",
                "(beschädigt)"
            )
        ),
    }
}

/// Intestazione delle colonne della tabella dei salvataggi.
fn intestazione_salvataggi(titolo_avventura: &str) -> String {
    tr!(
        format!(
            "SALVATAGGI DISPONIBILI — {titolo_avventura}\n\n     {}  Turno  {}  Punti  Data",
            riempi("Nome", 22),
            riempi("Stanza", 28)
        ),
        format!(
            "SAVED GAMES — {titolo_avventura}\n\n     {}   Turn  {}  Score  Date",
            riempi("Name", 22),
            riempi("Room", 28)
        ),
        format!(
            "SAUVEGARDES — {titolo_avventura}\n\n     {}   Tour  {}  Score  Date",
            riempi("Nom", 22),
            riempi("Lieu", 28)
        ),
        format!(
            "PARTIDAS GUARDADAS — {titolo_avventura}\n\n     {}  Turno  {}   Pts.  Fecha",
            riempi("Nombre", 22),
            riempi("Lugar", 28)
        ),
        format!(
            "SPIELSTÄNDE — {titolo_avventura}\n\n     {}    Zug  {}   Pkt.  Datum",
            riempi("Name", 22),
            riempi("Raum", 28)
        )
    )
}

/// Nota sotto la tabella quando c'è un salvataggio corrente.
fn nota_corrente() -> &'static str {
    tr!(
        "* = salvataggio corrente.",
        "* = current save.",
        "* = sauvegarde actuelle.",
        "* = partida guardada actual.",
        "* = aktueller Spielstand."
    )
}

/// Tabella dei salvataggi (formato di `/load`, §7.3), per `/saves` e `avventura salvataggi`.
pub fn formatta_salvataggi(
    titolo_avventura: &str,
    salvataggi: &[InfoSalvataggio],
    corrente: Option<&str>,
) -> String {
    let mut t = format!(
        "\n  {}\n",
        intestazione_salvataggi(titolo_avventura).replace('\n', "\n  ")
    );
    for (i, s) in salvataggi.iter().enumerate() {
        t.push_str(&format!(
            "\n  {:>2}) {}",
            i + 1,
            riga_salvataggio(s, corrente)
        ));
    }
    if salvataggi.iter().any(|s| e_corrente(corrente, &s.nome)) {
        t.push_str(&format!("\n\n  {}", nota_corrente()));
    }
    t
}

/// Lista dei salvataggi con scelta per numero o nome. `None` = annullato.
pub fn scegli_salvataggio(
    canale: &mut dyn Canale,
    titolo_avventura: &str,
    salvataggi: &[InfoSalvataggio],
    corrente: Option<&str>,
) -> Result<Option<String>> {
    if salvataggi.is_empty() {
        canale.mostra(nessun_salvataggio(), Stile::Motore)?;
        return Ok(None);
    }
    let voci: Vec<Voce> = salvataggi
        .iter()
        .map(|s| {
            let v = Voce::nuova(riga_salvataggio(s, corrente))
                .con_chiave(s.nome.clone())
                .con_breve(riga_breve_salvataggio(s, corrente));
            if s.danneggiato() { v.non_attiva() } else { v }
        })
        .collect();
    let mut titolo = intestazione_salvataggi(titolo_avventura);
    if salvataggi.iter().any(|s| e_corrente(corrente, &s.nome)) {
        titolo = format!("{titolo}\n     ({})", nota_corrente());
    }
    Ok(canale
        .scegli(&titolo, &voci)?
        .map(|i| salvataggi[i].nome.clone()))
}

/// Cerca un salvataggio per nome (confronto come per i nomi dei file).
fn trova_per_nome<'a>(
    salvataggi: &'a [InfoSalvataggio],
    nome: &str,
) -> Option<&'a InfoSalvataggio> {
    let cercato = crate::salvataggi::nome_file(nome).to_lowercase();
    salvataggi
        .iter()
        .find(|s| crate::salvataggi::nome_file(&s.nome).to_lowercase() == cercato)
}

/// Esegue il salvataggio e mostra la conferma.
fn esegui_salvataggio(
    canale: &mut dyn Canale,
    gestore: &GestoreSalvataggi,
    stato: &mut Stato,
    nome_stanza: &str,
    nome: &str,
) -> Result<String> {
    gestore.salva(nome, stato, nome_stanza, true)?;
    let nome = stato
        .salvataggio_corrente
        .clone()
        .unwrap_or_else(|| nome.to_string());
    canale.mostra(
        &tr!(
            format!("Partita salvata in «{nome}» (turno {}).", stato.turno),
            format!("Game saved as \"{nome}\" (turn {}).", stato.turno),
            format!("Partie sauvegardée sous « {nome} » (tour {}).", stato.turno),
            format!("Partida guardada como «{nome}» (turno {}).", stato.turno),
            format!("Spiel als „{nome}“ gespeichert (Zug {}).", stato.turno)
        ),
        Stile::Motore,
    )?;
    Ok(nome)
}

/// Messaggio di salvataggio annullato.
fn salvataggio_annullato() -> &'static str {
    tr!(
        "Salvataggio annullato.",
        "Save cancelled.",
        "Sauvegarde annulée.",
        "Guardado cancelado.",
        "Speichern abgebrochen."
    )
}

/// « (turno N)» / « (turn N)» per le domande di sovrascrittura.
fn tra_parentesi_turno(info: &InfoSalvataggio) -> String {
    info.riepilogo
        .as_ref()
        .map(|r| {
            tr!(
                format!(" (turno {})", r.turno),
                format!(" (turn {})", r.turno),
                format!(" (tour {})", r.turno),
                format!(" (turno {})", r.turno),
                format!(" (Zug {})", r.turno)
            )
        })
        .unwrap_or_default()
}

#[derive(Clone, Copy)]
enum VoceMenuSalva {
    Corrente,
    Esistente,
    Nuovo,
}

/// `/save` senza argomenti: menu (§7.3). Restituisce il nome usato, `None` se annullato.
pub fn menu_salva(
    canale: &mut dyn Canale,
    gestore: &GestoreSalvataggi,
    titolo_avventura: &str,
    stato: &mut Stato,
    nome_stanza: &str,
) -> Result<Option<String>> {
    let salvataggi = gestore.elenca();
    let corrente = stato
        .salvataggio_corrente
        .clone()
        .filter(|c| gestore.esiste(c));
    let mut voci = Vec::new();
    if corrente.is_some() {
        voci.push(VoceMenuSalva::Corrente);
    }
    if !salvataggi.is_empty() {
        voci.push(VoceMenuSalva::Esistente);
    }
    voci.push(VoceMenuSalva::Nuovo);
    let etichette: Vec<Voce> = voci
        .iter()
        .map(|v| {
            Voce::nuova(match v {
                VoceMenuSalva::Corrente => {
                    let c = corrente.as_deref().unwrap_or("");
                    tr!(
                        format!("Salva in «{c}» (salvataggio corrente)"),
                        format!("Save as \"{c}\" (current save)"),
                        format!("Sauvegarder sous « {c} » (sauvegarde actuelle)"),
                        format!("Guardar como «{c}» (partida guardada actual)"),
                        format!("Als „{c}“ speichern (aktueller Spielstand)")
                    )
                }
                VoceMenuSalva::Esistente => tr!(
                    "Sovrascrivi un salvataggio esistente",
                    "Overwrite an existing save",
                    "Écraser une sauvegarde existante",
                    "Sobrescribir una partida guardada",
                    "Vorhandenen Spielstand überschreiben"
                )
                .to_string(),
                VoceMenuSalva::Nuovo => tr!(
                    "Nuovo salvataggio",
                    "New save",
                    "Nouvelle sauvegarde",
                    "Nueva partida guardada",
                    "Neuer Spielstand"
                )
                .to_string(),
            })
        })
        .collect();
    let titolo = tr!(
        format!("SALVA PARTITA (turno {}, {nome_stanza})", stato.turno),
        format!("SAVE GAME (turn {}, {nome_stanza})", stato.turno),
        format!(
            "SAUVEGARDER LA PARTIE (tour {}, {nome_stanza})",
            stato.turno
        ),
        format!("GUARDAR PARTIDA (turno {}, {nome_stanza})", stato.turno),
        format!("SPIEL SPEICHERN (Zug {}, {nome_stanza})", stato.turno)
    );

    loop {
        let Some(i) = canale.scegli(&titolo, &etichette)? else {
            canale.mostra(salvataggio_annullato(), Stile::Motore)?;
            return Ok(None);
        };
        match voci[i] {
            VoceMenuSalva::Corrente => {
                let nome = corrente.clone().unwrap_or_default();
                return esegui_salvataggio(canale, gestore, stato, nome_stanza, &nome).map(Some);
            }
            VoceMenuSalva::Esistente => {
                let Some(nome) =
                    scegli_salvataggio(canale, titolo_avventura, &salvataggi, corrente.as_deref())?
                else {
                    continue;
                };
                let turno = salvataggi
                    .iter()
                    .find(|s| s.nome == nome)
                    .map(tra_parentesi_turno)
                    .unwrap_or_default();
                let domanda = tr!(
                    format!("Sovrascrivere «{nome}»{turno}?"),
                    format!("Overwrite \"{nome}\"{turno}?"),
                    format!("Écraser « {nome} »{turno} ?"),
                    format!("¿Sobrescribir «{nome}»{turno}?"),
                    format!("„{nome}“{turno} überschreiben?")
                );
                if canale.conferma(&domanda)? {
                    return esegui_salvataggio(canale, gestore, stato, nome_stanza, &nome)
                        .map(Some);
                }
            }
            VoceMenuSalva::Nuovo => {
                if let Some(nome) = chiedi_nome_nuovo(canale, gestore, stato)? {
                    return esegui_salvataggio(canale, gestore, stato, nome_stanza, &nome)
                        .map(Some);
                }
            }
        }
    }
}

/// Chiede il nome di un nuovo salvataggio proponendo quello predefinito.
fn chiedi_nome_nuovo(
    canale: &mut dyn Canale,
    gestore: &GestoreSalvataggi,
    stato: &Stato,
) -> Result<Option<String>> {
    let predefinito = nome_predefinito(&stato.stanza_corrente, stato.turno);
    loop {
        let Some(candidato) = canale.chiedi_testo(
            tr!(
                "Nome del salvataggio",
                "Name of the save",
                "Nom de la sauvegarde",
                "Nombre de la partida guardada",
                "Name des Spielstands"
            ),
            Some(&predefinito),
        )?
        else {
            return Ok(None);
        };
        let nome = match valida_nome(&candidato) {
            Ok(n) => n,
            Err(motivo) => {
                canale.mostra(
                    &tr!(
                        format!("Nome non valido: {motivo}. Riprova."),
                        format!("Invalid name: {motivo}. Try again."),
                        format!("Nom non valide : {motivo}. Réessayez."),
                        format!("Nombre no válido: {motivo}. Inténtalo de nuevo."),
                        format!("Ungültiger Name: {motivo}. Versuch es noch einmal.")
                    ),
                    Stile::Errore,
                )?;
                continue;
            }
        };
        match gestore.trova(&nome) {
            None => return Ok(Some(nome)),
            Some(esistente) => {
                let turno = tra_parentesi_turno(&esistente);
                if canale.conferma(&tr!(
                    format!(
                        "Esiste già un salvataggio «{}»{turno}. Sovrascriverlo?",
                        esistente.nome
                    ),
                    format!(
                        "A save named \"{}\"{turno} already exists. Overwrite it?",
                        esistente.nome
                    ),
                    format!(
                        "Une sauvegarde « {} »{turno} existe déjà. L'écraser ?",
                        esistente.nome
                    ),
                    format!(
                        "Ya existe una partida guardada «{}»{turno}. ¿Sobrescribirla?",
                        esistente.nome
                    ),
                    format!(
                        "Ein Spielstand „{}“{turno} existiert bereits. Überschreiben?",
                        esistente.nome
                    )
                ))? {
                    return Ok(Some(esistente.nome));
                }
            }
        }
    }
}

/// `/save NOME`: crea subito, o chiede conferma se esiste (non se è il corrente).
pub fn salva_con_nome(
    canale: &mut dyn Canale,
    gestore: &GestoreSalvataggi,
    stato: &mut Stato,
    nome_stanza: &str,
    nome: &str,
) -> Result<Option<String>> {
    let nome = match valida_nome(nome) {
        Ok(n) => n,
        Err(motivo) => {
            canale.mostra(
                &tr!(
                    format!("Nome non valido: {motivo}."),
                    format!("Invalid name: {motivo}."),
                    format!("Nom non valide : {motivo}."),
                    format!("Nombre no válido: {motivo}."),
                    format!("Ungültiger Name: {motivo}.")
                ),
                Stile::Errore,
            )?;
            return Ok(None);
        }
    };
    if let Some(esistente) = gestore.trova(&nome) {
        let corrente = e_corrente(stato.salvataggio_corrente.as_deref(), &esistente.nome);
        let turno = tra_parentesi_turno(&esistente);
        if !corrente
            && !canale.conferma(&tr!(
                format!("Sovrascrivere «{}»{turno}?", esistente.nome),
                format!("Overwrite \"{}\"{turno}?", esistente.nome),
                format!("Écraser « {} »{turno} ?", esistente.nome),
                format!("¿Sobrescribir «{}»{turno}?", esistente.nome),
                format!("„{}“{turno} überschreiben?", esistente.nome)
            ))?
        {
            canale.mostra(salvataggio_annullato(), Stile::Motore)?;
            return Ok(None);
        }
        return esegui_salvataggio(canale, gestore, stato, nome_stanza, &esistente.nome).map(Some);
    }
    esegui_salvataggio(canale, gestore, stato, nome_stanza, &nome).map(Some)
}

/// `/load` (con o senza nome): sceglie il salvataggio e chiede conferma se ci sono
/// progressi non salvati. Restituisce il nome da caricare.
pub fn menu_carica(
    canale: &mut dyn Canale,
    gestore: &GestoreSalvataggi,
    titolo_avventura: &str,
    stato: &Stato,
    nome: Option<&str>,
) -> Result<Option<String>> {
    let salvataggi = gestore.elenca();
    if salvataggi.is_empty() {
        canale.mostra(nessun_salvataggio(), Stile::Motore)?;
        return Ok(None);
    }
    let corrente = stato.salvataggio_corrente.as_deref();
    let diretto = match nome {
        Some(n) => {
            let trovato = trova_per_nome(&salvataggi, n);
            if trovato.is_none() {
                canale.mostra(&non_esiste(n), Stile::Motore)?;
            }
            trovato
        }
        None => None,
    };
    let scelto = match diretto {
        Some(s) if s.danneggiato() => {
            canale.mostra(
                &tr!(
                    format!(
                        "Il salvataggio «{}» è danneggiato e non può essere caricato.",
                        s.nome
                    ),
                    format!("The save \"{}\" is damaged and cannot be loaded.", s.nome),
                    format!(
                        "La sauvegarde « {} » est endommagée et ne peut pas être chargée.",
                        s.nome
                    ),
                    format!(
                        "La partida guardada «{}» está dañada y no se puede cargar.",
                        s.nome
                    ),
                    format!(
                        "Der Spielstand „{}“ ist beschädigt und kann nicht geladen werden.",
                        s.nome
                    )
                ),
                Stile::Errore,
            )?;
            return Ok(None);
        }
        Some(s) => s.nome.clone(),
        None => match scegli_salvataggio(canale, titolo_avventura, &salvataggi, corrente)? {
            Some(n) => n,
            None => return Ok(None),
        },
    };
    if progressi_non_salvati(stato)
        && !canale.conferma(&tr!(
            format!(
                "Hai {}. Caricare comunque?",
                turni_che_andranno_persi(stato)
            ),
            format!("You have {}. Load anyway?", turni_che_andranno_persi(stato)),
            format!(
                "Vous avez {}. Charger quand même ?",
                turni_che_andranno_persi(stato)
            ),
            format!(
                "Tienes {}. ¿Cargar de todos modos?",
                turni_che_andranno_persi(stato)
            ),
            format!(
                "Du hast {}. Trotzdem laden?",
                turni_che_andranno_persi(stato)
            )
        ))?
    {
        canale.mostra(
            tr!(
                "Caricamento annullato.",
                "Loading cancelled.",
                "Chargement annulé.",
                "Carga cancelada.",
                "Laden abgebrochen."
            ),
            Stile::Motore,
        )?;
        return Ok(None);
    }
    Ok(Some(scelto))
}

/// «Non esiste un salvataggio «X».»
pub fn non_esiste(nome: &str) -> String {
    tr!(
        format!("Non esiste un salvataggio «{nome}»."),
        format!("There is no save named \"{nome}\"."),
        format!("Il n'existe pas de sauvegarde « {nome} »."),
        format!("No existe ninguna partida guardada «{nome}»."),
        format!("Es gibt keinen Spielstand „{nome}“.")
    )
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::catalogo_avventure::Riepilogo;

    fn voce(id: &str, storia: &str, lingua: Lingua) -> VoceAvventura {
        VoceAvventura {
            id: id.into(),
            cartella: id.into(),
            esito: Ok(Riepilogo {
                titolo: id.into(),
                descrizione_breve: String::new(),
                numero_stanze: 1,
                partita: None,
                numero_salvataggi: 0,
                lingua,
                storia: storia.into(),
            }),
        }
    }

    #[test]
    fn una_versione_per_storia_nella_lingua_giusta() {
        let voci = vec![
            voce("faro", "faro", Lingua::It),
            voce("faro-en", "faro", Lingua::En),
            voce("solo-it", "solo-it", Lingua::It),
            VoceAvventura {
                id: "rotta".into(),
                cartella: "rotta".into(),
                esito: Err("x".into()),
            },
        ];
        assert_eq!(versioni_da_mostrare(&voci, Lingua::It), vec![0, 2, 3]);
        assert_eq!(versioni_da_mostrare(&voci, Lingua::En), vec![1, 2, 3]);
    }

    #[test]
    fn menu_lingua_con_la_proposta_per_prima() {
        let mut c = crate::canale::prova::CanaleProva::nuovo(&["1"]);
        assert_eq!(scegli_lingua(&mut c, Lingua::It).unwrap(), Some(Lingua::It));
        assert!(c.uscita.contains("1) Italiano") && c.uscita.contains("2) English"));
        let mut c = crate::canale::prova::CanaleProva::nuovo(&["it"]);
        assert_eq!(scegli_lingua(&mut c, Lingua::En).unwrap(), Some(Lingua::It));
    }
}
