//! Client minimale dell'API Bot di Telegram (§7.5): solo i metodi usati dal bot
//! (`getMe`, `deleteWebhook`, `getUpdates`, `sendMessage`, `editMessageText`,
//! `editMessageReplyMarkup`, `deleteMessage`, `answerCallbackQuery`,
//! `sendChatAction`, `setMyCommands`), con `reqwest` bloccante e strutture `serde` ridotte ai campi
//! necessari (https://core.telegram.org/bots/api).
//!
//! - Risposte `ok: false` → [`ErroreTelegram::Api`] con `description`.
//! - HTTP 429 / `parameters.retry_after` → attesa e nuovo tentativo.
//! - Il token non compare mai in log, errori o `Debug`: ogni testo d'errore passa
//!   da [`maschera`] (`bot<TOKEN>` → `bot***`).
//!
//! Contiene anche le utilità di formattazione: [`escape_html`] e [`dividi_testo`]
//! (parti ≤ 4.096 caratteri senza spezzare parole né tag HTML).

use std::fmt;
use std::time::Duration;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use thiserror::Error;
use tracing::{debug, warn};

/// Indirizzo predefinito dell'API Bot.
pub const BASE_URL_PREDEFINITO: &str = "https://api.telegram.org";

/// Lunghezza massima di un messaggio Telegram (caratteri dopo il parsing).
pub const LUNGHEZZA_MAX_MESSAGGIO: usize = 4096;

/// Tentativi dopo una risposta 429 prima di arrendersi.
const TENTATIVI_429: u32 = 5;

/// Attesa massima accettata per un singolo `retry_after` (secondi).
const ATTESA_MAX_429: u64 = 300;

/// Errori dell'API Bot. Nessuna variante contiene il token.
#[derive(Debug, Error)]
pub enum ErroreTelegram {
    /// Telegram ha risposto `ok: false`.
    #[error("Telegram ha rifiutato «{metodo}» (codice {codice}): {descrizione}")]
    Api {
        /// Metodo chiamato.
        metodo: String,
        /// `error_code`.
        codice: i64,
        /// `description`.
        descrizione: String,
    },
    /// Troppe richieste anche dopo le attese richieste da Telegram.
    #[error("Telegram limita le richieste per «{metodo}» (riprovare tra {secondi} s)")]
    TroppeRichieste {
        /// Metodo chiamato.
        metodo: String,
        /// Ultimo `retry_after`.
        secondi: u64,
    },
    /// Errore di rete (connessione, timeout).
    #[error("errore di rete verso Telegram («{metodo}»): {dettaglio}")]
    Rete {
        /// Metodo chiamato.
        metodo: String,
        /// Descrizione (mascherata).
        dettaglio: String,
    },
    /// Risposta non interpretabile.
    #[error("risposta di Telegram non valida per «{metodo}»: {dettaglio}")]
    Formato {
        /// Metodo chiamato.
        metodo: String,
        /// Descrizione (mascherata).
        dettaglio: String,
    },
}

impl ErroreTelegram {
    /// Vero se Telegram non è riuscito a interpretare l'HTML del messaggio.
    pub fn html_non_valido(&self) -> bool {
        matches!(self, ErroreTelegram::Api { codice: 400, descrizione, .. }
            if descrizione.to_lowercase().contains("parse entities"))
    }

    /// Vero se il messaggio da modificare non è cambiato (errore innocuo).
    pub fn non_modificato(&self) -> bool {
        matches!(self, ErroreTelegram::Api { codice: 400, descrizione, .. }
            if descrizione.to_lowercase().contains("not modified"))
    }
}

/// Risultato delle chiamate all'API.
pub type Risultato<T> = std::result::Result<T, ErroreTelegram>;

/// Sostituisce il token (e `bot<token>`) con `***` in un testo.
pub fn maschera(testo: &str, token: &str) -> String {
    if token.is_empty() {
        return testo.to_string();
    }
    testo
        .replace(&format!("bot{token}"), "bot***")
        .replace(token, "***")
}

/// Utente Telegram.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct Utente {
    /// Identificativo numerico.
    pub id: i64,
    /// Vero per i bot.
    #[serde(default)]
    pub is_bot: bool,
    /// Nome.
    #[serde(default)]
    pub first_name: String,
    /// Nome utente (senza `@`).
    #[serde(default)]
    pub username: Option<String>,
    /// Lingua dell'app Telegram dell'utente (codice IETF, es. `it`, `en-US`).
    #[serde(default)]
    pub language_code: Option<String>,
}

