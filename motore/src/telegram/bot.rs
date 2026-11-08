//! Il bot Telegram (§7.5): avvio (`getMe`, `deleteWebhook`, `setMyCommands`),
//! ciclo di long polling con `getUpdates`, smistamento degli aggiornamenti alle
//! sessioni per utente, controllo degli accessi, comandi amministratore,
//! persistenza dell'offset e chiusura ordinata.
//!
//! Il polling gira in un thread a parte che consegna ogni lotto al thread
//! principale e attende la conferma prima di chiedere il lotto successivo:
//! così l'offset avanza (e viene salvato) solo dopo che gli aggiornamenti sono
//! stati smistati, e un arresto con Ctrl+C non perde messaggi.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, RecvTimeoutError};
use std::time::{Duration, Instant};

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use tracing::{debug, warn};

use super::api::Utente;
use super::api::{Api, CallbackQuery, Messaggio, Update, escape_html};
use super::registra;
use super::sessione::{
    Condiviso, Evento, GestoreSessione, Limitatore, scelta_non_valida, sto_scrivendo,
};
use crate::ai::Narratore;
use crate::comandi::{self, Comando};
use crate::config::Config;
use crate::lingua::{self, Lingua};
use crate::stato::scrivi_json_atomico;
use crate::tr;

/// Risposta a chi non è ammesso (nessun'altra informazione).
pub fn messaggio_privato(id: i64) -> String {
    tr!(
        format!(
            "Questo bot è privato. Il tuo ID Telegram è {id}: chiedi al proprietario di \
             aggiungerlo."
        ),
        format!("This bot is private. Your Telegram ID is {id}: ask the owner to add it."),
        format!(
            "Ce bot est privé. Votre identifiant Telegram est {id} : demandez au propriétaire de l'ajouter."
        ),
        format!(
            "Este bot es privado. Tu ID de Telegram es {id}: pide al propietario que lo añada."
        ),
        format!(
            "Dieser Bot ist privat. Deine Telegram-ID ist {id}: Bitte den Betreiber, sie hinzuzufügen."
        )
    )
}

/// Risposta ai messaggi da gruppi e canali.
pub fn solo_private() -> &'static str {
    tr!(
        "Questo bot funziona solo nelle chat private: scrivimi in un messaggio diretto.",
        "This bot works only in private chats: write me a direct message.",
        "Ce bot ne fonctionne que dans les conversations privées : écrivez-moi un message direct.",
        "Este bot solo funciona en chats privados: escríbeme un mensaje directo.",
        "Dieser Bot funktioniert nur in privaten Chats: Schreib mir eine Direktnachricht."
    )
}

/// Risposta ai messaggi non testuali.
pub fn solo_testo() -> &'static str {
    tr!(
        "Scrivi la tua azione come testo.",
        "Write your action as text.",
        "Écrivez votre action sous forme de texte.",
        "Escribe tu acción como texto.",
        "Schreib deine Aktion als Text."
    )
}

/// Lingua dell'app Telegram di un utente, se supportata.
pub fn lingua_app(utente: Option<&Utente>) -> Option<Lingua> {
    utente
        .and_then(|u| u.language_code.as_deref())
        .and_then(Lingua::da_codice)
}

/// Testo dell'errore quando il token manca.
pub fn errore_token_mancante(variabile: &str) -> String {
    format!(
        "manca il token del bot Telegram: la variabile d'ambiente {variabile} non è impostata.\n\
         \n\
         Per ottenere un token: in Telegram scrivi a @BotFather, invia /newbot, scegli nome e \
         username del bot e copia il token che ti restituisce. Poi impostalo, ad esempio:\n\
         \x20 PowerShell:   $env:{variabile} = \"123456789:AA...\"\n\
         \x20 Linux/macOS:  export {variabile}=\"123456789:AA...\"\n\
         \n\
         Istruzioni complete nel manuale (docs/MANUALE.md, capitolo sul bot Telegram)."
    )
}

/// Legge il token dalla variabile indicata da `telegram.token_env`.
pub fn leggi_token(config: &Config, ambiente: &dyn Fn(&str) -> Option<String>) -> Result<String> {
    let variabile = &config.telegram.token_env;
    match ambiente(variabile).map(|t| t.trim().to_string()) {
        Some(t) if !t.is_empty() => Ok(t),
        _ => bail!("{}", errore_token_mancante(variabile)),
    }
}

