//! Separazione tra motore e canale di input/output (§8).
//!
//! Il motore (turni, menu, comandi, salvataggi) non scrive mai direttamente sul
//! terminale e non legge mai dallo standard input: usa il trait [`Canale`].
//! Implementazioni:
//! - [`terminale::Terminale`]: rustyline, testo a capo, colori, indicatore di
//!   attesa, trascrizione (§7.4);
//! - [`prova::CanaleProva`]: input da una lista di righe, output registrato (test);
//! - [`crate::telegram::sessione::CanaleTelegram`], che trasforma menu e
//!   conferme in pulsanti inline (§7.5).
//!
//! I canali che lavorano «a righe di testo» (terminale e prova) condividono la
//! stessa logica di menu, conferme e domande tramite [`IoRighe`] e le funzioni
//! `*_a_righe` di questo modulo: così i test esercitano esattamente il codice
//! usato dal terminale.

pub mod prova;
pub mod terminale;

use std::sync::mpsc;
use std::time::{Duration, Instant};

use anyhow::Result;

use crate::comandi::{self, Comando, Ingresso};

/// Stile di un testo mostrato al giocatore.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stile {
    /// Narrazione dell'AI o testi dell'avventura (vanno a capo).
    Narrazione,
    /// Titolo (nome della stanza, intestazioni).
    Titolo,
    /// Messaggio del motore.
    Motore,
    /// Avviso (problemi non bloccanti).
    Avviso,
    /// Errore.
    Errore,
    /// Riquadro finale di vittoria/sconfitta (preformattato).
    Finale,
    /// Testo preformattato (elenchi, tabelle, aiuto): mostrato così com'è.
    Elenco,
}

/// Una voce di menu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Voce {
    /// Testo principale.
    pub testo: String,
    /// Riga di dettaglio facoltativa (es. descrizione dell'avventura).
    pub dettaglio: Option<String>,
    /// Nome con cui la voce si può scegliere al posto del numero (es. salvataggio).
    pub chiave: Option<String>,
    /// Falso = mostrata ma non selezionabile (avventura non valida, file danneggiato).
    pub attiva: bool,
    /// Etichetta compatta per i canali a pulsanti (Telegram); assente = `testo`.
    pub breve: Option<String>,
}

impl Voce {
    /// Voce semplice e selezionabile.
    pub fn nuova(testo: impl Into<String>) -> Self {
        Voce {
            testo: testo.into(),
            dettaglio: None,
            chiave: None,
            attiva: true,
            breve: None,
        }
    }

    /// Etichetta compatta usata dai canali a pulsanti al posto di `testo`.
    pub fn con_breve(mut self, breve: impl Into<String>) -> Self {
        self.breve = Some(breve.into());
        self
    }

    /// Etichetta per un pulsante: `breve` oppure `testo` senza spazi ripetuti.
    pub fn etichetta_breve(&self) -> String {
        match &self.breve {
            Some(b) => b.clone(),
            None => self.testo.split_whitespace().collect::<Vec<_>>().join(" "),
        }
    }

    /// Aggiunge una riga di dettaglio.
    pub fn con_dettaglio(mut self, dettaglio: impl Into<String>) -> Self {
        let d = dettaglio.into();
        self.dettaglio = (!d.trim().is_empty()).then_some(d);
        self
    }

    /// Permette di scegliere la voce scrivendone il nome.
    pub fn con_chiave(mut self, chiave: impl Into<String>) -> Self {
        self.chiave = Some(chiave.into());
        self
    }

    /// Rende la voce non selezionabile.
    pub fn non_attiva(mut self) -> Self {
        self.attiva = false;
        self
    }
}

/// Input del giocatore durante la partita.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Input {
    /// Azione da inviare all'AI.
    Azione(String),
    /// Comando del motore (`/…`).
    Comando(Comando),
    /// Fine dell'input (EOF, Ctrl+C, chiusura della sessione).
    Fine,
}

