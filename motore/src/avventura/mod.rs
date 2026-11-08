//! Modello dati di un'avventura e caricamento dai file Markdown.
//!
//! Un'avventura è una cartella con `gioco.md` (ambientazione, obiettivi, regole)
//! e `stanze/*.md` (una stanza per file). Ogni file ha un front matter TOML
//! delimitato da `+++` e un corpo diviso in sezioni `## Titolo`.
//!
//! Il caricamento "grezzo" ([`carica_file`]) raccoglie tutti i problemi senza
//! fermarsi al primo, così la validazione statica può elencarli; [`Avventura::carica`]
//! invece accetta solo avventure senza errori.

pub mod frontmatter;
pub mod sezioni;

use std::collections::BTreeMap;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Result, bail};
use serde::Deserialize;

use crate::validazione_statica;
use sezioni::Sezioni;

/// Front matter e sezioni di `gioco.md`.
#[derive(Debug, Clone, Deserialize)]
pub struct Gioco {
    /// Titolo mostrato nel menu.
    pub titolo: String,
    /// Frase mostrata sotto il titolo nel menu di scelta.
    #[serde(default)]
    pub descrizione_breve: String,
    /// Autore dell'avventura.
    #[serde(default)]
    pub autore: String,
    /// Versione dell'avventura.
    #[serde(default)]
    pub versione: String,
    /// Lingua della narrazione (codice ISO, es. `it`).
    #[serde(default = "lingua_predefinita")]
    pub lingua: String,
    /// Identificativo comune alle versioni in lingue diverse della stessa
    /// storia (§7.6): il menu ne mostra una sola, nella lingua dell'interfaccia.
    /// Assente = l'id della cartella.
    #[serde(default)]
    pub storia: Option<String>,
    /// Id della stanza in cui inizia la partita.
    pub stanza_iniziale: String,
    /// Numero massimo di turni (superato → sconfitta).
    #[serde(default)]
    pub max_turni: Option<u32>,
    /// Oggetti posseduti all'inizio.
    #[serde(default)]
    pub inventario_iniziale: Vec<String>,
    /// Se vero l'AI può spostare il giocatore anche fuori dalle uscite dichiarate.
    #[serde(default)]
    pub spostamenti_liberi: bool,
    /// Flag con il loro valore iniziale.
    #[serde(default)]
    pub flag_iniziali: BTreeMap<String, serde_json::Value>,
    /// Catalogo `id oggetto → nome leggibile`.
    #[serde(default)]
    pub oggetti: BTreeMap<String, String>,
    /// Testo TOML originale del front matter (passato all'AI).
    #[serde(skip)]
    pub front_matter: String,
    /// Sezioni del corpo.
    #[serde(skip)]
    pub sezioni: Sezioni,
}

fn lingua_predefinita() -> String {
    "it".to_string()
}

impl Gioco {
    /// Lingua dell'avventura tra quelle supportate (inglese se non riconosciuta).
    pub fn lingua_supportata(&self) -> crate::lingua::Lingua {
        crate::lingua::Lingua::da_codice(&self.lingua).unwrap_or(crate::lingua::Lingua::En)
    }
}

/// Un'uscita di una stanza.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct Uscita {
    /// Id della stanza di destinazione.
    pub verso: String,
    /// Descrizione dell'uscita (es. "una botola nel pavimento").
    #[serde(default)]
    pub descrizione: String,
    /// Flag che deve essere "vero" perché l'uscita sia percorribile.
    #[serde(default)]
    pub richiede_flag: Option<String>,
    /// Oggetto da avere in inventario per percorrere l'uscita.
    #[serde(default)]
    pub richiede_oggetto: Option<String>,
    /// Se vera, l'uscita non va menzionata finché non è disponibile.
    #[serde(default)]
    pub nascosta: bool,
}

