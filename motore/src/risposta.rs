//! Estrazione e validazione della risposta dell'AI (§6.4–6.5).
//!
//! L'estrazione tollera blocchi ```json, testo prima e dopo il JSON (si prende
//! il primo oggetto `{…}` bilanciato) e virgole finali. La validazione in
//! modalità severa rifiuta la risposta alla prima violazione elencando tutti i
//! problemi; in modalità permissiva scarta le singole parti non valide e
//! applica il resto.

use std::collections::BTreeMap;

use serde::Deserialize;
use serde_json::Value;

use crate::avventura::Avventura;
use crate::config::Validazione;
use crate::lingua::Lingua;
use crate::stato::{Esito, Stato};
use crate::tr_in;
use crate::uscite;

/// Una nota: stringa singola o elenco di stringhe.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(untagged)]
pub enum Nota {
    /// Una sola nota.
    Una(String),
    /// Più note.
    Piu(Vec<String>),
}

impl Nota {
    /// Note come elenco.
    pub fn elenco(&self) -> Vec<String> {
        match self {
            Nota::Una(s) => vec![s.clone()],
            Nota::Piu(v) => v.clone(),
        }
    }
}

/// Campo `aggiornamenti` della risposta: tutti i campi sono facoltativi.
#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
#[serde(default)]
pub struct Aggiornamenti {
    /// Nuova stanza corrente.
    pub stanza_corrente: Option<String>,
    /// Oggetti presi.
    pub inventario_aggiungi: Option<Vec<String>>,
    /// Oggetti persi.
    pub inventario_rimuovi: Option<Vec<String>>,
    /// Oggetti comparsi nelle stanze.
    pub oggetti_stanza_aggiungi: Option<BTreeMap<String, Vec<String>>>,
    /// Oggetti spariti dalle stanze.
    pub oggetti_stanza_rimuovi: Option<BTreeMap<String, Vec<String>>>,
    /// Flag da impostare.
    pub flag: Option<BTreeMap<String, Value>>,
    /// Note per stanza.
    pub note_stanza: Option<BTreeMap<String, Nota>>,
    /// Variazione del punteggio.
    pub punteggio_delta: Option<f64>,
    /// Nuovo esito (testuale, validato).
    pub esito: Option<String>,
}

/// Risposta dell'AI.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct RispostaAi {
    /// Testo da mostrare.
    #[serde(default)]
    pub narrazione: String,
    /// Cambiamenti dello stato.
    #[serde(default)]
    pub aggiornamenti: Option<Aggiornamenti>,
    /// Nuova memoria (assente = invariata).
    #[serde(default)]
    pub memoria: Option<String>,
}

/// Estrae il testo JSON dalla risposta del modello.
pub fn estrai_json(testo: &str) -> Option<String> {
    let candidato = contenuto_fence(testo)
        .and_then(|f| primo_oggetto(&f))
        .or_else(|| primo_oggetto(testo))?;
    Some(togli_virgole_finali(&candidato))
}

/// Contenuto del primo blocco ``` … ``` che contiene una `{`.
fn contenuto_fence(testo: &str) -> Option<String> {
    let mut resto = testo;
    while let Some(inizio) = resto.find("```") {
        let dopo = &resto[inizio + 3..];
        // Salta l'eventuale etichetta di linguaggio (es. `json`).
        let corpo_inizio = dopo.find('\n').map_or(0, |p| p + 1);
        let corpo = &dopo[corpo_inizio..];
        let fine = corpo.find("```")?;
        let contenuto = &corpo[..fine];
        if contenuto.contains('{') {
            return Some(contenuto.to_string());
        }
        resto = &corpo[fine + 3..];
    }
    None
}

