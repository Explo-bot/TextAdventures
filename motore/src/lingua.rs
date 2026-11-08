//! Lingue dell'interfaccia (§7.6): italiano, inglese, francese, spagnolo e tedesco.
//!
//! - La lingua **dell'interfaccia** (menu, messaggi del motore, comandi, bot
//!   Telegram) è per thread: ogni sessione (il terminale, oppure il thread di un
//!   utente Telegram) imposta la propria con [`imposta`] e i testi la leggono con
//!   le macro [`tr!`](crate::tr) e [`tr_in!`](crate::tr_in).
//! - Il **prompt per l'AI** è nella lingua dell'avventura (`lingua` in
//!   `gioco.md`) se è l'italiano, altrimenti in inglese; la **narrazione**
//!   l'AI la scrive nella lingua dell'interfaccia del giocatore.
//!
//! Scelta della lingua dell'interfaccia (dalla più forte):
//! `--lingua` → preferenza salvata (`preferenze.json`, scritta al primo utilizzo
//! o con `/language`) → `AVVENTURA_LINGUA` o `interfaccia.lingua` diversa da
//! `auto` → domanda al primo utilizzo (con la lingua rilevata come prima voce) →
//! lingua rilevata dall'ambiente (sistema operativo o app Telegram) → inglese.

use std::cell::Cell;
use std::fs;
use std::path::Path;

use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Lingua supportata dall'interfaccia.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum Lingua {
    /// Italiano (valore predefinito del tipo, per compatibilità con le avventure
    /// senza campo `lingua`).
    #[default]
    It,
    /// Inglese (predefinita quando la lingua dell'ambiente non è supportata).
    En,
    /// Francese.
    Fr,
    /// Spagnolo.
    Es,
    /// Tedesco.
    De,
}

/// Tutte le lingue supportate, nell'ordine dei menu.
pub const TUTTE: [Lingua; 5] = [Lingua::En, Lingua::It, Lingua::Fr, Lingua::Es, Lingua::De];

impl Lingua {
    /// Codice ISO 639-1 (`it`, `en`, `fr`, `es`, `de`).
    pub fn codice(self) -> &'static str {
        match self {
            Lingua::It => "it",
            Lingua::En => "en",
            Lingua::Fr => "fr",
            Lingua::Es => "es",
            Lingua::De => "de",
        }
    }

    /// Nome della lingua scritto nella lingua stessa.
    pub fn nome(self) -> &'static str {
        match self {
            Lingua::It => "Italiano",
            Lingua::En => "English",
            Lingua::Fr => "Français",
            Lingua::Es => "Español",
            Lingua::De => "Deutsch",
        }
    }

    /// Posizione nelle tabelle per lingua (stesso ordine delle varianti).
    fn indice(self) -> usize {
        self as usize
    }

    /// Interpreta un codice o un nome di lingua: `it`, `it-IT`, `it_IT.UTF-8`,
    /// `italiano`, `en_US`, `english`, `fr`, `français`, `es-ES`, `de_DE`…
    /// `None` se non supportata.
    pub fn da_codice(testo: &str) -> Option<Lingua> {
        let t = testo.trim().to_lowercase();
        let base: String = t.chars().take_while(|c| c.is_alphabetic()).collect();
        match base.as_str() {
            "it" | "ita" | "italiano" | "italian" => Some(Lingua::It),
            "en" | "eng" | "english" | "inglese" => Some(Lingua::En),
            "fr" | "fra" | "fre" | "français" | "francais" | "french" | "francese" => {
                Some(Lingua::Fr)
            }
            "es" | "spa" | "español" | "espanol" | "spanish" | "spagnolo" => Some(Lingua::Es),
            "de" | "deu" | "ger" | "deutsch" | "german" | "tedesco" => Some(Lingua::De),
            _ => None,
        }
    }

    /// Lingua del prompt per l'AI quando l'avventura è scritta in questa
    /// lingua: le regole del narratore esistono in italiano e in inglese, per
    /// le altre lingue si usa l'inglese.
    pub fn lingua_prompt(self) -> Lingua {
        if self == Lingua::It {
            Lingua::It
        } else {
            Lingua::En
        }
    }
}