/// Canale di interazione con il giocatore.
pub trait Canale {
    /// Mostra un testo con lo stile indicato.
    fn mostra(&mut self, testo: &str, stile: Stile) -> Result<()>;
    /// Menu: restituisce l'indice della voce scelta; `None` = annulla/indietro/timeout.
    fn scegli(&mut self, titolo: &str, voci: &[Voce]) -> Result<Option<usize>>;
    /// Testo libero; con `proposta`, una risposta vuota la accetta. `None` = annulla.
    fn chiedi_testo(&mut self, domanda: &str, proposta: Option<&str>) -> Result<Option<String>>;
    /// Domanda sì/no; fine dell'input o timeout = no.
    fn conferma(&mut self, domanda: &str) -> Result<bool>;
    /// Legge la prossima azione o il prossimo comando del giocatore.
    fn leggi_azione(&mut self) -> Result<Input>;
    /// Chiamata periodicamente mentre l'AI risponde (spinner, "sta scrivendo…").
    fn attesa(&mut self, secondi_trascorsi: u64) -> Result<()>;
    /// Fine dell'attesa (toglie l'indicatore). Facoltativa.
    fn fine_attesa(&mut self) -> Result<()> {
        Ok(())
    }
    /// Vero se c'è una persona che risponde alle domande (terminale interattivo,
    /// Telegram): solo allora si chiede la lingua al primo utilizzo (§7.6).
    fn interattivo(&self) -> bool {
        false
    }
    /// Ogni quanto chiamare [`Canale::attesa`].
    fn intervallo_attesa(&self) -> Duration {
        Duration::from_millis(250)
    }
    /// Vero se il canale mostra la narrazione man mano che il modello la
    /// scrive (testo progressivo): solo allora si chiede lo streaming.
    fn testo_progressivo(&self) -> bool {
        false
    }
    /// Nuovo pezzo di narrazione arrivato durante l'attesa (testo progressivo).
    /// Alla fine dell'attesa la narrazione completa viene comunque passata a
    /// [`Canale::mostra`]: il canale decide se ristamparla o solo confermarla.
    fn frammento(&mut self, testo: &str) -> Result<()> {
        let _ = testo;
        Ok(())
    }
}

/// Esegue `lavoro` in un thread separato chiamando [`Canale::attesa`] finché
/// non termina: l'interfaccia non resta mai bloccata senza indicatore (§11).
pub fn con_attesa<T, F>(canale: &mut dyn Canale, lavoro: F) -> Result<T>
where
    T: Send,
    F: FnOnce() -> T + Send,
{
    con_attesa_e_frammenti(canale, move |_| lavoro())
}

/// Come [`con_attesa`], ma `lavoro` riceve una funzione per inviare pezzi di
/// testo: arrivano al canale ([`Canale::frammento`]) dal thread principale,
/// tra un'animazione dell'indicatore e l'altra.
pub fn con_attesa_e_frammenti<T, F>(canale: &mut dyn Canale, lavoro: F) -> Result<T>
where
    T: Send,
    F: FnOnce(&mut dyn FnMut(String)) -> T + Send,
{
    let inizio = Instant::now();
    let intervallo = canale.intervallo_attesa();
    // Il lavoro gira in un altro thread: gli si passa la lingua dell'interfaccia.
    let lingua = crate::lingua::attuale();
    let (invio, ricezione) = mpsc::channel::<String>();
    let risultato = std::thread::scope(|ambito| -> Result<T> {
        let gestore = ambito.spawn(move || {
            crate::lingua::imposta(lingua);
            let mut invia = |testo: String| {
                let _ = invio.send(testo);
            };
            lavoro(&mut invia)
        });
        let mut errore_canale = None;
        let consegna = |canale: &mut dyn Canale, errore: &mut Option<anyhow::Error>| {
            for testo in ricezione.try_iter() {
                if errore.is_none()
                    && let Err(e) = canale.frammento(&testo)
                {
                    *errore = Some(e);
                }
            }
        };
        while !gestore.is_finished() {
            if errore_canale.is_none()
                && let Err(e) = canale.attesa(inizio.elapsed().as_secs())
            {
                errore_canale = Some(e);
            }
            // Attesa a piccoli passi, così la fine del lavoro e i nuovi pezzi
            // di testo si notano subito.
            let prossima = Instant::now() + intervallo;
            while !gestore.is_finished() && Instant::now() < prossima {
                std::thread::sleep(Duration::from_millis(20));
                consegna(canale, &mut errore_canale);
            }
        }
        consegna(canale, &mut errore_canale);
        let valore = gestore.join().map_err(|_| {
            anyhow::anyhow!(crate::tr!(
                "errore interno durante la richiesta all'AI",
                "internal error during the AI request",
                "erreur interne pendant la requête à l'IA",
                "error interno durante la petición a la IA",
                "interner Fehler bei der KI-Anfrage"
            ))
        })?;
        match errore_canale {
            Some(e) => Err(e),
            None => Ok(valore),
        }
    });
    canale.fine_attesa()?;
    risultato
}

