//! Definizione della riga di comando (`clap`, §7). Senza sottocomando il
//! programma equivale a `avventura gioca`.

use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

/// Motore per avventure testuali narrate da un modello AI.
#[derive(Debug, Parser)]
#[command(
    name = "avventura",
    version,
    about = "Motore per avventure testuali narrate da un modello AI",
    long_about = "Motore per avventure testuali narrate da un modello AI.\n\
                  Senza sottocomando avvia il gioco (come `avventura gioca`)."
)]
pub struct Cli {
    /// Sottocomando (predefinito: gioca).
    #[command(subcommand)]
    pub comando: Option<Comando>,
}

/// Sottocomandi.
#[derive(Debug, Subcommand)]
pub enum Comando {
    /// Avvia il gioco con il menu di scelta dell'avventura e della partita.
    Gioca(ArgGioca),
    /// Elenca le avventure trovate.
    Elenca(ArgElenca),
    /// Elenca i salvataggi con nome di un'avventura.
    Salvataggi(ArgAvventura),
    /// Validazione statica di una o di tutte le avventure.
    Valida(ArgValida),
    /// Azzera la partita in corso di un'avventura (non tocca i salvataggi con nome).
    Nuova(ArgAvventura),
    /// Prova la connessione con il modello AI e consiglia la dimensione del contesto.
    #[command(name = "prova-ai")]
    ProvaAi(ArgProvaAi),
    /// Elenca il profilo predefinito e i profili AI configurati.
    Profili(ArgProfili),
    /// Avvia il bot Telegram (disponibile dopo lo script 03).
    Telegram(ArgTelegram),
}

/// Opzioni di `profili`.
#[derive(Debug, Clone, Args)]
pub struct ArgProfili {
    /// File di configurazione.
    #[arg(long, value_name = "FILE")]
    pub config: Option<PathBuf>,
    /// Profilo da considerare attivo (altrimenti AVVENTURA_PROFILO).
    #[arg(long, value_name = "NOME")]
    pub profilo: Option<String>,
}

/// Opzioni di `telegram`.
#[derive(Debug, Clone, Args)]
pub struct ArgTelegram {
    /// File di configurazione.
    #[arg(long, value_name = "FILE")]
    pub config: Option<PathBuf>,
    /// Profilo AI da usare.
    #[arg(long, value_name = "NOME")]
    pub profilo: Option<String>,
    /// Attesa massima della risposta dell'AI, in secondi (1–86400).
    #[arg(long, value_name = "SECONDI")]
    pub timeout: Option<u64>,
    /// Controlla token, AI e cartelle ed esce.
    #[arg(long)]
    pub verifica: bool,
    /// Log dettagliato.
    #[arg(long)]
    pub debug: bool,
}

/// Opzioni di `gioca`.
#[derive(Debug, Clone, Default, Args)]
pub struct ArgGioca {
    /// Avventura da giocare (id o percorso della cartella): salta la scelta.
    #[arg(long, value_name = "ID|DIR")]
    pub avventura: Option<String>,
    /// Carica subito il salvataggio con questo nome.
    #[arg(long, value_name = "NOME", conflicts_with = "nuova")]
    pub carica: Option<String>,
    /// Inizia subito una nuova partita.
    #[arg(long)]
    pub nuova: bool,
    /// Cartella che contiene le avventure.
    #[arg(long, value_name = "DIR")]
    pub cartella_avventure: Option<PathBuf>,
    /// File di configurazione.
    #[arg(long, value_name = "FILE")]
    pub config: Option<PathBuf>,
    /// Profilo AI da usare ([profili.NOME] nel file di configurazione).
    #[arg(long, value_name = "NOME")]
    pub profilo: Option<String>,
    /// Attesa massima della risposta dell'AI, in secondi (1–86400).
    #[arg(long, value_name = "SECONDI")]
    pub timeout: Option<u64>,
    /// Log dettagliato e comandi /state e /raw.
    #[arg(long)]
    pub debug: bool,
    /// Salva in Markdown tutto ciò che appare a schermo.
    #[arg(long, value_name = "FILE")]
    pub trascrizione: Option<PathBuf>,
    /// Lingua dell'interfaccia (it, en, fr, es, de) per questa sessione, senza salvarla.
    #[arg(long, value_name = "it|en|fr|es|de")]
    pub lingua: Option<String>,
}

