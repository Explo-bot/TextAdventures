//! Sessioni del bot Telegram (§7.5): un thread per utente che riceve gli eventi
//! di quell'utente da un canale `mpsc` ed esegue lo stesso ciclo di gioco del
//! terminale ([`crate::sessione::gioca`]) su un [`CanaleTelegram`].
//!
//! - `mostra` → `sendMessage` (HTML, divisione dei messaggi lunghi);
//! - `scegli` / `conferma` / `chiedi_testo` → messaggio con tastiera inline;
//!   la scelta modifica il messaggio (`editMessageText`) e toglie i pulsanti;
//! - le domande scadono dopo `inattivita_minuti` (= annulla);
//! - `attesa` → al primo giro un messaggio «✍️ Il narratore sta scrivendo…»
//!   (nella lingua del giocatore), poi `sendChatAction typing` ogni 4 s;
//!   `fine_attesa` cancella il messaggio (`deleteMessage`), così la risposta
//!   arriva come messaggio nuovo e fa partire la notifica;
//! - il limitatore globale ([`Limitatore`]) regola le richieste AI parallele;
//! - un panic o un errore nella sessione viene catturato: l'utente riceve un
//!   messaggio e la sessione viene ricreata al messaggio successivo.

use std::collections::VecDeque;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::sync::{Arc, Condvar, Mutex};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use anyhow::{Result, anyhow};
use tracing::{error, warn};

use super::api::{Api, LUNGHEZZA_MAX_MESSAGGIO, Pulsante, Tastiera, dividi_testo, escape_html};
use super::registra;
use crate::ai::{ErroreAi, Narratore, RichiestaAi, RispostaGrezza};
use crate::canale::{Canale, Input, RicercaVoce, Stile, Voce, interpreta_riga, trova_voce};
use crate::comandi::Comando;
use crate::config::Config;
use crate::lingua::Lingua;
use crate::sessione::{self, Ambiente, OpzioniGioco};
use crate::tr;

/// Messaggio a chi scrive mentre l'AI sta generando.
pub fn sto_scrivendo() -> &'static str {
    tr!(
        "Sto ancora scrivendo la risposta precedente…",
        "I am still writing the previous reply…",
        "J'écris encore la réponse précédente…",
        "Todavía estoy escribiendo la respuesta anterior…",
        "Ich schreibe noch an der vorherigen Antwort …"
    )
}
/// Notifica per i pulsanti di domande non più in sospeso.
pub fn scelta_non_valida() -> &'static str {
    tr!(
        "Questa scelta non è più valida.",
        "This choice is no longer valid.",
        "Ce choix n'est plus valable.",
        "Esta opción ya no es válida.",
        "Diese Auswahl ist nicht mehr gültig."
    )
}
/// Messaggio mostrato mentre l'AI prepara la risposta.
pub fn sta_scrivendo() -> &'static str {
    tr!(
        "Il narratore sta scrivendo…",
        "The narrator is writing…",
        "Le narrateur écrit…",
        "El narrador está escribiendo…",
        "Der Erzähler schreibt …"
    )
}

/// Messaggio a chi deve attendere il limitatore.
pub fn in_coda() -> &'static str {
    tr!(
        "Il narratore è occupato con un altro giocatore, sei in coda…",
        "The narrator is busy with another player, you are in the queue…",
        "Le narrateur est occupé avec un autre joueur, vous êtes dans la file d'attente…",
        "El narrador está ocupado con otro jugador, estás en la cola…",
        "Der Erzähler ist mit einem anderen Spieler beschäftigt, du bist in der Warteschlange …"
    )
}

fn annullato() -> &'static str {
    tr!(
        "Annullato",
        "Cancelled",
        "Annulé",
        "Cancelado",
        "Abgebrochen"
    )
}

fn si() -> &'static str {
    tr!("Sì", "Yes", "Oui", "Sí", "Ja")
}

/// Evento inviato dal thread di smistamento alla sessione di un utente.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Evento {
    /// Messaggio di testo.
    Testo(String),
    /// Pulsante premuto.
    Callback {
        /// Id da passare ad `answerCallbackQuery`.
        id: String,
        /// `callback_data`.
        dati: String,
        /// Messaggio che contiene il pulsante.
        message_id: i64,
    },
}

// ---------------------------------------------------------------------------
// Limitatore delle richieste AI parallele
// ---------------------------------------------------------------------------

/// Semaforo con `max_turni_paralleli` posti (Mutex + Condvar).
#[derive(Debug)]
pub struct Limitatore {
    liberi: Mutex<u32>,
    cambiato: Condvar,
}

/// Posto occupato nel limitatore: viene liberato quando esce dall'ambito.
#[derive(Debug)]
pub struct Posto<'a> {
    limitatore: &'a Limitatore,
}

impl Limitatore {
    /// Limitatore con `posti` posti (almeno 1).
    pub fn nuovo(posti: u32) -> Self {
        Limitatore {
            liberi: Mutex::new(posti.max(1)),
            cambiato: Condvar::new(),
        }
    }

