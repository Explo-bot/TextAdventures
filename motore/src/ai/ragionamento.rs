//! Ragionamento ("thinking") dei modelli (§5.4): livello configurato in modo
//! uniforme, traduzione nei parametri di ciascun fornitore e pulizia dei blocchi
//! di ragionamento dal testo della risposta prima dell'estrazione del JSON.
//!
//! I nomi dei parametri sono stati verificati sulla documentazione ufficiale
//! (novembre 2026):
//! - OpenAI: `reasoning_effort` (`none`/`minimal`/`low`/`medium`/`high`/…);
//!   si usa `minimal` per «spento» perché è il valore accettato dal maggior numero
//!   di modelli (alcuni rifiutano `none`).
//! - Anthropic: `thinking: {"type": "adaptive"}` + `output_config.effort`
//!   (`low`/`medium`/`high`); `budget_tokens` è rifiutato dai modelli recenti.
//! - Gemini: `generationConfig.thinkingConfig.thinkingLevel`
//!   (`minimal`/`low`/`medium`/`high`).
//! - Ollama: `think` (`false`, `true` o un livello `low`/`medium`/`high`).

use std::fmt;

use serde::Deserialize;

/// Livello di ragionamento richiesto.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Ragionamento {
    /// Nessun parametro: decide il fornitore/modello.
    #[default]
    Predefinito,
    /// Disattiva o minimizza il ragionamento.
    Spento,
    /// Livello basso.
    Basso,
    /// Livello medio.
    Medio,
    /// Livello alto.
    Alto,
}

impl Ragionamento {
    /// Interpreta il testo di configurazione o di una variabile d'ambiente.
    pub fn da_testo(testo: &str) -> Option<Self> {
        Some(match testo.trim().to_lowercase().as_str() {
            "predefinito" => Ragionamento::Predefinito,
            "spento" => Ragionamento::Spento,
            "basso" => Ragionamento::Basso,
            "medio" => Ragionamento::Medio,
            "alto" => Ragionamento::Alto,
            _ => return None,
        })
    }

    /// Nome testuale.
    pub fn nome(self) -> &'static str {
        match self {
            Ragionamento::Predefinito => "predefinito",
            Ragionamento::Spento => "spento",
            Ragionamento::Basso => "basso",
            Ragionamento::Medio => "medio",
            Ragionamento::Alto => "alto",
        }
    }

    /// Livello inglese comune (`low`/`medium`/`high`) per i livelli espliciti.
    fn livello(self) -> Option<&'static str> {
        match self {
            Ragionamento::Basso => Some("low"),
            Ragionamento::Medio => Some("medium"),
            Ragionamento::Alto => Some("high"),
            Ragionamento::Predefinito | Ragionamento::Spento => None,
        }
    }

    /// Valore di `reasoning_effort` per `openai`/`lmstudio` (None = non inviare).
    pub fn per_openai(self) -> Option<&'static str> {
        match self {
            Ragionamento::Predefinito => None,
            Ragionamento::Spento => Some("minimal"),
            altro => altro.livello(),
        }
    }

    /// `output_config.effort` per `anthropic` (None = nessun ragionamento esteso).
    pub fn per_anthropic(self) -> Option<&'static str> {
        self.livello()
    }

    /// `thinkingLevel` per `gemini` (None = non inviare).
    pub fn per_gemini(self) -> Option<&'static str> {
        match self {
            Ragionamento::Predefinito => None,
            Ragionamento::Spento => Some("minimal"),
            altro => altro.livello(),
        }
    }

    /// Valore del campo `think` per `ollama` (None = non inviare).
    pub fn per_ollama(self) -> Option<serde_json::Value> {
        match self {
            Ragionamento::Predefinito => None,
            Ragionamento::Spento => Some(serde_json::Value::Bool(false)),
            altro => altro.livello().map(serde_json::Value::from),
        }
    }
}

impl fmt::Display for Ragionamento {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.nome())
    }
}

/// Coppie di marcatori che delimitano il ragionamento nel testo di alcuni modelli.
const MARCATORI: [(&str, &str); 5] = [
    ("<think>", "</think>"),
    ("<thinking>", "</thinking>"),
    ("<reasoning>", "</reasoning>"),
    ("<thought>", "</thought>"),
    ("<|begin_of_thought|>", "<|end_of_thought|>"),
];