/// Cartella delle partite Telegram: `telegram.cartella_partite`, relativa alla
/// cartella che contiene le avventure (come `motore.cartella_partite`).
pub fn cartella_partite_telegram(cartella_avventure: &Path, config: &Config) -> PathBuf {
    let c = &config.telegram.cartella_partite;
    if c.is_absolute() {
        return c.clone();
    }
    cartella_avventure
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_default()
        .join(c)
}

// ---------------------------------------------------------------------------
// Accesso
// ---------------------------------------------------------------------------

/// Contenuto di `utenti.json`.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct FileUtenti {
    /// Utenti aggiunti con `codice_invito` o `/ammetti`.
    #[serde(default)]
    pub ammessi: Vec<i64>,
    /// Utenti revocati con `/revoca` (anche se presenti nel config).
    #[serde(default)]
    pub revocati: Vec<i64>,
}

/// Regole di accesso (§7.5).
#[derive(Debug)]
pub struct Accesso {
    da_config: Vec<i64>,
    amministratori: Vec<i64>,
    codice_invito: Option<String>,
    file: FileUtenti,
    percorso: PathBuf,
}

impl Accesso {
    /// Carica `utenti.json` (assente = vuoto; illeggibile = errore).
    pub fn carica(config: &Config, percorso: PathBuf) -> Result<Self> {
        let file = if percorso.is_file() {
            let testo = fs::read_to_string(&percorso)
                .with_context(|| format!("impossibile leggere «{}»", percorso.display()))?;
            serde_json::from_str(&testo)
                .with_context(|| format!("«{}» non è valido", percorso.display()))?
        } else {
            FileUtenti::default()
        };
        Ok(Accesso {
            da_config: config.telegram.utenti_ammessi.clone(),
            amministratori: config.telegram.amministratori.clone(),
            codice_invito: config
                .telegram
                .codice_invito
                .clone()
                .filter(|c| !c.trim().is_empty()),
            file,
            percorso,
        })
    }

    /// Vero per gli amministratori.
    pub fn amministratore(&self, id: i64) -> bool {
        self.amministratori.contains(&id)
    }

    /// Vero se l'utente può giocare.
    pub fn ammesso(&self, id: i64) -> bool {
        self.amministratore(id)
            || ((self.da_config.contains(&id) || self.file.ammessi.contains(&id))
                && !self.file.revocati.contains(&id))
    }

    /// Vero se l'utente è stato revocato: il codice d'invito non lo riammette
    /// (solo un amministratore con `/ammetti`).
    pub fn revocato(&self, id: i64) -> bool {
        self.file.revocati.contains(&id)
    }

    /// Vero se `codice` è il codice d'invito configurato.
    pub fn codice_valido(&self, codice: &str) -> bool {
        self.codice_invito
            .as_deref()
            .is_some_and(|c| c.trim() == codice.trim())
    }

    fn salva(&self) -> Result<()> {
        scrivi_json_atomico(&self.percorso, &self.file)
    }

    /// Aggiunge un utente e salva `utenti.json`.
    pub fn ammetti(&mut self, id: i64) -> Result<()> {
        self.file.revocati.retain(|&r| r != id);
        if !self.file.ammessi.contains(&id) {
            self.file.ammessi.push(id);
        }
        self.salva()
    }

    /// Revoca un utente e salva `utenti.json`.
    pub fn revoca(&mut self, id: i64) -> Result<()> {
        self.file.ammessi.retain(|&a| a != id);
        // Sempre tra i revocati, anche se era entrato con il codice d'invito:
        // così il codice non lo riammette (solo /ammetti di un amministratore).
        if !self.file.revocati.contains(&id) {
            self.file.revocati.push(id);
        }
        self.salva()
    }

    /// Utenti ammessi (amministratori compresi), ordinati.
    pub fn elenco(&self) -> Vec<i64> {
        let mut v: Vec<i64> = self
            .da_config
            .iter()
            .chain(&self.file.ammessi)
            .chain(&self.amministratori)
            .copied()
            .filter(|&id| self.ammesso(id))
            .collect();
        v.sort_unstable();
        v.dedup();
        v
    }
}

// ---------------------------------------------------------------------------
// Offset
// ---------------------------------------------------------------------------

#[derive(Debug, Default, Serialize, Deserialize)]
struct FileOffset {
    offset: i64,
}

