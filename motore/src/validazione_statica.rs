//! Validazione statica di un'avventura (§3.3 della specifica).
//!
//! Controlla struttura dei file, coerenza degli id, destinazioni delle uscite,
//! oggetti richiesti, raggiungibilità delle stanze e oggetti duplicati.
//! Gli errori impediscono di giocare; gli avvisi no.

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::path::{Path, PathBuf};

use crate::avventura::{Avventura, CARTELLA_TRADUZIONI, Gravita, Problema, Stanza, carica_file};

/// Esito della validazione di una cartella.
#[derive(Debug)]
pub struct Rapporto {
    /// Cartella validata.
    pub cartella: PathBuf,
    /// Errori e avvisi, nell'ordine in cui sono stati trovati.
    pub problemi: Vec<Problema>,
    /// L'avventura caricata, presente solo se non ci sono errori.
    pub avventura: Option<Avventura>,
}

impl Rapporto {
    /// Vero se c'è almeno un errore.
    pub fn ha_errori(&self) -> bool {
        self.numero(Gravita::Errore) > 0
    }

    /// Numero di problemi della gravità indicata.
    pub fn numero(&self, gravita: Gravita) -> usize {
        self.problemi
            .iter()
            .filter(|p| p.gravita == gravita)
            .count()
    }

    /// Primo errore (per i riepiloghi brevi, es. nel menu).
    pub fn primo_errore(&self) -> Option<&Problema> {
        self.problemi.iter().find(|p| p.gravita == Gravita::Errore)
    }

    /// Testo leggibile del rapporto.
    pub fn formatta(&self) -> String {
        let mut righe = Vec::new();
        for p in &self.problemi {
            righe.push(format!("  {p}"));
        }
        righe.push(format!(
            "  {} errori, {} avvisi.",
            self.numero(Gravita::Errore),
            self.numero(Gravita::Avviso)
        ));
        righe.join("\n")
    }
}