impl Lingua {
    /// Nome della lingua scritto in un'altra lingua (per il prompt dell'AI):
    /// `Lingua::It.nome_in(Lingua::En)` = `"Italian"`. Le righe sono le lingue
    /// da nominare, le colonne quelle in cui scrivere (ordine delle varianti).
    pub fn nome_in(self, lingua: Lingua) -> &'static str {
        const NOMI: [[&str; 5]; 5] = [
            ["italiano", "Italian", "italien", "italiano", "Italienisch"],
            ["inglese", "English", "anglais", "inglés", "Englisch"],
            ["francese", "French", "français", "francés", "Französisch"],
            ["spagnolo", "Spanish", "espagnol", "español", "Spanisch"],
            ["tedesco", "German", "allemand", "alemán", "Deutsch"],
        ];
        NOMI[self.indice()][lingua.indice()]
    }
}

/// Direzioni standard: per ciascuna, i nomi in ogni lingua (ordine delle
/// varianti di [`Lingua`]); il primo nome è quello mostrato, gli altri sono
/// sinonimi riconosciuti.
const DIREZIONI: [[&[&str]; 5]; 8] = [
    [
        &["nord"],
        &["north"],
        &["nord"],
        &["norte"],
        &["norden", "nord"],
    ],
    [
        &["sud"],
        &["south"],
        &["sud"],
        &["sur"],
        &["süden", "süd", "sueden"],
    ],
    [&["est"], &["east"], &["est"], &["este"], &["osten", "ost"]],
    [
        &["ovest"],
        &["west"],
        &["ouest"],
        &["oeste"],
        &["westen", "west"],
    ],
    [
        &["su"],
        &["up"],
        &["haut"],
        &["arriba"],
        &["hoch", "oben", "hinauf", "rauf"],
    ],
    [
        &["giù", "giu"],
        &["down"],
        &["bas"],
        &["abajo"],
        &["runter", "unten", "hinunter"],
    ],
    [
        &["fuori"],
        &["out"],
        &["dehors"],
        &["fuera", "afuera"],
        &["raus", "hinaus", "draußen"],
    ],
    [
        &["dentro"],
        &["in"],
        &["dedans"],
        &["dentro", "adentro"],
        &["rein", "hinein", "drinnen"],
    ],
];

/// Posizione nella tabella delle direzioni della riga che contiene `d` (in
/// qualunque lingua).
fn riga_direzione(d: &str) -> Option<usize> {
    let d = d.trim().to_lowercase();
    DIREZIONI
        .iter()
        .position(|riga| riga.iter().any(|nomi| nomi.contains(&d.as_str())))
}

/// Direzione standard scritta nella lingua `l` o in inglese, restituita nella
/// forma inglese (`norden` → `north`); `None` se `parola` non è una direzione.
pub fn direzione_canonica(parola: &str, l: Lingua) -> Option<&'static str> {
    let p = parola.trim().to_lowercase();
    DIREZIONI
        .iter()
        .find(|riga| riga[l.indice()].contains(&p.as_str()) || riga[1].contains(&p.as_str()))
        .map(|riga| riga[1][0])
}

/// Nome di una direzione standard nella lingua indicata (`north` → `nord`);
/// le altre direzioni restano come sono. In tedesco i punti cardinali sono
/// sostantivi e hanno l'iniziale maiuscola (`Norden`).
pub fn traduci_direzione(direzione: &str, lingua: Lingua) -> String {
    let Some(posizione) = riga_direzione(direzione) else {
        return direzione.to_string();
    };
    let nome = DIREZIONI[posizione][lingua.indice()][0];
    if lingua == Lingua::De && posizione < 4 {
        let mut lettere = nome.chars();
        lettere
            .next()
            .map(|p| p.to_uppercase().chain(lettere).collect())
            .unwrap_or_default()
    } else {
        nome.to_string()
    }
}

