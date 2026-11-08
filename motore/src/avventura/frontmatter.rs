//! Separazione del front matter TOML (delimitato da righe `+++`) dal corpo Markdown.
//!
//! Tollera il BOM UTF-8 iniziale e i fine riga Windows (CRLF): il testo restituito
//! usa sempre `\n` come fine riga.

use thiserror::Error;

/// Errori di struttura del front matter.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum ErroreFrontMatter {
    /// Il file non inizia con una riga `+++`.
    #[error("front matter mancante: il file deve iniziare con una riga `+++`")]
    Mancante,
    /// Manca la riga `+++` di chiusura.
    #[error("front matter non chiuso: manca la seconda riga `+++`")]
    NonChiuso,
}

/// Un file Markdown diviso nelle sue due parti.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Documento {
    /// Testo TOML compreso tra le due righe `+++` (senza i delimitatori).
    pub front_matter: String,
    /// Tutto ciò che segue la riga `+++` di chiusura.
    pub corpo: String,
}

/// Toglie il BOM UTF-8 e converte i fine riga in `\n`.
pub fn normalizza(testo: &str) -> String {
    let senza_bom = testo.strip_prefix('\u{feff}').unwrap_or(testo);
    senza_bom.replace("\r\n", "\n").replace('\r', "\n")
}

/// Separa front matter e corpo. Le righe vuote prima del primo `+++` sono ammesse.
pub fn separa(testo: &str) -> Result<Documento, ErroreFrontMatter> {
    let testo = normalizza(testo);
    let righe: Vec<&str> = testo.split('\n').collect();
    let inizio = righe
        .iter()
        .position(|r| !r.trim().is_empty())
        .ok_or(ErroreFrontMatter::Mancante)?;
    if righe[inizio].trim() != "+++" {
        return Err(ErroreFrontMatter::Mancante);
    }
    let fine = righe[inizio + 1..]
        .iter()
        .position(|r| r.trim() == "+++")
        .map(|p| inizio + 1 + p)
        .ok_or(ErroreFrontMatter::NonChiuso)?;
    Ok(Documento {
        front_matter: righe[inizio + 1..fine].join("\n"),
        corpo: righe[fine + 1..].join("\n"),
    })
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn separa_front_matter_e_corpo() {
        let d = separa("+++\ntitolo = \"x\"\n+++\n\n## Introduzione\nCiao").unwrap();
        assert_eq!(d.front_matter, "titolo = \"x\"");
        assert_eq!(d.corpo, "\n## Introduzione\nCiao");
    }

    #[test]
    fn tollera_bom_e_crlf() {
        let d = separa("\u{feff}+++\r\na = 1\r\nb = 2\r\n+++\r\n## S\r\ntesto\r\n").unwrap();
        assert_eq!(d.front_matter, "a = 1\nb = 2");
        assert!(d.corpo.contains("## S\ntesto"));
        assert!(!d.corpo.contains('\r'));
    }

    #[test]
    fn righe_vuote_iniziali_ammesse() {
        let d = separa("\n\n+++\na = 1\n+++\ncorpo").unwrap();
        assert_eq!(d.front_matter, "a = 1");
        assert_eq!(d.corpo, "corpo");
    }

    #[test]
    fn front_matter_mancante() {
        assert_eq!(separa("## Solo corpo\n"), Err(ErroreFrontMatter::Mancante));
        assert_eq!(separa(""), Err(ErroreFrontMatter::Mancante));
    }

    #[test]
    fn front_matter_non_chiuso() {
        assert_eq!(
            separa("+++\na = 1\n## corpo"),
            Err(ErroreFrontMatter::NonChiuso)
        );
    }
}
