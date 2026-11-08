//! Scansione della cartella delle avventure (§2, §7.1) e riepilogo per il menu:
//! titolo, descrizione breve, numero di stanze, partita in corso, numero di
//! salvataggi. Le avventure non valide sono incluse con il motivo, in fondo
//! all'elenco.
//!
//! Contiene anche la risoluzione delle cartelle (`avventure/`, `partite/`) e
//! dell'opzione `--avventura` (id oppure percorso).

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Result, bail};
use chrono::{DateTime, FixedOffset};

use crate::avventura::id_da_cartella;
use crate::config::{Config, cartella_eseguibile};
use crate::lingua::Lingua;
use crate::salvataggi::GestoreSalvataggi;
use crate::stato::{Esito, PercorsiPartita, Stato};
use crate::tr;
use crate::validazione_statica::valida_cartella;

/// Partita in corso trovata in `stato.json`.
#[derive(Debug, Clone, PartialEq)]
pub struct InfoPartita {
    /// Turno.
    pub turno: u32,
    /// Nome della stanza corrente.
    pub nome_stanza: String,
    /// Ultimo aggiornamento.
    pub aggiornato_il: DateTime<FixedOffset>,
    /// Esito.
    pub esito: Esito,
}

/// Riepilogo di un'avventura valida.
#[derive(Debug, Clone, PartialEq)]
pub struct Riepilogo {
    /// Titolo.
    pub titolo: String,
    /// Descrizione breve.
    pub descrizione_breve: String,
    /// Numero di stanze.
    pub numero_stanze: usize,
    /// Partita salvata in `stato.json`, se esiste.
    pub partita: Option<InfoPartita>,
    /// Numero di salvataggi con nome.
    pub numero_salvataggi: usize,
    /// Lingua dell'avventura (§7.6).
    pub lingua: Lingua,
    /// Storia di cui l'avventura è una versione (campo `storia`, altrimenti l'id).
    pub storia: String,
}

impl Riepilogo {
    /// Etichetta tra parentesi quadre mostrata nel menu.
    pub fn etichetta(&self) -> String {
        let salvataggi = match self.numero_salvataggi {
            0 => None,
            1 => Some(
                tr!(
                    "1 salvataggio",
                    "1 save",
                    "1 sauvegarde",
                    "1 partida guardada",
                    "1 Spielstand"
                )
                .to_string(),
            ),
            n => Some(tr!(
                format!("{n} salvataggi"),
                format!("{n} saves"),
                format!("{n} sauvegardes"),
                format!("{n} partidas guardadas"),
                format!("{n} Spielstände")
            )),
        };
        let partita = self.partita.as_ref().map(|p| match p.esito {
            Esito::InCorso => tr!(
                format!("partita in corso: turno {}", p.turno),
                format!("game in progress: turn {}", p.turno),
                format!("partie en cours : tour {}", p.turno),
                format!("partida en curso: turno {}", p.turno),
                format!("Spiel läuft: Zug {}", p.turno)
            ),
            Esito::Vittoria => tr!(
                "partita vinta",
                "game won",
                "partie gagnée",
                "partida ganada",
                "Spiel gewonnen"
            )
            .to_string(),
            Esito::Sconfitta => tr!(
                "partita persa",
                "game lost",
                "partie perdue",
                "partida perdida",
                "Spiel verloren"
            )
            .to_string(),
        });
        let parti: Vec<String> = partita.into_iter().chain(salvataggi).collect();
        if parti.is_empty() {
            tr!("[nuova]", "[new]", "[nouvelle]", "[nueva]", "[neu]").to_string()
        } else {
            format!("[{}]", parti.join(" · "))
        }
    }
}

/// Una sottocartella con `gioco.md`.
#[derive(Debug, Clone, PartialEq)]
pub struct VoceAvventura {
    /// Id (nome della cartella).
    pub id: String,
    /// Cartella.
    pub cartella: PathBuf,
    /// Riepilogo, oppure il motivo per cui non è valida.
    pub esito: std::result::Result<Riepilogo, String>,
}

impl VoceAvventura {
    /// Vero se si può giocare.
    pub fn valida(&self) -> bool {
        self.esito.is_ok()
    }
}

