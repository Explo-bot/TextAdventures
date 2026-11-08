//! Stima dei token e controllo del contesto del modello (§5.2).
//!
//! La stima è volutamente prudente: `ceil(caratteri / 3.5)` sommando tutti i
//! messaggi. Serve per avvisare prima di inviare un prompt più grande del
//! contesto configurato, per accorgersi dopo la risposta che il server ha
//! troncato il prompt e per consigliare il valore di `contesto_token` adatto a
//! un'avventura.

use crate::avventura::Avventura;
use crate::config::Config;
use crate::prompt::{AzioneTurno, costruisci_messaggi};
use crate::stato::{Stato, TurnoRecente};

use super::Messaggio;

/// Contesto minimo consigliato.
pub const CONTESTO_MINIMO: u64 = 8192;

/// Token stimati per un testo.
pub fn stima_testo(testo: &str) -> u64 {
    caratteri_in_token(testo.chars().count() as u64)
}

fn caratteri_in_token(caratteri: u64) -> u64 {
    // ceil(c / 3.5) = ceil(2c / 7)
    (caratteri * 2).div_ceil(7)
}

/// Token stimati per i soli messaggi.
pub fn stima_messaggi(messaggi: &[Messaggio]) -> u64 {
    caratteri_in_token(
        messaggi
            .iter()
            .map(|m| m.contenuto.chars().count() as u64)
            .sum(),
    )
}

/// Token stimati per l'intera richiesta (messaggi + risposta).
pub fn stima_richiesta(messaggi: &[Messaggio], max_token: u32) -> u64 {
    stima_messaggi(messaggi) + u64::from(max_token)
}

/// Vero se la stima supera il contesto configurato (0 = non controllare).
pub fn supera_contesto(stima: u64, contesto_token: u32) -> bool {
    contesto_token > 0 && stima > u64::from(contesto_token)
}

/// Vero se i token dichiarati dal server sono meno del 60% della stima dei messaggi.
pub fn troncamento_probabile(dichiarati: u64, stima_messaggi: u64) -> bool {
    dichiarati * 10 < stima_messaggi * 6
}

/// Contesto consigliato: (stima + max_token) + 25%, alla potenza di 2 superiore, minimo 8192.
pub fn contesto_consigliato(stima_messaggi: u64, max_token: u32) -> u64 {
    let con_margine = (stima_messaggi + u64::from(max_token)) * 5;
    let con_margine = con_margine.div_ceil(4);
    con_margine.next_power_of_two().max(CONTESTO_MINIMO)
}

/// Numero con il punto come separatore delle migliaia (`11200` → `11.200`).
pub fn migliaia(n: u64) -> String {
    let cifre = n.to_string();
    let mut risultato = String::new();
    for (i, c) in cifre.chars().enumerate() {
        if i > 0 && (cifre.len() - i).is_multiple_of(3) {
            risultato.push('.');
        }
        risultato.push(c);
    }
    risultato
}

/// Avviso mostrato quando la stima supera il contesto.
pub fn avviso_contesto(stima: u64, contesto_token: u32) -> String {
    let (s, c) = (migliaia(stima), migliaia(u64::from(contesto_token)));
    crate::tr!(
        format!(
            "Il prompt stimato (≈ {s} token) supera il contesto configurato ({c}): il modello \
             potrebbe ignorare parte delle regole. Aumenta `contesto_token` (e il contesto del \
             server) — vedi manuale."
        ),
        format!(
            "The estimated prompt (≈ {s} tokens) exceeds the configured context ({c}): the \
             model may ignore part of the rules. Increase `contesto_token` (and the server \
             context) — see the manual."
        ),
        format!(
            "Le prompt estimé (≈ {s} jetons) dépasse le contexte configuré ({c}) : le modèle risque d'ignorer une partie des règles. Augmentez contesto_token (et le contexte du serveur) — voir le manuel."
        ),
        format!(
            "El prompt estimado (≈ {s} tokens) supera el contexto configurado ({c}): el modelo puede ignorar parte de las reglas. Aumenta contesto_token (y el contexto del servidor) — consulta el manual."
        ),
        format!(
            "Der geschätzte Prompt (≈ {s} Token) überschreitet den konfigurierten Kontext ({c}): Das Modell könnte einen Teil der Regeln ignorieren. Erhöhe contesto_token (und den Kontext des Servers) — siehe Handbuch."
        )
    )
}