/// Una stanza, letta da `stanze/<id>.md`.
#[derive(Debug, Clone, Deserialize)]
pub struct Stanza {
    /// Identificatore (coincide col nome del file).
    pub id: String,
    /// Nome leggibile.
    pub nome: String,
    /// Oggetti visibili all'inizio.
    #[serde(default)]
    pub oggetti: Vec<String>,
    /// Oggetti presenti ma da scoprire.
    #[serde(default)]
    pub oggetti_nascosti: Vec<String>,
    /// Uscite per direzione.
    #[serde(default)]
    pub uscite: BTreeMap<String, Uscita>,
    /// Testo TOML originale del front matter.
    #[serde(skip)]
    pub front_matter: String,
    /// Sezioni del corpo.
    #[serde(skip)]
    pub sezioni: Sezioni,
    /// Percorso del file da cui è stata letta.
    #[serde(skip)]
    pub file: PathBuf,
}

impl Stanza {
    /// Sezione `Descrizione` (vuota se manca).
    pub fn descrizione(&self) -> &str {
        self.sezioni.get("Descrizione").unwrap_or("")
    }
}

/// Gravità di un problema trovato durante il caricamento o la validazione.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Gravita {
    /// Impedisce di giocare l'avventura.
    Errore,
    /// Segnalazione che non blocca il gioco.
    Avviso,
}

/// Un problema riferito a un file e a un campo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Problema {
    /// Errore o avviso.
    pub gravita: Gravita,
    /// File relativo alla cartella dell'avventura (es. `stanze/ingresso.md`).
    pub file: String,
    /// Campo o sezione interessata.
    pub campo: String,
    /// Spiegazione in italiano.
    pub messaggio: String,
}

impl Problema {
    /// Crea un errore.
    pub fn errore(file: &str, campo: &str, messaggio: impl Into<String>) -> Self {
        Problema {
            gravita: Gravita::Errore,
            file: file.to_string(),
            campo: campo.to_string(),
            messaggio: messaggio.into(),
        }
    }

    /// Crea un avviso.
    pub fn avviso(file: &str, campo: &str, messaggio: impl Into<String>) -> Self {
        Problema {
            gravita: Gravita::Avviso,
            ..Problema::errore(file, campo, messaggio)
        }
    }
}

impl fmt::Display for Problema {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let etichetta = match self.gravita {
            Gravita::Errore => "errore",
            Gravita::Avviso => "avviso",
        };
        if self.campo.is_empty() {
            write!(f, "[{etichetta}] {}: {}", self.file, self.messaggio)
        } else {
            write!(
                f,
                "[{etichetta}] {} ({}): {}",
                self.file, self.campo, self.messaggio
            )
        }
    }
}

/// Traduzione facoltativa dei testi che il motore mostra direttamente al
/// giocatore (§7.6), letta da `traduzioni/<codice lingua>.toml`. Tutto ciò che
/// manca resta nella lingua dell'avventura; la narrazione invece la scrive
/// l'AI direttamente nella lingua dell'interfaccia.
#[derive(Debug, Clone, Default, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Traduzione {
    /// Titolo.
    #[serde(default)]
    pub titolo: Option<String>,
    /// Frase sotto il titolo nel menu.
    #[serde(default)]
    pub descrizione_breve: Option<String>,
    /// Testo della sezione `Introduzione`.
    #[serde(default)]
    pub introduzione: Option<String>,
    /// `id stanza → nome`.
    #[serde(default)]
    pub stanze: BTreeMap<String, String>,
    /// `id oggetto → nome`.
    #[serde(default)]
    pub oggetti: BTreeMap<String, String>,
    /// `"id_stanza.direzione" → descrizione dell'uscita`.
    #[serde(default)]
    pub uscite: BTreeMap<String, String>,
}

/// Cartella delle traduzioni dentro l'avventura.
pub const CARTELLA_TRADUZIONI: &str = "traduzioni";

/// Risultato del caricamento grezzo dei file di un'avventura.
#[derive(Debug, Default)]
pub struct Caricamento {
    /// `gioco.md`, se leggibile.
    pub gioco: Option<Gioco>,
    /// Stanze lette correttamente (anche se con problemi non strutturali).
    pub stanze: Vec<Stanza>,
    /// Traduzioni lette da `traduzioni/*.toml`, per lingua.
    pub traduzioni: BTreeMap<crate::lingua::Lingua, Traduzione>,
    /// Problemi trovati.
    pub problemi: Vec<Problema>,
}