/// Valida la cartella di un'avventura.
pub fn valida_cartella(cartella: &Path) -> Rapporto {
    let caricamento = carica_file(cartella);
    let mut problemi = caricamento.problemi;
    let Some(gioco) = caricamento.gioco else {
        return Rapporto {
            cartella: cartella.to_path_buf(),
            problemi,
            avventura: None,
        };
    };

    // Id duplicati (possibili solo se un id non coincide col nome del file).
    let mut stanze: BTreeMap<String, Stanza> = BTreeMap::new();
    for stanza in caricamento.stanze {
        if let Some(prima) = stanze.get(&stanza.id) {
            problemi.push(Problema::errore(
                &file_relativo(&stanza),
                "id",
                format!(
                    "id «{}» duplicato (già usato in {})",
                    stanza.id,
                    file_relativo(prima)
                ),
            ));
        } else {
            stanze.insert(stanza.id.clone(), stanza);
        }
    }

    if !stanze.contains_key(&gioco.stanza_iniziale) {
        problemi.push(Problema::errore(
            "gioco.md",
            "stanza_iniziale",
            format!("la stanza «{}» non esiste", gioco.stanza_iniziale),
        ));
    }

    for stanza in stanze.values() {
        for (direzione, uscita) in &stanza.uscite {
            if !stanze.contains_key(&uscita.verso) {
                problemi.push(Problema::errore(
                    &file_relativo(stanza),
                    &format!("uscite.{direzione}.verso"),
                    format!("la stanza di destinazione «{}» non esiste", uscita.verso),
                ));
            }
        }
    }

    // Oggetti noti: stanze, inventario iniziale, catalogo.
    let mut noti: BTreeSet<&str> = BTreeSet::new();
    for stanza in stanze.values() {
        noti.extend(stanza.oggetti.iter().map(String::as_str));
        noti.extend(stanza.oggetti_nascosti.iter().map(String::as_str));
    }
    noti.extend(gioco.inventario_iniziale.iter().map(String::as_str));
    noti.extend(gioco.oggetti.keys().map(String::as_str));
    for stanza in stanze.values() {
        for (direzione, uscita) in &stanza.uscite {
            if let Some(oggetto) = &uscita.richiede_oggetto
                && !noti.contains(oggetto.as_str())
            {
                problemi.push(Problema::errore(
                    &file_relativo(stanza),
                    &format!("uscite.{direzione}.richiede_oggetto"),
                    format!(
                        "l'oggetto «{oggetto}» non compare in nessun elenco di oggetti \
                         (stanze, inventario iniziale, catalogo [oggetti])"
                    ),
                ));
            }
        }
    }

    // Oggetti dichiarati in più stanze.
    let mut posizioni: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for stanza in stanze.values() {
        let propri: BTreeSet<&str> = stanza
            .oggetti
            .iter()
            .chain(&stanza.oggetti_nascosti)
            .map(String::as_str)
            .collect();
        for oggetto in propri {
            posizioni.entry(oggetto).or_default().push(&stanza.id);
        }
    }
    for (oggetto, dove) in &posizioni {
        if dove.len() > 1 {
            problemi.push(Problema::avviso(
                "stanze/",
                "oggetti",
                format!(
                    "l'oggetto «{oggetto}» è dichiarato in più stanze: {}",
                    dove.join(", ")
                ),
            ));
        }
    }

    // Raggiungibilità (ignorando le condizioni delle uscite).
    if stanze.contains_key(&gioco.stanza_iniziale) {
        let mut visitate: BTreeSet<&str> = BTreeSet::new();
        let mut coda: VecDeque<&str> = VecDeque::from([gioco.stanza_iniziale.as_str()]);
        while let Some(id) = coda.pop_front() {
            if !visitate.insert(id) {
                continue;
            }
            if let Some(stanza) = stanze.get(id) {
                coda.extend(stanza.uscite.values().map(|u| u.verso.as_str()));
            }
        }
        for stanza in stanze.values() {
            if !visitate.contains(stanza.id.as_str()) && !gioco.spostamenti_liberi {
                problemi.push(Problema::avviso(
                    &file_relativo(stanza),
                    "",
                    format!(
                        "la stanza «{}» non è raggiungibile dalla stanza iniziale",
                        stanza.id
                    ),
                ));
            }
        }
    }

    // Traduzioni (§7.6): id sconosciuti e voci mancanti sono solo avvisi.
    for (lingua, traduzione) in &caricamento.traduzioni {
        let file = format!("{CARTELLA_TRADUZIONI}/{}.toml", lingua.codice());
        for id in traduzione.stanze.keys() {
            if !stanze.contains_key(id) {
                problemi.push(Problema::avviso(
                    &file,
                    "stanze",
                    format!("la stanza «{id}» non esiste"),
                ));
            }
        }
        for id in traduzione.oggetti.keys() {
            if !noti.contains(id.as_str()) {
                problemi.push(Problema::avviso(
                    &file,
                    "oggetti",
                    format!("l'oggetto «{id}» non esiste"),
                ));
            }
        }
        for chiave in traduzione.uscite.keys() {
            let esiste = chiave
                .split_once('.')
                .is_some_and(|(s, d)| stanze.get(s).is_some_and(|s| s.uscite.contains_key(d)));
            if !esiste {
                problemi.push(Problema::avviso(
                    &file,
                    "uscite",
                    format!("l'uscita «{chiave}» non esiste (formato: id_stanza.direzione)"),
                ));
            }
        }
        let mancanti: Vec<&str> = stanze
            .keys()
            .filter(|id| !traduzione.stanze.contains_key(*id))
            .map(String::as_str)
            .collect();
        if !mancanti.is_empty() {
            problemi.push(Problema::avviso(
                &file,
                "stanze",
                format!("nomi non tradotti: {}", mancanti.join(", ")),
            ));
        }
    }

    let ha_errori = problemi.iter().any(|p| p.gravita == Gravita::Errore);
    Rapporto {
        cartella: cartella.to_path_buf(),
        problemi,
        avventura: (!ha_errori).then(|| {
            let mut avventura = Avventura::da_parti(cartella, gioco, stanze);
            avventura.traduzioni = caricamento.traduzioni;
            avventura
        }),
    }
}

fn file_relativo(stanza: &Stanza) -> String {
    let nome = stanza
        .file
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| format!("{}.md", stanza.id));
    format!("stanze/{nome}")
}

#[cfg(test)]
mod test {
    use super::*;
    use std::fs;

