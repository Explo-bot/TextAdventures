//! Stato della partita (`stato.json`, §4): struttura, creazione iniziale,
//! lettura, scrittura atomica, storico per turno e `/annulla`.
//!
//! Lo stato è l'unica memoria dell'AI: viene riletto dal disco a ogni turno.
//! Ogni avventura ha la propria cartella `partite/<id>/` (vedi [`PercorsiPartita`]).
//!
//! Lo storico contiene una copia dello stato precedente a ogni modifica
//! (`storico/turno_NNNN.json`); il file `storico/indice.json` ne registra l'ordine
//! cronologico, così `/annulla` torna sempre allo stato immediatamente precedente,
//! anche dopo un `/load` che ha riportato indietro il numero di turno.

use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use chrono::{DateTime, FixedOffset, Local};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::avventura::Avventura;

/// Versione del formato del file di stato.
pub const VERSIONE_FORMATO: u32 = 1;

/// Esito della partita.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Esito {
    /// Partita in corso.
    #[default]
    InCorso,
    /// Il giocatore ha vinto.
    Vittoria,
    /// Il giocatore ha perso.
    Sconfitta,
}

impl Esito {
    /// Valore testuale usato nel JSON.
    pub fn come_testo(self) -> &'static str {
        match self {
            Esito::InCorso => "in_corso",
            Esito::Vittoria => "vittoria",
            Esito::Sconfitta => "sconfitta",
        }
    }

    /// Interpreta il valore testuale del JSON.
    pub fn da_testo(testo: &str) -> Option<Esito> {
        match testo.trim() {
            "in_corso" => Some(Esito::InCorso),
            "vittoria" => Some(Esito::Vittoria),
            "sconfitta" => Some(Esito::Sconfitta),
            _ => None,
        }
    }

    /// Vero per vittoria o sconfitta.
    pub fn concluso(self) -> bool {
        self != Esito::InCorso
    }
}

/// Stato dinamico di una stanza.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct StatoStanza {
    /// Il giocatore ci è già stato.
    #[serde(default)]
    pub visitata: bool,
    /// Oggetti presenti ora nella stanza.
    #[serde(default)]
    pub oggetti: Vec<String>,
    /// Annotazioni dell'AI su cambiamenti della stanza.
    #[serde(default)]
    pub note: Vec<String>,
}

/// Un turno recente (buffer circolare `ultimi_turni`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TurnoRecente {
    /// Numero del turno.
    pub turno: u32,
    /// Azione del giocatore.
    pub azione: String,
    /// Narrazione prodotta.
    pub narrazione: String,
}

/// Contenuto di `stato.json`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Stato {
    /// Versione del formato.
    pub versione_formato: u32,
    /// Id dell'avventura.
    pub avventura: String,
    /// Data di creazione della partita.
    pub creato_il: DateTime<FixedOffset>,
    /// Data dell'ultimo aggiornamento.
    pub aggiornato_il: DateTime<FixedOffset>,
    /// Turni giocati.
    pub turno: u32,
    /// Stanza in cui si trova il giocatore.
    pub stanza_corrente: String,
    /// Oggetti posseduti.
    #[serde(default)]
    pub inventario: Vec<String>,
    /// Flag di gioco (valori JSON liberi).
    #[serde(default)]
    pub flag: BTreeMap<String, Value>,
    /// Stato dinamico delle stanze.
    #[serde(default)]
    pub stanze: BTreeMap<String, StatoStanza>,
    /// Riassunto narrativo cumulativo scritto dall'AI.
    #[serde(default)]
    pub memoria: String,
    /// Ultimi turni giocati.
    #[serde(default)]
    pub ultimi_turni: Vec<TurnoRecente>,
    /// Punteggio.
    #[serde(default)]
    pub punteggio: i64,
    /// Esito della partita.
    #[serde(default)]
    pub esito: Esito,
    /// Salvataggio con nome da cui è stata caricata o in cui è stata salvata (gestito dal motore).
    #[serde(default)]
    pub salvataggio_corrente: Option<String>,
    /// Turno dell'ultimo `/save` o `/load` (gestito dal motore).
    #[serde(default)]
    pub turno_ultimo_salvataggio: u32,
}

