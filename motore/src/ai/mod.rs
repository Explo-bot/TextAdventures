//! Comunicazione con il modello AI che fa da narratore.
//!
//! Il trait [`Narratore`] riceve una [`RichiestaAi`]: i messaggi costruiti dal
//! motore (sistema + utente, più l'eventuale coppia assistente/utente di
//! correzione) e un [`ContestoTurno`] strutturato, usato solo dall'AI simulata.
//! Restituisce una [`RispostaGrezza`] con il testo, i token di prompt dichiarati
//! dal server e la durata. Il resto del motore non sa quale fornitore sta usando.
//!
//! Fornitori (§5): [`openai`] (anche `lmstudio`), [`anthropic`], [`gemini`],
//! [`ollama`], [`locale`] (GGUF dal disco, feature `locale`) e [`finto`].
//! I narratori sono `Send + Sync`: il bot Telegram li condivide tra le sessioni.
//!
//! [`Narratore::turno_in_flusso`] chiede la risposta in streaming e passa i
//! pezzi di testo man mano che arrivano (testo progressivo, [`flusso`]); i
//! fornitori che non lo supportano rispondono tutto in una volta.

pub mod anthropic;
pub mod finto;
pub mod flusso;
pub mod gemini;
pub mod http;
pub mod locale;
pub mod ollama;
pub mod openai;
pub mod profili;
pub mod ragionamento;
pub mod schema;
pub mod token;

use std::time::Duration;

use anyhow::Result;
use serde::Serialize;
use thiserror::Error;

use crate::config::{ConfigAi, Fornitore};
use crate::uscite::InfoUscita;

/// Ruolo di un messaggio.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Ruolo {
    /// Messaggio di sistema (regole e contesto).
    System,
    /// Messaggio dell'utente (stato + azione).
    User,
    /// Risposta del modello.
    Assistant,
}

/// Un messaggio della conversazione, serializzato come `{"role", "content"}`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Messaggio {
    /// Ruolo.
    #[serde(rename = "role")]
    pub ruolo: Ruolo,
    /// Testo.
    #[serde(rename = "content")]
    pub contenuto: String,
}

impl Messaggio {
    /// Messaggio di sistema.
    pub fn sistema(testo: impl Into<String>) -> Self {
        Messaggio {
            ruolo: Ruolo::System,
            contenuto: testo.into(),
        }
    }

    /// Messaggio utente.
    pub fn utente(testo: impl Into<String>) -> Self {
        Messaggio {
            ruolo: Ruolo::User,
            contenuto: testo.into(),
        }
    }

    /// Messaggio dell'assistente (risposta precedente, nei tentativi di correzione).
    pub fn assistente(testo: impl Into<String>) -> Self {
        Messaggio {
            ruolo: Ruolo::Assistant,
            contenuto: testo.into(),
        }
    }
}

/// Testo di sistema (concatenato) e messaggi non di sistema, per i fornitori
/// che tengono le istruzioni di sistema fuori dall'elenco dei messaggi.
pub fn separa_sistema(messaggi: &[Messaggio]) -> (String, Vec<&Messaggio>) {
    let sistema: Vec<&str> = messaggi
        .iter()
        .filter(|m| m.ruolo == Ruolo::System)
        .map(|m| m.contenuto.as_str())
        .collect();
    let altri = messaggi
        .iter()
        .filter(|m| m.ruolo != Ruolo::System)
        .collect();
    (sistema.join("\n\n"), altri)
}

/// Un oggetto con id e nome leggibile.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OggettoNominato {
    /// Id.
    pub id: String,
    /// Nome leggibile.
    pub nome: String,
}

/// Contesto strutturato del turno (usato dall'AI simulata; i client reali lo ignorano).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ContestoTurno {
    /// Id della stanza corrente.
    pub stanza: String,
    /// Nome della stanza corrente.
    pub nome_stanza: String,
    /// Sezione `Descrizione` della stanza.
    pub descrizione: String,
    /// Uscite calcolate dal motore.
    pub uscite: Vec<InfoUscita>,
    /// Oggetti visibili nella stanza.
    pub oggetti_stanza: Vec<OggettoNominato>,
    /// Inventario.
    pub inventario: Vec<OggettoNominato>,
    /// Testo dell'azione (o marcatore di inizio/ripresa).
    pub azione: String,
    /// Memoria attuale.
    pub memoria: String,
    /// Lingua dell'avventura (l'AI simulata risponde in questa lingua).
    pub lingua: crate::lingua::Lingua,
}

/// Una richiesta al narratore.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RichiestaAi {
    /// Messaggi da inviare.
    pub messaggi: Vec<Messaggio>,
    /// Contesto strutturato (solo per l'AI simulata).
    pub contesto: ContestoTurno,
}

