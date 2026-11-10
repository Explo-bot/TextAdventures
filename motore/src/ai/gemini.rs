//! Fornitore `gemini`: API Google Gemini
//! (`POST {base_url}/models/{modello}:generateContent`, §6.1).
//!
//! Verificato sulla documentazione corrente (novembre 2026): `generateContent`
//! è ancora pubblicato e supportato accanto alla più recente "Interactions
//! API"; si resta su `generateContent` come prevede la specifica.
//!
//! - Header `x-goog-api-key` (solo se c'è una chiave).
//! - Istruzioni in `systemInstruction`; messaggi in `contents` con ruoli
//!   `user`/`model` (l'assistente è `model`).
//! - `generationConfig`: `temperature`, `maxOutputTokens`, `responseMimeType =
//!   "application/json"` (se `json_mode`), `thinkingConfig.thinkingLevel`
//!   (`minimal`/`low`/`medium`/`high`) secondo `ragionamento`. Sui modelli che
//!   non ammettono un livello (es. i Pro non scendono a `minimal`) il 400 fa
//!   ripetere la richiesta senza `thinkingConfig`.
//! - Testo: `candidates[0].content.parts[*].text`, ignorando le parti con
//!   `thought: true`. `finishReason` diverso da `STOP` (es. `SAFETY`,
//!   `MAX_TOKENS`) e `promptFeedback.blockReason` producono un messaggio chiaro.
//!   Token di prompt da `usageMetadata.promptTokenCount`.
//! - In streaming (testo progressivo) usa `:streamGenerateContent?alt=sse`:
//!   ogni evento `data:` ha lo stesso formato della risposta completa.

use std::time::{Duration, Instant};

use serde_json::{Value, json};
use tracing::{debug, info};

use super::flusso::dati_sse;
use super::http::{
    ClientHttp, ParametriRifiutati, StileChiave, avviso_parametro, base_senza_barra, corpo_json,
    errore_da_stato, inviati,
};
use super::ragionamento::Ragionamento;
use super::{ErroreAi, Narratore, RichiestaAi, RispostaGrezza, Ruolo, separa_sistema};
use crate::config::ConfigAi;

/// Parametri facoltativi che il server può rifiutare, in ordine di priorità.
const FACOLTATIVI: [&str; 4] = [
    "thinkingLevel",
    "thinkingConfig",
    "responseMimeType",
    "temperature",
];

/// Client Gemini.
#[derive(Debug)]
pub struct ClientGemini {
    http: ClientHttp,
    url: String,
    url_flusso: String,
    modello: String,
    temperatura: f32,
    max_token: u32,
    ragionamento: Ragionamento,
    json_mode: bool,
    rifiutati: ParametriRifiutati,
}

impl ClientGemini {
    /// Crea il client dalla configurazione.
    pub fn nuovo(config: &ConfigAi) -> Result<Self, ErroreAi> {
        let modello = config
            .modello
            .trim()
            .trim_start_matches("models/")
            .to_string();
        if modello.is_empty() {
            return Err(ErroreAi::Configurazione(
                "per il fornitore gemini indica il modello (es. modello = \"gemini-3.6-flash\")"
                    .to_string(),
            ));
        }
        let base = base_senza_barra(&config.base_url);
        Ok(ClientGemini {
            http: ClientHttp::nuovo(config, StileChiave::Google)?,
            url: format!("{base}/models/{modello}:generateContent"),
            url_flusso: format!("{base}/models/{modello}:streamGenerateContent?alt=sse"),
            modello,
            temperatura: config.temperatura,
            max_token: config.max_token,
            ragionamento: config.ragionamento,
            json_mode: config.json_mode,
            rifiutati: ParametriRifiutati::default(),
        })
    }

    /// Cambia l'attesa base del backoff (per i test).
    pub fn con_backoff(mut self, base: Duration) -> Self {
        self.http.imposta_backoff(base);
        self
    }

    /// Corpo JSON della richiesta.
    pub fn corpo(&self, richiesta: &RichiestaAi) -> Value {
        let (sistema, altri) = separa_sistema(&richiesta.messaggi);
        let contenuti: Vec<Value> = altri
            .iter()
            .map(|m| {
                let ruolo = if m.ruolo == Ruolo::Assistant {
                    "model"
                } else {
                    "user"
                };
                json!({ "role": ruolo, "parts": [ { "text": m.contenuto } ] })
            })
            .collect();
        let mut generazione = json!({ "maxOutputTokens": self.max_token });
        if self.rifiutati.attivo("temperature") {
            generazione["temperature"] = json!(self.temperatura);
        }
        if self.json_mode && self.rifiutati.attivo("responseMimeType") {
            generazione["responseMimeType"] = json!("application/json");
        }
        if let Some(livello) = self.ragionamento.per_gemini()
            && self.rifiutati.attivo("thinkingConfig")
            && self.rifiutati.attivo("thinkingLevel")
        {
            generazione["thinkingConfig"] = json!({ "thinkingLevel": livello });
        }
        let mut corpo = json!({
            "contents": contenuti,
            "generationConfig": generazione,
        });
        if !sistema.is_empty() {
            corpo["systemInstruction"] = json!({ "parts": [ { "text": sistema } ] });
        }
        corpo
    }
}