/// Ora corrente con il fuso orario locale.
pub fn adesso() -> DateTime<FixedOffset> {
    Local::now().fixed_offset()
}

impl Stato {
    /// Stato iniziale di una nuova partita.
    pub fn nuovo(avventura: &Avventura) -> Stato {
        let ora = adesso();
        let stanze = avventura
            .stanze
            .values()
            .map(|s| {
                (
                    s.id.clone(),
                    StatoStanza {
                        visitata: false,
                        oggetti: s.oggetti.clone(),
                        note: Vec::new(),
                    },
                )
            })
            .collect();
        Stato {
            versione_formato: VERSIONE_FORMATO,
            avventura: avventura.id.clone(),
            creato_il: ora,
            aggiornato_il: ora,
            turno: 0,
            stanza_corrente: avventura.gioco.stanza_iniziale.clone(),
            inventario: avventura.gioco.inventario_iniziale.clone(),
            flag: avventura.gioco.flag_iniziali.clone(),
            stanze,
            memoria: String::new(),
            ultimi_turni: Vec::new(),
            punteggio: 0,
            esito: Esito::InCorso,
            salvataggio_corrente: None,
            turno_ultimo_salvataggio: 0,
        }
    }

    /// Legge uno stato da file.
    pub fn carica(percorso: &Path) -> Result<Stato> {
        let testo = fs::read_to_string(percorso)
            .with_context(|| format!("impossibile leggere «{}»", percorso.display()))?;
        serde_json::from_str(&testo)
            .with_context(|| format!("il file «{}» non è uno stato valido", percorso.display()))
    }

    /// Legge lo stato se il file esiste.
    pub fn carica_se_esiste(percorso: &Path) -> Result<Option<Stato>> {
        if percorso.exists() {
            Stato::carica(percorso).map(Some)
        } else {
            Ok(None)
        }
    }

    /// Scrive lo stato in modo atomico (file temporaneo + rinomina).
    pub fn salva_atomico(&self, percorso: &Path) -> Result<()> {
        scrivi_json_atomico(percorso, self)
    }

    /// Stato da inviare all'AI: senza i campi gestiti solo dal motore.
    pub fn per_ai(&self) -> Value {
        let mut valore = serde_json::to_value(self).unwrap_or(Value::Null);
        if let Some(oggetto) = valore.as_object_mut() {
            oggetto.remove("salvataggio_corrente");
            oggetto.remove("turno_ultimo_salvataggio");
        }
        valore
    }

    /// Stato dinamico di una stanza (creato vuoto se manca).
    pub fn stanza_mut(&mut self, id: &str) -> &mut StatoStanza {
        self.stanze.entry(id.to_string()).or_default()
    }
}

/// Percorsi dei file di partita di un'avventura (`partite/<id>/…`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PercorsiPartita {
    /// `partite/<id>`.
    pub cartella: PathBuf,
}

impl PercorsiPartita {
    /// Percorsi per l'avventura `id` dentro `cartella_partite`.
    pub fn nuovi(cartella_partite: &Path, id: &str) -> Self {
        PercorsiPartita {
            cartella: cartella_partite.join(id),
        }
    }

    /// `stato.json`.
    pub fn stato(&self) -> PathBuf {
        self.cartella.join("stato.json")
    }

    /// Cartella `storico/`.
    pub fn storico(&self) -> PathBuf {
        self.cartella.join("storico")
    }

    /// Cartella `salvataggi/`.
    pub fn salvataggi(&self) -> PathBuf {
        self.cartella.join("salvataggi")
    }

    fn indice_storico(&self) -> PathBuf {
        self.storico().join("indice.json")
    }
}