/// Chat.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct Chat {
    /// Identificativo.
    pub id: i64,
    /// `private`, `group`, `supergroup` o `channel`.
    #[serde(rename = "type", default)]
    pub tipo: String,
}

impl Chat {
    /// Vero per le chat private.
    pub fn privata(&self) -> bool {
        self.tipo == "private"
    }
}

/// Messaggio (anche `MaybeInaccessibleMessage` delle callback: hanno in comune
/// `message_id` e `chat`).
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct Messaggio {
    /// Identificativo nella chat.
    pub message_id: i64,
    /// Mittente (assente nei canali).
    #[serde(default)]
    pub from: Option<Utente>,
    /// Chat.
    pub chat: Chat,
    /// Testo (assente per foto, vocali, sticker…).
    #[serde(default)]
    pub text: Option<String>,
}

/// Pressione di un pulsante inline.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct CallbackQuery {
    /// Identificativo da passare ad `answerCallbackQuery`.
    pub id: String,
    /// Chi ha premuto.
    pub from: Utente,
    /// Messaggio con il pulsante.
    #[serde(default)]
    pub message: Option<Messaggio>,
    /// `callback_data` del pulsante.
    #[serde(default)]
    pub data: Option<String>,
}

/// Un aggiornamento di `getUpdates`.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct Update {
    /// Identificativo progressivo.
    pub update_id: i64,
    /// Nuovo messaggio.
    #[serde(default)]
    pub message: Option<Messaggio>,
    /// Messaggio modificato (ignorato).
    #[serde(default)]
    pub edited_message: Option<Messaggio>,
    /// Messaggio di un canale (ignorato).
    #[serde(default)]
    pub channel_post: Option<Messaggio>,
    /// Pulsante premuto.
    #[serde(default)]
    pub callback_query: Option<CallbackQuery>,
}

/// Pulsante di una tastiera inline (`InlineKeyboardButton` con `callback_data`).
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Pulsante {
    /// Testo mostrato.
    pub text: String,
    /// Dati restituiti nella callback (1–64 byte).
    pub callback_data: String,
}

impl Pulsante {
    /// Nuovo pulsante.
    pub fn nuovo(testo: impl Into<String>, dati: impl Into<String>) -> Self {
        Pulsante {
            text: testo.into(),
            callback_data: dati.into(),
        }
    }
}

/// Tastiera inline: righe di pulsanti.
pub type Tastiera = Vec<Vec<Pulsante>>;

fn markup(tastiera: &Tastiera) -> Value {
    json!({ "inline_keyboard": tastiera })
}

#[derive(Deserialize)]
struct ParametriRisposta {
    #[serde(default)]
    retry_after: Option<u64>,
}

#[derive(Deserialize)]
struct RispostaApi<T> {
    ok: bool,
    #[serde(default = "Option::default")]
    result: Option<T>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    error_code: Option<i64>,
    #[serde(default)]
    parameters: Option<ParametriRisposta>,
}

/// Client dell'API Bot. Clonabile e condivisibile tra thread.
#[derive(Clone)]
pub struct Api {
    client: reqwest::blocking::Client,
    base_url: String,
    token: String,
    timeout: Duration,
}

impl fmt::Debug for Api {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Api")
            .field("base_url", &self.base_url)
            .field("token", &"***")
            .finish()
    }
}

impl Api {
    /// Client verso l'API ufficiale.
    pub fn nuovo(token: &str) -> Risultato<Self> {
        Api::con_base_url(token, BASE_URL_PREDEFINITO)
    }

    /// Client verso un indirizzo diverso: solo test e collaudo (server di prova, `AVVENTURA_TELEGRAM_API`).
    pub fn con_base_url(token: &str, base_url: &str) -> Risultato<Self> {
        let client = reqwest::blocking::Client::builder()
            .connect_timeout(Duration::from_secs(10))
            // Il timeout di ogni richiesta è impostato chiamata per chiamata.
            .timeout(None::<Duration>)
            .build()
            .map_err(|e| ErroreTelegram::Rete {
                metodo: "inizializzazione".into(),
                dettaglio: maschera(&e.without_url().to_string(), token),
            })?;
        Ok(Api {
            client,
            base_url: base_url.trim_end_matches('/').to_string(),
            token: token.to_string(),
            timeout: Duration::from_secs(30),
        })
    }