/// Primo oggetto `{…}` bilanciato, ignorando le graffe dentro le stringhe.
fn primo_oggetto(testo: &str) -> Option<String> {
    let inizio = testo.find('{')?;
    let mut profondita = 0usize;
    let mut in_stringa = false;
    let mut escape = false;
    for (i, c) in testo[inizio..].char_indices() {
        if in_stringa {
            match (escape, c) {
                (true, _) => escape = false,
                (false, '\\') => escape = true,
                (false, '"') => in_stringa = false,
                _ => {}
            }
            continue;
        }
        match c {
            '"' => in_stringa = true,
            '{' => profondita += 1,
            '}' => {
                profondita = profondita.saturating_sub(1);
                if profondita == 0 {
                    return Some(testo[inizio..inizio + i + 1].to_string());
                }
            }
            _ => {}
        }
    }
    None
}

/// Toglie le virgole seguite (a meno di spazi) da `}` o `]`, fuori dalle stringhe.
fn togli_virgole_finali(json: &str) -> String {
    let caratteri: Vec<char> = json.chars().collect();
    let mut risultato = String::with_capacity(json.len());
    let mut in_stringa = false;
    let mut escape = false;
    for (i, &c) in caratteri.iter().enumerate() {
        if in_stringa {
            match (escape, c) {
                (true, _) => escape = false,
                (false, '\\') => escape = true,
                (false, '"') => in_stringa = false,
                _ => {}
            }
            risultato.push(c);
            continue;
        }
        if c == '"' {
            in_stringa = true;
        }
        if c == ',' {
            let successivo = caratteri[i + 1..].iter().find(|c| !c.is_whitespace());
            if matches!(successivo, Some('}') | Some(']')) {
                continue;
            }
        }
        risultato.push(c);
    }
    risultato
}

/// Estrae e deserializza la risposta; in caso di errore restituisce i motivi.
///
/// Prima dell'estrazione toglie i blocchi di ragionamento (`<think>…</think>`),
/// dopo l'estrazione riporta alla forma del §6.4 le mappe scritte come elenchi
/// di coppie (schema rigoroso di Anthropic, vedi [`crate::ai::schema`]).
pub fn analizza(testo: &str) -> Result<RispostaAi, Vec<String>> {
    analizza_in(testo, Lingua::It)
}

/// Come [`analizza`], con i motivi d'errore nella lingua indicata (quella
/// dell'avventura: i motivi tornano all'AI nel messaggio di correzione).
pub fn analizza_in(testo: &str, l: Lingua) -> Result<RispostaAi, Vec<String>> {
    let pulito = crate::ai::ragionamento::rimuovi_ragionamento(testo);
    let json = estrai_json(&pulito).ok_or_else(|| {
        vec![
            tr_in!(
                l,
                "la risposta non contiene un oggetto JSON",
                "the reply does not contain a JSON object"
            )
            .to_string(),
        ]
    })?;
    let mut valore: Value = serde_json::from_str(&json).map_err(|e| {
        vec![tr_in!(
            l,
            format!("il JSON non è valido: {e}"),
            format!("the JSON is not valid: {e}")
        )]
    })?;
    normalizza_coppie(&mut valore);
    serde_json::from_value::<RispostaAi>(valore).map_err(|e| {
        vec![tr_in!(
            l,
            format!("il JSON non rispetta lo schema: {e}"),
            format!("the JSON does not follow the schema: {e}")
        )]
    })
}

/// Converte `[{"stanza": "a", "oggetti": [...]}, …]` in `{"a": [...], …}` per i
/// campi di `aggiornamenti` che lo schema rigoroso rappresenta come coppie.
pub fn normalizza_coppie(risposta: &mut Value) {
    let Some(agg) = risposta
        .get_mut("aggiornamenti")
        .and_then(Value::as_object_mut)
    else {
        return;
    };
    for (campo, chiave, valore) in crate::ai::schema::MAPPE_COME_COPPIE {
        let Some(Value::Array(elenco)) = agg.get(campo) else {
            continue;
        };
        let convertibile = elenco
            .iter()
            .all(|v| v.get(chiave).is_some_and(Value::is_string) && v.get(valore).is_some());
        if !convertibile {
            continue;
        }
        let mut mappa = serde_json::Map::new();
        for v in elenco {
            if let (Some(k), Some(x)) = (v.get(chiave).and_then(Value::as_str), v.get(valore)) {
                match (mappa.get_mut(k), x) {
                    // Più voci per la stessa stanza: si uniscono gli elenchi.
                    (Some(Value::Array(esistente)), Value::Array(nuovi)) => {
                        esistente.extend(nuovi.iter().cloned())
                    }
                    _ => {
                        mappa.insert(k.to_string(), x.clone());
                    }
                }
            }
        }
        agg.insert(campo.to_string(), Value::Object(mappa));
    }
}