/// Legge `offset.json` (assente o illeggibile = 0).
pub fn carica_offset(percorso: &Path) -> i64 {
    fs::read_to_string(percorso)
        .ok()
        .and_then(|t| serde_json::from_str::<FileOffset>(&t).ok())
        .map_or(0, |f| f.offset)
}

/// Scrive `offset.json` in modo atomico.
pub fn salva_offset(percorso: &Path, offset: i64) -> Result<()> {
    scrivi_json_atomico(percorso, &FileOffset { offset })
}

// ---------------------------------------------------------------------------
// Bot
// ---------------------------------------------------------------------------

/// Opzioni di esecuzione (i valori dei test differiscono da quelli reali).
#[derive(Debug, Clone)]
pub struct OpzioniBot {
    /// Timeout del long polling (50 s).
    pub timeout_polling: u64,
    /// Scadenza delle domande in sospeso (`inattivita_minuti`).
    pub scadenza_domande: Duration,
    /// Chiusura delle sessioni ferme senza messaggi (facoltativa).
    pub chiusura_inattive: Option<Duration>,
    /// Attesa massima dei turni in corso alla chiusura (60 s).
    pub attesa_chiusura: Duration,
    /// Attesa massima tra i tentativi dopo un errore di rete (60 s).
    pub backoff_max: Duration,
    /// Diventa vero per fermare il bot (Ctrl+C).
    pub ferma: Arc<AtomicBool>,
}

impl OpzioniBot {
    /// Valori reali derivati dalla configurazione.
    pub fn da_config(config: &Config, ferma: Arc<AtomicBool>) -> Self {
        let scadenza = Duration::from_secs(config.telegram.inattivita_minuti.max(1) * 60);
        OpzioniBot {
            timeout_polling: 50,
            scadenza_domande: scadenza,
            chiusura_inattive: Some(scadenza * 4),
            attesa_chiusura: Duration::from_secs(60),
            backoff_max: Duration::from_secs(60),
            ferma,
        }
    }
}

/// Tutto ciò che serve per avviare il bot.
pub struct DatiBot {
    /// Configurazione (AI già risolta).
    pub config: Config,
    /// Cartella delle avventure.
    pub cartella_avventure: PathBuf,
    /// Cartella `partite/telegram`.
    pub cartella_partite: PathBuf,
    /// Narratore, creato una sola volta.
    pub narratore: Arc<dyn Narratore>,
    /// Client dell'API.
    pub api: Api,
}

/// Normalizza un comando: `/start@NomeBot arg` → `/start arg`.
pub fn normalizza_comando(testo: &str) -> String {
    let t = testo.trim();
    if !t.starts_with('/') {
        return t.to_string();
    }
    let (primo, resto) = t.split_once(char::is_whitespace).unwrap_or((t, ""));
    let primo = primo.split('@').next().unwrap_or(primo);
    let resto = resto.trim();
    if resto.is_empty() {
        primo.to_string()
    } else {
        format!("{primo} {resto}")
    }
}

/// Codici d'invito sbagliati consentiti prima del blocco.
pub const TENTATIVI_INVITO: u32 = 5;
/// Durata del blocco dopo troppi codici d'invito sbagliati.
pub const BLOCCO_INVITO: Duration = Duration::from_secs(15 * 60);

/// Conteggio dei codici d'invito sbagliati per utente (solo in memoria): dopo
/// [`TENTATIVI_INVITO`] errori l'utente è bloccato per [`BLOCCO_INVITO`] e nel
/// frattempo nessun codice, nemmeno quello giusto, viene accettato.
#[derive(Debug, Default)]
pub struct TentativiInvito {
    per_utente: HashMap<i64, (u32, Option<Instant>)>,
}

impl TentativiInvito {
    /// Vero se l'utente è bloccato in questo momento (un blocco scaduto si azzera).
    pub fn bloccato(&mut self, id: i64, ora: Instant) -> bool {
        match self.per_utente.get(&id) {
            Some((_, Some(fino))) if ora < *fino => true,
            Some((_, Some(_))) => {
                self.per_utente.remove(&id);
                false
            }
            _ => false,
        }
    }

    /// Registra un codice sbagliato; vero se da ora l'utente è bloccato.
    pub fn sbagliato(&mut self, id: i64, ora: Instant) -> bool {
        let voce = self.per_utente.entry(id).or_insert((0, None));
        voce.0 += 1;
        if voce.0 >= TENTATIVI_INVITO {
            *voce = (0, Some(ora + BLOCCO_INVITO));
            return true;
        }
        false
    }

