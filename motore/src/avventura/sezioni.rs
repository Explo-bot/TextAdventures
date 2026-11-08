//! Estrazione delle sezioni di secondo livello (`## Titolo`) dal corpo Markdown.
//!
//! Le sezioni restano nell'ordine del file; la ricerca per titolo ignora
//! maiuscole/minuscole e spazi iniziali/finali. Il testo prima della prima
//! sezione viene ignorato; le righe dentro i blocchi di codice (```) non
//! vengono interpretate come titoli.

/// Mappa ordinata `titolo sezione → testo`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Sezioni {
    voci: Vec<(String, String)>,
}

impl Sezioni {
    /// Analizza il corpo di un file Markdown (già normalizzato a `\n`).
    pub fn analizza(corpo: &str) -> Self {
        let mut voci = Vec::new();
        let mut corrente: Option<(String, Vec<&str>)> = None;
        let mut in_codice = false;
        for riga in corpo.lines() {
            if riga.trim_start().starts_with("```") {
                in_codice = !in_codice;
            }
            let titolo = if in_codice {
                None
            } else {
                titolo_sezione(riga)
            };
            match titolo {
                Some(t) => {
                    if let Some((titolo, righe)) = corrente.take() {
                        voci.push((titolo, righe.join("\n").trim().to_string()));
                    }
                    corrente = Some((t.to_string(), Vec::new()));
                }
                None => {
                    if let Some((_, righe)) = corrente.as_mut() {
                        righe.push(riga);
                    }
                }
            }
        }
        if let Some((titolo, righe)) = corrente {
            voci.push((titolo, righe.join("\n").trim().to_string()));
        }
        Sezioni { voci }
    }

    /// Testo della sezione con il titolo indicato (senza distinzione maiuscole/minuscole).
    /// I titoli standard valgono anche nell'altra lingua (§3): `Descrizione` trova
    /// anche `## Description`, e viceversa.
    pub fn get(&self, titolo: &str) -> Option<&str> {
        let cercato = canonico(titolo);
        self.voci
            .iter()
            .find(|(t, _)| canonico(t) == cercato)
            .map(|(_, c)| c.as_str())
    }

    /// Vero se la sezione esiste.
    pub fn contiene(&self, titolo: &str) -> bool {
        self.get(titolo).is_some()
    }

    /// Itera le sezioni nell'ordine del file.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        self.voci.iter().map(|(t, c)| (t.as_str(), c.as_str()))
    }

    /// Numero di sezioni.
    pub fn len(&self) -> usize {
        self.voci.len()
    }

    /// Vero se non ci sono sezioni.
    pub fn is_empty(&self) -> bool {
        self.voci.is_empty()
    }
}

/// Titoli standard delle sezioni nelle due lingue: `(italiano, inglese…)`.
pub const TITOLI_EQUIVALENTI: &[(&str, &[&str])] = &[
    ("introduzione", &["introduction"]),
    ("ambientazione", &["setting", "background"]),
    ("obiettivi", &["goals", "objectives"]),
    (
        "condizioni di vittoria",
        &["victory conditions", "win conditions"],
    ),
    (
        "condizioni di sconfitta",
        &["defeat conditions", "lose conditions"],
    ),
    ("tono e stile", &["tone and style", "tone"]),
    ("regole del mondo", &["world rules", "rules of the world"]),
    ("descrizione", &["description"]),
    ("dettagli", &["details"]),
    ("segreti", &["secrets"]),
    ("meccaniche", &["mechanics"]),
];

/// Forma di confronto di un titolo: minuscole, spazi normalizzati, titoli
/// inglesi standard ricondotti a quelli italiani.
fn canonico(titolo: &str) -> String {
    let t = titolo
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase();
    TITOLI_EQUIVALENTI
        .iter()
        .find(|(_, inglesi)| inglesi.contains(&t.as_str()))
        .map_or(t, |(italiano, _)| italiano.to_string())
}

/// Se la riga è un titolo di secondo livello restituisce il titolo.
fn titolo_sezione(riga: &str) -> Option<&str> {
    riga.trim()
        .strip_prefix("## ")
        .map(str::trim)
        .filter(|t| !t.is_empty())
}

#[cfg(test)]
mod test {
    use super::*;

    const CORPO: &str = "Preambolo ignorato\n## Descrizione\nUna stanza.\n\nSeconda riga.\n### Sotto\nancora descrizione\n##  Segreti  \nNascosto\n";

    #[test]
    fn estrae_sezioni_in_ordine() {
        let s = Sezioni::analizza(CORPO);
        let titoli: Vec<&str> = s.iter().map(|(t, _)| t).collect();
        assert_eq!(titoli, vec!["Descrizione", "Segreti"]);
        assert_eq!(
            s.get("descrizione").unwrap(),
            "Una stanza.\n\nSeconda riga.\n### Sotto\nancora descrizione"
        );
    }

    #[test]
    fn confronto_senza_maiuscole_e_spazi() {
        let s = Sezioni::analizza(CORPO);
        assert_eq!(s.get("  SEGRETI "), Some("Nascosto"));
        assert!(s.contiene("Descrizione"));
        assert!(!s.contiene("Meccaniche"));
    }

    #[test]
    fn titoli_inglesi_equivalenti() {
        let s = Sezioni::analizza(
            "## Description
A room.
## Victory Conditions
Win.",
        );
        assert_eq!(s.get("Descrizione"), Some("A room."));
        assert_eq!(s.get("description"), Some("A room."));
        assert_eq!(s.get("Condizioni di vittoria"), Some("Win."));
        let it = Sezioni::analizza(
            "## Segreti
x",
        );
        assert_eq!(it.get("Secrets"), Some("x"));
    }

    #[test]
    fn ignora_titoli_nei_blocchi_di_codice() {
        let s = Sezioni::analizza("## A\n```\n## non titolo\n```\n## B\nx");
        assert_eq!(s.len(), 2);
        assert!(s.get("A").unwrap().contains("## non titolo"));
    }

    #[test]
    fn sezione_vuota() {
        let s = Sezioni::analizza("## Vuota\n## Piena\nx");
        assert_eq!(s.get("vuota"), Some(""));
        assert!(Sezioni::analizza("niente").is_empty());
    }
}