    /// Testo con il token mascherato.
    pub fn maschera(&self, testo: &str) -> String {
        maschera(testo, &self.token)
    }

    /// Chiamata generica con il timeout HTTP predefinito.
    pub fn chiama<T: DeserializeOwned>(&self, metodo: &str, corpo: &Value) -> Risultato<T> {
        self.chiama_con_timeout(metodo, corpo, self.timeout)
    }

    /// Chiamata generica: POST JSON, gestione di `ok: false` e dei 429.
    pub fn chiama_con_timeout<T: DeserializeOwned>(
        &self,
        metodo: &str,
        corpo: &Value,
        timeout: Duration,
    ) -> Risultato<T> {
        let url = format!("{}/bot{}/{metodo}", self.base_url, self.token);
        let mut ultimo_retry = 0;
        for tentativo in 0..=TENTATIVI_429 {
            let risposta = self
                .client
                .post(&url)
                .timeout(timeout)
                .json(corpo)
                .send()
                .map_err(|e| ErroreTelegram::Rete {
                    metodo: metodo.to_string(),
                    dettaglio: self.maschera(&e.without_url().to_string()),
                })?;
            let stato = risposta.status().as_u16();
            let testo = risposta.text().map_err(|e| ErroreTelegram::Rete {
                metodo: metodo.to_string(),
                dettaglio: self.maschera(&e.without_url().to_string()),
            })?;
            let r: RispostaApi<T> =
                serde_json::from_str(&testo).map_err(|e| ErroreTelegram::Formato {
                    metodo: metodo.to_string(),
                    dettaglio: self.maschera(&format!("HTTP {stato}: {e}")),
                })?;
            if r.ok {
                return r.result.ok_or_else(|| ErroreTelegram::Formato {
                    metodo: metodo.to_string(),
                    dettaglio: "campo «result» mancante".into(),
                });
            }
            let retry = r.parameters.and_then(|p| p.retry_after);
            if stato == 429 || r.error_code == Some(429) || retry.is_some() {
                let secondi = retry.unwrap_or(1).min(ATTESA_MAX_429);
                ultimo_retry = secondi;
                if tentativo < TENTATIVI_429 {
                    warn!(metodo, secondi, "Telegram chiede di attendere (429)");
                    std::thread::sleep(Duration::from_secs(secondi));
                    continue;
                }
                break;
            }
            return Err(ErroreTelegram::Api {
                metodo: metodo.to_string(),
                codice: r.error_code.unwrap_or(i64::from(stato)),
                descrizione: self
                    .maschera(r.description.as_deref().unwrap_or("nessuna descrizione")),
            });
        }
        Err(ErroreTelegram::TroppeRichieste {
            metodo: metodo.to_string(),
            secondi: ultimo_retry,
        })
    }

    /// `getMe`.
    pub fn get_me(&self) -> Risultato<Utente> {
        self.chiama("getMe", &json!({}))
    }

    /// `deleteWebhook` (senza scartare gli aggiornamenti in attesa).
    pub fn delete_webhook(&self) -> Risultato<bool> {
        self.chiama("deleteWebhook", &json!({ "drop_pending_updates": false }))
    }

    /// `setMyCommands`; con `lingua` (codice ISO 639-1) i comandi valgono per gli
    /// utenti con quella lingua nell'app, senza per tutti gli altri.
    pub fn set_my_commands(
        &self,
        comandi: &[(&str, &str)],
        lingua: Option<&str>,
    ) -> Risultato<bool> {
        let lista: Vec<Value> = comandi
            .iter()
            .map(|(c, d)| json!({ "command": c, "description": d }))
            .collect();
        let mut corpo = json!({ "commands": lista });
        if let Some(l) = lingua {
            corpo["language_code"] = json!(l);
        }
        self.chiama("setMyCommands", &corpo)
    }

    /// `getUpdates` in long polling: il timeout HTTP supera quello del polling di 15 s.
    pub fn get_updates(&self, offset: i64, timeout_polling: u64) -> Risultato<Vec<Update>> {
        let corpo = json!({
            "offset": offset,
            "timeout": timeout_polling,
            "allowed_updates": ["message", "callback_query"],
        });
        debug!(offset, "getUpdates");
        self.chiama_con_timeout(
            "getUpdates",
            &corpo,
            Duration::from_secs(timeout_polling + 15),
        )
    }