    /// Codice accettato: si dimenticano gli errori.
    pub fn riuscito(&mut self, id: i64) {
        self.per_utente.remove(&id);
    }
}

struct Bot {
    api: Api,
    condiviso: Arc<Condiviso>,
    accesso: Accesso,
    sessioni: HashMap<i64, GestoreSessione>,
    tentativi_invito: TentativiInvito,
}

impl Bot {
    fn invia(&self, chat_id: i64, testo_html: &str) {
        if let Err(e) = self.api.send_message(chat_id, testo_html, true, None) {
            warn!(chat_id, errore = %e, "messaggio non inviato");
        }
    }

    fn gestisci(&mut self, u: Update) {
        if let Some(m) = u.message {
            self.gestisci_messaggio(m);
        } else if let Some(c) = u.callback_query {
            self.gestisci_callback(c);
        } else {
            debug!(update = u.update_id, "aggiornamento ignorato");
        }
    }

    /// Lingua dei messaggi del bot per un utente (§7.6): preferenza salvata →
    /// `AVVENTURA_LINGUA` o `interfaccia.lingua` → lingua dell'app → inglese.
    fn lingua_utente(&self, utente: i64, app: Option<Lingua>) -> Lingua {
        lingua::preferita(&self.condiviso.cartella_partite.join(utente.to_string()))
            .or_else(lingua::da_variabile)
            .or_else(|| Lingua::da_codice(&self.condiviso.config.interfaccia.lingua))
            .or(app)
            .unwrap_or(Lingua::En)
    }

    fn gestisci_messaggio(&mut self, m: Messaggio) {
        let chat = m.chat.id;
        let app = lingua_app(m.from.as_ref());
        let utente = m.from.as_ref().map_or(chat, |u| u.id);
        lingua::imposta(self.lingua_utente(utente, app));
        if !m.chat.privata() {
            registra(&format!("messaggio da una chat «{}» ignorato", m.chat.tipo));
            self.invia(chat, &escape_html(solo_private()));
            return;
        }
        let testo = m.text.as_deref().map(normalizza_comando);
        if !self.accesso.ammesso(utente) {
            let codice = testo
                .as_deref()
                .and_then(|t| t.strip_prefix("/start "))
                .map(str::trim)
                .filter(|c| !c.is_empty());
            let ora = Instant::now();
            match codice {
                Some(_) if self.tentativi_invito.bloccato(utente, ora) => {
                    registra(&format!(
                        "utente {utente}: codice d'invito ignorato (troppi tentativi sbagliati)"
                    ));
                    self.invia(chat, &escape_html(&messaggio_privato(utente)));
                    return;
                }
                Some(c) if self.accesso.codice_valido(c) && self.accesso.revocato(utente) => {
                    registra(&format!(
                        "utente {utente} revocato: il codice d'invito non lo riammette"
                    ));
                    self.invia(chat, &escape_html(&messaggio_privato(utente)));
                    return;
                }
                Some(c) if self.accesso.codice_valido(c) => {
                    self.tentativi_invito.riuscito(utente);
                    if let Err(e) = self.accesso.ammetti(utente) {
                        warn!(errore = %e, "utenti.json non salvato");
                    }
                    registra(&format!("utente {utente} ammesso con il codice d'invito"));
                    self.invia(
                        chat,
                        tr!(
                            "<i>Codice d'invito accettato: ora puoi giocare.</i>",
                            "<i>Invitation code accepted: you can play now.</i>",
                            "<i>Code d'invitation accepté : vous pouvez jouer.</i>",
                            "<i>Código de invitación aceptado: ya puedes jugar.</i>",
                            "<i>Einladungscode akzeptiert: Du kannst jetzt spielen.</i>"
                        ),
                    );
                }
                altro => {
                    if altro.is_some() && self.tentativi_invito.sbagliato(utente, ora) {
                        registra(&format!(
                            "utente {utente}: troppi codici d'invito sbagliati, bloccato per {} minuti",
                            BLOCCO_INVITO.as_secs() / 60
                        ));
                    }
                    registra(&format!("utente {utente} non ammesso"));
                    self.invia(chat, &escape_html(&messaggio_privato(utente)));
                    return;
                }
            }
        }
        let Some(testo) = testo else {
            self.invia(chat, &escape_html(solo_testo()));
            return;
        };
        let admin = self.accesso.amministratore(utente);
        if testo.starts_with('/') {
            if admin && self.comando_amministratore(chat, &testo) {
                return;
            }
            if !admin && matches!(comandi::analizza(&testo), Comando::Stato | Comando::Grezzo) {
                self.invia(
                    chat,
                    tr!(
                        "<i>Comando riservato agli amministratori.</i>",
                        "<i>Command reserved to administrators.</i>",
                        "<i>Commande réservée aux administrateurs.</i>",
                        "<i>Comando reservado a los administradores.</i>",
                        "<i>Befehl nur für Administratoren.</i>"
                    ),
                );
                return;
            }
        }
        self.inoltra(utente, chat, Evento::Testo(testo), app);
    }