    /// Occupa un posto; se non ce ne sono chiama `in_coda` (una volta) e attende.
    pub fn acquisisci(&self, in_coda: impl FnOnce()) -> Posto<'_> {
        let mut liberi = self.liberi.lock().unwrap_or_else(|e| e.into_inner());
        if *liberi == 0 {
            drop(liberi);
            in_coda();
            liberi = self.liberi.lock().unwrap_or_else(|e| e.into_inner());
            while *liberi == 0 {
                liberi = self
                    .cambiato
                    .wait(liberi)
                    .unwrap_or_else(|e| e.into_inner());
            }
        }
        *liberi -= 1;
        Posto { limitatore: self }
    }
}

impl Drop for Posto<'_> {
    fn drop(&mut self) {
        let mut liberi = self
            .limitatore
            .liberi
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        *liberi += 1;
        self.limitatore.cambiato.notify_one();
    }
}

// ---------------------------------------------------------------------------
// Narratore di una sessione: limitatore, stato «occupato», log
// ---------------------------------------------------------------------------

/// Rimette a falso il flag «occupato» anche in caso di panic.
struct Libera<'a>(&'a AtomicBool);

impl Drop for Libera<'_> {
    fn drop(&mut self) {
        self.0.store(false, Ordering::SeqCst);
    }
}

/// Avvolge il narratore condiviso: passa dal limitatore globale, segnala al
/// thread di smistamento che un turno è in corso e registra la durata.
pub struct NarratoreSessione {
    interno: Arc<dyn Narratore>,
    limitatore: Arc<Limitatore>,
    occupato: Arc<AtomicBool>,
    api: Api,
    chat_id: i64,
    utente: i64,
}

/// Avventura e turno letti dallo stato inviato all'AI (solo per il log).
fn avventura_e_turno(richiesta: &RichiestaAi) -> (String, String) {
    let stato = richiesta
        .messaggi
        .iter()
        .filter_map(|m| serde_json::from_str::<serde_json::Value>(&m.contenuto).ok())
        .find_map(|v| v.get("stato").cloned());
    match stato {
        Some(s) => (
            s.get("avventura")
                .and_then(|a| a.as_str())
                .unwrap_or("?")
                .to_string(),
            s.get("turno")
                .map(|t| t.to_string())
                .unwrap_or_else(|| "?".into()),
        ),
        None => ("?".into(), "?".into()),
    }
}

impl Narratore for NarratoreSessione {
    fn turno(&self, richiesta: &RichiestaAi) -> std::result::Result<RispostaGrezza, ErroreAi> {
        self.occupato.store(true, Ordering::SeqCst);
        let _libera = Libera(&self.occupato);
        let _posto = self.limitatore.acquisisci(|| {
            if let Err(e) = self.api.send_message(
                self.chat_id,
                &format!("<i>{}</i>", escape_html(in_coda())),
                true,
                None,
            ) {
                warn!(utente = self.utente, errore = %e, "avviso di coda non inviato");
            }
        });
        let inizio = Instant::now();
        let esito = self.interno.turno(richiesta);
        let (avventura, turno) = avventura_e_turno(richiesta);
        let durata = inizio.elapsed().as_secs_f64();
        match &esito {
            Ok(_) => registra(&format!(
                "utente {} · {avventura} · turno {turno} · risposta AI in {durata:.1} s",
                self.utente
            )),
            Err(e) => registra(&format!(
                "utente {} · {avventura} · turno {turno} · errore AI dopo {durata:.1} s: {e}",
                self.utente
            )),
        }
        esito
    }

    fn descrizione(&self) -> String {
        self.interno.descrizione()
    }

    fn json_mode_attivo(&self) -> Option<bool> {
        self.interno.json_mode_attivo()
    }

    fn diagnostica(&self) -> Vec<String> {
        self.interno.diagnostica()
    }
}

// ---------------------------------------------------------------------------
// Canale Telegram
// ---------------------------------------------------------------------------

/// Esito dell'attesa di una risposta a una domanda con pulsanti.
enum Risposta {
    /// Pulsante della domanda: indice (o `None` = «Annulla»), id della callback.
    Pulsante(Option<usize>, String),
    /// Testo scritto al posto di premere un pulsante.
    Testo(String),
    /// Scadenza della domanda.
    Scaduta,
    /// Il canale degli eventi è chiuso (bot in chiusura).
    Chiuso,
}

/// Implementazione di [`Canale`] su una chat Telegram.
pub struct CanaleTelegram {
    api: Api,
    chat_id: i64,
    eventi: Receiver<Evento>,
    scadenza: Duration,
    /// Contatore delle domande: identifica i pulsanti validi (`callback_data`).
    domanda: u64,
    /// Testi (comandi) scritti durante una domanda, da elaborare dopo.
    rimandati: VecDeque<String>,
    /// Messaggio «sta scrivendo…» dell'attesa in corso (`None` se non inviato).
    segnaposto: Option<i64>,
    /// Il messaggio «sta scrivendo…» è già stato tentato in questa attesa.
    segnaposto_tentato: bool,
}

impl CanaleTelegram {
    /// Nuovo canale per una chat.
    pub fn nuovo(api: Api, chat_id: i64, eventi: Receiver<Evento>, scadenza: Duration) -> Self {
        CanaleTelegram {
            api,
            chat_id,
            eventi,
            scadenza,
            domanda: 0,
            rimandati: VecDeque::new(),
            segnaposto: None,
            segnaposto_tentato: false,
        }
    }