/// Un'avventura completa e valida.
#[derive(Debug, Clone)]
pub struct Avventura {
    /// Id = nome della sottocartella.
    pub id: String,
    /// Cartella dell'avventura.
    pub cartella: PathBuf,
    /// Contenuto di `gioco.md`.
    pub gioco: Gioco,
    /// Stanze per id.
    pub stanze: BTreeMap<String, Stanza>,
    /// Catalogo complessivo `id oggetto → nome leggibile`.
    pub catalogo_oggetti: BTreeMap<String, String>,
    /// Traduzioni facoltative per lingua (§7.6).
    pub traduzioni: BTreeMap<crate::lingua::Lingua, Traduzione>,
}

impl Avventura {
    /// Carica e valida un'avventura; fallisce se la validazione statica trova errori.
    pub fn carica(cartella: &Path) -> Result<Avventura> {
        let rapporto = validazione_statica::valida_cartella(cartella);
        if let Some(avventura) = rapporto.avventura {
            return Ok(avventura);
        }
        let errori: Vec<String> = rapporto
            .problemi
            .iter()
            .filter(|p| p.gravita == Gravita::Errore)
            .map(ToString::to_string)
            .collect();
        bail!(
            "l'avventura in «{}» non è valida:\n  {}",
            cartella.display(),
            errori.join("\n  ")
        )
    }

    /// Costruisce l'avventura dalle parti già lette e calcola il catalogo oggetti.
    pub fn da_parti(cartella: &Path, gioco: Gioco, stanze: BTreeMap<String, Stanza>) -> Self {
        let mut catalogo: BTreeMap<String, String> = BTreeMap::new();
        let mut aggiungi = |id: &str| {
            catalogo
                .entry(id.to_string())
                .or_insert_with(|| nome_da_id(id));
        };
        for stanza in stanze.values() {
            stanza
                .oggetti
                .iter()
                .chain(&stanza.oggetti_nascosti)
                .for_each(|o| aggiungi(o));
        }
        gioco.inventario_iniziale.iter().for_each(|o| aggiungi(o));
        for (id, nome) in &gioco.oggetti {
            catalogo.insert(id.clone(), nome.clone());
        }
        Avventura {
            id: id_da_cartella(cartella),
            cartella: cartella.to_path_buf(),
            gioco,
            stanze,
            catalogo_oggetti: catalogo,
            traduzioni: BTreeMap::new(),
        }
    }

    /// Lingua in cui è scritta l'avventura (inglese se non supportata). Il
    /// prompt per l'AI è in italiano per le avventure in italiano e in inglese
    /// per tutte le altre ([`crate::lingua::Lingua::lingua_prompt`]).
    pub fn lingua(&self) -> crate::lingua::Lingua {
        self.gioco.lingua_supportata()
    }

    /// Stanza per id.
    pub fn stanza(&self, id: &str) -> Option<&Stanza> {
        self.stanze.get(id)
    }

    /// Traduzione per la lingua dell'interfaccia del thread, se l'avventura è
    /// scritta in un'altra lingua e la traduzione esiste.
    pub fn traduzione(&self) -> Option<&Traduzione> {
        self.traduzione_in(crate::lingua::attuale())
    }

    /// Traduzione per una lingua (nessuna se coincide con quella dell'avventura).
    pub fn traduzione_in(&self, lingua: crate::lingua::Lingua) -> Option<&Traduzione> {
        if crate::lingua::Lingua::da_codice(&self.gioco.lingua) == Some(lingua) {
            return None;
        }
        self.traduzioni.get(&lingua)
    }

    /// Titolo nella lingua dell'interfaccia (l'originale se manca la traduzione).
    pub fn titolo(&self) -> String {
        self.traduzione()
            .and_then(|t| t.titolo.clone())
            .unwrap_or_else(|| self.gioco.titolo.clone())
    }

    /// Descrizione breve nella lingua dell'interfaccia.
    pub fn descrizione_breve(&self) -> String {
        self.traduzione()
            .and_then(|t| t.descrizione_breve.clone())
            .unwrap_or_else(|| self.gioco.descrizione_breve.clone())
    }