    /// `sendMessage` con `parse_mode = HTML` (o testo semplice) e tastiera facoltativa.
    pub fn send_message(
        &self,
        chat_id: i64,
        testo: &str,
        html: bool,
        tastiera: Option<&Tastiera>,
    ) -> Risultato<Messaggio> {
        let mut corpo = json!({
            "chat_id": chat_id,
            "text": testo,
            "link_preview_options": { "is_disabled": true },
        });
        if html {
            corpo["parse_mode"] = json!("HTML");
        }
        if let Some(t) = tastiera {
            corpo["reply_markup"] = markup(t);
        }
        self.chiama("sendMessage", &corpo)
    }

    /// `editMessageText` (HTML); senza tastiera i pulsanti vengono rimossi.
    pub fn edit_message_text(
        &self,
        chat_id: i64,
        message_id: i64,
        testo: &str,
        tastiera: Option<&Tastiera>,
    ) -> Risultato<()> {
        let mut corpo = json!({
            "chat_id": chat_id,
            "message_id": message_id,
            "text": testo,
            "parse_mode": "HTML",
            "link_preview_options": { "is_disabled": true },
        });
        if let Some(t) = tastiera {
            corpo["reply_markup"] = markup(t);
        }
        // Restituisce il messaggio modificato (o `true`): il contenuto non serve.
        self.chiama::<Value>("editMessageText", &corpo).map(|_| ())
    }

    /// `editMessageReplyMarkup` senza tastiera: rimuove i pulsanti.
    pub fn rimuovi_pulsanti(&self, chat_id: i64, message_id: i64) -> Risultato<()> {
        let corpo = json!({ "chat_id": chat_id, "message_id": message_id });
        self.chiama::<Value>("editMessageReplyMarkup", &corpo)
            .map(|_| ())
    }

    /// `answerCallbackQuery` con una notifica facoltativa (max 200 caratteri).
    pub fn answer_callback_query(&self, id: &str, testo: Option<&str>) -> Risultato<bool> {
        let mut corpo = json!({ "callback_query_id": id });
        if let Some(t) = testo {
            corpo["text"] = json!(t.chars().take(200).collect::<String>());
        }
        self.chiama("answerCallbackQuery", &corpo)
    }

    /// `deleteMessage`: cancella un messaggio inviato dal bot.
    pub fn delete_message(&self, chat_id: i64, message_id: i64) -> Risultato<bool> {
        self.chiama(
            "deleteMessage",
            &json!({ "chat_id": chat_id, "message_id": message_id }),
        )
    }

    /// `sendChatAction` = `typing` (dura al massimo 5 s).
    pub fn sta_scrivendo(&self, chat_id: i64) -> Risultato<bool> {
        self.chiama(
            "sendChatAction",
            &json!({ "chat_id": chat_id, "action": "typing" }),
        )
    }
}

/// Escape HTML per `parse_mode = HTML`: `&`, `<`, `>` (e `"`, innocuo).
pub fn escape_html(testo: &str) -> String {
    let mut s = String::with_capacity(testo.len());
    for c in testo.chars() {
        match c {
            '&' => s.push_str("&amp;"),
            '<' => s.push_str("&lt;"),
            '>' => s.push_str("&gt;"),
            '"' => s.push_str("&quot;"),
            _ => s.push(c),
        }
    }
    s
}

/// Lunghezza come la conta Telegram (unità UTF-16: un'emoji può valere 2).
pub fn lunghezza_telegram(testo: &str) -> usize {
    testo.encode_utf16().count()
}