/// Scansiona `cartella_avventure`: valide in ordine alfabetico di titolo, poi le non valide.
pub fn scansiona(cartella_avventure: &Path, cartella_partite: &Path) -> Vec<VoceAvventura> {
    let Ok(voci) = fs::read_dir(cartella_avventure) else {
        return Vec::new();
    };
    let mut elenco: Vec<VoceAvventura> = voci
        .filter_map(|v| v.ok().map(|v| v.path()))
        .filter(|p| p.is_dir() && p.join("gioco.md").is_file())
        .map(|cartella| riepiloga(&cartella, cartella_partite))
        .collect();
    elenco.sort_by(|a, b| match (&a.esito, &b.esito) {
        (Ok(x), Ok(y)) => x
            .titolo
            .to_lowercase()
            .cmp(&y.titolo.to_lowercase())
            .then_with(|| a.id.cmp(&b.id)),
        (Ok(_), Err(_)) => std::cmp::Ordering::Less,
        (Err(_), Ok(_)) => std::cmp::Ordering::Greater,
        (Err(_), Err(_)) => a.id.cmp(&b.id),
    });
    elenco
}

/// Riepilogo di una singola avventura.
pub fn riepiloga(cartella: &Path, cartella_partite: &Path) -> VoceAvventura {
    let id = id_da_cartella(cartella);
    let rapporto = valida_cartella(cartella);
    let esito = match rapporto.avventura {
        Some(avventura) => {
            let percorsi = PercorsiPartita::nuovi(cartella_partite, &id);
            let partita = Stato::carica(&percorsi.stato()).ok().map(|s| InfoPartita {
                turno: s.turno,
                nome_stanza: avventura.nome_stanza(&s.stanza_corrente),
                aggiornato_il: s.aggiornato_il,
                esito: s.esito,
            });
            let numero_salvataggi = GestoreSalvataggi::nuovo(percorsi, &id).elenca().len();
            Ok(Riepilogo {
                // Nella lingua dell'interfaccia, se l'avventura ha la traduzione (§7.6).
                titolo: avventura.titolo(),
                descrizione_breve: avventura.descrizione_breve(),
                numero_stanze: avventura.stanze.len(),
                partita,
                numero_salvataggi,
                lingua: avventura.lingua(),
                storia: avventura
                    .gioco
                    .storia
                    .clone()
                    .filter(|s| !s.trim().is_empty())
                    .unwrap_or_else(|| id.clone()),
            })
        }
        None => {
            let errori = rapporto
                .problemi
                .iter()
                .filter(|p| p.gravita == crate::avventura::Gravita::Errore)
                .count();
            let primo = rapporto
                .primo_errore()
                .map(ToString::to_string)
                .unwrap_or_else(|| "errore sconosciuto".to_string());
            Err(if errori > 1 {
                format!("{primo} (e altri {} errori)", errori - 1)
            } else {
                primo
            })
        }
    };
    VoceAvventura {
        id,
        cartella: cartella.to_path_buf(),
        esito,
    }
}

/// Cartella delle avventure: `--cartella-avventure` → configurazione →
/// `./avventure` → `avventure/` accanto all'eseguibile.
pub fn cartella_avventure(da_cli: Option<&Path>, config: &Config) -> PathBuf {
    if let Some(c) = da_cli {
        return c.to_path_buf();
    }
    let accanto_exe = |relativo: &Path| cartella_eseguibile().map(|d| d.join(relativo));
    if let Some(c) = &config.motore.cartella_avventure {
        if c.is_absolute() || c.is_dir() {
            return c.clone();
        }
        if let Some(e) = accanto_exe(c).filter(|e| e.is_dir()) {
            return e;
        }
        return c.clone();
    }
    let locale = PathBuf::from("avventure");
    if locale.is_dir() {
        return locale;
    }
    accanto_exe(Path::new("avventure"))
        .filter(|e| e.is_dir())
        .unwrap_or(locale)
}

/// Cartella delle partite: `motore.cartella_partite` (relativa alla cartella che
/// contiene le avventure) oppure `partite/` accanto alla cartella delle avventure.
pub fn cartella_partite(cartella_avventure: &Path, config: &Config) -> PathBuf {
    let base = cartella_avventure
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_default();
    match &config.motore.cartella_partite {
        Some(c) if c.is_absolute() => c.clone(),
        Some(c) => base.join(c),
        None => base.join("partite"),
    }
}