    /// Introduzione nella lingua dell'interfaccia (vuota se manca).
    pub fn introduzione(&self) -> String {
        self.traduzione()
            .and_then(|t| t.introduzione.clone())
            .or_else(|| self.gioco.sezioni.get("Introduzione").map(str::to_string))
            .unwrap_or_default()
    }

    /// Nome di una stanza nella lingua dell'interfaccia (l'id se non esiste).
    pub fn nome_stanza(&self, id: &str) -> String {
        if let Some(nome) = self.traduzione().and_then(|t| t.stanze.get(id)) {
            return nome.clone();
        }
        self.nome_stanza_originale(id)
    }

    /// Nome di una stanza come scritto nell'avventura.
    pub fn nome_stanza_originale(&self, id: &str) -> String {
        self.stanze
            .get(id)
            .map_or_else(|| id.to_string(), |s| s.nome.clone())
    }

    /// Nome di un oggetto nella lingua dell'interfaccia (dall'id se non è nel catalogo).
    pub fn nome_oggetto(&self, id: &str) -> String {
        if let Some(nome) = self.traduzione().and_then(|t| t.oggetti.get(id)) {
            return nome.clone();
        }
        self.nome_oggetto_originale(id)
    }

    /// Nome di un oggetto come scritto nell'avventura (dall'id se non è nel catalogo).
    pub fn nome_oggetto_originale(&self, id: &str) -> String {
        self.catalogo_oggetti
            .get(id)
            .cloned()
            .unwrap_or_else(|| nome_da_id(id))
    }

    /// Descrizione di un'uscita nella lingua dell'interfaccia.
    pub fn descrizione_uscita(&self, stanza: &str, direzione: &str, originale: &str) -> String {
        self.traduzione()
            .and_then(|t| t.uscite.get(&format!("{stanza}.{direzione}")))
            .cloned()
            .unwrap_or_else(|| originale.to_string())
    }

    /// Vero se l'oggetto è nel catalogo complessivo.
    pub fn oggetto_esiste(&self, id: &str) -> bool {
        self.catalogo_oggetti.contains_key(id)
    }
}

/// Id di un'avventura = nome della cartella.
pub fn id_da_cartella(cartella: &Path) -> String {
    let nome = cartella
        .file_name()
        .map(|n| n.to_string_lossy().to_string());
    match nome {
        Some(n) if !n.is_empty() && n != "." && n != ".." => n,
        _ => cartella
            .canonicalize()
            .ok()
            .and_then(|c| c.file_name().map(|n| n.to_string_lossy().to_string()))
            .unwrap_or_else(|| "avventura".to_string()),
    }
}

/// Nome leggibile ricavato da un id (`chiave_ottone` → `chiave ottone`).
pub fn nome_da_id(id: &str) -> String {
    id.replace(['_', '-'], " ")
}