/// Riga del giocatore interpretata: `None` = vuota, `Err` = rifiutata con motivo.
pub fn interpreta_riga(riga: &str) -> Option<std::result::Result<Input, String>> {
    match comandi::classifica(riga) {
        Ingresso::Vuoto => None,
        Ingresso::Rifiutato(motivo) => Some(Err(motivo)),
        Ingresso::Comando(c) => Some(Ok(Input::Comando(c))),
        Ingresso::Azione(a) => Some(Ok(Input::Azione(a))),
    }
}

/// Esito della ricerca di una voce dalla risposta del giocatore.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RicercaVoce {
    /// Voce selezionabile.
    Trovata(usize),
    /// Voce esistente ma non selezionabile.
    NonAttiva(usize),
    /// Nessuna voce corrisponde.
    Nessuna,
}

/// Normalizza un nome per il confronto (minuscole, spazi → `-`).
pub fn normalizza_nome(nome: &str) -> String {
    nome.split_whitespace()
        .collect::<Vec<_>>()
        .join("-")
        .to_lowercase()
}

/// Numeri mostrati accanto alle voci: solo le attive sono numerate (1, 2, …).
pub fn numerazione(voci: &[Voce]) -> Vec<Option<usize>> {
    let mut n = 0;
    voci.iter()
        .map(|v| {
            v.attiva.then(|| {
                n += 1;
                n
            })
        })
        .collect()
}

/// Cerca la voce indicata con il numero mostrato o con la sua chiave.
pub fn trova_voce(voci: &[Voce], risposta: &str) -> RicercaVoce {
    let r = risposta.trim();
    if let Ok(numero) = r.parse::<usize>() {
        return numerazione(voci)
            .iter()
            .position(|n| *n == Some(numero))
            .map_or(RicercaVoce::Nessuna, RicercaVoce::Trovata);
    }
    let cercato = normalizza_nome(r);
    match voci.iter().position(|v| {
        v.chiave
            .as_deref()
            .is_some_and(|c| normalizza_nome(c) == cercato)
    }) {
        Some(i) if voci[i].attiva => RicercaVoce::Trovata(i),
        Some(i) => RicercaVoce::NonAttiva(i),
        None => RicercaVoce::Nessuna,
    }
}

/// Operazioni elementari di un canale a righe di testo.
pub trait IoRighe {
    /// Legge una riga dopo aver mostrato `prompt`; `None` = fine dell'input.
    fn leggi(&mut self, prompt: &str) -> Option<String>;
    /// Scrive un testo con lo stile indicato.
    fn scrivi(&mut self, testo: &str, stile: Stile);
}