/// Risposta grezza del modello.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RispostaGrezza {
    /// Testo del messaggio del modello (dovrebbe contenere il JSON).
    pub testo: String,
    /// Token di prompt dichiarati dal server, se presenti (esatti con `locale`).
    pub token_prompt: Option<u64>,
    /// Token generati nella risposta, se noti.
    pub token_generati: Option<u64>,
    /// Durata della richiesta (tentativi automatici compresi).
    pub durata: Duration,
    /// Avvisi da mostrare una volta all'utente (es. parametro rifiutato).
    pub avvisi: Vec<String>,
}

/// Errori di comunicazione con l'AI.
#[derive(Debug, Error)]
pub enum ErroreAi {
    /// Nessuna risposta entro il timeout di lettura (non si ritenta in automatico).
    #[error("nessuna risposta entro {secondi} s")]
    TimeoutLettura {
        /// Limite configurato.
        secondi: u64,
    },
    /// 401/403.
    #[error(
        "il server ha rifiutato la chiave API (HTTP {stato}): controlla la chiave e la variabile d'ambiente configurata"
    )]
    Autenticazione {
        /// Codice HTTP.
        stato: u16,
    },
    /// 404: URL o modello errato (con Ollama: modello non scaricato).
    #[error("{0}")]
    NonTrovato(String),
    /// Connessione rifiutata verso un server locale.
    #[error("il server locale non risponde su {0}: è avviato?")]
    ServerLocaleNonRisponde(String),
    /// Errore di rete dopo tutti i tentativi.
    #[error("errore di connessione: {0}")]
    Connessione(String),
    /// Altri errori HTTP.
    #[error("il server ha risposto con HTTP {stato}: {estratto}")]
    Http {
        /// Codice HTTP.
        stato: u16,
        /// Inizio del corpo della risposta.
        estratto: String,
    },
    /// Risposta HTTP valida ma con un formato inatteso o incompleta.
    #[error("risposta del server non utilizzabile: {0}")]
    Formato(String),
    /// Configurazione del client non valida o fornitore non disponibile.
    #[error("{0}")]
    Configurazione(String),
}

/// Un narratore: riceve i messaggi e restituisce il testo della risposta.
pub trait Narratore: Send + Sync {
    /// Esegue una richiesta (bloccante).
    fn turno(&self, richiesta: &RichiestaAi) -> std::result::Result<RispostaGrezza, ErroreAi>;

    /// Come [`Narratore::turno`], ma chiede la risposta in streaming e chiama
    /// `pezzo` con ogni frammento di testo grezzo appena arriva. La risposta
    /// restituita è comunque completa. Predefinito: nessuno streaming.
    fn turno_in_flusso(
        &self,
        richiesta: &RichiestaAi,
        pezzo: &mut dyn FnMut(&str),
    ) -> std::result::Result<RispostaGrezza, ErroreAi> {
        let _ = pezzo;
        self.turno(richiesta)
    }

    /// Descrizione breve (fornitore e modello) per log e diagnostica.
    fn descrizione(&self) -> String;

    /// Stato del JSON garantito dopo le richieste (None se non applicabile).
    fn json_mode_attivo(&self) -> Option<bool> {
        None
    }

    /// Righe di diagnostica aggiuntive per `prova-ai` (es. file e velocità del
    /// modello `locale`); vuoto se il fornitore non ha nulla da aggiungere.
    fn diagnostica(&self) -> Vec<String> {
        Vec::new()
    }
}

/// Crea il narratore adatto alla configurazione.
pub fn crea_narratore(config: &ConfigAi) -> Result<Box<dyn Narratore>> {
    Ok(match config.fornitore {
        Fornitore::Finto => Box::new(finto::NarratoreFinto::new()),
        Fornitore::Openai | Fornitore::Lmstudio => Box::new(openai::ClientOpenAi::nuovo(config)?),
        Fornitore::Anthropic => Box::new(anthropic::ClientAnthropic::nuovo(config)?),
        Fornitore::Gemini => Box::new(gemini::ClientGemini::nuovo(config)?),
        Fornitore::Ollama => Box::new(ollama::ClientOllama::nuovo(config)?),
        Fornitore::Locale => locale::crea(config)?,
    })
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn messaggi_serializzati_come_openai() {
        let m = Messaggio::sistema("regole");
        assert_eq!(
            serde_json::to_string(&m).unwrap(),
            r#"{"role":"system","content":"regole"}"#
        );
        assert_eq!(
            serde_json::to_string(&Messaggio::assistente("x")).unwrap(),
            r#"{"role":"assistant","content":"x"}"#
        );
    }

    #[test]
    fn separazione_del_sistema() {
        let m = vec![
            Messaggio::sistema("a"),
            Messaggio::utente("u"),
            Messaggio::assistente("r"),
            Messaggio::utente("c"),
        ];
        let (s, altri) = separa_sistema(&m);
        assert_eq!(s, "a");
        assert_eq!(altri.len(), 3);
    }

    #[test]
    fn narratore_condivisibile_tra_thread() {
        fn richiede_send_sync<T: Send + Sync + ?Sized>() {}
        richiede_send_sync::<dyn Narratore>();
    }
}
