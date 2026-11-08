//! Calcolo della disponibilità delle uscite di una stanza in base a flag e
//! inventario, con il motivo in forma testuale (usato nel prompt, da `/uscite`
//! e nella validazione degli spostamenti).

use serde_json::Value;

use crate::avventura::Avventura;
use crate::stato::Stato;

/// Un'uscita con la sua disponibilità calcolata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InfoUscita {
    /// Direzione (chiave in `[uscite.<direzione>]`).
    pub direzione: String,
    /// Id della destinazione.
    pub verso: String,
    /// Nome leggibile della destinazione.
    pub nome_verso: String,
    /// Descrizione dell'uscita.
    pub descrizione: String,
    /// Percorribile ora.
    pub disponibile: bool,
    /// Motivo della (non) disponibilità.
    pub motivo: String,
    /// Dichiarata `nascosta` nel file.
    pub nascosta: bool,
    /// Da menzionare al giocatore (non nascosta, oppure nascosta ma già disponibile).
    pub visibile: bool,
}

/// Regola di verità dei flag (§3.2): `true`, numero diverso da 0, stringa non
/// vuota diversa da `"false"`, `"no"`, `"0"`.
pub fn flag_vero(valore: Option<&Value>) -> bool {
    match valore {
        Some(Value::Bool(b)) => *b,
        Some(Value::Number(n)) => n.as_f64().is_some_and(|x| x != 0.0),
        Some(Value::String(s)) => {
            let s = s.trim().to_lowercase();
            !s.is_empty() && s != "false" && s != "no" && s != "0"
        }
        _ => false,
    }
}

/// Uscite della stanza `id_stanza` con disponibilità calcolata sullo stato.
pub fn calcola(avventura: &Avventura, stato: &Stato, id_stanza: &str) -> Vec<InfoUscita> {
    let Some(stanza) = avventura.stanza(id_stanza) else {
        return Vec::new();
    };
    // Il motivo finisce nel prompt e nella narrazione dell'AI simulata: è
    // nella lingua dell'avventura.
    let l = avventura.lingua();
    stanza
        .uscite
        .iter()
        .map(|(direzione, uscita)| {
            let mut mancanze = Vec::new();
            if let Some(flag) = &uscita.richiede_flag
                && !flag_vero(stato.flag.get(flag))
            {
                mancanze.push(crate::tr_in!(
                    l,
                    format!("richiede che il flag «{flag}» sia vero"),
                    format!("requires the flag \"{flag}\" to be true")
                ));
            }
            if let Some(oggetto) = &uscita.richiede_oggetto
                && !stato.inventario.contains(oggetto)
            {
                let nome = avventura.nome_oggetto_originale(oggetto);
                mancanze.push(crate::tr_in!(
                    l,
                    format!("richiede l'oggetto «{oggetto}» in inventario ({nome})"),
                    format!("requires the item \"{oggetto}\" in the inventory ({nome})")
                ));
            }
            let disponibile = mancanze.is_empty();
            InfoUscita {
                direzione: direzione.clone(),
                verso: uscita.verso.clone(),
                nome_verso: avventura.nome_stanza(&uscita.verso),
                descrizione: uscita.descrizione.clone(),
                disponibile,
                motivo: if disponibile {
                    crate::tr_in!(l, "condizioni soddisfatte", "conditions met").to_string()
                } else {
                    mancanze.join("; ")
                },
                nascosta: uscita.nascosta,
                visibile: !uscita.nascosta || disponibile,
            }
        })
        .collect()
}

/// Vero se dalla stanza corrente si può raggiungere `destinazione` con un'uscita disponibile.
pub fn raggiungibile(avventura: &Avventura, stato: &Stato, destinazione: &str) -> bool {
    calcola(avventura, stato, &stato.stanza_corrente)
        .iter()
        .any(|u| u.disponibile && u.verso == destinazione)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::stato::test::avventura_di_prova;
    use serde_json::json;

    #[test]
    fn verita_dei_flag() {
        assert!(flag_vero(Some(&json!(true))));
        assert!(!flag_vero(Some(&json!(false))));
        assert!(flag_vero(Some(&json!(2))));
        assert!(!flag_vero(Some(&json!(0))));
        assert!(!flag_vero(Some(&json!(0.0))));
        assert!(flag_vero(Some(&json!("sì"))));
        for falso in ["", "false", "No", "0", "  "] {
            assert!(!flag_vero(Some(&json!(falso))), "{falso}");
        }
        assert!(!flag_vero(None));
        assert!(!flag_vero(Some(&Value::Null)));
    }

    #[test]
    fn disponibilita_con_oggetti_e_flag() {
        let a = avventura_di_prova();
        let mut s = Stato::nuovo(&a);
        let u = calcola(&a, &s, "a");
        let nord = u.iter().find(|u| u.direzione == "nord").unwrap();
        assert!(!nord.disponibile);
        assert!(nord.motivo.contains("chiave"));
        assert!(nord.visibile);
        let giu = u.iter().find(|u| u.direzione == "giu").unwrap();
        assert!(!giu.disponibile && !giu.visibile && giu.nascosta);
        let est = u.iter().find(|u| u.direzione == "est").unwrap();
        assert!(est.disponibile);
        assert_eq!(est.nome_verso, "Stanza C");

        s.inventario.push("chiave".into());
        s.flag.insert("botola_aperta".into(), json!(true));
        let u = calcola(&a, &s, "a");
        assert!(u.iter().all(|u| u.disponibile && u.visibile));
        assert!(raggiungibile(&a, &s, "b"));
    }

    #[test]
    fn stanza_inesistente() {
        let a = avventura_di_prova();
        let s = Stato::nuovo(&a);
        assert!(calcola(&a, &s, "zz").is_empty());
        assert!(!raggiungibile(&a, &s, "zz"));
    }
}