    /// Comandi solo per amministratori: `/utenti` (`/users`), `/ammetti ID`
    /// (`/allow`), `/revoca ID` (`/revoke`).
    /// Restituisce falso se il testo non è uno di questi comandi.
    fn comando_amministratore(&mut self, chat: i64, testo: &str) -> bool {
        let (nome, arg) = testo.split_once(' ').unwrap_or((testo, ""));
        let id = arg.trim().parse::<i64>();
        let nome = match nome {
            "/users" => "/utenti",
            "/allow" => "/ammetti",
            "/revoke" => "/revoca",
            altro => altro,
        };
        let risposta = match nome {
            "/utenti" => {
                let elenco: Vec<String> = self
                    .accesso
                    .elenco()
                    .iter()
                    .map(|&id| {
                        if self.accesso.amministratore(id) {
                            tr!(
                                format!("{id} (amministratore)"),
                                format!("{id} (administrator)"),
                                format!("{id} (administrateur)"),
                                format!("{id} (administrador)"),
                                format!("{id} (Administrator)")
                            )
                        } else {
                            id.to_string()
                        }
                    })
                    .collect();
                if elenco.is_empty() {
                    tr!(
                        "Nessun utente ammesso.",
                        "No allowed users.",
                        "Aucun utilisateur autorisé.",
                        "Ningún usuario autorizado.",
                        "Keine zugelassenen Benutzer."
                    )
                    .to_string()
                } else {
                    format!(
                        "{}\n{}",
                        tr!(
                            "Utenti ammessi:",
                            "Allowed users:",
                            "Utilisateurs autorisés :",
                            "Usuarios autorizados:",
                            "Zugelassene Benutzer:"
                        ),
                        elenco.join("\n")
                    )
                }
            }
            "/ammetti" | "/revoca" => match id {
                Err(_) => tr!(
                    format!("Uso: {nome} ID (ID numerico Telegram)."),
                    format!("Usage: {nome} ID (numeric Telegram ID)."),
                    format!("Usage : {nome} ID (identifiant Telegram numérique)."),
                    format!("Uso: {nome} ID (ID numérico de Telegram)."),
                    format!("Verwendung: {nome} ID (numerische Telegram-ID).")
                ),
                Ok(id) if nome == "/revoca" && self.accesso.amministratore(id) => tr!(
                    format!("{id} è un amministratore: toglilo da «amministratori» nel config."),
                    format!(
                        "{id} is an administrator: remove it from \"amministratori\" in the config."
                    ),
                    format!(
                        "{id} est administrateur : retirez-le de « amministratori » dans la configuration."
                    ),
                    format!(
                        "{id} es administrador: quítalo de «amministratori» en la configuración."
                    ),
                    format!(
                        "{id} ist Administrator: Entferne die ID aus „amministratori“ in der Konfiguration."
                    )
                ),
                Ok(id) => {
                    let esito = if nome == "/ammetti" {
                        self.accesso.ammetti(id)
                    } else {
                        self.accesso.revoca(id)
                    };
                    match esito {
                        Ok(()) if nome == "/ammetti" => {
                            registra(&format!("utente {id} ammesso da un amministratore"));
                            tr!(
                                format!("Utente {id} ammesso."),
                                format!("User {id} allowed."),
                                format!("Utilisateur {id} autorisé."),
                                format!("Usuario {id} autorizado."),
                                format!("Benutzer {id} zugelassen.")
                            )
                        }
                        Ok(()) => {
                            registra(&format!("utente {id} revocato da un amministratore"));
                            tr!(
                                format!("Accesso revocato all'utente {id}."),
                                format!("Access revoked for user {id}."),
                                format!("Accès révoqué pour l'utilisateur {id}."),
                                format!("Acceso revocado al usuario {id}."),
                                format!("Zugang für Benutzer {id} entzogen.")
                            )
                        }
                        Err(e) => tr!(
                            format!("Errore: {e:#}"),
                            format!("Error: {e:#}"),
                            format!("Erreur : {e:#}"),
                            format!("Error: {e:#}"),
                            format!("Fehler: {e:#}")
                        ),
                    }
                }
            },
            _ => return false,
        };
        self.invia(chat, &format!("<i>{}</i>", escape_html(&risposta)));
        true
    }