    /// Invia un testo HTML (diviso se lungo); se Telegram rifiuta l'HTML lo
    /// reinvia come testo semplice.
    fn invia_html(&self, html: &str, semplice: &str) -> Result<()> {
        for parte in dividi_testo(html, LUNGHEZZA_MAX_MESSAGGIO) {
            match self.api.send_message(self.chat_id, &parte, true, None) {
                Ok(_) => {}
                Err(e) if e.html_non_valido() => {
                    warn!(errore = %e, "HTML rifiutato da Telegram: invio come testo semplice");
                    for p in dividi_testo(&escape_html(semplice), LUNGHEZZA_MAX_MESSAGGIO) {
                        self.api.send_message(self.chat_id, &p, false, None)?;
                    }
                    return Ok(());
                }
                Err(e) => return Err(e.into()),
            }
        }
        Ok(())
    }

    /// Risponde a una callback ignorando gli errori (la notifica non è essenziale).
    fn rispondi_callback(&self, id: &str, testo: Option<&str>) {
        if let Err(e) = self.api.answer_callback_query(id, testo) {
            warn!(errore = %e, "answerCallbackQuery non riuscita");
        }
    }

    /// Callback di una domanda non più in sospeso: avvisa e toglie i pulsanti.
    fn callback_vecchia(&self, id: &str, message_id: i64) {
        self.rispondi_callback(id, Some(scelta_non_valida()));
        if let Err(e) = self.api.rimuovi_pulsanti(self.chat_id, message_id)
            && !e.non_modificato()
        {
            warn!(errore = %e, "pulsanti non rimossi");
        }
    }

    /// Prefisso dei `callback_data` della domanda corrente.
    fn prefisso(&self) -> String {
        format!("{}:", self.domanda)
    }

    /// Invia una domanda con pulsanti; restituisce l'id del messaggio.
    fn invia_domanda(&mut self, testo_html: &str, etichette: &[String]) -> Result<i64> {
        self.domanda += 1;
        let p = self.prefisso();
        let mut tastiera: Tastiera = etichette
            .iter()
            .enumerate()
            .map(|(i, e)| vec![Pulsante::nuovo(etichetta_pulsante(e), format!("{p}{i}"))])
            .collect();
        tastiera.push(vec![Pulsante::nuovo(
            tr!(
                "✖ Annulla",
                "✖ Cancel",
                "✖ Annuler",
                "✖ Cancelar",
                "✖ Abbrechen"
            ),
            format!("{p}x"),
        )]);
        // Un testo molto lungo va in messaggi separati; i pulsanti sull'ultimo.
        let mut parti = dividi_testo(testo_html, LUNGHEZZA_MAX_MESSAGGIO);
        let ultima = parti.pop().unwrap_or_else(|| "…".to_string());
        for parte in parti {
            self.api.send_message(self.chat_id, &parte, true, None)?;
        }
        let m = self
            .api
            .send_message(self.chat_id, &ultima, true, Some(&tastiera))?;
        Ok(m.message_id)
    }

    /// Chiude una domanda: modifica il messaggio mostrando l'esito e senza pulsanti.
    fn chiudi_domanda(&self, message_id: i64, testo_html: &str, esito: &str) {
        let testo = format!("{testo_html}\n\n➜ <i>{}</i>", escape_html(esito));
        let testo = if super::api::lunghezza_telegram(&testo) > LUNGHEZZA_MAX_MESSAGGIO {
            format!("➜ <i>{}</i>", escape_html(esito))
        } else {
            testo
        };
        if let Err(e) = self
            .api
            .edit_message_text(self.chat_id, message_id, &testo, None)
            && !e.non_modificato()
        {
            warn!(errore = %e, "messaggio della domanda non modificato");
            let _ = self.api.rimuovi_pulsanti(self.chat_id, message_id);
        }
    }

    /// Attende la risposta alla domanda corrente (pulsante, testo o scadenza).
    fn attendi_risposta(&mut self, message_id: i64, n_voci: usize) -> Risposta {
        let fine = Instant::now() + self.scadenza;
        let prefisso = self.prefisso();
        loop {
            let resto = fine.saturating_duration_since(Instant::now());
            if resto.is_zero() {
                return Risposta::Scaduta;
            }
            match self.eventi.recv_timeout(resto) {
                Err(RecvTimeoutError::Timeout) => return Risposta::Scaduta,
                Err(RecvTimeoutError::Disconnected) => return Risposta::Chiuso,
                Ok(Evento::Testo(t)) => return Risposta::Testo(t),
                Ok(Evento::Callback {
                    id,
                    dati,
                    message_id: m,
                }) => {
                    let scelta = dati.strip_prefix(&prefisso).and_then(|r| {
                        if r == "x" {
                            Some(None)
                        } else {
                            r.parse::<usize>().ok().filter(|&i| i < n_voci).map(Some)
                        }
                    });
                    match scelta {
                        Some(s) if m == message_id => return Risposta::Pulsante(s, id),
                        _ => self.callback_vecchia(&id, m),
                    }
                }
            }
        }
    }

