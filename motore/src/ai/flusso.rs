//! Risposte in streaming (testo progressivo, §7.4).
//!
//! I fornitori che lo permettono inviano la risposta a pezzi (Server-Sent
//! Events per OpenAI/LM Studio, Anthropic e Gemini, JSON a righe per Ollama,
//! token per token per `locale`). Il narratore risponde però con un oggetto
//! JSON: [`EstrattoreNarrazione`] riceve il testo grezzo man mano che arriva e
//! restituisce solo i caratteri già decodificati del campo `narrazione` di
//! primo livello, così il canale può mostrarli mentre il modello scrive.
//!
//! Lo streaming serve solo alla visualizzazione: la risposta completa viene
//! comunque ricostruita, validata e applicata come prima.

/// Contenuto `data:` di una riga Server-Sent Events (`None` per le altre righe
/// e per la riga finale `[DONE]` di OpenAI).
pub fn dati_sse(riga: &str) -> Option<&str> {
    let dati = riga.strip_prefix("data:")?.trim();
    (!dati.is_empty() && dati != "[DONE]").then_some(dati)
}

/// Punto dell'analisi incrementale.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum Fase {
    /// Prima dell'oggetto JSON.
    #[default]
    Fuori,
    /// Dentro un blocco `<think>…</think>` prima del JSON.
    Ragionamento,
    /// Dentro l'oggetto JSON, fuori dalle stringhe.
    Json,
    /// Dentro una stringa qualsiasi del JSON.
    Stringa,
    /// Dopo la chiave `"narrazione"` di primo livello, in attesa del valore.
    DopoChiave,
    /// Dentro il valore della narrazione (i caratteri vengono restituiti).
    Narrazione,
    /// Narrazione completa: il resto viene ignorato.
    Finito,
}

/// Estrae progressivamente il campo `narrazione` da una risposta JSON in arrivo.
///
/// Tollera testo e blocchi ```json prima dell'oggetto e un blocco
/// `<think>…</think>` iniziale; ignora le chiavi `narrazione` annidate.
#[derive(Debug, Default)]
pub struct EstrattoreNarrazione {
    grezzo: String,
    posizione: usize,
    fase: Fase,
    profondita: usize,
    /// Ultima stringa chiusa al primo livello (candidata come chiave).
    ultima_stringa: Option<String>,
    /// Stringa in lettura (solo al primo livello).
    stringa: String,
}

const APRI_RAGIONAMENTO: &str = "<think>";
const CHIUDI_RAGIONAMENTO: &str = "</think>";

impl EstrattoreNarrazione {
    /// Aggiunge un pezzo di risposta e restituisce il nuovo testo della narrazione.
    pub fn aggiungi(&mut self, pezzo: &str) -> String {
        self.grezzo.push_str(pezzo);
        let mut uscita = String::new();
        while self.posizione < self.grezzo.len() && self.fase != Fase::Finito {
            let resto = &self.grezzo[self.posizione..];
            let Some(c) = resto.chars().next() else {
                break;
            };
            let mut avanza = c.len_utf8();
            match self.fase {
                Fase::Fuori => {
                    if c == '<' {
                        if resto.starts_with(APRI_RAGIONAMENTO) {
                            self.fase = Fase::Ragionamento;
                            avanza = APRI_RAGIONAMENTO.len();
                        } else if APRI_RAGIONAMENTO.starts_with(resto) {
                            break; // tag ancora incompleto
                        }
                    } else if c == '{' {
                        self.fase = Fase::Json;
                        self.profondita = 1;
                        self.ultima_stringa = None;
                    }
                }
                Fase::Ragionamento => {
                    if c == '<' {
                        if resto.starts_with(CHIUDI_RAGIONAMENTO) {
                            self.fase = Fase::Fuori;
                            avanza = CHIUDI_RAGIONAMENTO.len();
                        } else if CHIUDI_RAGIONAMENTO.starts_with(resto) {
                            break;
                        }
                    }
                }
                Fase::Json => match c {
                    '{' | '[' => {
                        self.profondita += 1;
                        self.ultima_stringa = None;
                    }
                    '}' | ']' => {
                        self.profondita = self.profondita.saturating_sub(1);
                        self.ultima_stringa = None;
                        if self.profondita == 0 {
                            // Oggetto chiuso senza narrazione: si cerca il prossimo.
                            self.fase = Fase::Fuori;
                        }
                    }
                    '"' => {
                        self.fase = Fase::Stringa;
                        self.stringa.clear();
                    }
                    ':' => {
                        if self.profondita == 1
                            && self.ultima_stringa.as_deref() == Some("narrazione")
                        {
                            self.fase = Fase::DopoChiave;
                        }
                        self.ultima_stringa = None;
                    }
                    c if c.is_whitespace() => {}
                    _ => self.ultima_stringa = None,
                },
                Fase::Stringa => match c {
                    '\\' => {
                        // Basta saltare il carattere dopo la barra: nessuna
                        // sequenza (nemmeno \uXXXX) contiene virgolette.
                        let Some(dopo) = resto[1..].chars().next() else {
                            break;
                        };
                        avanza = 1 + dopo.len_utf8();
                        if self.profondita == 1 {
                            self.stringa.push('\\');
                            self.stringa.push(dopo);
                        }
                    }
                    '"' => {
                        self.fase = Fase::Json;
                        self.ultima_stringa = Some(std::mem::take(&mut self.stringa));
                    }
                    c => {
                        if self.profondita == 1 {
                            self.stringa.push(c);
                        }
                    }
                },
                Fase::DopoChiave => match c {
                    '"' => self.fase = Fase::Narrazione,
                    c if c.is_whitespace() => {}
                    // Valore non stringa (es. null): nessuna narrazione da mostrare.
                    _ => self.fase = Fase::Finito,
                },
                Fase::Narrazione => match c {
                    '"' => self.fase = Fase::Finito,
                    '\\' => match decodifica_escape(resto) {
                        Some((testo, lunghezza)) => {
                            uscita.push_str(&testo);
                            avanza = lunghezza;
                        }
                        None => break, // sequenza incompleta: si aspetta il resto
                    },
                    c => uscita.push(c),
                },
                Fase::Finito => break,
            }
            self.posizione += avanza;
        }
        uscita
    }