/// Risolve `--avventura` (id o percorso) nella cartella dell'avventura.
pub fn risolvi_avventura(argomento: &str, cartella_avventure: &Path) -> Result<PathBuf> {
    let come_percorso = PathBuf::from(argomento);
    if come_percorso.join("gioco.md").is_file() {
        return Ok(come_percorso);
    }
    let come_id = cartella_avventure.join(argomento);
    if come_id.join("gioco.md").is_file() {
        return Ok(come_id);
    }
    bail!(
        "avventura «{argomento}» non trovata: non è una cartella con gioco.md né una \
         sottocartella di «{}»",
        cartella_avventure.display()
    )
}

#[cfg(test)]
mod test {
    use super::*;

    fn crea(dir: &Path, id: &str, titolo: &str, valida: bool) {
        let c = dir.join(id);
        fs::create_dir_all(c.join("stanze")).unwrap();
        let iniziale = if valida { "a" } else { "nessuna" };
        fs::write(
            c.join("gioco.md"),
            format!("+++\ntitolo = \"{titolo}\"\ndescrizione_breve = \"desc {id}\"\nstanza_iniziale = \"{iniziale}\"\n+++\n"),
        )
        .unwrap();
        fs::write(
            c.join("stanze/a.md"),
            "+++\nid = \"a\"\nnome = \"A\"\n+++\n## Descrizione\nx\n",
        )
        .unwrap();
    }

    #[test]
    fn scansione_ordinata_con_non_valide_in_fondo() {
        let dir = tempfile::tempdir().unwrap();
        let av = dir.path().join("avventure");
        crea(&av, "zeta", "Alba", true);
        crea(&av, "alfa", "Zenit", true);
        crea(&av, "rotta", "Rotta", false);
        fs::create_dir_all(av.join("senza-gioco")).unwrap();
        let elenco = scansiona(&av, &dir.path().join("partite"));
        let id: Vec<&str> = elenco.iter().map(|v| v.id.as_str()).collect();
        assert_eq!(id, vec!["zeta", "alfa", "rotta"]);
        assert!(!elenco[2].valida());
        assert!(
            elenco[2]
                .esito
                .as_ref()
                .unwrap_err()
                .contains("stanza_iniziale")
        );
        let r = elenco[0].esito.as_ref().unwrap();
        assert_eq!(r.etichetta(), "[nuova]");
        assert_eq!(r.numero_stanze, 1);
    }

    #[test]
    fn etichette() {
        let mut r = Riepilogo {
            titolo: "T".into(),
            descrizione_breve: String::new(),
            numero_stanze: 1,
            partita: Some(InfoPartita {
                turno: 12,
                nome_stanza: "x".into(),
                aggiornato_il: crate::stato::adesso(),
                esito: Esito::InCorso,
            }),
            numero_salvataggi: 2,
            lingua: Lingua::It,
            storia: "t".into(),
        };
        assert_eq!(r.etichetta(), "[partita in corso: turno 12 · 2 salvataggi]");
        crate::lingua::imposta(Lingua::En);
        assert_eq!(r.etichetta(), "[game in progress: turn 12 · 2 saves]");
        crate::lingua::imposta(Lingua::It);
        r.partita = None;
        r.numero_salvataggi = 1;
        assert_eq!(r.etichetta(), "[1 salvataggio]");
    }

    #[test]
    fn risoluzione_cartelle() {
        let dir = tempfile::tempdir().unwrap();
        let av = dir.path().join("avventure");
        crea(&av, "mini", "Mini", true);
        assert_eq!(risolvi_avventura("mini", &av).unwrap(), av.join("mini"));
        let percorso = av.join("mini");
        assert_eq!(
            risolvi_avventura(&percorso.to_string_lossy(), Path::new("altrove")).unwrap(),
            percorso
        );
        assert!(risolvi_avventura("manca", &av).is_err());
        let config = Config::default();
        assert_eq!(cartella_partite(&av, &config), dir.path().join("partite"));
        assert_eq!(cartella_avventure(Some(&av), &config), av);
    }
}