/// Vero se due direzioni sono la stessa in lingue diverse (`nord` = `north`).
pub fn stessa_direzione(a: &str, b: &str) -> bool {
    let (a, b) = (a.trim().to_lowercase(), b.trim().to_lowercase());
    a == b
        || DIREZIONI.iter().any(|riga| {
            riga.iter().any(|n| n.contains(&a.as_str()))
                && riga.iter().any(|n| n.contains(&b.as_str()))
        })
}

/// Vero se `risposta` è un «sì» in una delle lingue supportate
/// (`s`, `sì`, `y`, `yes`, `o`, `oui`, `j`, `ja`…).
pub fn e_si(risposta: &str) -> bool {
    matches!(
        risposta.trim().to_lowercase().as_str(),
        "s" | "si" | "sì" | "sí" | "y" | "yes" | "o" | "oui" | "j" | "ja"
    )
}

/// Vero se `risposta` è un «no» in una delle lingue supportate.
pub fn e_no(risposta: &str) -> bool {
    matches!(
        risposta.trim().to_lowercase().as_str(),
        "n" | "no" | "non" | "nein"
    )
}

/// Etichetta «No» del pulsante o della risposta nella lingua corrente.
pub fn testo_no() -> &'static str {
    crate::tr!("No", "No", "Non", "No", "Nein")
}

thread_local! {
    /// Lingua dell'interfaccia del thread. Il valore iniziale (italiano) vale
    /// solo finché la sessione non imposta la propria.
    static ATTUALE: Cell<Lingua> = const { Cell::new(Lingua::It) };
}

/// Lingua dell'interfaccia del thread corrente.
pub fn attuale() -> Lingua {
    ATTUALE.with(Cell::get)
}

/// Imposta la lingua dell'interfaccia del thread corrente.
pub fn imposta(lingua: Lingua) {
    ATTUALE.with(|c| c.set(lingua));
}

/// Testo nella lingua del thread corrente:
/// `tr!("italiano", "english", "français", "español", "deutsch")`.
/// Valuta solo l'espressione della lingua scelta (vanno bene anche `format!`).
#[macro_export]
macro_rules! tr {
    ($it:expr, $en:expr, $fr:expr, $es:expr, $de:expr $(,)?) => {
        match $crate::lingua::attuale() {
            $crate::lingua::Lingua::It => $it,
            $crate::lingua::Lingua::En => $en,
            $crate::lingua::Lingua::Fr => $fr,
            $crate::lingua::Lingua::Es => $es,
            $crate::lingua::Lingua::De => $de,
        }
    };
}

/// Testo nella lingua indicata. Due forme:
/// - `tr_in!(lingua, it, en, fr, es, de)`: una variante per lingua;
/// - `tr_in!(lingua, it, en)`: per i testi destinati all'AI (prompt, motivi di
///   validazione), che esistono solo in italiano e in inglese: le altre lingue
///   usano l'inglese ([`Lingua::lingua_prompt`]).
#[macro_export]
macro_rules! tr_in {
    ($lingua:expr, $it:expr, $en:expr $(,)?) => {
        match $lingua {
            $crate::lingua::Lingua::It => $it,
            _ => $en,
        }
    };
    ($lingua:expr, $it:expr, $en:expr, $fr:expr, $es:expr, $de:expr $(,)?) => {
        match $lingua {
            $crate::lingua::Lingua::It => $it,
            $crate::lingua::Lingua::En => $en,
            $crate::lingua::Lingua::Fr => $fr,
            $crate::lingua::Lingua::Es => $es,
            $crate::lingua::Lingua::De => $de,
        }
    };
}

/// Lingua del sistema operativo: variabili `LC_ALL`, `LC_MESSAGES`, `LANG`
/// (Linux/macOS), poi le impostazioni internazionali del sistema (Windows,
/// macOS). `None` se non si riesce a determinarla o non è supportata.
pub fn rileva_sistema() -> Option<Lingua> {
    rileva_da(&|k| std::env::var(k).ok(), sys_locale::get_locale())
}