/// Messaggio per un `finishReason` diverso da `STOP`.
fn spiega_fine(motivo: &str) -> String {
    match motivo {
        "MAX_TOKENS" => "la risposta è stata troncata da max_token: aumenta max_token o \
                         abbassa ragionamento"
            .to_string(),
        "SAFETY" | "PROHIBITED_CONTENT" | "BLOCKLIST" | "SPII" => {
            "Gemini ha bloccato la risposta per i filtri di sicurezza: riformula l'azione"
                .to_string()
        }
        "RECITATION" => "Gemini ha bloccato la risposta (citazione di testi protetti): \
                         riformula l'azione"
            .to_string(),
        altro => format!("Gemini ha interrotto la risposta (finishReason = {altro})"),
    }
}

/// Testo e metadati di una risposta (letta per intero o ricomposta dallo streaming).
#[derive(Debug, Default)]
struct Raccolta {
    testo: String,
    /// Almeno un candidato ricevuto.
    candidato: bool,
    fine: Option<String>,
    blocco: Option<String>,
    token_prompt: Option<u64>,
    token_generati: Option<u64>,
    /// Token spesi nel ragionamento (solo per il log).
    token_pensiero: Option<u64>,
    /// `modelVersion` dichiarato dal server (solo per il log).
    versione_modello: Option<String>,
    errore: Option<String>,
}

impl Raccolta {
    /// Aggiunge una risposta completa o un pezzo dello streaming (stesso formato).
    fn aggiungi(&mut self, valore: &Value, pezzo: &mut dyn FnMut(&str)) {
        if let Some(e) = valore.get("error") {
            self.errore = Some(e["message"].as_str().unwrap_or(&e.to_string()).to_string());
            return;
        }
        if let Some(b) = valore["promptFeedback"]["blockReason"].as_str() {
            self.blocco = Some(b.to_string());
        }
        let candidato = &valore["candidates"][0];
        if !candidato.is_null() {
            self.candidato = true;
        }
        for parte in candidato["content"]["parts"]
            .as_array()
            .into_iter()
            .flatten()
        {
            if parte["thought"] != json!(true)
                && let Some(t) = parte["text"].as_str()
                && !t.is_empty()
            {
                self.testo.push_str(t);
                pezzo(t);
            }
        }
        if let Some(f) = candidato["finishReason"].as_str() {
            self.fine = Some(f.to_string());
        }
        if let Some(n) = valore["usageMetadata"]["promptTokenCount"].as_u64() {
            self.token_prompt = Some(n);
        }
        if let Some(n) = valore["usageMetadata"]["candidatesTokenCount"].as_u64() {
            self.token_generati = Some(n);
        }
        if let Some(n) = valore["usageMetadata"]["thoughtsTokenCount"].as_u64() {
            self.token_pensiero = Some(n);
        }
        if let Some(v) = valore["modelVersion"].as_str() {
            self.versione_modello = Some(v.to_string());
        }
    }
}