    /// Gestisce una risposta di testo a una domanda: i comandi annullano la
    /// domanda e vengono rimandati al ciclo di gioco. Restituisce vero se la
    /// domanda va annullata.
    fn testo_annulla_domanda(&mut self, testo: &str) -> bool {
        if testo.trim_start().starts_with('/') {
            self.rimandati.push_back(testo.to_string());
            return true;
        }
        false
    }

    /// Avvisa che la domanda è scaduta.
    fn scaduta(&self, message_id: i64, testo_html: &str) -> Result<()> {
        self.chiudi_domanda(
            message_id,
            testo_html,
            tr!(
                "Domanda scaduta",
                "Question expired",
                "Question expirée",
                "Pregunta caducada",
                "Frage abgelaufen"
            ),
        );
        let testo = tr!(
            "Tempo scaduto: la domanda è stata annullata (la partita è salvata).",
            "Time is up: the question has been cancelled (the game is saved).",
            "Temps écoulé : la question a été annulée (la partie est sauvegardée).",
            "Se acabó el tiempo: la pregunta se ha cancelado (la partida está guardada).",
            "Die Zeit ist um: Die Frage wurde abgebrochen (das Spiel ist gespeichert)."
        );
        self.invia_html(&format!("<i>{testo}</i>"), testo)
    }

    /// Attende il primo messaggio che (ri)avvia la sessione. `false` = chiusa
    /// (canale chiuso o inattività oltre `limite`).
    pub fn attendi_avvio(&mut self, limite: Option<Duration>) -> bool {
        if self.rimandati.pop_front().is_some() {
            return true;
        }
        loop {
            let evento = match limite {
                Some(l) => match self.eventi.recv_timeout(l) {
                    Ok(e) => e,
                    Err(_) => return false,
                },
                None => match self.eventi.recv() {
                    Ok(e) => e,
                    Err(_) => return false,
                },
            };
            match evento {
                Evento::Testo(_) => return true,
                Evento::Callback { id, message_id, .. } => self.callback_vecchia(&id, message_id),
            }
        }
    }
}

/// Testo di un pulsante: una riga, al massimo 60 caratteri.
fn etichetta_pulsante(testo: &str) -> String {
    let t = testo.split_whitespace().collect::<Vec<_>>().join(" ");
    if t.chars().count() > 60 {
        let mut c: String = t.chars().take(59).collect();
        c.push('…');
        c
    } else {
        t
    }
}

/// Titolo di un menu per Telegram: la prima riga non vuota (le intestazioni di
/// colonna del terminale non servono con i pulsanti).
fn titolo_menu(titolo: &str) -> String {
    titolo
        .lines()
        .map(str::trim)
        .find(|r| !r.is_empty())
        .unwrap_or("")
        .to_string()
}

/// Riquadro finale in HTML con emoji sobrie (§7.5).
fn finale_html(testo: &str) -> String {
    let righe: Vec<String> = testo
        .lines()
        .map(|r| {
            r.trim_matches(|c: char| c.is_whitespace() || "╔╗╚╝║═".contains(c))
                .to_string()
        })
        .filter(|r| !r.is_empty())
        .collect();
    let Some((titolo, resto)) = righe.split_first() else {
        return String::new();
    };
    let emoji = if titolo.starts_with("VITTORIA") || titolo.starts_with("VICTORY") {
        "🏆"
    } else if titolo.starts_with("SCONFITTA") || titolo.starts_with("DEFEAT") {
        "🕯"
    } else {
        "📜"
    };
    let mut t = format!("{emoji} <b>{}</b>", escape_html(titolo));
    for r in resto {
        t.push_str(&format!("\n{}", escape_html(r)));
    }
    t
}

/// Testo HTML per uno stile (§7.5): stanza in grassetto, motore in corsivo.
pub fn formatta(testo: &str, stile: Stile) -> String {
    match stile {
        Stile::Narrazione => escape_html(testo.trim()),
        Stile::Titolo => format!("<b>{}</b>", escape_html(testo.trim())),
        Stile::Motore => format!("<i>{}</i>", escape_html(testo.trim())),
        Stile::Avviso => format!("⚠️ <i>{}</i>", escape_html(testo.trim())),
        Stile::Errore => format!("❗ {}", escape_html(testo.trim())),
        Stile::Finale => finale_html(testo),
        Stile::Elenco => {
            let righe: Vec<&str> = testo.lines().collect();
            // Toglie le righe vuote iniziali/finali e il rientro comune.
            let righe: Vec<&str> = righe
                .iter()
                .copied()
                .skip_while(|r| r.trim().is_empty())
                .collect();
            let fine = righe
                .iter()
                .rposition(|r| !r.trim().is_empty())
                .map_or(0, |i| i + 1);
            let righe = &righe[..fine];
            let rientro = righe
                .iter()
                .filter(|r| !r.trim().is_empty())
                .map(|r| r.len() - r.trim_start().len())
                .min()
                .unwrap_or(0);
            let corpo: Vec<&str> = righe
                .iter()
                .map(|r| {
                    if r.len() >= rientro {
                        &r[rientro..]
                    } else {
                        r.trim_start()
                    }
                })
                .collect();
            format!("<pre>{}</pre>", escape_html(&corpo.join("\n")))
        }
    }
}