/// Come [`rileva_sistema`], con ambiente e impostazione di sistema forniti (test).
pub fn rileva_da(
    ambiente: &dyn Fn(&str) -> Option<String>,
    sistema: Option<String>,
) -> Option<Lingua> {
    for variabile in ["LC_ALL", "LC_MESSAGES", "LANG"] {
        if let Some(v) = ambiente(variabile) {
            let v = v.trim();
            // "C" e "POSIX" non indicano una lingua: si passa alla fonte successiva.
            if v.is_empty() || v == "C" || v == "POSIX" || v.starts_with("C.") {
                continue;
            }
            return Lingua::da_codice(v);
        }
    }
    sistema.as_deref().and_then(Lingua::da_codice)
}

/// Lingua forzata dalla variabile `AVVENTURA_LINGUA` (`it`, `en`; `auto` = nessuna).
pub fn da_variabile() -> Option<Lingua> {
    std::env::var("AVVENTURA_LINGUA")
        .ok()
        .and_then(|v| Lingua::da_codice(&v))
}

/// Preferenze di un giocatore (`preferenze.json` nella sua cartella delle partite).
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Preferenze {
    /// Lingua scelta (`it`, `en`).
    #[serde(default)]
    pub lingua: Option<String>,
}

/// Nome del file delle preferenze.
pub const FILE_PREFERENZE: &str = "preferenze.json";

/// Lingua salvata nelle preferenze di `cartella_partite`, se presente e valida.
pub fn preferita(cartella_partite: &Path) -> Option<Lingua> {
    let testo = fs::read_to_string(cartella_partite.join(FILE_PREFERENZE)).ok()?;
    let p: Preferenze = serde_json::from_str(&testo).ok()?;
    p.lingua.as_deref().and_then(Lingua::da_codice)
}