    fn gestisci_callback(&mut self, c: CallbackQuery) {
        let app = lingua_app(Some(&c.from));
        lingua::imposta(self.lingua_utente(c.from.id, app));
        let Some(m) = &c.message else {
            let _ = self
                .api
                .answer_callback_query(&c.id, Some(scelta_non_valida()));
            return;
        };
        let chat = m.chat.id;
        let message_id = m.message_id;
        if !m.chat.privata() {
            let _ = self.api.answer_callback_query(&c.id, Some(solo_private()));
            return;
        }
        let utente = c.from.id;
        if !self.accesso.ammesso(utente) {
            let _ = self
                .api
                .answer_callback_query(&c.id, Some(&messaggio_privato(utente)));
            return;
        }
        let attiva = self
            .sessioni
            .get(&utente)
            .is_some_and(GestoreSessione::attiva);
        if !attiva {
            let _ = self
                .api
                .answer_callback_query(&c.id, Some(scelta_non_valida()));
            let _ = self.api.rimuovi_pulsanti(chat, message_id);
            return;
        }
        let evento = Evento::Callback {
            id: c.id,
            dati: c.data.unwrap_or_default(),
            message_id,
        };
        self.inoltra(utente, chat, evento, app);
    }

    /// Consegna un evento alla sessione dell'utente (creandola se serve).
    fn inoltra(&mut self, utente: i64, chat: i64, evento: Evento, app: Option<Lingua>) {
        let mut evento = evento;
        if let Some(s) = self.sessioni.get(&utente)
            && s.attiva()
        {
            if s.occupata() {
                match &evento {
                    Evento::Testo(_) => {
                        self.invia(chat, &format!("<i>{}</i>", escape_html(sto_scrivendo())))
                    }
                    Evento::Callback { id, .. } => {
                        let _ = self.api.answer_callback_query(id, Some(sto_scrivendo()));
                    }
                }
                return;
            }
            match s.invia(evento) {
                Ok(()) => return,
                Err(e) => evento = e,
            }
        }
        if let Evento::Callback { id, message_id, .. } = &evento {
            let _ = self
                .api
                .answer_callback_query(id, Some(scelta_non_valida()));
            let _ = self.api.rimuovi_pulsanti(chat, *message_id);
            return;
        }
        let admin = self.accesso.amministratore(utente);
        match GestoreSessione::avvia(Arc::clone(&self.condiviso), utente, chat, admin, app) {
            Ok(s) => {
                if s.invia(evento).is_err() {
                    warn!(utente, "la nuova sessione si è chiusa subito");
                }
                self.sessioni.insert(utente, s);
            }
            Err(e) => {
                warn!(utente, errore = %e, "sessione non creata");
                self.invia(
                    chat,
                    tr!(
"❗ Il bot non riesce a creare la sessione: riprova più tardi.",
"❗ The bot cannot create the session: try again later.",
"❗ Le bot ne peut pas créer la session : réessayez plus tard.",
"❗ El bot no puede crear la sesión: inténtalo más tarde.",
"❗ Der Bot kann die Sitzung nicht erstellen: Versuch es später noch einmal."
),
                );
            }
        }
    }

    fn turni_in_corso(&self) -> usize {
        self.sessioni
            .values()
            .filter(|s| s.attiva() && s.occupata())
            .count()
    }
}