/// Opzioni di `elenca`.
#[derive(Debug, Clone, Args)]
pub struct ArgElenca {
    /// Cartella che contiene le avventure.
    #[arg(long, value_name = "DIR")]
    pub cartella_avventure: Option<PathBuf>,
    /// File di configurazione.
    #[arg(long, value_name = "FILE")]
    pub config: Option<PathBuf>,
}

/// Opzioni dei comandi che richiedono un'avventura.
#[derive(Debug, Clone, Args)]
pub struct ArgAvventura {
    /// Avventura (id o percorso della cartella).
    #[arg(long, value_name = "ID|DIR")]
    pub avventura: String,
    /// Cartella che contiene le avventure.
    #[arg(long, value_name = "DIR")]
    pub cartella_avventure: Option<PathBuf>,
    /// File di configurazione.
    #[arg(long, value_name = "FILE")]
    pub config: Option<PathBuf>,
}

/// Opzioni di `valida`.
#[derive(Debug, Clone, Args)]
pub struct ArgValida {
    /// Avventura da validare (id o percorso); senza: tutte.
    #[arg(long, value_name = "ID|DIR")]
    pub avventura: Option<String>,
    /// Cartella che contiene le avventure.
    #[arg(long, value_name = "DIR")]
    pub cartella_avventure: Option<PathBuf>,
    /// File di configurazione.
    #[arg(long, value_name = "FILE")]
    pub config: Option<PathBuf>,
}

/// Opzioni di `prova-ai`.
#[derive(Debug, Clone, Args)]
pub struct ArgProvaAi {
    /// File di configurazione.
    #[arg(long, value_name = "FILE")]
    pub config: Option<PathBuf>,
    /// Invia anche il prompt reale più grande di questa avventura.
    #[arg(long, value_name = "ID|DIR")]
    pub avventura: Option<String>,
    /// Cartella che contiene le avventure.
    #[arg(long, value_name = "DIR")]
    pub cartella_avventure: Option<PathBuf>,
    /// Attesa massima della risposta, in secondi (1–86400).
    #[arg(long, value_name = "SECONDI")]
    pub timeout: Option<u64>,
    /// Profilo AI da provare.
    #[arg(long, value_name = "NOME")]
    pub profilo: Option<String>,
    /// Verifica solo configurazione, chiave e raggiungibilità (nessun prompt, nessun costo).
    #[arg(long, conflicts_with = "avventura")]
    pub solo_config: bool,
}

#[cfg(test)]
mod test {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn definizione_valida() {
        Cli::command().debug_assert();
    }

    #[test]
    fn senza_argomenti_nessun_sottocomando() {
        let cli = Cli::try_parse_from(["avventura"]).unwrap();
        assert!(cli.comando.is_none());
    }

    #[test]
    fn opzioni_di_gioca() {
        let cli = Cli::try_parse_from([
            "avventura",
            "gioca",
            "--avventura",
            "mini",
            "--carica",
            "inizio",
            "--timeout",
            "60",
            "--cartella-avventure",
            "x",
        ])
        .unwrap();
        match cli.comando {
            Some(Comando::Gioca(a)) => {
                assert_eq!(a.avventura.as_deref(), Some("mini"));
                assert_eq!(a.carica.as_deref(), Some("inizio"));
                assert_eq!(a.timeout, Some(60));
            }
            altro => panic!("{altro:?}"),
        }
        assert!(Cli::try_parse_from(["avventura", "gioca", "--carica", "a", "--nuova"]).is_err());
        assert!(Cli::try_parse_from(["avventura", "prova-ai", "--avventura", "mini"]).is_ok());
    }

    #[test]
    fn profili_telegram_e_solo_config() {
        let cli = Cli::try_parse_from(["avventura", "gioca", "--profilo", "claude"]).unwrap();
        assert!(
            matches!(cli.comando, Some(Comando::Gioca(a)) if a.profilo.as_deref() == Some("claude"))
        );
        assert!(Cli::try_parse_from(["avventura", "profili"]).is_ok());
        assert!(Cli::try_parse_from(["avventura", "telegram", "--verifica"]).is_ok());
        assert!(Cli::try_parse_from(["avventura", "prova-ai", "--solo-config"]).is_ok());
        assert!(
            Cli::try_parse_from(["avventura", "prova-ai", "--solo-config", "--avventura", "x"])
                .is_err()
        );
    }
}