/// Salva la lingua scelta nelle preferenze di `cartella_partite`.
pub fn salva_preferita(cartella_partite: &Path, lingua: Lingua) -> Result<()> {
    let percorso = cartella_partite.join(FILE_PREFERENZE);
    let mut p: Preferenze = fs::read_to_string(&percorso)
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or_default();
    p.lingua = Some(lingua.codice().to_string());
    crate::stato::scrivi_json_atomico(&percorso, &p)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn codici_e_nomi() {
        assert_eq!(Lingua::da_codice("it-IT"), Some(Lingua::It));
        assert_eq!(Lingua::da_codice("it_IT.UTF-8"), Some(Lingua::It));
        assert_eq!(Lingua::da_codice("Italiano"), Some(Lingua::It));
        assert_eq!(Lingua::da_codice("en-GB"), Some(Lingua::En));
        assert_eq!(Lingua::da_codice("english"), Some(Lingua::En));
        assert_eq!(Lingua::da_codice("fr-FR"), Some(Lingua::Fr));
        assert_eq!(Lingua::da_codice("Français"), Some(Lingua::Fr));
        assert_eq!(Lingua::da_codice("es_ES.UTF-8"), Some(Lingua::Es));
        assert_eq!(Lingua::da_codice("de-AT"), Some(Lingua::De));
        assert_eq!(Lingua::da_codice("ja-JP"), None);
        assert_eq!(Lingua::da_codice("auto"), None);
        for l in TUTTE {
            assert_eq!(Lingua::da_codice(l.codice()), Some(l));
            assert_eq!(Lingua::da_codice(l.nome()), Some(l));
        }
        assert_eq!(Lingua::Fr.lingua_prompt(), Lingua::En);
        assert_eq!(Lingua::It.lingua_prompt(), Lingua::It);
        assert_eq!(Lingua::De.nome_in(Lingua::It), "tedesco");
        assert_eq!(Lingua::Es.nome_in(Lingua::En), "Spanish");
        assert_eq!(Lingua::En.nome(), "English");
        assert_eq!(Lingua::It.nome_in(Lingua::En), "Italian");
        assert_eq!(traduci_direzione("north", Lingua::It), "nord");
        assert_eq!(traduci_direzione("giu", Lingua::En), "down");
        assert_eq!(traduci_direzione("down", Lingua::It), "giù");
        assert_eq!(
            traduci_direzione("verso il pozzo", Lingua::It),
            "verso il pozzo"
        );
        assert!(stessa_direzione("nord", "North"));
        assert!(stessa_direzione("giù", "down") && stessa_direzione("giu", "down"));
        assert!(!stessa_direzione("su", "down"));
        assert_eq!(traduci_direzione("north", Lingua::Es), "norte");
        assert_eq!(traduci_direzione("ouest", Lingua::De), "Westen");
        assert_eq!(traduci_direzione("up", Lingua::De), "hoch");
        assert_eq!(traduci_direzione("in", Lingua::Fr), "dedans");
        assert!(stessa_direzione("Osten", "est") && stessa_direzione("abajo", "giù"));
        assert_eq!(direzione_canonica("norden", Lingua::De), Some("north"));
        assert_eq!(direzione_canonica("north", Lingua::De), Some("north"));
        assert_eq!(direzione_canonica("norte", Lingua::Fr), None);
        assert!(e_si("Oui") && e_si("ja") && e_si("sí") && !e_si("non"));
        assert!(e_no("nein") && e_no("non") && !e_no("ja"));
    }

    #[test]
    fn rilevamento_dall_ambiente() {
        let amb = |coppie: &'static [(&'static str, &'static str)]| {
            move |k: &str| {
                coppie
                    .iter()
                    .find(|(c, _)| *c == k)
                    .map(|(_, v)| v.to_string())
            }
        };
        assert_eq!(
            rileva_da(&amb(&[("LANG", "it_IT.UTF-8")]), None),
            Some(Lingua::It)
        );
        assert_eq!(
            rileva_da(&amb(&[("LC_ALL", "en_US.UTF-8"), ("LANG", "it_IT")]), None),
            Some(Lingua::En)
        );
        // "C" non indica una lingua: vale l'impostazione di sistema.
        assert_eq!(
            rileva_da(&amb(&[("LANG", "C.UTF-8")]), Some("it-IT".into())),
            Some(Lingua::It)
        );
        assert_eq!(rileva_da(&amb(&[]), Some("en-US".into())), Some(Lingua::En));
        assert_eq!(
            rileva_da(&amb(&[("LANG", "ja_JP")]), Some("it-IT".into())),
            None
        );
        assert_eq!(
            rileva_da(&amb(&[("LANG", "de_DE.UTF-8")]), None),
            Some(Lingua::De)
        );
        assert_eq!(rileva_da(&amb(&[]), None), None);
    }

    #[test]
    fn lingua_per_thread_e_macro() {
        let saluto = || tr!("ciao", "hello", "salut", "hola", "hallo");
        imposta(Lingua::En);
        assert_eq!(saluto(), "hello");
        let n = 3;
        assert_eq!(
            tr!(
                format!("{n} turni"),
                format!("{n} turns"),
                format!("{n} tours"),
                format!("{n} turnos"),
                format!("{n} Züge")
            ),
            "3 turns"
        );
        imposta(Lingua::Es);
        assert_eq!(saluto(), "hola");
        imposta(Lingua::De);
        assert_eq!(saluto(), "hallo");
        std::thread::spawn(|| assert_eq!(attuale(), Lingua::It))
            .join()
            .unwrap();
        imposta(Lingua::It);
        assert_eq!(saluto(), "ciao");
        assert_eq!(tr_in!(Lingua::En, "sì", "yes"), "yes");
        // Forma a due varianti (testi per l'AI): le altre lingue usano l'inglese.
        assert_eq!(tr_in!(Lingua::Fr, "sì", "yes"), "yes");
        assert_eq!(tr_in!(Lingua::Fr, "sì", "yes", "oui", "sí", "ja"), "oui");
    }

    #[test]
    fn preferenze_su_disco() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(preferita(dir.path()), None);
        salva_preferita(dir.path(), Lingua::En).unwrap();
        assert_eq!(preferita(dir.path()), Some(Lingua::En));
        salva_preferita(dir.path(), Lingua::It).unwrap();
        assert_eq!(preferita(dir.path()), Some(Lingua::It));
    }
}
