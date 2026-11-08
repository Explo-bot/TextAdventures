//! Modalità bot Telegram (§7.5): tutta l'interazione con il giocatore avviene
//! in una chat Telegram, con lo stesso motore, le stesse avventure, gli stessi
//! menu e comandi della modalità terminale.
//!
//! - [`api`]: client minimale dell'API Bot (reqwest bloccante, niente async);
//! - [`sessione`]: thread per utente e implementazione di [`crate::canale::Canale`];
//! - [`bot`]: polling, smistamento, accesso, comandi amministratore, offset.

pub mod api;
pub mod bot;
pub mod sessione;

use tracing::info;

/// Una riga di log per evento significativo: sul terminale del server (con
/// l'ora) e nel file di log. Mai il token né il contenuto dei messaggi.
pub fn registra(evento: &str) {
    info!("{evento}");
    crate::canale::terminale::scrivi(&format!(
        "[{}] {evento}",
        chrono::Local::now().format("%H:%M:%S")
    ));
}