impl Canale for CanaleTelegram {
    fn mostra(&mut self, testo: &str, stile: Stile) -> Result<()> {
        if testo.trim().is_empty() {
            return Ok(());
        }
        self.invia_html(&formatta(testo, stile), testo)
    }

    fn scegli(&mut self, titolo: &str, voci: &[Voce]) -> Result<Option<usize>> {
        let attive: Vec<usize> = (0..voci.len()).filter(|&i| voci[i].attiva).collect();
        let mut testo = format!("<b>{}</b>", escape_html(&titolo_menu(titolo)));
        for v in voci {
            match (&v.dettaglio, v.attiva) {
                (Some(d), true) => testo.push_str(&format!(
                    "\n\n<b>{}</b>\n{}",
                    escape_html(&v.etichetta_breve()),
                    escape_html(d)
                )),
                (_, false) => {
                    testo.push_str(&format!("\n\n<s>{}</s>", escape_html(&v.etichetta_breve())))
                }
                _ => {}
            }
        }
        if attive.is_empty() {
            self.invia_html(&testo, titolo)?;
            return Ok(None);
        }
        let etichette: Vec<String> = attive.iter().map(|&i| voci[i].etichetta_breve()).collect();
        let message_id = self.invia_domanda(&testo, &etichette)?;
        loop {
            match self.attendi_risposta(message_id, attive.len()) {
                Risposta::Pulsante(scelta, id) => {
                    self.rispondi_callback(&id, None);
                    let esito = match scelta {
                        Some(i) => etichette[i].clone(),
                        None => annullato().to_string(),
                    };
                    self.chiudi_domanda(message_id, &testo, &esito);
                    return Ok(scelta.map(|i| attive[i]));
                }
                Risposta::Testo(t) => {
                    if self.testo_annulla_domanda(&t) {
                        self.chiudi_domanda(message_id, &testo, annullato());
                        return Ok(None);
                    }
                    match trova_voce(voci, &t) {
                        RicercaVoce::Trovata(i) => {
                            self.chiudi_domanda(message_id, &testo, &voci[i].etichetta_breve());
                            return Ok(Some(i));
                        }
                        _ => {
                            let aiuto = tr!(
                                "Scegli con i pulsanti qui sopra (o scrivi il numero o il nome \
                                 della voce).",
                                "Choose with the buttons above (or type the number or the name \
                                 of the option).",
                                "Choisissez avec les boutons ci-dessus (ou tapez le numéro ou le nom de l'option).",
                                "Elige con los botones de arriba (o escribe el número o el nombre de la opción).",
                                "Wähle mit den Schaltflächen oben (oder gib die Nummer oder den Namen der Option ein)."
                            );
                            self.invia_html(&format!("<i>{aiuto}</i>"), aiuto)?
                        }
                    }
                }
                Risposta::Scaduta => {
                    self.scaduta(message_id, &testo)?;
                    return Ok(None);
                }
                Risposta::Chiuso => return Ok(None),
            }
        }
    }

    fn chiedi_testo(&mut self, domanda: &str, proposta: Option<&str>) -> Result<Option<String>> {
        let mut testo = format!("<b>{}</b>", escape_html(domanda));
        let etichette: Vec<String> = match proposta {
            Some(p) => {
                testo.push_str(&tr!(
                    format!(
                        "\n<i>Scrivi un nome oppure usa quello proposto: «{}».</i>",
                        escape_html(p)
                    ),
                    format!(
                        "\n<i>Type a name or use the suggested one: \"{}\".</i>",
                        escape_html(p)
                    ),
                    format!(
                        "\n<i>Tapez un nom ou utilisez celui proposé : « {} ».</i>",
                        escape_html(p)
                    ),
                    format!(
                        "\n<i>Escribe un nombre o usa el propuesto: «{}».</i>",
                        escape_html(p)
                    ),
                    format!(
                        "\n<i>Gib einen Namen ein oder nimm den vorgeschlagenen: „{}“.</i>",
                        escape_html(p)
                    )
                ));
                vec![format!("✔ {p}")]
            }
            None => {
                testo.push_str(tr!(
                    "\n<i>Scrivi la risposta.</i>",
                    "\n<i>Type your answer.</i>",
                    "\n<i>Tapez votre réponse.</i>",
                    "\n<i>Escribe tu respuesta.</i>",
                    "\n<i>Gib deine Antwort ein.</i>"
                ));
                Vec::new()
            }
        };
        let message_id = self.invia_domanda(&testo, &etichette)?;
        match self.attendi_risposta(message_id, etichette.len()) {
            Risposta::Pulsante(Some(_), id) => {
                self.rispondi_callback(&id, None);
                let p = proposta.unwrap_or_default().to_string();
                self.chiudi_domanda(message_id, &testo, &p);
                Ok(Some(p))
            }
            Risposta::Pulsante(None, id) => {
                self.rispondi_callback(&id, None);
                self.chiudi_domanda(message_id, &testo, annullato());
                Ok(None)
            }
            Risposta::Testo(t) => {
                if self.testo_annulla_domanda(&t) {
                    self.chiudi_domanda(message_id, &testo, annullato());
                    return Ok(None);
                }
                let t = t.trim().to_string();
                self.chiudi_domanda(message_id, &testo, &t);
                Ok(Some(t)
                    .filter(|t| !t.is_empty())
                    .or(proposta.map(str::to_string)))
            }
            Risposta::Scaduta => {
                self.scaduta(message_id, &testo)?;
                Ok(None)
            }
            Risposta::Chiuso => Ok(None),
        }
    }