/// Menu numerato su un canale a righe (q o 0 = annulla, fine input = annulla).
pub fn scegli_a_righe(io: &mut dyn IoRighe, titolo: &str, voci: &[Voce]) -> Option<usize> {
    let numeri = numerazione(voci);
    let mut testo = String::new();
    if !titolo.trim().is_empty() {
        testo.push_str(&format!(
            "\n  {}\n",
            titolo.trim_end().replace('\n', "\n  ")
        ));
    }
    for (v, n) in voci.iter().zip(&numeri) {
        let segno = match n {
            Some(n) => format!("{n:>2})"),
            None => "  -".to_string(),
        };
        testo.push_str(&format!("\n {segno} {}", v.testo));
        if let Some(d) = &v.dettaglio {
            testo.push_str(&format!("\n     {d}"));
        }
    }
    testo.push('\n');
    io.scrivi(&testo, Stile::Elenco);
    let attive = numeri.iter().flatten().count();
    if attive == 0 {
        return None;
    }
    // Le chiavi di una sola lettera (s/n) sono scorciatoie, non nomi da elencare nel prompt.
    let per_nome = voci
        .iter()
        .any(|v| v.chiave.as_deref().is_some_and(|c| c.chars().count() > 1));
    let prompt = if per_nome {
        crate::tr!(
            "  Scegli (numero o nome, q per annullare): ",
            "  Choose (number or name, q to cancel): ",
            "  Choisissez (numéro ou nom, q pour annuler) : ",
            "  Elige (número o nombre, q para cancelar): ",
            "  Wähle (Nummer oder Name, q zum Abbrechen): ",
        )
    } else {
        crate::tr!(
            "  Scegli un numero (q per annullare): ",
            "  Choose a number (q to cancel): ",
            "  Choisissez un numéro (q pour annuler) : ",
            "  Elige un número (q para cancelar): ",
            "  Wähle eine Nummer (q zum Abbrechen): ",
        )
    };
    loop {
        let r = io.leggi(prompt)?;
        let r = r.trim();
        if r.eq_ignore_ascii_case("q") || r == "0" {
            return None;
        }
        if r.is_empty() {
            continue;
        }
        match trova_voce(voci, r) {
            RicercaVoce::Trovata(i) => return Some(i),
            RicercaVoce::NonAttiva(i) => io.scrivi(
                &crate::tr!(
                    format!("«{}» non è selezionabile.", voci[i].testo.trim()),
                    format!("\"{}\" cannot be selected.", voci[i].testo.trim()),
                    format!("« {} » ne peut pas être sélectionné.", voci[i].testo.trim()),
                    format!("«{}» no se puede seleccionar.", voci[i].testo.trim()),
                    format!("„{}“ kann nicht ausgewählt werden.", voci[i].testo.trim())
                ),
                Stile::Errore,
            ),
            RicercaVoce::Nessuna => io.scrivi(
                &crate::tr!(
                    format!(
                        "Scelta non valida: scrivi un numero da 1 a {attive}{} oppure q.",
                        if per_nome {
                            " o un nome dell'elenco"
                        } else {
                            ""
                        }
                    ),
                    format!(
                        "Invalid choice: type a number from 1 to {attive}{} or q.",
                        if per_nome {
                            ", a name from the list,"
                        } else {
                            ""
                        }
                    ),
                    format!(
                        "Choix non valide : tapez un nombre de 1 à {attive}{} ou q.",
                        if per_nome { ", un nom de la liste" } else { "" }
                    ),
                    format!(
                        "Opción no válida: escribe un número del 1 al {attive}{} o q.",
                        if per_nome {
                            ", un nombre de la lista"
                        } else {
                            ""
                        }
                    ),
                    format!(
                        "Ungültige Auswahl: Gib eine Zahl von 1 bis {attive}{} oder q ein.",
                        if per_nome {
                            ", einen Namen aus der Liste"
                        } else {
                            ""
                        }
                    )
                ),
                Stile::Motore,
            ),
        }
    }
}