/// Toglie dal testo i blocchi di ragionamento (`<think>…</think>` e simili).
///
/// Un blocco aperto e mai chiuso viene tolto fino alla fine del testo, a meno che
/// dopo l'apertura compaia un oggetto JSON: in quel caso si tiene il JSON (alcuni
/// server tagliano il marcatore di chiusura). Un marcatore di chiusura senza
/// apertura (il server ha già tolto l'apertura) elimina tutto ciò che lo precede.
pub fn rimuovi_ragionamento(testo: &str) -> String {
    let mut risultato = testo.to_string();
    for (apertura, chiusura) in MARCATORI {
        // Chiusura senza apertura: tutto ciò che precede è ragionamento.
        if let Some(fine) = trova_ci(&risultato, chiusura)
            && trova_ci(&risultato[..fine], apertura).is_none()
        {
            risultato = risultato[fine + chiusura.len()..].to_string();
        }
        while let Some(inizio) = trova_ci(&risultato, apertura) {
            let dopo = inizio + apertura.len();
            match trova_ci(&risultato[dopo..], chiusura) {
                Some(rel) => {
                    let fine = dopo + rel + chiusura.len();
                    risultato.replace_range(inizio..fine, "");
                }
                None => {
                    // Blocco non chiuso: se c'è un JSON dopo, lo si conserva.
                    let resto = &risultato[dopo..];
                    // Il JSON di solito inizia a capo; altrimenti si prende la
                    // prima graffa.
                    let da = resto
                        .rfind("\n{")
                        .map(|n| n + 1)
                        .or_else(|| resto.find('{'));
                    let tenuto = da.map(|p| resto[p..].to_string());
                    risultato = format!("{}{}", &risultato[..inizio], tenuto.unwrap_or_default());
                    break;
                }
            }
        }
    }
    risultato.trim().to_string()
}

/// Ricerca senza distinzione tra maiuscole e minuscole (solo ASCII nei marcatori).
fn trova_ci(testo: &str, cercato: &str) -> Option<usize> {
    let t = testo.to_ascii_lowercase();
    t.find(&cercato.to_ascii_lowercase())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn mappatura_per_fornitore() {
        assert_eq!(Ragionamento::Predefinito.per_openai(), None);
        assert_eq!(Ragionamento::Spento.per_openai(), Some("minimal"));
        assert_eq!(Ragionamento::Alto.per_openai(), Some("high"));
        assert_eq!(Ragionamento::Spento.per_anthropic(), None);
        assert_eq!(Ragionamento::Medio.per_anthropic(), Some("medium"));
        assert_eq!(Ragionamento::Spento.per_gemini(), Some("minimal"));
        assert_eq!(Ragionamento::Basso.per_gemini(), Some("low"));
        assert_eq!(
            Ragionamento::Spento.per_ollama(),
            Some(serde_json::json!(false))
        );
        assert_eq!(
            Ragionamento::Alto.per_ollama(),
            Some(serde_json::json!("high"))
        );
        assert_eq!(Ragionamento::Predefinito.per_ollama(), None);
        assert_eq!(Ragionamento::da_testo(" Medio "), Some(Ragionamento::Medio));
        assert_eq!(Ragionamento::da_testo("forte"), None);
    }

    #[test]
    fn pulizia_dei_blocchi_di_ragionamento() {
        assert_eq!(
            rimuovi_ragionamento("<think>devo pensare {x}</think>\n{\"narrazione\":\"ok\"}"),
            "{\"narrazione\":\"ok\"}"
        );
        assert_eq!(
            rimuovi_ragionamento("<THINKING>a</THINKING>{\"a\":1}<think>b</think>"),
            "{\"a\":1}"
        );
        assert_eq!(
            rimuovi_ragionamento("ragiono a lungo…</think>{\"a\":1}"),
            "{\"a\":1}"
        );
        assert_eq!(
            rimuovi_ragionamento("<think>non finisce mai\n{\"a\":1}"),
            "{\"a\":1}"
        );
        assert_eq!(rimuovi_ragionamento("{\"a\":\"<b>\"}"), "{\"a\":\"<b>\"}");
        assert_eq!(rimuovi_ragionamento("<think>solo pensieri"), "");
    }
}