/// Scrive un valore JSON indentato in modo atomico: `<file>.tmp` e poi rinomina.
pub fn scrivi_json_atomico<T: Serialize + ?Sized>(percorso: &Path, valore: &T) -> Result<()> {
    if let Some(cartella) = percorso.parent() {
        fs::create_dir_all(cartella)
            .with_context(|| format!("impossibile creare la cartella «{}»", cartella.display()))?;
    }
    let nome = percorso
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "file".to_string());
    let temporaneo = percorso.with_file_name(format!("{nome}.tmp"));
    let testo = serde_json::to_string_pretty(valore).context("serializzazione JSON fallita")?;
    {
        let mut file = fs::File::create(&temporaneo)
            .with_context(|| format!("impossibile scrivere «{}»", temporaneo.display()))?;
        file.write_all(testo.as_bytes())
            .and_then(|()| file.write_all(b"\n"))
            .and_then(|()| file.sync_all())
            .with_context(|| format!("impossibile scrivere «{}»", temporaneo.display()))?;
    }
    fs::rename(&temporaneo, percorso).with_context(|| {
        format!(
            "impossibile rinominare «{}» in «{}»",
            temporaneo.display(),
            percorso.display()
        )
    })
}

fn leggi_indice(percorsi: &PercorsiPartita) -> Vec<String> {
    let da_file = fs::read_to_string(percorsi.indice_storico())
        .ok()
        .and_then(|t| serde_json::from_str::<Vec<String>>(&t).ok());
    if let Some(indice) = da_file {
        return indice;
    }
    // Indice assente o danneggiato: ricostruito in ordine di data di modifica.
    let mut file: Vec<(std::time::SystemTime, String)> = fs::read_dir(percorsi.storico())
        .into_iter()
        .flatten()
        .filter_map(|v| v.ok())
        .filter_map(|v| {
            let nome = v.file_name().to_string_lossy().to_string();
            let data = v.metadata().and_then(|m| m.modified()).ok()?;
            (nome.starts_with("turno_") && nome.ends_with(".json")).then_some((data, nome))
        })
        .collect();
    file.sort();
    file.into_iter().map(|(_, n)| n).collect()
}

/// Copia lo `stato.json` attuale (se esiste) nello storico.
pub fn archivia_nello_storico(percorsi: &PercorsiPartita) -> Result<()> {
    let percorso = percorsi.stato();
    if !percorso.exists() {
        return Ok(());
    }
    let stato = Stato::carica(&percorso)?;
    // Dopo una nuova partita o un /load i numeri di turno si ripetono: il nome
    // riceve un suffisso invece di sovrascrivere uno stato più vecchio, così
    // /annulla ripercorre sempre la storia vera, in ordine.
    let mut nome = format!("turno_{:04}.json", stato.turno);
    let mut n = 2;
    while percorsi.storico().join(&nome).exists() {
        nome = format!("turno_{:04}_{n}.json", stato.turno);
        n += 1;
    }
    scrivi_json_atomico(&percorsi.storico().join(&nome), &stato)?;
    let mut indice = leggi_indice(percorsi);
    indice.retain(|n| n != &nome);
    indice.push(nome);
    // Si tengono solo gli stati più recenti.
    let eccesso = indice.len().saturating_sub(MAX_STORICO);
    for vecchio in indice.drain(..eccesso) {
        let _ = fs::remove_file(percorsi.storico().join(vecchio));
    }
    scrivi_json_atomico(&percorsi.indice_storico(), &indice)
}

/// Stati conservati nello storico (passi possibili con `/annulla`).
pub const MAX_STORICO: usize = 200;

/// Archivia lo stato precedente e scrive il nuovo (fine di ogni turno).
pub fn salva_con_storico(percorsi: &PercorsiPartita, stato: &Stato) -> Result<()> {
    archivia_nello_storico(percorsi)?;
    stato.salva_atomico(&percorsi.stato())
}