/// Domanda sì/no su un canale a righe (fine input = no).
pub fn conferma_a_righe(io: &mut dyn IoRighe, domanda: &str) -> bool {
    loop {
        let Some(r) = io.leggi(&crate::tr!(
            format!("{domanda} (s/n) "),
            format!("{domanda} (y/n) "),
            format!("{domanda} (o/n) "),
            format!("{domanda} (s/n) "),
            format!("{domanda} (j/n) ")
        )) else {
            return false;
        };
        match r.trim().to_lowercase().as_str() {
            r if crate::lingua::e_si(r) => return true,
            r if crate::lingua::e_no(r) => return false,
            _ => io.scrivi(
                crate::tr!(
                    "Rispondi s oppure n.",
                    "Please answer y or n.",
                    "Répondez o ou n.",
                    "Responde s o n.",
                    "Bitte antworte j oder n."
                ),
                Stile::Motore,
            ),
        }
    }
}

/// Testo libero su un canale a righe.
pub fn chiedi_testo_a_righe(
    io: &mut dyn IoRighe,
    domanda: &str,
    proposta: Option<&str>,
) -> Option<String> {
    let prompt = match proposta {
        Some(p) => format!("  {domanda} [{p}]: "),
        None => format!("  {domanda}: "),
    };
    let r = io.leggi(&prompt)?;
    let r = r.trim();
    if r.is_empty() {
        proposta.map(str::to_string)
    } else {
        Some(r.to_string())
    }
}

/// Azione del giocatore su un canale a righe (prompt `> `).
pub fn leggi_azione_a_righe(io: &mut dyn IoRighe) -> Input {
    loop {
        let Some(riga) = io.leggi("> ") else {
            return Input::Fine;
        };
        match interpreta_riga(&riga) {
            None => {}
            Some(Ok(input)) => return input,
            Some(Err(motivo)) => io.scrivi(&motivo, Stile::Errore),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn voci() -> Vec<Voce> {
        vec![
            Voce::nuova("Prima").con_chiave("prima-della-scala"),
            Voce::nuova("Rotta").con_chiave("vecchio").non_attiva(),
            Voce::nuova("Seconda").con_chiave("inizio"),
        ]
    }

    #[test]
    fn ricerca_per_numero_e_per_nome() {
        let v = voci();
        assert_eq!(numerazione(&v), vec![Some(1), None, Some(2)]);
        assert_eq!(trova_voce(&v, "2"), RicercaVoce::Trovata(2));
        assert_eq!(trova_voce(&v, "3"), RicercaVoce::Nessuna);
        assert_eq!(trova_voce(&v, "Prima della Scala"), RicercaVoce::Trovata(0));
        assert_eq!(trova_voce(&v, "VECCHIO"), RicercaVoce::NonAttiva(1));
        assert_eq!(trova_voce(&v, "boh"), RicercaVoce::Nessuna);
    }

    #[test]
    fn righe_interpretate() {
        assert_eq!(interpreta_riga("  "), None);
        assert_eq!(
            interpreta_riga("apro la porta"),
            Some(Ok(Input::Azione("apro la porta".into())))
        );
        assert_eq!(
            interpreta_riga("/help"),
            Some(Ok(Input::Comando(Comando::Aiuto)))
        );
        assert!(matches!(interpreta_riga("[INIZIO PARTITA]"), Some(Err(_))));
    }

    #[test]
    fn attesa_chiamata_durante_il_lavoro() {
        let mut c = prova::CanaleProva::nuovo(&[] as &[&str]);
        let valore = con_attesa(&mut c, || {
            std::thread::sleep(Duration::from_millis(600));
            42
        })
        .unwrap();
        assert_eq!(valore, 42);
        assert!(c.attese >= 2, "attese: {}", c.attese);
    }

    #[test]
    fn frammenti_consegnati_nell_ordine() {
        let mut c = prova::CanaleProva::nuovo(&[] as &[&str]);
        let valore = con_attesa_e_frammenti(&mut c, |invia| {
            for parola in ["C'era ", "una ", "volta"] {
                invia(parola.to_string());
                std::thread::sleep(Duration::from_millis(30));
            }
            7
        })
        .unwrap();
        assert_eq!(valore, 7);
        assert_eq!(c.frammenti.concat(), "C'era una volta");
    }
}