/// Pezzo di un testo HTML: tag, entità, spazio o parola.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Pezzo<'a> {
    Tag(&'a str),
    Spazio(&'a str),
    Parola(&'a str),
}

fn pezzi(testo: &str) -> Vec<Pezzo<'_>> {
    let mut v = Vec::new();
    let mut resto = testo;
    while let Some(c) = resto.chars().next() {
        if c == '<'
            && let Some(fine) = resto.find('>')
        {
            v.push(Pezzo::Tag(&resto[..=fine]));
            resto = &resto[fine + 1..];
        } else if c.is_whitespace() {
            let fine = resto
                .find(|c: char| !c.is_whitespace())
                .unwrap_or(resto.len());
            v.push(Pezzo::Spazio(&resto[..fine]));
            resto = &resto[fine..];
        } else {
            // Una parola termina a uno spazio o all'inizio di un tag; le entità
            // (`&amp;`) restano dentro la parola e quindi non vengono spezzate.
            let fine = resto
                .char_indices()
                .skip(1)
                .find(|(_, c)| c.is_whitespace() || *c == '<')
                .map_or(resto.len(), |(i, _)| i);
            v.push(Pezzo::Parola(&resto[..fine]));
            resto = &resto[fine..];
        }
    }
    v
}

/// Nome di un tag (`<b>` → `b`, `</b>` → `b`, `<a href=…>` → `a`).
fn nome_tag(tag: &str) -> String {
    tag.trim_start_matches('<')
        .trim_start_matches('/')
        .trim_end_matches('>')
        .split(|c: char| c.is_whitespace() || c == '/')
        .next()
        .unwrap_or("")
        .to_lowercase()
}

/// Divide un testo HTML in parti di al massimo `massimo` caratteri (unità
/// UTF-16), senza spezzare parole, entità o tag: i tag aperti al punto di
/// divisione vengono chiusi alla fine di una parte e riaperti nella successiva.
/// Si preferisce dividere a un a capo; una parola più lunga del limite (caso
/// estremo) viene spezzata tra caratteri.
pub fn dividi_testo(testo: &str, massimo: usize) -> Vec<String> {
    let massimo = massimo.max(64);
    if lunghezza_telegram(testo) <= massimo {
        return if testo.trim().is_empty() {
            Vec::new()
        } else {
            vec![testo.to_string()]
        };
    }
    let mut parti: Vec<String> = Vec::new();
    let mut corrente = String::new();
    // Tag aperti (testo completo del tag di apertura).
    let mut aperti: Vec<String> = Vec::new();
    // Ultimo a capo nella parte corrente: (posizione in byte, tag aperti lì).
    let mut ultimo_a_capo: Option<(usize, Vec<String>)> = None;

    let chiusure = |aperti: &[String]| -> String {
        aperti
            .iter()
            .rev()
            .map(|t| format!("</{}>", nome_tag(t)))
            .collect()
    };
    let aperture = |aperti: &[String]| -> String { aperti.concat() };

    let chiudi_parte = |corrente: &mut String,
                        parti: &mut Vec<String>,
                        aperti_qui: &[String],
                        aperti_dopo: &[String]|
     -> String {
        let mut p = corrente.trim_end().to_string();
        p.push_str(&chiusure(aperti_qui));
        if !p.trim().is_empty() {
            parti.push(p);
        }
        aperture(aperti_dopo)
    };

    for pezzo in pezzi(testo) {
        match pezzo {
            Pezzo::Tag(t) => {
                if t.starts_with("</") {
                    let nome = nome_tag(t);
                    if let Some(i) = aperti.iter().rposition(|a| nome_tag(a) == nome) {
                        aperti.remove(i);
                    }
                } else if !t.ends_with("/>") {
                    aperti.push(t.to_string());
                }
                corrente.push_str(t);
            }
            Pezzo::Spazio(s) => {
                if s.contains('\n') {
                    ultimo_a_capo = Some((corrente.len(), aperti.clone()));
                }
                corrente.push_str(s);
            }
            Pezzo::Parola(p) => {
                let riserva = lunghezza_telegram(&chiusure(&aperti));
                if lunghezza_telegram(&corrente) + lunghezza_telegram(p) + riserva > massimo {
                    // Divisione: preferibilmente all'ultimo a capo, se non troppo indietro.
                    let soglia = lunghezza_telegram(&corrente) / 2;
                    match ultimo_a_capo.take() {
                        Some((pos, aperti_li)) if lunghezza_telegram(&corrente[..pos]) > soglia => {
                            let coda = corrente[pos..].trim_start().to_string();
                            let mut testa = corrente[..pos].to_string();
                            let inizio =
                                chiudi_parte(&mut testa, &mut parti, &aperti_li, &aperti_li);
                            corrente = format!("{inizio}{coda}");
                        }
                        _ => {
                            let inizio = chiudi_parte(&mut corrente, &mut parti, &aperti, &aperti);
                            corrente = inizio;
                        }
                    }
                }
                if lunghezza_telegram(&corrente) + lunghezza_telegram(p) + riserva > massimo {
                    // Parola più lunga del limite: si spezza tra caratteri.
                    for c in p.chars() {
                        if lunghezza_telegram(&corrente) + c.len_utf16() + riserva > massimo {
                            let inizio = chiudi_parte(&mut corrente, &mut parti, &aperti, &aperti);
                            corrente = inizio;
                        }
                        corrente.push(c);
                    }
                } else {
                    corrente.push_str(p);
                }
            }
        }
    }
    let ultima = corrente.trim_end().to_string();
    if !ultima.trim().is_empty() {
        parti.push(ultima);
    }
    parti
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn maschera_il_token() {
        let t = "123:ABC";
        assert_eq!(
            maschera("https://api.telegram.org/bot123:ABC/getMe e 123:ABC", t),
            "https://api.telegram.org/bot***/getMe e ***"
        );
        assert_eq!(maschera("niente", ""), "niente");
        let api = Api::con_base_url(t, "http://127.0.0.1:1").unwrap();
        assert!(!format!("{api:?}").contains("ABC"));
    }

    #[test]
    fn escape() {
        assert_eq!(
            escape_html("a < b && c > \"d\""),
            "a &lt; b &amp;&amp; c &gt; &quot;d&quot;"
        );
    }

    #[test]
    fn testo_corto_intatto() {
        assert_eq!(
            dividi_testo("ciao <b>mondo</b>", 4096),
            vec!["ciao <b>mondo</b>"]
        );
        assert!(dividi_testo("   ", 4096).is_empty());
    }

    #[test]
    fn divisione_senza_spezzare_parole() {
        let parola = "abcdefghij";
        let testo = vec![parola; 100].join(" ");
        let parti = dividi_testo(&testo, 100);
        assert!(parti.len() > 5);
        for p in &parti {
            assert!(lunghezza_telegram(p) <= 100, "{p}");
            for w in p.split_whitespace() {
                assert_eq!(w, parola);
            }
        }
        let ricomposto: Vec<&str> = parti.iter().flat_map(|p| p.split_whitespace()).collect();
        assert_eq!(ricomposto.len(), 100);
    }

    #[test]
    fn divisione_con_tag_chiusi_e_riaperti() {
        let corpo = vec!["parola &amp; altra"; 40].join(" ");
        let testo = format!("<b>Titolo</b>\n<i>{corpo}</i>");
        let parti = dividi_testo(&testo, 120);
        assert!(parti.len() > 2);
        for p in &parti {
            assert!(lunghezza_telegram(p) <= 120, "{p}");
            assert_eq!(p.matches("<i>").count(), p.matches("</i>").count(), "{p}");
            assert!(!p.contains("&amp ") && !p.ends_with('&'), "{p}");
        }
        assert!(parti[1].starts_with("<i>"));
    }

    #[test]
    fn preferisce_gli_a_capo() {
        let riga = vec!["aaaa"; 15].join(" "); // 74 caratteri
        let testo = format!("{riga}\n{riga}\n{riga}");
        let parti = dividi_testo(&testo, 100);
        assert_eq!(parti[0], riga);
    }

    #[test]
    fn parola_enorme_spezzata() {
        let testo = "x".repeat(250);
        let parti = dividi_testo(&testo, 100);
        assert_eq!(parti.len(), 3);
        assert_eq!(parti.concat(), testo);
    }

    #[test]
    fn deserializza_aggiornamenti() {
        let u: Vec<Update> = serde_json::from_value(json!([
            {"update_id": 5, "message": {"message_id": 1, "date": 0,
              "from": {"id": 7, "is_bot": false, "first_name": "A"},
              "chat": {"id": 7, "type": "private"}, "text": "/start"}},
            {"update_id": 6, "callback_query": {"id": "q", "chat_instance": "z",
              "from": {"id": 7, "is_bot": false, "first_name": "A"},
              "message": {"message_id": 3, "date": 0, "chat": {"id": 7, "type": "private"}},
              "data": "1:0"}},
            {"update_id": 7, "message": {"message_id": 2, "date": 0,
              "chat": {"id": -5, "type": "group"}, "photo": []}}
        ]))
        .unwrap();
        assert_eq!(
            u[0].message.as_ref().unwrap().text.as_deref(),
            Some("/start")
        );
        assert!(u[0].message.as_ref().unwrap().chat.privata());
        assert_eq!(
            u[1].callback_query.as_ref().unwrap().data.as_deref(),
            Some("1:0")
        );
        assert!(u[2].message.as_ref().unwrap().text.is_none());
    }
}