    fn conferma(&mut self, domanda: &str) -> Result<bool> {
        let testo = format!("<b>{}</b>", escape_html(domanda));
        let etichette = [si().to_string(), crate::lingua::testo_no().to_string()];
        let message_id = self.invia_domanda(&testo, &etichette)?;
        loop {
            match self.attendi_risposta(message_id, etichette.len()) {
                Risposta::Pulsante(scelta, id) => {
                    self.rispondi_callback(&id, None);
                    let vero = scelta == Some(0);
                    self.chiudi_domanda(
                        message_id,
                        &testo,
                        if vero {
                            si()
                        } else {
                            crate::lingua::testo_no()
                        },
                    );
                    return Ok(vero);
                }
                Risposta::Testo(t) => {
                    if self.testo_annulla_domanda(&t) {
                        self.chiudi_domanda(message_id, &testo, crate::lingua::testo_no());
                        return Ok(false);
                    }
                    match t.trim().to_lowercase().as_str() {
                        r if crate::lingua::e_si(r) => {
                            self.chiudi_domanda(message_id, &testo, si());
                            return Ok(true);
                        }
                        r if crate::lingua::e_no(r) => {
                            self.chiudi_domanda(message_id, &testo, crate::lingua::testo_no());
                            return Ok(false);
                        }
                        _ => {
                            let aiuto = tr!(
                                "Rispondi con i pulsanti «Sì» o «No».",
                                "Answer with the \"Yes\" or \"No\" buttons.",
                                "Répondez avec les boutons « Oui » ou « Non ».",
                                "Responde con los botones «Sí» o «No».",
                                "Antworte mit den Schaltflächen „Ja“ oder „Nein“."
                            );
                            self.invia_html(&format!("<i>{aiuto}</i>"), aiuto)?
                        }
                    }
                }
                Risposta::Scaduta => {
                    self.scaduta(message_id, &testo)?;
                    return Ok(false);
                }
                Risposta::Chiuso => return Ok(false),
            }
        }
    }

    fn leggi_azione(&mut self) -> Result<Input> {
        loop {
            let testo = match self.rimandati.pop_front() {
                Some(t) => t,
                None => match self.eventi.recv() {
                    Ok(Evento::Testo(t)) => t,
                    Ok(Evento::Callback { id, message_id, .. }) => {
                        self.callback_vecchia(&id, message_id);
                        continue;
                    }
                    Err(_) => return Ok(Input::Fine),
                },
            };
            // `/start` durante la partita equivale a `/menu` (con l'avviso open source).
            if testo.split_whitespace().next() == Some("/start") {
                self.mostra(&sessione::avviso_open_source(), Stile::Motore)?;
                return Ok(Input::Comando(Comando::Menu));
            }
            match interpreta_riga(&testo) {
                None => {}
                Some(Ok(input)) => return Ok(input),
                Some(Err(motivo)) => self.mostra(&motivo, Stile::Errore)?,
            }
        }
    }

    fn interattivo(&self) -> bool {
        true
    }

    fn attesa(&mut self, _secondi_trascorsi: u64) -> Result<()> {
        // Gli errori degli indicatori non devono interrompere il turno.
        if !self.segnaposto_tentato {
            self.segnaposto_tentato = true;
            let testo = format!("<i>✍️ {}</i>", escape_html(sta_scrivendo()));
            match self.api.send_message(self.chat_id, &testo, true, None) {
                Ok(m) => self.segnaposto = Some(m.message_id),
                Err(e) => warn!(errore = %e, "messaggio «sta scrivendo» non inviato"),
            }
        }
        // Dopo un messaggio del bot Telegram toglie l'indicatore: lo si invia dopo.
        if let Err(e) = self.api.sta_scrivendo(self.chat_id) {
            warn!(errore = %e, "sendChatAction non riuscita");
        }
        Ok(())
    }

    fn fine_attesa(&mut self) -> Result<()> {
        self.segnaposto_tentato = false;
        if let Some(id) = self.segnaposto.take()
            && let Err(e) = self.api.delete_message(self.chat_id, id)
        {
            warn!(errore = %e, "messaggio «sta scrivendo» non cancellato");
        }
        Ok(())
    }

    fn intervallo_attesa(&self) -> Duration {
        Duration::from_secs(4)
    }
}

// ---------------------------------------------------------------------------
// Thread della sessione
// ---------------------------------------------------------------------------

/// Dati condivisi da tutte le sessioni.
pub struct Condiviso {
    /// Configurazione (con la sezione AI già risolta).
    pub config: Config,
    /// Cartella delle avventure (condivisa).
    pub cartella_avventure: PathBuf,
    /// `partite/telegram`: una sottocartella per utente.
    pub cartella_partite: PathBuf,
    /// Narratore creato una sola volta.
    pub narratore: Arc<dyn Narratore>,
    /// Limitatore delle richieste AI parallele.
    pub limitatore: Arc<Limitatore>,
    /// Client dell'API.
    pub api: Api,
    /// Scadenza delle domande in sospeso.
    pub scadenza_domande: Duration,
    /// Dopo quanto tempo senza messaggi una sessione ferma si chiude.
    pub chiusura_inattive: Option<Duration>,
}

