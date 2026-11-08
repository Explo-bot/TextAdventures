//! Funzioni di supporto condivise dai test di integrazione.

#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};

use avventura::config::{Config, Fornitore};
use avventura::sessione::Ambiente;

/// Cartella dei dati di prova (`tests/dati/avventure`).
pub fn dati_avventure() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/dati/avventure")
}

fn copia_cartella(da: &Path, a: &Path) {
    fs::create_dir_all(a).unwrap();
    for voce in fs::read_dir(da).unwrap() {
        let voce = voce.unwrap();
        let destinazione = a.join(voce.file_name());
        if voce.path().is_dir() {
            copia_cartella(&voce.path(), &destinazione);
        } else {
            fs::copy(voce.path(), destinazione).unwrap();
        }
    }
}

/// Copia le avventure di prova in una cartella temporanea: `<tmp>/avventure`.
pub fn copia_avventure() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    copia_cartella(&dati_avventure(), &dir.path().join("avventure"));
    dir
}

/// Configurazione con l'AI simulata.
pub fn config_finta() -> Config {
    let mut c = Config::default();
    c.ai.fornitore = Fornitore::Finto;
    c.ai.base_url = String::new();
    c.interfaccia.mostra_turno = true;
    // I test controllano i testi italiani: lingua fissata, nessuna domanda.
    c.interfaccia.lingua = "it".into();
    c
}

/// Ambiente di gioco nella cartella temporanea.
pub fn ambiente<'a>(dir: &Path, config: &'a Config) -> Ambiente<'a> {
    Ambiente {
        config,
        cartella_avventure: dir.join("avventure"),
        cartella_partite: dir.join("partite"),
    }
}
