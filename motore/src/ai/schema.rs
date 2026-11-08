//! Schema JSON della risposta del narratore (§6.4).
//!
//! Ne esistono due varianti:
//! - [`schema_risposta`]: la forma «naturale» del §6.4, con le mappe
//!   (`flag`, `note_stanza`, `oggetti_stanza_*`) come oggetti a chiavi libere.
//!   Usata con LM Studio (`response_format` di tipo `json_schema`) e, in futuro,
//!   per la grammatica del fornitore `locale`.
//! - [`schema_rigoroso`]: per gli output strutturati di Anthropic, che esigono
//!   `additionalProperties: false` su **tutti** gli oggetti e quindi non
//!   ammettono chiavi libere. Qui ogni mappa diventa un elenco di coppie
//!   (es. `flag: [{"nome": "botola_aperta", "valore": true}]`).
//!   [`crate::risposta`] accetta entrambe le forme e le riporta alla forma
//!   naturale prima della validazione.

use serde_json::{Value, json};

/// Valori ammessi per `esito`.
const ESITI: [&str; 3] = ["in_corso", "vittoria", "sconfitta"];

fn elenco_di_stringhe() -> Value {
    json!({ "type": "array", "items": { "type": "string" } })
}

/// Schema nella forma del §6.4 (mappe come oggetti).
pub fn schema_risposta() -> Value {
    let mappa_di_elenchi = json!({
        "type": "object",
        "additionalProperties": elenco_di_stringhe()
    });
    json!({
        "type": "object",
        "additionalProperties": false,
        "required": ["narrazione"],
        "properties": {
            "narrazione": { "type": "string" },
            "aggiornamenti": {
                "type": "object",
                "additionalProperties": false,
                "properties": {
                    "stanza_corrente": { "type": "string" },
                    "inventario_aggiungi": elenco_di_stringhe(),
                    "inventario_rimuovi": elenco_di_stringhe(),
                    "oggetti_stanza_aggiungi": mappa_di_elenchi,
                    "oggetti_stanza_rimuovi": mappa_di_elenchi,
                    "flag": {
                        "type": "object",
                        "additionalProperties": {
                            "type": ["boolean", "number", "string"]
                        }
                    },
                    "note_stanza": {
                        "type": "object",
                        "additionalProperties": { "type": "string" }
                    },
                    "punteggio_delta": { "type": "integer" },
                    "esito": { "type": "string", "enum": ESITI }
                }
            },
            "memoria": { "type": "string" }
        }
    })
}

fn coppie(chiave: &str, valore: &str, schema_valore: Value) -> Value {
    json!({
        "type": "array",
        "items": {
            "type": "object",
            "additionalProperties": false,
            "required": [chiave, valore],
            "properties": {
                chiave: { "type": "string" },
                valore: schema_valore
            }
        }
    })
}

/// Schema con `additionalProperties: false` ovunque (mappe come elenchi di coppie).
pub fn schema_rigoroso() -> Value {
    json!({
        "type": "object",
        "additionalProperties": false,
        "required": ["narrazione"],
        "properties": {
            "narrazione": { "type": "string" },
            "aggiornamenti": {
                "type": "object",
                "additionalProperties": false,
                "properties": {
                    "stanza_corrente": { "type": "string" },
                    "inventario_aggiungi": elenco_di_stringhe(),
                    "inventario_rimuovi": elenco_di_stringhe(),
                    "oggetti_stanza_aggiungi": coppie("stanza", "oggetti", elenco_di_stringhe()),
                    "oggetti_stanza_rimuovi": coppie("stanza", "oggetti", elenco_di_stringhe()),
                    "flag": coppie(
                        "nome",
                        "valore",
                        json!({ "anyOf": [
                            { "type": "boolean" },
                            { "type": "number" },
                            { "type": "string" }
                        ]})
                    ),
                    "note_stanza": coppie("stanza", "nota", json!({ "type": "string" })),
                    "punteggio_delta": { "type": "integer" },
                    "esito": { "type": "string", "enum": ESITI }
                }
            },
            "memoria": { "type": "string" }
        }
    })
}

/// Campi di `aggiornamenti` che nello schema rigoroso sono elenchi di coppie:
/// (campo, nome della chiave, nome del valore).
pub const MAPPE_COME_COPPIE: [(&str, &str, &str); 5] = [
    ("oggetti_stanza_aggiungi", "stanza", "oggetti"),
    ("oggetti_stanza_rimuovi", "stanza", "oggetti"),
    ("flag", "nome", "valore"),
    ("note_stanza", "stanza", "nota"),
    // Variante tollerata: alcuni modelli scrivono "stanza"/"note".
    ("note_stanza", "stanza", "note"),
];

#[cfg(test)]
mod test {
    use super::*;

    /// Verifica ricorsivamente che ogni oggetto dello schema abbia
    /// `additionalProperties: false`.
    fn tutti_chiusi(v: &Value) -> bool {
        match v {
            Value::Object(m) => {
                let chiuso = m.get("type") != Some(&json!("object"))
                    || m.get("additionalProperties") == Some(&json!(false));
                chiuso && m.values().all(tutti_chiusi)
            }
            Value::Array(a) => a.iter().all(tutti_chiusi),
            _ => true,
        }
    }

    #[test]
    fn schema_rigoroso_senza_chiavi_libere() {
        assert!(tutti_chiusi(&schema_rigoroso()));
        assert!(!tutti_chiusi(&schema_risposta()));
    }

    #[test]
    fn campi_principali() {
        for s in [schema_risposta(), schema_rigoroso()] {
            assert_eq!(s["required"], json!(["narrazione"]));
            let agg = &s["properties"]["aggiornamenti"]["properties"];
            for campo in [
                "stanza_corrente",
                "inventario_aggiungi",
                "flag",
                "note_stanza",
                "esito",
            ] {
                assert!(agg.get(campo).is_some(), "{campo}");
            }
        }
    }
}