/// Una sessione attiva vista dal thread di smistamento.
pub struct GestoreSessione {
    invio: Sender<Evento>,
    /// Vero mentre l'AI sta generando per questo utente.
    pub occupato: Arc<AtomicBool>,
    thread: JoinHandle<()>,
}

impl GestoreSessione {
    /// Crea il thread della sessione per un utente; `lingua_app` è la lingua
    /// dell'app Telegram dell'utente (proposta al primo utilizzo, §7.6).
    pub fn avvia(
        condiviso: Arc<Condiviso>,
        utente: i64,
        chat_id: i64,
        admin: bool,
        lingua_app: Option<Lingua>,
    ) -> Result<Self> {
        let (invio, ricezione) = mpsc::channel();
        let occupato = Arc::new(AtomicBool::new(false));
        let occ = Arc::clone(&occupato);
        let thread = std::thread::Builder::new()
            .name(format!("sessione-{utente}"))
            .spawn(move || {
                // Fino alla scelta della lingua i messaggi usano quella dell'app.
                crate::lingua::imposta(lingua_app.unwrap_or(Lingua::En));
                esegui_sessione(
                    condiviso, utente, chat_id, admin, lingua_app, ricezione, occ,
                )
            })
            .map_err(|e| anyhow!("impossibile creare il thread della sessione: {e}"))?;
        Ok(GestoreSessione {
            invio,
            occupato,
            thread,
        })
    }

    /// Vero se il thread è ancora attivo.
    pub fn attiva(&self) -> bool {
        !self.thread.is_finished()
    }

    /// Vero mentre l'AI sta generando una risposta per questo utente.
    pub fn occupata(&self) -> bool {
        self.occupato.load(Ordering::SeqCst)
    }

    /// Invia un evento; restituisce l'evento se la sessione è terminata.
    pub fn invia(&self, evento: Evento) -> std::result::Result<(), Evento> {
        self.invio.send(evento).map_err(|e| e.0)
    }
}

/// Corpo del thread: cattura errori e panic al confine del thread.
fn esegui_sessione(
    condiviso: Arc<Condiviso>,
    utente: i64,
    chat_id: i64,
    admin: bool,
    lingua_app: Option<Lingua>,
    eventi: Receiver<Evento>,
    occupato: Arc<AtomicBool>,
) {
    let api = condiviso.api.clone();
    let esito = catch_unwind(AssertUnwindSafe(|| {
        ciclo_sessione(
            &condiviso, utente, chat_id, admin, lingua_app, eventi, occupato,
        )
    }));
    let messaggio = match esito {
        Ok(Ok(())) => return,
        Ok(Err(e)) => {
            let testo = api.maschera(&format!("{e:#}"));
            error!(utente, errore = %testo, "errore nella sessione");
            registra(&format!("utente {utente} · errore nella sessione: {testo}"));
            tr!(
                format!(
                    "❗ Si è verificato un errore: {}\n<i>La partita è salvata: scrivi di nuovo \
                     per ripartire.</i>",
                    escape_html(&testo)
                ),
                format!(
                    "❗ An error occurred: {}\n<i>The game is saved: write again to start \
                     over.</i>",
                    escape_html(&testo)
                ),
                format!(
                    "❗ Une erreur s'est produite : {}\n<i>La partie est sauvegardée : écrivez à nouveau pour reprendre.</i>",
                    escape_html(&testo)
                ),
                format!(
                    "❗ Se ha producido un error: {}\n<i>La partida está guardada: escribe de nuevo para volver a empezar.</i>",
                    escape_html(&testo)
                ),
                format!(
                    "❗ Ein Fehler ist aufgetreten: {}\n<i>Das Spiel ist gespeichert: Schreib erneut, um weiterzumachen.</i>",
                    escape_html(&testo)
                )
            )
        }
        Err(panico) => {
            let testo = panico
                .downcast_ref::<&str>()
                .map(|s| s.to_string())
                .or_else(|| panico.downcast_ref::<String>().cloned())
                .unwrap_or_else(|| "errore sconosciuto".into());
            let testo = api.maschera(&testo);
            error!(utente, errore = %testo, "panic nella sessione");
            registra(&format!(
                "utente {utente} · errore interno (panic): {testo}"
            ));
            tr!(
"❗ Si è verificato un errore interno.\n<i>La partita è salvata: scrivi di nuovo \
                 per ripartire.</i>",
"❗ An internal error occurred.\n<i>The game is saved: write again to start \
                 over.</i>",
"❗ Une erreur interne s'est produite.\n<i>La partie est sauvegardée : écrivez à nouveau pour reprendre.</i>",
"❗ Se ha producido un error interno.\n<i>La partida está guardada: escribe de nuevo para volver a empezar.</i>",
"❗ Ein interner Fehler ist aufgetreten.\n<i>Das Spiel ist gespeichert: Schreib erneut, um weiterzumachen.</i>"
)
            .to_string()
        }
    };
    if let Err(e) = api.send_message(chat_id, &messaggio, true, None) {
        warn!(utente, errore = %e, "messaggio d'errore non inviato");
    }
}