/// Legge `gioco.md` e tutte le stanze raccogliendo i problemi.
pub fn carica_file(cartella: &Path) -> Caricamento {
    let mut c = Caricamento::default();
    let percorso_gioco = cartella.join("gioco.md");
    match leggi_documento::<Gioco>(&percorso_gioco, "gioco.md") {
        Ok((mut gioco, documento)) => {
            gioco.sezioni = Sezioni::analizza(&documento.corpo);
            gioco.front_matter = documento.front_matter;
            c.gioco = Some(gioco);
        }
        Err(p) => c.problemi.push(p),
    }

    carica_traduzioni(cartella, &mut c);

    let cartella_stanze = cartella.join("stanze");
    let voci = match fs::read_dir(&cartella_stanze) {
        Ok(v) => v,
        Err(_) => {
            c.problemi.push(Problema::errore(
                "stanze/",
                "",
                "cartella delle stanze mancante o illeggibile",
            ));
            return c;
        }
    };
    let mut file: Vec<PathBuf> = voci
        .filter_map(|v| v.ok().map(|v| v.path()))
        .filter(|p| {
            p.is_file()
                && p.extension()
                    .is_some_and(|e| e.to_string_lossy().eq_ignore_ascii_case("md"))
        })
        .collect();
    file.sort();
    if file.is_empty() {
        c.problemi.push(Problema::errore(
            "stanze/",
            "",
            "nessuna stanza trovata (servono file .md)",
        ));
    }
    for percorso in file {
        let nome_file = percorso
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        let relativo = format!("stanze/{nome_file}");
        match leggi_documento::<Stanza>(&percorso, &relativo) {
            Ok((mut stanza, documento)) => {
                stanza.sezioni = Sezioni::analizza(&documento.corpo);
                stanza.front_matter = documento.front_matter;
                stanza.file = percorso.clone();
                let atteso = percorso
                    .file_stem()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_default();
                if stanza.id != atteso {
                    c.problemi.push(Problema::errore(
                        &relativo,
                        "id",
                        format!(
                            "l'id «{}» è diverso dal nome del file («{atteso}»)",
                            stanza.id
                        ),
                    ));
                }
                if !stanza.sezioni.contiene("Descrizione") {
                    c.problemi.push(Problema::errore(
                        &relativo,
                        "## Descrizione",
                        "sezione «Descrizione» mancante",
                    ));
                }
                c.stanze.push(stanza);
            }
            Err(p) => c.problemi.push(p),
        }
    }
    c
}

/// Legge `traduzioni/*.toml` (facoltativa). Un file con un nome di lingua non
/// supportato o non leggibile è segnalato come avviso e ignorato.
fn carica_traduzioni(cartella: &Path, c: &mut Caricamento) {
    let Ok(voci) = fs::read_dir(cartella.join(CARTELLA_TRADUZIONI)) else {
        return;
    };
    let mut file: Vec<PathBuf> = voci
        .filter_map(|v| v.ok().map(|v| v.path()))
        .filter(|p| {
            p.is_file()
                && p.extension()
                    .is_some_and(|e| e.to_string_lossy().eq_ignore_ascii_case("toml"))
        })
        .collect();
    file.sort();
    for percorso in file {
        let codice = percorso
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
        let relativo = format!("{CARTELLA_TRADUZIONI}/{codice}.toml");
        let Some(lingua) = crate::lingua::Lingua::da_codice(&codice) else {
            c.problemi.push(Problema::avviso(
                &relativo,
                "",
                format!(
                    "lingua «{codice}» non supportata (usa it, en, fr, es o de): file ignorato"
                ),
            ));
            continue;
        };
        let letto = fs::read_to_string(&percorso)
            .map_err(|e| format!("impossibile leggere il file: {e}"))
            .and_then(|t| {
                toml::from_str::<Traduzione>(t.trim_start_matches('\u{feff}'))
                    .map_err(|e| descrivi_errore_toml(&e, &t))
            });
        match letto {
            Ok(t) => {
                c.traduzioni.insert(lingua, t);
            }
            Err(e) => c.problemi.push(Problema::avviso(
                &relativo,
                "",
                format!("{e}: file ignorato"),
            )),
        }
    }
}

/// Legge un file Markdown e deserializza il suo front matter.
fn leggi_documento<T: serde::de::DeserializeOwned>(
    percorso: &Path,
    relativo: &str,
) -> std::result::Result<(T, frontmatter::Documento), Problema> {
    let testo = fs::read_to_string(percorso)
        .map_err(|e| Problema::errore(relativo, "", format!("impossibile leggere il file: {e}")))?;
    let documento = frontmatter::separa(&testo)
        .map_err(|e| Problema::errore(relativo, "front matter", e.to_string()))?;
    let valore: T = toml::from_str(&documento.front_matter).map_err(|e| {
        Problema::errore(
            relativo,
            "front matter",
            descrivi_errore_toml(&e, &documento.front_matter),
        )
    })?;
    Ok((valore, documento))
}