/// Valida la risposta (§6.5). In modalità permissiva le parti non valide vengono
/// tolte dalla risposta e restituite come avvisi; in modalità severa qualsiasi
/// violazione è un errore. Una narrazione vuota è sempre un errore.
pub fn valida(
    risposta: &mut RispostaAi,
    avventura: &Avventura,
    stato: &Stato,
    modalita: Validazione,
) -> Result<Vec<String>, Vec<String>> {
    if risposta.narrazione.trim().is_empty() {
        return Err(vec![
            tr_in!(
                avventura.lingua(),
                "il campo \"narrazione\" è vuoto o mancante",
                "the \"narrazione\" field is empty or missing"
            )
            .to_string(),
        ]);
    }
    let mut problemi = Vec::new();
    if let Some(agg) = risposta.aggiornamenti.as_mut() {
        controlla_aggiornamenti(agg, avventura, stato, &mut problemi);
    }
    match modalita {
        Validazione::Severa if !problemi.is_empty() => Err(problemi),
        _ => Ok(problemi),
    }
}

fn controlla_aggiornamenti(
    agg: &mut Aggiornamenti,
    avventura: &Avventura,
    stato: &Stato,
    problemi: &mut Vec<String>,
) {
    let l = avventura.lingua();
    let stanza_inesistente = |campo: &str, stanza: &str| {
        tr_in!(
            l,
            format!("{campo}: la stanza «{stanza}» non esiste"),
            format!("{campo}: the room \"{stanza}\" does not exist")
        )
    };
    if let Some(destinazione) = agg.stanza_corrente.clone() {
        let ok = if destinazione == stato.stanza_corrente {
            true
        } else if !avventura.stanze.contains_key(&destinazione) {
            problemi.push(stanza_inesistente("stanza_corrente", &destinazione));
            false
        } else if !avventura.gioco.spostamenti_liberi
            && !uscite::raggiungibile(avventura, stato, &destinazione)
        {
            let da = &stato.stanza_corrente;
            problemi.push(tr_in!(
                l,
                format!(
                    "stanza_corrente: non c'è un'uscita disponibile da «{da}» verso «{destinazione}»"
                ),
                format!(
                    "stanza_corrente: there is no available exit from \"{da}\" to \"{destinazione}\""
                )
            ));
            false
        } else {
            true
        };
        if !ok {
            agg.stanza_corrente = None;
        }
    }

    let oggetti_validi = |campo: &str, elenco: &mut Vec<String>, problemi: &mut Vec<String>| {
        elenco.retain(|o| {
            let esiste = avventura.oggetto_esiste(o);
            if !esiste {
                problemi.push(tr_in!(
                    l,
                    format!("{campo}: l'oggetto «{o}» non esiste nel catalogo"),
                    format!("{campo}: the item \"{o}\" is not in the catalogue")
                ));
            }
            esiste
        });
    };

    if let Some(elenco) = agg.inventario_aggiungi.as_mut() {
        oggetti_validi("inventario_aggiungi", elenco, problemi);
    }
    if let Some(elenco) = agg.inventario_rimuovi.as_mut() {
        oggetti_validi("inventario_rimuovi", elenco, problemi);
        let aggiunti = agg.inventario_aggiungi.clone().unwrap_or_default();
        elenco.retain(|o| {
            let posseduto = stato.inventario.contains(o) || aggiunti.contains(o);
            if !posseduto {
                problemi.push(tr_in!(
                    l,
                    format!("inventario_rimuovi: il giocatore non possiede «{o}»"),
                    format!("inventario_rimuovi: the player does not have \"{o}\"")
                ));
            }
            posseduto
        });
    }
    for (campo, mappa) in [
        (
            "oggetti_stanza_aggiungi",
            agg.oggetti_stanza_aggiungi.as_mut(),
        ),
        (
            "oggetti_stanza_rimuovi",
            agg.oggetti_stanza_rimuovi.as_mut(),
        ),
    ] {
        let Some(mappa) = mappa else { continue };
        mappa.retain(|stanza, _| {
            let esiste = avventura.stanze.contains_key(stanza);
            if !esiste {
                problemi.push(stanza_inesistente(campo, stanza));
            }
            esiste
        });
        for elenco in mappa.values_mut() {
            oggetti_validi(campo, elenco, problemi);
        }
    }
    if let Some(note) = agg.note_stanza.as_mut() {
        note.retain(|stanza, _| {
            let esiste = avventura.stanze.contains_key(stanza);
            if !esiste {
                problemi.push(stanza_inesistente("note_stanza", stanza));
            }
            esiste
        });
    }
    if let Some(esito) = &agg.esito
        && Esito::da_testo(esito).is_none()
    {
        problemi.push(tr_in!(
            l,
            format!("esito: «{esito}» non è ammesso (usa in_corso, vittoria o sconfitta)"),
            format!("esito: \"{esito}\" is not allowed (use in_corso, vittoria or sconfitta)")
        ));
        agg.esito = None;
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::stato::test::avventura_di_prova;

    #[test]
    fn estrazione_da_fence() {
        let t = "Ecco:\n```json\n{\"narrazione\": \"ciao\"}\n```\nfine";
        assert_eq!(estrai_json(t).unwrap(), "{\"narrazione\": \"ciao\"}");
    }

    #[test]
    fn estrazione_con_testo_extra_e_annidato() {
        let t = "Certo! {\"narrazione\": \"una } graffa\", \"aggiornamenti\": {\"flag\": {\"a\": {\"b\": 1}}}} e poi {altro}";
        let j = estrai_json(t).unwrap();
        let v: Value = serde_json::from_str(&j).unwrap();
        assert_eq!(v["aggiornamenti"]["flag"]["a"]["b"], 1);
        assert_eq!(v["narrazione"], "una } graffa");
    }

    #[test]
    fn estrazione_con_virgole_finali() {
        let t =
            "{\"narrazione\": \"a, ]\", \"aggiornamenti\": {\"inventario_aggiungi\": [\"x\",],},}";
        let j = estrai_json(t).unwrap();
        let v: Value = serde_json::from_str(&j).unwrap();
        assert_eq!(v["narrazione"], "a, ]");
        assert_eq!(v["aggiornamenti"]["inventario_aggiungi"][0], "x");
    }

    #[test]
    fn estrazione_fallita() {
        assert!(estrai_json("nessun json qui").is_none());
        assert!(estrai_json("{\"aperto\": ").is_none());
        assert!(analizza("niente").is_err());
        let e = analizza("{\"narrazione\": 5}").unwrap_err();
        assert!(e[0].contains("schema"));
    }

    #[test]
    fn analisi_completa() {
        let r = analizza(
            r#"{"narrazione":"x","aggiornamenti":{"note_stanza":{"a":"n1","b":["n2","n3"]},"punteggio_delta":5,"esito":null},"memoria":"m"}"#,
        )
        .unwrap();
        let agg = r.aggiornamenti.unwrap();
        assert_eq!(agg.note_stanza.unwrap()["b"].elenco(), vec!["n2", "n3"]);
        assert_eq!(agg.punteggio_delta, Some(5.0));
        assert_eq!(agg.esito, None);
        assert_eq!(r.memoria.as_deref(), Some("m"));
    }

    fn risposta(agg: &str) -> RispostaAi {
        analizza(&format!(r#"{{"narrazione":"ok","aggiornamenti":{agg}}}"#)).unwrap()
    }

    #[test]
    fn risposte_valide() {
        let a = avventura_di_prova();
        let s = Stato::nuovo(&a);
        for agg in [
            r#"{}"#,
            r#"{"stanza_corrente":"a"}"#,
            r#"{"stanza_corrente":"c"}"#,
            r#"{"inventario_aggiungi":["chiave"],"inventario_rimuovi":["moneta"]}"#,
            r#"{"inventario_aggiungi":["chiave"],"inventario_rimuovi":["chiave"]}"#,
            r#"{"oggetti_stanza_aggiungi":{"b":["gemma"]},"esito":"vittoria"}"#,
        ] {
            let mut r = risposta(agg);
            assert_eq!(
                valida(&mut r, &a, &s, Validazione::Severa),
                Ok(vec![]),
                "{agg}"
            );
        }
    }

    #[test]
    fn risposte_non_valide_in_modalita_severa() {
        let a = avventura_di_prova();
        let s = Stato::nuovo(&a);
        let casi = [
            (r#"{"stanza_corrente":"zz"}"#, "non esiste"),
            (r#"{"stanza_corrente":"b"}"#, "uscita disponibile"),
            (r#"{"inventario_aggiungi":["spada"]}"#, "catalogo"),
            (r#"{"inventario_rimuovi":["chiave"]}"#, "non possiede"),
            (
                r#"{"oggetti_stanza_rimuovi":{"zz":["chiave"]}}"#,
                "stanza «zz»",
            ),
            (r#"{"note_stanza":{"zz":"x"}}"#, "stanza «zz»"),
            (r#"{"esito":"pareggio"}"#, "non è ammesso"),
        ];
        for (agg, atteso) in casi {
            let mut r = risposta(agg);
            let errori = valida(&mut r, &a, &s, Validazione::Severa).unwrap_err();
            assert!(
                errori.iter().any(|e| e.contains(atteso)),
                "{agg}: {errori:?}"
            );
        }
        let mut vuota = analizza(r#"{"narrazione":"  "}"#).unwrap();
        assert!(valida(&mut vuota, &a, &s, Validazione::Permissiva).is_err());
    }

    #[test]
    fn modalita_permissiva_scarta_solo_le_parti_errate() {
        let a = avventura_di_prova();
        let s = Stato::nuovo(&a);
        let mut r = risposta(
            r#"{"stanza_corrente":"b","inventario_aggiungi":["chiave","spada"],"esito":"boh","punteggio_delta":3}"#,
        );
        let avvisi = valida(&mut r, &a, &s, Validazione::Permissiva).unwrap();
        assert_eq!(avvisi.len(), 3);
        let agg = r.aggiornamenti.unwrap();
        assert_eq!(agg.stanza_corrente, None);
        assert_eq!(agg.inventario_aggiungi.unwrap(), vec!["chiave"]);
        assert_eq!(agg.esito, None);
        assert_eq!(agg.punteggio_delta, Some(3.0));
    }

    #[test]
    fn spostamenti_liberi() {
        let mut a = avventura_di_prova();
        a.gioco.spostamenti_liberi = true;
        let s = Stato::nuovo(&a);
        let mut r = risposta(r#"{"stanza_corrente":"b"}"#);
        assert!(valida(&mut r, &a, &s, Validazione::Severa).is_ok());
    }

    #[test]
    fn mappe_come_coppie_e_ragionamento() {
        let r = analizza(
            r#"<think>prima ragiono {forse}</think>
            {"narrazione":"n","aggiornamenti":{
              "flag":[{"nome":"botola_aperta","valore":true},{"nome":"conta","valore":2}],
              "oggetti_stanza_aggiungi":[{"stanza":"a","oggetti":["x"]},{"stanza":"a","oggetti":["y"]}],
              "note_stanza":[{"stanza":"b","nota":"Polvere."}]}}"#,
        )
        .unwrap();
        let agg = r.aggiornamenti.unwrap();
        let flag = agg.flag.unwrap();
        assert_eq!(flag["botola_aperta"], serde_json::json!(true));
        assert_eq!(flag["conta"], serde_json::json!(2));
        assert_eq!(agg.oggetti_stanza_aggiungi.unwrap()["a"], vec!["x", "y"]);
        assert_eq!(agg.note_stanza.unwrap()["b"], Nota::Una("Polvere.".into()));
    }
}