fn ciclo_sessione(
    condiviso: &Condiviso,
    utente: i64,
    chat_id: i64,
    admin: bool,
    lingua_app: Option<Lingua>,
    eventi: Receiver<Evento>,
    occupato: Arc<AtomicBool>,
) -> Result<()> {
    let mut canale = CanaleTelegram::nuovo(
        condiviso.api.clone(),
        chat_id,
        eventi,
        condiviso.scadenza_domande,
    );
    let narratore = NarratoreSessione {
        interno: Arc::clone(&condiviso.narratore),
        limitatore: Arc::clone(&condiviso.limitatore),
        occupato,
        api: condiviso.api.clone(),
        chat_id,
        utente,
    };
    let ambiente = Ambiente {
        config: &condiviso.config,
        cartella_avventure: condiviso.cartella_avventure.clone(),
        cartella_partite: condiviso.cartella_partite.join(utente.to_string()),
    };
    let opzioni = OpzioniGioco {
        debug: admin,
        lingua_rilevata: lingua_app,
        ..Default::default()
    };
    while canale.attendi_avvio(condiviso.chiusura_inattive) {
        registra(&format!("utente {utente} · inizio sessione"));
        // Al primo utilizzo chiede la lingua (proponendo quella dell'app) prima
        // del benvenuto; poi vale la preferenza salvata.
        sessione::prepara_lingua(&ambiente, &opzioni, &mut canale)?;
        canale.mostra(
            tr!(
"Benvenuto! Scegli un'avventura con i pulsanti. Durante il gioco scrivi le tue \
                 azioni come messaggi; /help mostra i comandi, /language cambia lingua.",
"Welcome! Choose an adventure with the buttons. While playing, write your \
                 actions as messages; /help shows the commands, /language changes language.",
"Bienvenue ! Choisissez une aventure avec les boutons. Pendant la partie, écrivez vos actions sous forme de messages ; /help affiche les commandes, /language change la langue.",
"¡Bienvenido! Elige una aventura con los botones. Mientras juegas, escribe tus acciones como mensajes; /help muestra los comandos, /language cambia el idioma.",
"Willkommen! Wähle ein Abenteuer mit den Schaltflächen. Schreib beim Spielen deine Aktionen als Nachrichten; /help zeigt die Befehle, /language ändert die Sprache."
),
            Stile::Motore,
        )?;
        sessione::gioca(&ambiente, &opzioni, &mut canale, &narratore)?;
        canale.mostra(
            tr!(
                "Partita chiusa. Scrivi /start (o un messaggio qualsiasi) per ricominciare.",
                "Game closed. Write /start (or any message) to start again.",
                "Partie fermée. Écrivez /start (ou n'importe quel message) pour recommencer.",
                "Partida cerrada. Escribe /start (o cualquier mensaje) para volver a empezar.",
                "Spiel geschlossen. Schreib /start (oder irgendeine Nachricht), um neu zu beginnen."
            ),
            Stile::Motore,
        )?;
        registra(&format!("utente {utente} · fine sessione"));
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn limitatore_un_posto() {
        let l = Arc::new(Limitatore::nuovo(1));
        let p = l.acquisisci(|| panic!("non deve attendere"));
        let l2 = Arc::clone(&l);
        let (tx, rx) = mpsc::channel();
        let h = std::thread::spawn(move || {
            let mut in_coda = false;
            let _p = l2.acquisisci(|| in_coda = true);
            tx.send(in_coda).unwrap();
        });
        std::thread::sleep(Duration::from_millis(100));
        assert!(rx.try_recv().is_err(), "il secondo non deve passare");
        drop(p);
        assert!(rx.recv_timeout(Duration::from_secs(2)).unwrap());
        h.join().unwrap();
    }

    #[test]
    fn formattazione_html() {
        assert_eq!(
            formatta("La <cantina>", Stile::Titolo),
            "<b>La &lt;cantina&gt;</b>"
        );
        assert_eq!(formatta("ok & via", Stile::Motore), "<i>ok &amp; via</i>");
        let elenco = formatta("\n  COMANDI\n\n  /help   aiuto\n", Stile::Elenco);
        assert_eq!(elenco, "<pre>COMANDI\n\n/help   aiuto</pre>");
        let riquadro = "\n  ╔════════════════════════════╗\n  ║  VITTORIA!                 ║\n  \
                        ║  Turni: 3 · Punteggio: 7   ║\n  ╚════════════════════════════╝";
        let finale = formatta(riquadro, Stile::Finale);
        assert!(finale.starts_with("🏆 <b>VITTORIA!</b>"), "{finale}");
        assert!(finale.contains("Turni: 3 · Punteggio: 7"));
        assert!(!finale.contains('║'));
    }

    #[test]
    fn titoli_ed_etichette() {
        assert_eq!(
            titolo_menu("\nSALVATAGGI — X\n\n  Nome  Turno"),
            "SALVATAGGI — X"
        );
        assert_eq!(etichetta_pulsante("a    b"), "a b");
        assert_eq!(etichetta_pulsante(&"x".repeat(80)).chars().count(), 60);
    }
}