/// `/annulla`: ripristina lo stato più recente dello storico e lo rimuove dallo storico.
/// Restituisce `None` se non c'è nulla da annullare.
pub fn annulla(percorsi: &PercorsiPartita) -> Result<Option<Stato>> {
    let mut indice = leggi_indice(percorsi);
    while let Some(nome) = indice.pop() {
        let file = percorsi.storico().join(&nome);
        let Ok(stato) = Stato::carica(&file) else {
            continue;
        };
        stato.salva_atomico(&percorsi.stato())?;
        let _ = fs::remove_file(&file);
        scrivi_json_atomico(&percorsi.indice_storico(), &indice)?;
        return Ok(Some(stato));
    }
    Ok(None)
}

#[cfg(test)]
pub(crate) mod test {
    use super::*;
    use crate::avventura::{Gioco, Stanza};

    /// Avventura costruita in memoria per i test unitari.
    pub(crate) fn avventura_di_prova() -> Avventura {
        let gioco: Gioco = toml::from_str(
            "titolo = \"Prova\"\nstanza_iniziale = \"a\"\ninventario_iniziale = [\"moneta\"]\n\
             [flag_iniziali]\nbotola_aperta = false\n[oggetti]\nchiave = \"una chiave\"\n",
        )
        .unwrap();
        let a: Stanza = toml::from_str(
            "id = \"a\"\nnome = \"Stanza A\"\noggetti = [\"chiave\"]\n\
             [uscite.nord]\nverso = \"b\"\nrichiede_oggetto = \"chiave\"\n\
             [uscite.giu]\nverso = \"c\"\nrichiede_flag = \"botola_aperta\"\nnascosta = true\n\
             [uscite.est]\nverso = \"c\"\ndescrizione = \"un corridoio\"\n",
        )
        .unwrap();
        let b: Stanza = toml::from_str(
            "id = \"b\"\nnome = \"Stanza B\"\noggetti_nascosti = [\"gemma\"]\n[uscite.sud]\nverso = \"a\"\n",
        )
        .unwrap();
        let c: Stanza =
            toml::from_str("id = \"c\"\nnome = \"Stanza C\"\n[uscite.ovest]\nverso = \"a\"\n")
                .unwrap();
        let stanze = [a, b, c].into_iter().map(|s| (s.id.clone(), s)).collect();
        Avventura::da_parti(Path::new("prova"), gioco, stanze)
    }

    #[test]
    fn stato_iniziale() {
        let s = Stato::nuovo(&avventura_di_prova());
        assert_eq!(s.turno, 0);
        assert_eq!(s.stanza_corrente, "a");
        assert_eq!(s.inventario, vec!["moneta"]);
        assert_eq!(s.stanze["a"].oggetti, vec!["chiave"]);
        assert!(!s.stanze["a"].visitata);
        assert_eq!(s.flag["botola_aperta"], Value::Bool(false));
        assert_eq!(s.esito, Esito::InCorso);
    }

    #[test]
    fn per_ai_toglie_campi_del_motore() {
        let mut s = Stato::nuovo(&avventura_di_prova());
        s.salvataggio_corrente = Some("x".into());
        let v = s.per_ai();
        assert!(v.get("salvataggio_corrente").is_none());
        assert!(v.get("turno_ultimo_salvataggio").is_none());
        assert_eq!(v["stanza_corrente"], "a");
    }

    #[test]
    fn scrittura_atomica_e_rilettura() {
        let dir = tempfile::tempdir().unwrap();
        let p = PercorsiPartita::nuovi(dir.path(), "prova");
        let s = Stato::nuovo(&avventura_di_prova());
        s.salva_atomico(&p.stato()).unwrap();
        assert!(!p.cartella.join("stato.json.tmp").exists());
        let letto = Stato::carica(&p.stato()).unwrap();
        assert_eq!(letto, s);
        let testo = fs::read_to_string(p.stato()).unwrap();
        assert!(testo.contains("\n  \"turno\": 0"), "JSON indentato");
    }