impl ClientGemini {
    /// Richiesta completa; con `flusso` la risposta arriva in streaming
    /// (`:streamGenerateContent?alt=sse`).
    fn esegui(
        &self,
        richiesta: &RichiestaAi,
        mut flusso: Option<&mut dyn FnMut(&str)>,
    ) -> Result<RispostaGrezza, ErroreAi> {
        let inizio = Instant::now();
        let mut avvisi = Vec::new();
        loop {
            let corpo = self.corpo(richiesta);
            let mut raccolta = Raccolta::default();
            let url = if flusso.is_some() {
                &self.url_flusso
            } else {
                &self.url
            };
            info!(
                url = %url,
                modello = %self.modello,
                flusso = flusso.is_some(),
                messaggi = richiesta.messaggi.len(),
                caratteri_prompt = richiesta.messaggi.iter().map(|m| m.contenuto.len()).sum::<usize>(),
                max_token = self.max_token,
                ragionamento = ?corpo["generationConfig"]["thinkingConfig"]["thinkingLevel"].as_str(),
                "richiesta al narratore (gemini)"
            );
            debug!(corpo = %corpo, "corpo della richiesta");
            let risposta = match flusso.as_deref_mut() {
                Some(pezzo) => self.http.post_json_flusso(url, &corpo, &mut |riga| {
                    if let Some(evento) =
                        dati_sse(riga).and_then(|d| serde_json::from_str::<Value>(d).ok())
                    {
                        raccolta.aggiungi(&evento, pezzo);
                    }
                })?,
                None => self.http.post_json(url, &corpo)?,
            };
            info!(
                stato = risposta.stato,
                byte = risposta.corpo.len(),
                "risposta ricevuta (gemini)"
            );
            debug!(corpo = %risposta.corpo, "risposta grezza (gemini)");
            if let Some(parametro) = self
                .rifiutati
                .gestisci(&risposta, &inviati(&corpo, &FACOLTATIVI))
            {
                avvisi.push(avviso_parametro(&parametro));
                continue;
            }
            if risposta.stato >= 400 {
                return Err(errore_da_stato(
                    risposta.stato,
                    &risposta.corpo,
                    &format!(
                        "controlla base_url e il nome del modello («{}»)",
                        self.modello
                    ),
                ));
            }
            if flusso.is_none() {
                raccolta.aggiungi(&corpo_json(&risposta)?, &mut |_| {});
            }
            info!(
                fine = ?raccolta.fine,
                blocco = ?raccolta.blocco,
                errore = ?raccolta.errore,
                token_prompt = ?raccolta.token_prompt,
                token_generati = ?raccolta.token_generati,
                token_pensiero = ?raccolta.token_pensiero,
                versione_modello = ?raccolta.versione_modello,
                caratteri_testo = raccolta.testo.len(),
                ms_dall_inizio_turno = u64::try_from(inizio.elapsed().as_millis()).unwrap_or(u64::MAX),
                "esito della risposta (gemini)"
            );
            if let Some(errore) = raccolta.errore.take() {
                return Err(ErroreAi::Formato(format!("errore di Gemini: {errore}")));
            }
            if let Some(blocco) = &raccolta.blocco {
                return Err(ErroreAi::Formato(format!(
                    "Gemini ha bloccato la richiesta (blockReason = {blocco}): riformula l'azione"
                )));
            }
            let testo = raccolta.testo;
            let fine = raccolta.fine.as_deref().unwrap_or("STOP");
            if fine != "STOP" && (testo.trim().is_empty() || fine != "MAX_TOKENS") {
                return Err(ErroreAi::Formato(spiega_fine(fine)));
            }
            if testo.trim().is_empty() && !raccolta.candidato {
                return Err(ErroreAi::Formato(
                    "nessun candidato nella risposta".to_string(),
                ));
            }
            return Ok(RispostaGrezza {
                testo,
                token_prompt: raccolta.token_prompt,
                token_generati: raccolta.token_generati,
                durata: inizio.elapsed(),
                avvisi,
            });
        }
    }
}

impl Narratore for ClientGemini {
    fn turno(&self, richiesta: &RichiestaAi) -> Result<RispostaGrezza, ErroreAi> {
        self.esegui(richiesta, None)
    }

    fn turno_in_flusso(
        &self,
        richiesta: &RichiestaAi,
        pezzo: &mut dyn FnMut(&str),
    ) -> Result<RispostaGrezza, ErroreAi> {
        self.esegui(richiesta, Some(pezzo))
    }

    fn descrizione(&self) -> String {
        format!("gemini · {} · {}", self.modello, self.url)
    }

    fn json_mode_attivo(&self) -> Option<bool> {
        Some(self.json_mode && self.rifiutati.attivo("responseMimeType"))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::ai::Messaggio;
    use crate::config::Fornitore;

    #[test]
    fn corpo_con_ruoli_gemini() {
        let config = ConfigAi {
            fornitore: Fornitore::Gemini,
            base_url: "https://generativelanguage.googleapis.com/v1beta/".into(),
            modello: "models/gemini-3.6-flash".into(),
            ragionamento: Ragionamento::Spento,
            ..ConfigAi::default()
        };
        let c = ClientGemini::nuovo(&config).unwrap();
        assert_eq!(
            c.url,
            "https://generativelanguage.googleapis.com/v1beta/models/gemini-3.6-flash:generateContent"
        );
        let corpo = c.corpo(&RichiestaAi {
            messaggi: vec![
                Messaggio::sistema("regole"),
                Messaggio::utente("u"),
                Messaggio::assistente("r"),
                Messaggio::utente("c"),
            ],
            contesto: Default::default(),
        });
        assert_eq!(corpo["systemInstruction"]["parts"][0]["text"], "regole");
        assert_eq!(corpo["contents"][1]["role"], "model");
        assert_eq!(
            corpo["generationConfig"]["responseMimeType"],
            "application/json"
        );
        assert_eq!(corpo["generationConfig"]["maxOutputTokens"], 1200);
        assert_eq!(
            corpo["generationConfig"]["thinkingConfig"]["thinkingLevel"],
            "minimal"
        );
    }
}