/// Traduce in italiano (per quanto possibile) un errore di parsing TOML.
fn descrivi_errore_toml(e: &toml::de::Error, sorgente: &str) -> String {
    let messaggio = e.message().trim().to_string();
    let tradotto = if let Some(resto) = messaggio.strip_prefix("missing field ") {
        format!("campo obbligatorio mancante: {resto}")
    } else if let Some(resto) = messaggio.strip_prefix("invalid type: ") {
        format!("tipo di valore non valido: {resto}")
    } else {
        messaggio
    };
    match e.span() {
        // +1 per la riga `+++` iniziale.
        Some(span) => {
            let riga = sorgente[..span.start.min(sorgente.len())]
                .matches('\n')
                .count()
                + 2;
            format!("TOML non valido (riga {riga} del file): {tradotto}")
        }
        None => format!("TOML non valido: {tradotto}"),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn scrivi(dir: &Path, relativo: &str, testo: &str) {
        let p = dir.join(relativo);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, testo).unwrap();
    }

    fn avventura_minima(dir: &Path) {
        scrivi(
            dir,
            "gioco.md",
            "+++\ntitolo = \"Prova\"\nstanza_iniziale = \"a\"\ninventario_iniziale = [\"moneta\"]\n[oggetti]\nchiave = \"una chiave\"\n+++\n## Introduzione\nCiao\n",
        );
        scrivi(
            dir,
            "stanze/a.md",
            "+++\nid = \"a\"\nnome = \"Stanza A\"\noggetti = [\"chiave\"]\noggetti_nascosti = [\"gemma_rossa\"]\n[uscite.nord]\nverso = \"b\"\n+++\n## Descrizione\nA.\n",
        );
        scrivi(
            dir,
            "stanze/b.md",
            "+++\nid = \"b\"\nnome = \"Stanza B\"\n[uscite.sud]\nverso = \"a\"\nrichiede_oggetto = \"chiave\"\n+++\n## Descrizione\nB.\n## Segreti\nS.\n",
        );
    }

    #[test]
    fn carica_avventura_valida() {
        let dir = tempfile::tempdir().unwrap();
        let cartella = dir.path().join("prova");
        avventura_minima(&cartella);
        let a = Avventura::carica(&cartella).unwrap();
        assert_eq!(a.id, "prova");
        assert_eq!(a.gioco.titolo, "Prova");
        assert_eq!(a.gioco.lingua, "it");
        assert_eq!(a.stanze.len(), 2);
        assert_eq!(a.nome_oggetto("chiave"), "una chiave");
        assert_eq!(a.nome_oggetto("gemma_rossa"), "gemma rossa");
        assert!(a.oggetto_esiste("moneta"));
        assert!(!a.oggetto_esiste("spada"));
        let b = a.stanza("b").unwrap();
        assert_eq!(b.descrizione(), "B.");
        assert_eq!(b.uscite["sud"].richiede_oggetto.as_deref(), Some("chiave"));
        assert_eq!(a.gioco.sezioni.get("introduzione"), Some("Ciao"));
    }

    #[test]
    fn segnala_toml_non_valido_e_id_errato() {
        let dir = tempfile::tempdir().unwrap();
        let cartella = dir.path().join("rotta");
        avventura_minima(&cartella);
        scrivi(
            &cartella,
            "stanze/c.md",
            "+++\nid = \"cc\"\nnome = \"C\"\n+++\n## Altro\nx\n",
        );
        scrivi(&cartella, "stanze/d.md", "+++\nid = = 3\n+++\n");
        let c = carica_file(&cartella);
        let testi: Vec<String> = c.problemi.iter().map(ToString::to_string).collect();
        assert!(testi.iter().any(|t| t.contains("stanze/c.md (id)")));
        assert!(testi.iter().any(|t| t.contains("«Descrizione» mancante")));
        assert!(
            testi
                .iter()
                .any(|t| t.contains("stanze/d.md (front matter)"))
        );
        assert!(Avventura::carica(&cartella).is_err());
    }

    #[test]
    fn campo_obbligatorio_mancante() {
        let dir = tempfile::tempdir().unwrap();
        scrivi(dir.path(), "gioco.md", "+++\ntitolo = \"x\"\n+++\n");
        let c = carica_file(dir.path());
        assert!(
            c.problemi
                .iter()
                .any(|p| p.messaggio.contains("campo obbligatorio mancante"))
        );
    }
}