    #[test]
    fn storico_e_annulla() {
        let dir = tempfile::tempdir().unwrap();
        let p = PercorsiPartita::nuovi(dir.path(), "prova");
        let mut s = Stato::nuovo(&avventura_di_prova());
        assert!(annulla(&p).unwrap().is_none());
        salva_con_storico(&p, &s).unwrap();
        for t in 1..=3 {
            s.turno = t;
            salva_con_storico(&p, &s).unwrap();
        }
        assert!(p.storico().join("turno_0002.json").exists());
        assert_eq!(annulla(&p).unwrap().unwrap().turno, 2);
        assert_eq!(Stato::carica(&p.stato()).unwrap().turno, 2);
        assert_eq!(annulla(&p).unwrap().unwrap().turno, 1);
        assert_eq!(annulla(&p).unwrap().unwrap().turno, 0);
        assert!(annulla(&p).unwrap().is_none());
    }

    #[test]
    fn annulla_segue_l_ordine_cronologico() {
        let dir = tempfile::tempdir().unwrap();
        let p = PercorsiPartita::nuovi(dir.path(), "prova");
        let mut s = Stato::nuovo(&avventura_di_prova());
        s.turno = 12;
        s.salva_atomico(&p.stato()).unwrap();
        // Simula un /load che riporta al turno 5.
        s.turno = 5;
        salva_con_storico(&p, &s).unwrap();
        s.turno = 6;
        salva_con_storico(&p, &s).unwrap();
        assert_eq!(annulla(&p).unwrap().unwrap().turno, 5);
        assert_eq!(annulla(&p).unwrap().unwrap().turno, 12);
    }

    #[test]
    fn turni_ripetuti_non_sovrascrivono_lo_storico() {
        let dir = tempfile::tempdir().unwrap();
        let p = PercorsiPartita::nuovi(dir.path(), "prova");
        let mut s = Stato::nuovo(&avventura_di_prova());
        // Prima partita: turni 0–2, con memoria «vecchia».
        s.memoria = "vecchia".into();
        for t in 0..=2 {
            s.turno = t;
            salva_con_storico(&p, &s).unwrap();
        }
        // Nuova partita: di nuovo turni 0–2, memoria «nuova».
        s.memoria = "nuova".into();
        for t in 0..=2 {
            s.turno = t;
            salva_con_storico(&p, &s).unwrap();
        }
        assert!(p.storico().join("turno_0001_2.json").exists());
        let memorie: Vec<(u32, String)> = std::iter::from_fn(|| annulla(&p).unwrap())
            .map(|s| (s.turno, s.memoria))
            .collect();
        let atteso: Vec<(u32, String)> = [
            (1, "nuova"),
            (0, "nuova"),
            (2, "vecchia"),
            (1, "vecchia"),
            (0, "vecchia"),
        ]
        .into_iter()
        .map(|(t, m)| (t, m.to_string()))
        .collect();
        assert_eq!(memorie, atteso, "la storia vera, in ordine, senza miscugli");
    }

    #[test]
    fn storico_limitato() {
        let dir = tempfile::tempdir().unwrap();
        let p = PercorsiPartita::nuovi(dir.path(), "prova");
        let mut s = Stato::nuovo(&avventura_di_prova());
        for t in 0..(MAX_STORICO as u32 + 5) {
            s.turno = t;
            salva_con_storico(&p, &s).unwrap();
        }
        assert_eq!(leggi_indice(&p).len(), MAX_STORICO);
        // Gli stati più indice.json.
        assert_eq!(fs::read_dir(p.storico()).unwrap().count(), MAX_STORICO + 1);
        assert!(!p.storico().join("turno_0000.json").exists());
    }

    #[test]
    fn esito_testuale() {
        assert_eq!(Esito::da_testo("vittoria"), Some(Esito::Vittoria));
        assert_eq!(Esito::da_testo("boh"), None);
        assert!(Esito::Sconfitta.concluso());
        assert_eq!(
            serde_json::to_string(&Esito::InCorso).unwrap(),
            "\"in_corso\""
        );
    }
}