/// Primo passo dell'avvio: `getMe`, `deleteWebhook`, `setMyCommands`.
/// Restituisce il nome del bot (`@username`).
pub fn prepara(api: &Api) -> Result<String> {
    let io = api
        .get_me()
        .context("il token non è valido o Telegram non è raggiungibile (getMe)")?;
    api.delete_webhook().context("deleteWebhook non riuscita")?;
    // Comandi in inglese per tutti, e nella lingua dell'app per chi la usa in
    // italiano, francese, spagnolo o tedesco (§7.6).
    for lingua in crate::lingua::TUTTE {
        let codice = (lingua != Lingua::En).then(|| lingua.codice());
        let voci = comandi::comandi_bot(lingua);
        let voci: Vec<(&str, &str)> = voci.iter().map(|(c, d)| (*c, d.as_str())).collect();
        if let Err(e) = api.set_my_commands(&voci, codice) {
            warn!(errore = %e, "setMyCommands non riuscita");
            registra(&format!("avviso: comandi non registrati ({e})"));
        }
    }
    Ok(match io.username {
        Some(u) => format!("@{u}"),
        None => io.first_name,
    })
}

/// Esegue il bot finché `opzioni.ferma` non diventa vero.
pub fn esegui(dati: DatiBot, opzioni: OpzioniBot) -> Result<()> {
    let DatiBot {
        config,
        cartella_avventure,
        cartella_partite,
        narratore,
        api,
    } = dati;
    fs::create_dir_all(&cartella_partite).with_context(|| {
        format!(
            "impossibile creare la cartella delle partite «{}»",
            cartella_partite.display()
        )
    })?;
    let nome = prepara(&api)?;
    let percorso_offset = cartella_partite.join("offset.json");
    let accesso = Accesso::carica(&config, cartella_partite.join("utenti.json"))?;
    let mut offset = carica_offset(&percorso_offset);
    let n_ammessi = accesso.elenco().len();
    let profilo = config
        .ai
        .profilo
        .clone()
        .unwrap_or_else(|| "[ai] (predefinito)".into());
    let descrizione_ai = narratore.descrizione();
    let condiviso = Arc::new(Condiviso {
        limitatore: Arc::new(Limitatore::nuovo(config.telegram.max_turni_paralleli)),
        config,
        cartella_avventure,
        cartella_partite,
        narratore,
        api: api.clone(),
        scadenza_domande: opzioni.scadenza_domande,
        chiusura_inattive: opzioni.chiusura_inattive,
    });
    registra(&format!(
        "Bot {nome} avviato · profilo AI: {profilo} ({descrizione_ai})"
    ));
    registra(&format!(
        "Utenti ammessi: {n_ammessi} · richieste AI parallele: {}",
        condiviso.config.telegram.max_turni_paralleli
    ));
    registra("In ascolto… (Ctrl+C per fermare)");

    let mut bot = Bot {
        api: api.clone(),
        condiviso,
        accesso,
        sessioni: HashMap::new(),
        tentativi_invito: TentativiInvito::default(),
    };

    // Thread di polling: consegna un lotto e attende la conferma con il nuovo offset.
    let (invio_lotti, lotti) = mpsc::channel::<Vec<Update>>();
    let (invio_conferme, conferme) = mpsc::channel::<i64>();
    {
        let api = api.clone();
        let ferma = Arc::clone(&opzioni.ferma);
        let timeout = opzioni.timeout_polling;
        let backoff_max = opzioni.backoff_max;
        let mut offset_polling = offset;
        std::thread::Builder::new()
            .name("polling".into())
            .spawn(move || {
                let mut attesa = Duration::from_secs(1);
                while !ferma.load(Ordering::SeqCst) {
                    match api.get_updates(offset_polling, timeout) {
                        Ok(lotto) => {
                            attesa = Duration::from_secs(1);
                            if lotto.is_empty() {
                                continue;
                            }
                            if invio_lotti.send(lotto).is_err() {
                                return;
                            }
                            match conferme.recv() {
                                Ok(o) => offset_polling = o,
                                Err(_) => return,
                            }
                        }
                        Err(e) => {
                            registra(&format!(
                                "errore di polling: {e}; nuovo tentativo tra {} s",
                                attesa.as_secs()
                            ));
                            let fine = Instant::now() + attesa;
                            while Instant::now() < fine && !ferma.load(Ordering::SeqCst) {
                                std::thread::sleep(Duration::from_millis(100));
                            }
                            attesa = (attesa * 2).min(backoff_max);
                        }
                    }
                }
            })
            .context("impossibile avviare il thread di polling")?;
    }

    while !opzioni.ferma.load(Ordering::SeqCst) {
        match lotti.recv_timeout(Duration::from_millis(200)) {
            Ok(lotto) => {
                for u in lotto {
                    let id = u.update_id;
                    bot.gestisci(u);
                    offset = offset.max(id + 1);
                }
                if let Err(e) = salva_offset(&percorso_offset, offset) {
                    warn!(errore = %e, "offset non salvato");
                }
                bot.sessioni.retain(|_, s| s.attiva());
                if invio_conferme.send(offset).is_err() {
                    break;
                }
            }
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => break,
        }
    }

    // Chiusura ordinata: niente nuovi aggiornamenti, attesa dei turni in corso.
    let in_corso = bot.turni_in_corso();
    if in_corso > 0 {
        registra(&format!(
            "Arresto: attendo la fine di {in_corso} turni in corso (max {} s)…",
            opzioni.attesa_chiusura.as_secs()
        ));
    }
    let limite = Instant::now() + opzioni.attesa_chiusura;
    while bot.turni_in_corso() > 0 && Instant::now() < limite {
        std::thread::sleep(Duration::from_millis(100));
    }
    salva_offset(&percorso_offset, offset)?;
    registra("Bot fermato.");
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn comandi_validi_per_telegram() {
        for (c, d) in [Lingua::It, Lingua::En]
            .into_iter()
            .flat_map(comandi::comandi_bot)
        {
            assert!(
                (1..=32).contains(&c.len())
                    && c.chars()
                        .all(|x| x.is_ascii_lowercase() || x.is_ascii_digit() || x == '_'),
                "{c}"
            );
            assert!((1..=256).contains(&d.chars().count()));
        }
    }

    #[test]
    fn normalizzazione_comandi() {
        assert_eq!(normalizza_comando("/start@MioBot abc"), "/start abc");
        assert_eq!(normalizza_comando("/help@MioBot"), "/help");
        assert_eq!(normalizza_comando("  apro la porta "), "apro la porta");
    }

    #[test]
    fn token_mancante() {
        let c = Config::default();
        let e = leggi_token(&c, &|_| None).unwrap_err().to_string();
        assert!(
            e.contains("TELEGRAM_BOT_TOKEN") && e.contains("@BotFather"),
            "{e}"
        );
        assert_eq!(leggi_token(&c, &|_| Some(" 1:A ".into())).unwrap(), "1:A");
    }

    #[test]
    fn accesso_e_utenti_json() {
        let dir = tempfile::tempdir().unwrap();
        let mut c = Config::default();
        c.telegram.utenti_ammessi = vec![1];
        c.telegram.amministratori = vec![9];
        c.telegram.codice_invito = Some("segreto".into());
        let p = dir.path().join("utenti.json");
        let mut a = Accesso::carica(&c, p.clone()).unwrap();
        assert!(a.ammesso(1) && a.ammesso(9) && !a.ammesso(2));
        assert!(a.codice_valido("segreto") && !a.codice_valido("altro"));
        a.ammetti(2).unwrap();
        a.revoca(1).unwrap();
        assert_eq!(a.elenco(), vec![2, 9]);
        let b = Accesso::carica(&c, p).unwrap();
        assert!(b.ammesso(2) && !b.ammesso(1));
        assert!(b.revocato(1) && !b.revocato(2));
    }

    #[test]
    fn blocco_dopo_troppi_codici_sbagliati() {
        let mut t = TentativiInvito::default();
        let ora = Instant::now();
        for _ in 1..TENTATIVI_INVITO {
            assert!(!t.sbagliato(7, ora));
            assert!(!t.bloccato(7, ora));
        }
        assert!(t.sbagliato(7, ora), "il quinto errore blocca");
        assert!(t.bloccato(7, ora + Duration::from_secs(60)));
        assert!(!t.bloccato(8, ora), "gli altri utenti non sono bloccati");
        assert!(!t.bloccato(7, ora + BLOCCO_INVITO), "il blocco scade");
        assert!(!t.sbagliato(7, ora), "dopo la scadenza si riparte da zero");
        t.riuscito(7);
        assert!(t.per_utente.is_empty());
    }

    #[test]
    fn offset_su_file() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("offset.json");
        assert_eq!(carica_offset(&p), 0);
        salva_offset(&p, 42).unwrap();
        assert_eq!(carica_offset(&p), 42);
    }
}