/// Avviso mostrato quando il server sembra aver troncato il prompt.
pub fn avviso_troncamento(dichiarati: u64, stima: u64) -> String {
    let (d, s) = (migliaia(dichiarati), migliaia(stima));
    crate::tr!(
        format!(
            "Il server ha elaborato solo ≈ {d} token di prompt su ≈ {s} stimati: probabilmente \
             ha troncato il contesto. Aumenta il contesto del server e `contesto_token` — vedi \
             manuale."
        ),
        format!(
            "The server processed only ≈ {d} prompt tokens out of ≈ {s} estimated: it has \
             probably truncated the context. Increase the server context and `contesto_token` \
             — see the manual."
        ),
        format!(
            "Le serveur n'a traité que ≈ {d} jetons du prompt sur ≈ {s} estimés : il a probablement tronqué le contexte. Augmentez le contexte du serveur et contesto_token — voir le manuel."
        ),
        format!(
            "El servidor solo ha procesado ≈ {d} tokens del prompt de ≈ {s} estimados: probablemente ha recortado el contexto. Aumenta el contexto del servidor y contesto_token — consulta el manual."
        ),
        format!(
            "Der Server hat nur ≈ {d} von ≈ {s} geschätzten Prompt-Token verarbeitet: Wahrscheinlich hat er den Kontext gekürzt. Erhöhe den Kontext des Servers und contesto_token — siehe Handbuch."
        )
    )
}

/// Il prompt più grande previsto per un'avventura.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StimaAvventura {
    /// Token stimati dei messaggi.
    pub token_messaggi: u64,
    /// Stanza che produce il prompt più grande.
    pub stanza: String,
    /// Contesto consigliato.
    pub consigliato: u64,
    /// Messaggi della stanza più grande (per `prova-ai --avventura`).
    pub messaggi: Vec<Messaggio>,
}

/// Calcola il prompt più grande dell'avventura: per ogni stanza, stato iniziale con
/// `memoria` e `ultimi_turni` riempiti fino ai limiti configurati.
pub fn stima_avventura(avventura: &Avventura, config: &Config) -> Option<StimaAvventura> {
    let mut base = Stato::nuovo(avventura);
    base.memoria = "m".repeat(config.motore.lunghezza_max_memoria);
    base.ultimi_turni = (1..=config.motore.turni_in_memoria as u32)
        .map(|turno| TurnoRecente {
            turno,
            azione: "a".repeat(200),
            narrazione: "n".repeat(1200),
        })
        .collect();
    let azione = AzioneTurno::Giocatore("x".repeat(500));
    let mut migliore: Option<StimaAvventura> = None;
    for id in avventura.stanze.keys() {
        let mut stato = base.clone();
        stato.stanza_corrente = id.clone();
        let messaggi = costruisci_messaggi(avventura, &stato, &azione, crate::lingua::attuale());
        let token = stima_messaggi(&messaggi);
        if migliore.as_ref().is_none_or(|m| token > m.token_messaggi) {
            migliore = Some(StimaAvventura {
                token_messaggi: token,
                stanza: id.clone(),
                consigliato: contesto_consigliato(token, config.ai.max_token),
                messaggi,
            });
        }
    }
    migliore
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::stato::test::avventura_di_prova;

    #[test]
    fn stima_prudente() {
        assert_eq!(stima_testo(""), 0);
        assert_eq!(stima_testo("abc"), 1);
        assert_eq!(stima_testo(&"x".repeat(7)), 2);
        assert_eq!(stima_testo(&"x".repeat(8)), 3);
        assert_eq!(stima_testo("àèìòù"), 2, "conta i caratteri, non i byte");
        let m = vec![
            Messaggio::sistema("x".repeat(35)),
            Messaggio::utente("y".repeat(35)),
        ];
        assert_eq!(stima_messaggi(&m), 20);
        assert_eq!(stima_richiesta(&m, 1200), 1220);
    }

    #[test]
    fn controllo_del_contesto() {
        assert!(!supera_contesto(100_000, 0));
        assert!(supera_contesto(9000, 8192));
        assert!(!supera_contesto(8192, 8192));
    }

    #[test]
    fn rilevamento_troncamento() {
        assert!(troncamento_probabile(2048, 7900));
        assert!(!troncamento_probabile(6000, 7900));
        assert!(!troncamento_probabile(60, 100));
        assert!(troncamento_probabile(59, 100));
    }

    #[test]
    fn valore_consigliato() {
        assert_eq!(contesto_consigliato(1000, 1200), 8192);
        // (7000 + 1200) * 1.25 = 10250 → 16384
        assert_eq!(contesto_consigliato(7000, 1200), 16384);
        // (12000 + 1200) * 1.25 = 16500 → 32768
        assert_eq!(contesto_consigliato(12000, 1200), 32768);
    }

    #[test]
    fn formati() {
        assert_eq!(migliaia(11200), "11.200");
        assert_eq!(migliaia(999), "999");
        assert_eq!(migliaia(1_000_000), "1.000.000");
        assert!(avviso_contesto(11200, 8192).contains("≈ 11.200 token"));
        assert!(avviso_contesto(11200, 8192).contains("(8.192)"));
        assert!(avviso_troncamento(2048, 7900).contains("≈ 2.048 token di prompt su ≈ 7.900"));
    }

    #[test]
    fn stima_dell_avventura() {
        let a = avventura_di_prova();
        let config = Config::default();
        let s = stima_avventura(&a, &config).unwrap();
        assert!(s.token_messaggi > 1000);
        assert!(s.consigliato >= CONTESTO_MINIMO);
        assert!(s.consigliato.is_power_of_two());
        assert!(a.stanze.contains_key(&s.stanza));
    }
}