    fn scrivi(dir: &Path, relativo: &str, testo: &str) {
        let p = dir.join(relativo);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, testo).unwrap();
    }

    fn stanza(id: &str, extra: &str) -> String {
        format!("+++\nid = \"{id}\"\nnome = \"{id}\"\n{extra}\n+++\n## Descrizione\nx\n")
    }

    #[test]
    fn avventura_corretta_senza_problemi() {
        let dir = tempfile::tempdir().unwrap();
        let d = dir.path();
        scrivi(
            d,
            "gioco.md",
            "+++\ntitolo = \"T\"\nstanza_iniziale = \"a\"\n+++\n",
        );
        scrivi(
            d,
            "stanze/a.md",
            &stanza("a", "[uscite.est]\nverso = \"b\""),
        );
        scrivi(
            d,
            "stanze/b.md",
            &stanza("b", "[uscite.ovest]\nverso = \"a\""),
        );
        let r = valida_cartella(d);
        assert!(r.problemi.is_empty(), "{}", r.formatta());
        assert!(r.avventura.is_some());
    }

    #[test]
    fn trova_errori_e_avvisi() {
        let dir = tempfile::tempdir().unwrap();
        let d = dir.path();
        scrivi(
            d,
            "gioco.md",
            "+++\ntitolo = \"T\"\nstanza_iniziale = \"zz\"\n+++\n",
        );
        scrivi(
            d,
            "stanze/a.md",
            &stanza(
                "a",
                "oggetti = [\"sasso\"]\n[uscite.est]\nverso = \"nessuna\"\nrichiede_oggetto = \"spada\"",
            ),
        );
        scrivi(d, "stanze/b.md", &stanza("a", "oggetti = [\"sasso\"]"));
        let r = valida_cartella(d);
        let t = r.formatta();
        assert!(t.contains("gioco.md (stanza_iniziale)"), "{t}");
        assert!(t.contains("uscite.est.verso"), "{t}");
        assert!(t.contains("uscite.est.richiede_oggetto"), "{t}");
        assert!(t.contains("diverso dal nome del file"), "{t}");
        assert!(t.contains("duplicato"), "{t}");
        assert!(r.ha_errori());
        assert!(r.avventura.is_none());
    }

    #[test]
    fn stanze_irraggiungibili_e_oggetti_doppi_sono_avvisi() {
        let dir = tempfile::tempdir().unwrap();
        let d = dir.path();
        scrivi(
            d,
            "gioco.md",
            "+++\ntitolo = \"T\"\nstanza_iniziale = \"a\"\n+++\n",
        );
        scrivi(d, "stanze/a.md", &stanza("a", "oggetti = [\"sasso\"]"));
        scrivi(
            d,
            "stanze/b.md",
            &stanza("b", "oggetti_nascosti = [\"sasso\"]"),
        );
        let r = valida_cartella(d);
        assert!(!r.ha_errori(), "{}", r.formatta());
        assert_eq!(r.numero(Gravita::Avviso), 2, "{}", r.formatta());
        assert!(r.avventura.is_some());
    }

    #[test]
    fn traduzioni_caricate_e_controllate() {
        use crate::lingua::{self, Lingua};
        let dir = tempfile::tempdir().unwrap();
        scrivi(
            dir.path(),
            "gioco.md",
            "+++\ntitolo = \"T\"\nlingua = \"en\"\nstanza_iniziale = \"a\"\n+++\n## Introduction\nHi\n",
        );
        scrivi(dir.path(), "stanze/a.md", &stanza("a", ""));
        scrivi(
            dir.path(),
            "traduzioni/it.toml",
            "titolo = \"Titolo\"\nintroduzione = \"Ciao\"\n[stanze]\nzz = \"Z\"\n[uscite]\n\"a.nord\" = \"x\"\n",
        );
        scrivi(dir.path(), "traduzioni/ja.toml", "titolo = \"Taitoru\"\n");
        scrivi(dir.path(), "traduzioni/de.toml", "titolo = \"Titel\"\n");
        let r = valida_cartella(dir.path());
        let testi: Vec<String> = r.problemi.iter().map(ToString::to_string).collect();
        let tutti = testi.join("\n");
        assert!(!r.ha_errori(), "{tutti}");
        assert!(tutti.contains("la stanza «zz» non esiste"), "{tutti}");
        assert!(tutti.contains("l'uscita «a.nord» non esiste"), "{tutti}");
        assert!(tutti.contains("nomi non tradotti: a"), "{tutti}");
        assert!(tutti.contains("lingua «ja» non supportata"), "{tutti}");
        let a = r.avventura.unwrap();
        assert_eq!(a.traduzioni.len(), 2);
        lingua::imposta(Lingua::De);
        assert_eq!(a.titolo(), "Titel");
        lingua::imposta(Lingua::Fr);
        assert_eq!(a.titolo(), "T");
        lingua::imposta(Lingua::It);
        assert_eq!(a.titolo(), "Titolo");
        assert_eq!(a.introduzione(), "Ciao");
        assert_eq!(a.nome_stanza("a"), "a");
        lingua::imposta(Lingua::En);
        assert_eq!(a.titolo(), "T");
        assert_eq!(a.introduzione(), "Hi");
        lingua::imposta(Lingua::It);
    }
}