    /// Vero se la narrazione è stata letta per intero.
    pub fn completa(&self) -> bool {
        self.fase == Fase::Finito
    }
}

/// Decodifica la sequenza di escape all'inizio di `testo` (che comincia con
/// `\`): restituisce il testo e i byte consumati, `None` se è ancora incompleta.
fn decodifica_escape(testo: &str) -> Option<(String, usize)> {
    let mut caratteri = testo[1..].chars();
    let c = caratteri.next()?;
    let semplice = match c {
        'n' => Some('\n'),
        't' => Some('\t'),
        'r' => Some('\r'),
        'b' | 'f' => Some(' '),
        'u' => None,
        altro => Some(altro), // \" \\ \/ e sequenze non standard
    };
    if let Some(s) = semplice {
        return Some((s.to_string(), 1 + c.len_utf8()));
    }
    // `None` = testo ancora troppo corto; `Some(None)` = byte non validi.
    let porzione = |da: usize, a: usize| -> Option<Option<&str>> {
        if testo.len() < a {
            return None;
        }
        Some(testo.get(da..a))
    };
    let codice = |da: usize| -> Option<Option<u32>> {
        Some(porzione(da, da + 4)?.and_then(|esa| u32::from_str_radix(esa, 16).ok()))
    };
    let Some(alto) = codice(2)? else {
        return Some((String::new(), 2)); // \u non valida: si scarta
    };
    if (0xD800..0xDC00).contains(&alto) {
        // Coppia surrogata: serve la seconda metà (\uDC00–\uDFFF).
        let seconda = porzione(6, 8)?;
        if seconda == Some("\\u")
            && let Some(basso) = codice(8)?
            && (0xDC00..0xE000).contains(&basso)
        {
            let unito = 0x10000 + ((alto - 0xD800) << 10) + (basso - 0xDC00);
            return Some((
                char::from_u32(unito).map(String::from).unwrap_or_default(),
                12,
            ));
        }
        return Some(('\u{FFFD}'.to_string(), 6));
    }
    Some((char::from_u32(alto).unwrap_or('\u{FFFD}').to_string(), 6))
}

#[cfg(test)]
mod test {
    use super::*;

    /// Passa il testo all'estrattore a pezzi di `n` caratteri.
    fn a_pezzi(testo: &str, n: usize) -> String {
        let mut e = EstrattoreNarrazione::default();
        let caratteri: Vec<char> = testo.chars().collect();
        let mut uscita = String::new();
        for pezzo in caratteri.chunks(n) {
            uscita.push_str(&e.aggiungi(&pezzo.iter().collect::<String>()));
        }
        uscita
    }

    #[test]
    fn narrazione_a_pezzi_di_ogni_dimensione() {
        let json = r#"{"aggiornamenti":{"narrazione":"no","flag":{"x":"}"}},"narrazione":"Apri la porta.\nUn \"soffio\" d'aria è fredda 😀 😀.","memoria":"m"}"#;
        let atteso = "Apri la porta.\nUn \"soffio\" d'aria è fredda 😀 😀.";
        for n in 1..12 {
            assert_eq!(a_pezzi(json, n), atteso, "pezzi da {n}");
        }
    }

    #[test]
    fn testo_e_ragionamento_prima_del_json() {
        let t = "<think>forse {\"narrazione\":\"falsa\"}</think>Ecco:\n```json\n{ \"narrazione\" : \"vera\" }\n```";
        for n in 1..6 {
            assert_eq!(a_pezzi(t, n), "vera");
        }
    }

    #[test]
    fn narrazione_dopo_altre_chiavi_e_completamento() {
        let mut e = EstrattoreNarrazione::default();
        assert_eq!(e.aggiungi(r#"{"memoria":"narrazione","narr"#), "");
        assert_eq!(e.aggiungi(r#"azione":"ci"#), "ci");
        assert!(!e.completa());
        assert_eq!(e.aggiungi(r#"ao", "x": "y"}"#), "ao");
        assert!(e.completa());
        assert_eq!(e.aggiungi(r#"{"narrazione":"altro"}"#), "");
    }

    #[test]
    fn righe_sse() {
        assert_eq!(dati_sse("data: {\"a\":1}"), Some("{\"a\":1}"));
        assert_eq!(dati_sse("data:[DONE]"), None);
        assert_eq!(dati_sse("event: ping"), None);
        assert_eq!(dati_sse(""), None);
    }
}
