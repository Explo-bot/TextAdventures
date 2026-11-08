//! Configurazione (`config.toml`, §5): tipi della configurazione, valori
//! predefiniti, ricerca del file, sezioni `[motore]`, `[interfaccia]` e
//! `[telegram]`, riconoscimento dei server locali e regole della chiave API.
//!
//! La risoluzione della sezione `[ai]` (fornitore, profili `[profili.*]`,
//! variabili d'ambiente, precedenze, predefiniti per fornitore) è in
//! [`crate::ai::profili`].
//!
//! Nessun segreto (chiave API, token Telegram, codice d'invito) compare mai
//! nell'output `Debug` né nei messaggi d'errore.

use std::collections::BTreeMap;
use std::fmt;
use std::fs;
use std::net::IpAddr;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use serde::Deserialize;

use crate::ai::profili;
use crate::ai::ragionamento::Ragionamento;

/// Valore minimo ammesso per i timeout (secondi).
pub const TIMEOUT_MIN: u64 = 1;
/// Valore massimo ammesso per i timeout (secondi).
pub const TIMEOUT_MAX: u64 = 86_400;

/// Testo mostrato al posto dei segreti.
pub const MASCHERATO: &str = "«nascosta»";

/// Fornitore del modello AI (§5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Fornitore {
    /// API OpenAI `/chat/completions` e servizi compatibili.
    #[default]
    Openai,
    /// API Anthropic Messages.
    Anthropic,
    /// API Google Gemini (`generateContent`).
    Gemini,
    /// API nativa di Ollama (`/api/chat`).
    Ollama,
    /// Server locale di LM Studio (compatibile OpenAI).
    Lmstudio,
    /// Modello GGUF caricato dal disco (llama.cpp, feature `locale`).
    Locale,
    /// AI simulata offline (§9).
    Finto,
}

impl Fornitore {
    /// Tutti i fornitori, nell'ordine della documentazione.
    pub const TUTTI: [Fornitore; 7] = [
        Fornitore::Openai,
        Fornitore::Anthropic,
        Fornitore::Gemini,
        Fornitore::Ollama,
        Fornitore::Lmstudio,
        Fornitore::Locale,
        Fornitore::Finto,
    ];

    /// Nome usato nel file di configurazione.
    pub fn nome(self) -> &'static str {
        match self {
            Fornitore::Openai => "openai",
            Fornitore::Anthropic => "anthropic",
            Fornitore::Gemini => "gemini",
            Fornitore::Ollama => "ollama",
            Fornitore::Lmstudio => "lmstudio",
            Fornitore::Locale => "locale",
            Fornitore::Finto => "finto",
        }
    }

    /// Interpreta il nome (senza distinzione tra maiuscole e minuscole).
    pub fn da_testo(testo: &str) -> Option<Self> {
        let t = testo.trim().to_lowercase();
        Fornitore::TUTTI.into_iter().find(|f| f.nome() == t)
    }

    /// Elenco dei nomi validi, per i messaggi d'errore.
    pub fn nomi_validi() -> String {
        Fornitore::TUTTI
            .iter()
            .map(|f| f.nome())
            .collect::<Vec<_>>()
            .join(", ")
    }

    /// `base_url` predefinito (vuoto per i fornitori senza HTTP).
    pub fn base_url_predefinito(self) -> &'static str {
        match self {
            Fornitore::Openai => "https://api.openai.com/v1",
            Fornitore::Anthropic => "https://api.anthropic.com",
            Fornitore::Gemini => "https://generativelanguage.googleapis.com/v1beta",
            Fornitore::Ollama => "http://localhost:11434",
            Fornitore::Lmstudio => "http://localhost:1234/v1",
            Fornitore::Locale | Fornitore::Finto => "",
        }
    }

    /// Variabile d'ambiente predefinita con la chiave (vuota = nessuna).
    pub fn api_key_env_predefinita(self) -> &'static str {
        match self {
            Fornitore::Openai => "OPENAI_API_KEY",
            Fornitore::Anthropic => "ANTHROPIC_API_KEY",
            Fornitore::Gemini => "GEMINI_API_KEY",
            _ => "",
        }
    }

    /// Modello predefinito (esempio modificabile; vuoto = da indicare).
    pub fn modello_predefinito(self) -> &'static str {
        match self {
            Fornitore::Openai => "gpt-6-luna",
            Fornitore::Anthropic => "claude-sonnet-5-5",
            Fornitore::Gemini => "gemini-3.6-flash",
            Fornitore::Ollama => "gemma4:12b",
            Fornitore::Lmstudio | Fornitore::Locale | Fornitore::Finto => "",
        }
    }

    /// Livello di ragionamento predefinito (§5.4).
    pub fn ragionamento_predefinito(self) -> Ragionamento {
        match self {
            Fornitore::Ollama | Fornitore::Lmstudio | Fornitore::Locale => Ragionamento::Spento,
            _ => Ragionamento::Predefinito,
        }
    }

    /// Vero se il fornitore comunica via HTTP.
    pub fn usa_http(self) -> bool {
        !matches!(self, Fornitore::Locale | Fornitore::Finto)
    }

    /// Vero se il fornitore, per sua natura, non richiede una chiave (§5.1).
    pub fn senza_chiave(self) -> bool {
        matches!(
            self,
            Fornitore::Ollama | Fornitore::Lmstudio | Fornitore::Locale | Fornitore::Finto
        )
    }
}

impl fmt::Display for Fornitore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.nome())
    }
}

/// Modalità di validazione delle risposte dell'AI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Validazione {
    /// Qualsiasi violazione fa ripetere la richiesta.
    #[default]
    Severa,
    /// Le violazioni non bloccanti vengono scartate singolarmente.
    Permissiva,
}

/// Sezione `[ai.locale]` (fornitore `locale`, §5.5).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigLocale {
    /// File GGUF (già risolto rispetto alla cartella del file di configurazione).
    pub percorso_modello: Option<PathBuf>,
    /// Strati sulla GPU: -1 = tutti, 0 = solo CPU.
    pub gpu_layers: i32,
    /// Thread di calcolo: 0 = automatico.
    pub thread: u32,
    /// Template di chat forzato (vuoto = quello del file GGUF).
    pub template_chat: Option<String>,
}

impl Default for ConfigLocale {
    fn default() -> Self {
        ConfigLocale {
            percorso_modello: None,
            gpu_layers: -1,
            thread: 0,
            template_chat: None,
        }
    }
}

/// Sezione `[ai]` già risolta (profilo, ambiente e predefiniti applicati).
#[derive(Clone)]
pub struct ConfigAi {
    /// Fornitore.
    pub fornitore: Fornitore,
    /// URL di base (vuoto per `locale` e `finto`).
    pub base_url: String,
    /// Variabile d'ambiente con la chiave (vuota = nessuna).
    pub api_key_env: String,
    /// Chiave scritta in chiaro nel file (sconsigliata).
    pub api_key: Option<String>,
    /// Chiave obbligatoria? `None` = automatico (§5.1).
    pub chiave_richiesta: Option<bool>,
    /// Nome del modello.
    pub modello: String,
    /// Temperatura.
    pub temperatura: f32,
    /// Token massimi della risposta.
    pub max_token: u32,
    /// Dimensione del contesto del modello (0 = sconosciuta, nessun controllo).
    pub contesto_token: u32,
    /// Livello di ragionamento.
    pub ragionamento: Ragionamento,
    /// Chiede al fornitore un output JSON garantito.
    pub json_mode: bool,
    /// Attesa massima della risposta (secondi).
    pub timeout_secondi: u64,
    /// Attesa massima della connessione (secondi).
    pub timeout_connessione_secondi: u64,
    /// Tentativi totali per turno.
    pub max_tentativi: u32,
    /// Header HTTP aggiuntivi.
    pub header_extra: BTreeMap<String, String>,
    /// Sezione `[ai.locale]`.
    pub locale: ConfigLocale,
    /// Chiave effettiva risolta da ambiente/file (mai stampata).
    pub chiave: Option<String>,
    /// Profilo usato (None = sezione `[ai]`).
    pub profilo: Option<String>,
    /// Avvisi di configurazione da mostrare una volta (es. chiave `protocollo`).
    pub avvisi: Vec<String>,
}

impl Default for ConfigAi {
    fn default() -> Self {
        let f = Fornitore::Openai;
        ConfigAi {
            fornitore: f,
            base_url: f.base_url_predefinito().to_string(),
            api_key_env: f.api_key_env_predefinita().to_string(),
            api_key: None,
            chiave_richiesta: None,
            modello: f.modello_predefinito().to_string(),
            temperatura: 0.7,
            max_token: 1200,
            contesto_token: 0,
            ragionamento: f.ragionamento_predefinito(),
            json_mode: true,
            timeout_secondi: 900,
            timeout_connessione_secondi: 10,
            max_tentativi: 3,
            header_extra: BTreeMap::new(),
            locale: ConfigLocale::default(),
            chiave: None,
            profilo: None,
            avvisi: Vec::new(),
        }
    }
}

/// Vero se il nome di un header fa pensare a un segreto.
pub fn header_sensibile(nome: &str) -> bool {
    let n = nome.to_lowercase();
    ["key", "auth", "token", "secret", "cookie"]
        .iter()
        .any(|s| n.contains(s))
}

impl fmt::Debug for ConfigAi {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let header: BTreeMap<&str, &str> = self
            .header_extra
            .iter()
            .map(|(k, v)| {
                let v = if header_sensibile(k) {
                    MASCHERATO
                } else {
                    v.as_str()
                };
                (k.as_str(), v)
            })
            .collect();
        f.debug_struct("ConfigAi")
            .field("fornitore", &self.fornitore)
            .field("profilo", &self.profilo)
            .field("base_url", &self.base_url)
            .field("api_key_env", &self.api_key_env)
            .field("api_key", &self.api_key.as_ref().map(|_| MASCHERATO))
            .field("chiave_richiesta", &self.chiave_richiesta)
            .field("modello", &self.modello)
            .field("temperatura", &self.temperatura)
            .field("max_token", &self.max_token)
            .field("contesto_token", &self.contesto_token)
            .field("ragionamento", &self.ragionamento)
            .field("json_mode", &self.json_mode)
            .field("timeout_secondi", &self.timeout_secondi)
            .field(
                "timeout_connessione_secondi",
                &self.timeout_connessione_secondi,
            )
            .field("max_tentativi", &self.max_tentativi)
            .field("header_extra", &header)
            .field("locale", &self.locale)
            .field("chiave", &self.chiave.as_ref().map(|_| MASCHERATO))
            .finish()
    }
}

impl ConfigAi {
    /// Vero se il fornitore è l'AI simulata.
    pub fn finto(&self) -> bool {
        self.fornitore == Fornitore::Finto
    }

    /// Vero se il modello gira su questa macchina (§5.1).
    pub fn server_locale(&self) -> bool {
        !self.fornitore.usa_http() || url_locale(&self.base_url)
    }

    /// Chiave obbligatoria? Usa il valore esplicito o il default automatico (§5.1).
    pub fn chiave_richiesta_effettiva(&self) -> bool {
        if !self.fornitore.usa_http() {
            return false;
        }
        if let Some(esplicito) = self.chiave_richiesta {
            return esplicito;
        }
        match self.fornitore {
            Fornitore::Openai => !url_locale(&self.base_url),
            Fornitore::Anthropic | Fornitore::Gemini => true,
            _ => false,
        }
    }

    /// Verifica che la chiave sia presente se richiesta (da chiamare prima del menu).
    pub fn verifica_chiave(&self) -> Result<()> {
        if self.chiave.is_some() || !self.chiave_richiesta_effettiva() {
            return Ok(());
        }
        let variabile = if self.api_key_env.is_empty() {
            "AVVENTURA_API_KEY"
        } else {
            self.api_key_env.as_str()
        };
        bail!(
            "manca la chiave API per il fornitore «{}» ({}).\n\
             Imposta la variabile d'ambiente {variabile} (oppure AVVENTURA_API_KEY) con la tua chiave, ad esempio:\n\
             \x20 Windows (PowerShell, permanente):  setx {variabile} \"la-tua-chiave\"   (poi riapri il terminale)\n\
             \x20 Linux/macOS:                       export {variabile}=\"la-tua-chiave\"\n\
             Se il server è nella tua rete locale e non usa chiavi, aggiungi \
             `chiave_richiesta = false` nella sezione [ai] (o nel profilo) di config.toml.",
            self.fornitore,
            self.base_url,
        )
    }

    /// `host:porta` del server (per i messaggi d'errore).
    pub fn host_porta(&self) -> String {
        reqwest::Url::parse(&self.base_url)
            .ok()
            .map(|u| {
                let host = u.host_str().unwrap_or("?").to_string();
                match u.port_or_known_default() {
                    Some(p) => format!("{host}:{p}"),
                    None => host,
                }
            })
            .unwrap_or_else(|| self.base_url.clone())
    }

    /// Controlla che i valori siano nei limiti ammessi.
    pub fn valida(&self) -> Result<()> {
        for (nome, valore) in [
            ("timeout_secondi", self.timeout_secondi),
            (
                "timeout_connessione_secondi",
                self.timeout_connessione_secondi,
            ),
        ] {
            if !(TIMEOUT_MIN..=TIMEOUT_MAX).contains(&valore) {
                bail!(
                    "valore non valido per {nome}: {valore}. Sono ammessi valori da \
                     {TIMEOUT_MIN} a {TIMEOUT_MAX} secondi"
                );
            }
        }
        if self.max_tentativi == 0 {
            bail!("max_tentativi deve essere almeno 1");
        }
        if !(0.0..=2.0).contains(&self.temperatura) {
            bail!("temperatura deve essere compresa tra 0 e 2");
        }
        if self.max_token == 0 {
            bail!("max_token deve essere maggiore di 0");
        }
        if self.fornitore.usa_http() {
            if self.base_url.trim().is_empty() {
                bail!(
                    "base_url non può essere vuoto per il fornitore «{}»",
                    self.fornitore
                );
            }
            if reqwest::Url::parse(self.base_url.trim()).is_err() {
                bail!("base_url «{}» non è un URL valido", self.base_url);
            }
        }
        Ok(())
    }
}

/// Vero se l'host dell'URL è `localhost`, `127.0.0.0/8`, `::1` o `host.docker.internal`.
pub fn url_locale(url: &str) -> bool {
    let Ok(u) = reqwest::Url::parse(url.trim()) else {
        return false;
    };
    let Some(host) = u.host_str() else {
        return false;
    };
    let host = host.trim_start_matches('[').trim_end_matches(']');
    match host.parse::<IpAddr>() {
        Ok(ip) => ip.is_loopback(),
        Err(_) => {
            let d = host.to_lowercase();
            d == "localhost" || d == "host.docker.internal" || d.ends_with(".localhost")
        }
    }
}

/// Sezione `[motore]`.
#[derive(Debug, Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ConfigMotore {
    /// Lunghezza del buffer `ultimi_turni`.
    pub turni_in_memoria: usize,
    /// Lunghezza massima di `memoria` in caratteri.
    pub lunghezza_max_memoria: usize,
    /// Note conservate per ogni stanza (le più vecchie vengono tolte).
    pub max_note_stanza: usize,
    /// Cartella delle avventure (se assente: ricerca automatica).
    pub cartella_avventure: Option<PathBuf>,
    /// Cartella delle partite (se assente: `partite/` accanto alle avventure).
    pub cartella_partite: Option<PathBuf>,
    /// Modalità di validazione.
    pub validazione: Validazione,
}

impl Default for ConfigMotore {
    fn default() -> Self {
        ConfigMotore {
            turni_in_memoria: 5,
            lunghezza_max_memoria: 2000,
            max_note_stanza: 10,
            cartella_avventure: None,
            cartella_partite: None,
            validazione: Validazione::Severa,
        }
    }
}

/// Sezione `[interfaccia]`.
#[derive(Debug, Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ConfigInterfaccia {
    /// Larghezza del testo (0 = larghezza del terminale).
    pub larghezza_testo: usize,
    /// Colori attivi.
    pub colori: bool,
    /// Mostra numero di turno e punteggio dopo ogni turno.
    pub mostra_turno: bool,
    /// Mostra la narrazione man mano che il modello la scrive (streaming),
    /// con un cursore animato finché la risposta non è completa.
    pub testo_progressivo: bool,
    /// Lingua dell'interfaccia: `auto` (chiede al primo utilizzo, proponendo
    /// quella del sistema o di Telegram), `it` o `en` (§7.6).
    pub lingua: String,
}

impl Default for ConfigInterfaccia {
    fn default() -> Self {
        ConfigInterfaccia {
            larghezza_testo: 90,
            colori: true,
            mostra_turno: true,
            testo_progressivo: true,
            lingua: "auto".to_string(),
        }
    }
}

/// Sezione `[telegram]` (§7.5, usata dallo script 03).
#[derive(Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ConfigTelegram {
    /// Variabile d'ambiente con il token del bot.
    pub token_env: String,
    /// ID Telegram ammessi.
    pub utenti_ammessi: Vec<i64>,
    /// ID con i comandi di amministrazione.
    pub amministratori: Vec<i64>,
    /// Codice d'invito facoltativo per `/start <codice>`.
    pub codice_invito: Option<String>,
    /// Cartella delle partite degli utenti Telegram.
    pub cartella_partite: PathBuf,
    /// Richieste AI contemporanee.
    pub max_turni_paralleli: u32,
    /// Minuti dopo i quali una domanda in sospeso scade.
    pub inattivita_minuti: u64,
    /// Profilo AI da usare per il bot.
    pub profilo: Option<String>,
}

impl Default for ConfigTelegram {
    fn default() -> Self {
        ConfigTelegram {
            token_env: "TELEGRAM_BOT_TOKEN".to_string(),
            utenti_ammessi: Vec::new(),
            amministratori: Vec::new(),
            codice_invito: None,
            cartella_partite: PathBuf::from("partite").join("telegram"),
            max_turni_paralleli: 1,
            inattivita_minuti: 30,
            profilo: None,
        }
    }
}

impl fmt::Debug for ConfigTelegram {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ConfigTelegram")
            .field("token_env", &self.token_env)
            .field("utenti_ammessi", &self.utenti_ammessi)
            .field("amministratori", &self.amministratori)
            .field(
                "codice_invito",
                &self.codice_invito.as_ref().map(|_| MASCHERATO),
            )
            .field("cartella_partite", &self.cartella_partite)
            .field("max_turni_paralleli", &self.max_turni_paralleli)
            .field("inattivita_minuti", &self.inattivita_minuti)
            .field("profilo", &self.profilo)
            .finish()
    }
}

/// Sezioni AI grezze lette dal file, conservate per risolvere altri profili
/// (sottocomando `profili`, profilo del bot Telegram).
#[derive(Clone, Default)]
pub struct SorgenteAi {
    /// Tabella `[ai]`.
    pub ai: toml::Table,
    /// Tabelle `[profili.*]`.
    pub profili: BTreeMap<String, toml::Table>,
    /// Cartella del file di configurazione (per i percorsi relativi).
    pub cartella: Option<PathBuf>,
}

/// Configurazione completa.
#[derive(Clone, Default)]
pub struct Config {
    /// `[ai]` risolta (con il profilo scelto).
    pub ai: ConfigAi,
    /// `[motore]`.
    pub motore: ConfigMotore,
    /// `[interfaccia]`.
    pub interfaccia: ConfigInterfaccia,
    /// `[telegram]`.
    pub telegram: ConfigTelegram,
    /// Sezioni AI grezze (mai stampate: possono contenere chiavi).
    pub sorgente: SorgenteAi,
    /// File da cui è stata letta (assente = valori predefiniti).
    pub file: Option<PathBuf>,
}

impl fmt::Debug for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Config")
            .field("file", &self.file)
            .field("ai", &self.ai)
            .field("profili", &self.sorgente.profili.keys().collect::<Vec<_>>())
            .field("motore", &self.motore)
            .field("interfaccia", &self.interfaccia)
            .field("telegram", &self.telegram)
            .finish()
    }
}

/// Opzioni da riga di comando che influenzano la configurazione.
#[derive(Debug, Clone, Default)]
pub struct OpzioniConfig {
    /// `--config FILE`.
    pub file: Option<PathBuf>,
    /// `--timeout SECONDI`.
    pub timeout: Option<u64>,
    /// `--profilo NOME`.
    pub profilo: Option<String>,
}

/// File di configurazione candidati, nell'ordine di ricerca (§5).
pub fn candidati(opzioni: &OpzioniConfig) -> Vec<PathBuf> {
    if let Some(f) = &opzioni.file {
        return vec![f.clone()];
    }
    let mut c = vec![
        PathBuf::from("config.toml"),
        PathBuf::from("config").join("config.toml"),
    ];
    if let Some(dir) = cartella_eseguibile() {
        c.push(dir.join("config.toml"));
        c.push(dir.join("config").join("config.toml"));
    }
    c
}

/// Cartella che contiene l'eseguibile.
pub fn cartella_eseguibile() -> Option<PathBuf> {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(Path::to_path_buf))
}

/// Parti del file diverse dalla sezione AI.
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct AltreSezioni {
    motore: ConfigMotore,
    interfaccia: ConfigInterfaccia,
    telegram: ConfigTelegram,
}

/// Lettore dell'ambiente (iniettabile nei test).
pub type LettoreAmbiente<'a> = &'a dyn Fn(&str) -> Option<String>;

impl Config {
    /// Carica la configurazione usando le variabili d'ambiente del processo.
    pub fn carica(opzioni: &OpzioniConfig) -> Result<Config> {
        Config::carica_con_ambiente(opzioni, &|k| std::env::var(k).ok())
    }

    /// Carica la configurazione con una funzione di lettura dell'ambiente (testabile).
    pub fn carica_con_ambiente(
        opzioni: &OpzioniConfig,
        ambiente: LettoreAmbiente<'_>,
    ) -> Result<Config> {
        let trovato = candidati(opzioni).into_iter().find(|p| p.is_file());
        if opzioni.file.is_some() && trovato.is_none() {
            bail!(
                "il file di configurazione «{}» non esiste",
                opzioni.file.as_deref().unwrap_or(Path::new("")).display()
            );
        }
        let mut config = match &trovato {
            Some(file) => {
                let testo = fs::read_to_string(file)
                    .with_context(|| format!("impossibile leggere «{}»", file.display()))?;
                let mut c = Config::da_toml(&testo).with_context(|| {
                    format!("configurazione non valida in «{}»", file.display())
                })?;
                c.sorgente.cartella = file.parent().map(Path::to_path_buf);
                c.file = Some(file.clone());
                c
            }
            None => Config::default(),
        };
        config.ai = config
            .risolvi_ai(opzioni.profilo.as_deref(), opzioni.timeout, ambiente)
            .with_context(|| match &config.file {
                Some(f) => format!("configurazione AI non valida in «{}»", f.display()),
                None => "configurazione AI non valida".to_string(),
            })?;
        config.valida()?;
        Ok(config)
    }

    /// Interpreta il testo TOML: sezioni lette, sezione AI non ancora risolta.
    pub fn da_toml(testo: &str) -> Result<Config> {
        let testo = crate::avventura::frontmatter::normalizza(testo);
        let mut tabella: toml::Table =
            toml::from_str(&testo).map_err(|e| anyhow::anyhow!("{}", e.message().trim()))?;
        let ai = match tabella.remove("ai") {
            Some(toml::Value::Table(t)) => t,
            Some(_) => bail!("[ai] deve essere una sezione"),
            None => toml::Table::new(),
        };
        let profili = match tabella.remove("profili") {
            Some(toml::Value::Table(t)) => t
                .into_iter()
                .map(|(nome, v)| match v {
                    toml::Value::Table(p) => Ok((nome, p)),
                    _ => bail!("il profilo «{nome}» deve essere una sezione [profili.{nome}]"),
                })
                .collect::<Result<BTreeMap<_, _>>>()?,
            Some(_) => bail!("[profili] deve contenere sezioni [profili.NOME]"),
            None => BTreeMap::new(),
        };
        let altre: AltreSezioni = toml::Value::Table(tabella)
            .try_into()
            .map_err(|e: toml::de::Error| anyhow::anyhow!("{}", e.message().trim()))?;
        Ok(Config {
            ai: ConfigAi::default(),
            motore: altre.motore,
            interfaccia: altre.interfaccia,
            telegram: altre.telegram,
            sorgente: SorgenteAi {
                ai,
                profili,
                cartella: None,
            },
            file: None,
        })
    }

    /// Risolve la sezione AI per un profilo (None = `AVVENTURA_PROFILO` o `[ai]`).
    pub fn risolvi_ai(
        &self,
        profilo: Option<&str>,
        timeout: Option<u64>,
        ambiente: LettoreAmbiente<'_>,
    ) -> Result<ConfigAi> {
        profili::risolvi(&self.sorgente, profilo, timeout, ambiente)
    }

    /// Controlla i valori delle sezioni diverse da `[ai]`.
    pub fn valida(&self) -> Result<()> {
        self.ai.valida()?;
        if self.motore.turni_in_memoria == 0 {
            bail!("turni_in_memoria deve essere almeno 1");
        }
        if self.telegram.max_turni_paralleli == 0 {
            bail!("telegram.max_turni_paralleli deve essere almeno 1");
        }
        let lingua = self.interfaccia.lingua.trim();
        if !lingua.eq_ignore_ascii_case("auto")
            && crate::lingua::Lingua::da_codice(lingua).is_none()
        {
            bail!(
                "interfaccia.lingua deve essere \"auto\", \"it\", \"en\", \"fr\", \"es\" o \"de\" (trovato \"{lingua}\")"
            );
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    pub(crate) fn ambiente(coppie: &[(&str, &str)]) -> impl Fn(&str) -> Option<String> + use<> {
        let mappa: BTreeMap<String, String> = coppie
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        move |k| mappa.get(k).cloned()
    }

    fn da_file(testo: &str, env: &[(&str, &str)], timeout: Option<u64>) -> Result<Config> {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("config.toml");
        fs::write(&file, testo).unwrap();
        let opzioni = OpzioniConfig {
            file: Some(file),
            timeout,
            profilo: None,
        };
        Config::carica_con_ambiente(&opzioni, &ambiente(env))
    }

    #[test]
    fn valori_predefiniti() {
        let c = da_file("", &[], None).unwrap();
        assert_eq!(c.ai.timeout_secondi, 900);
        assert_eq!(c.ai.timeout_connessione_secondi, 10);
        assert_eq!(c.ai.max_tentativi, 3);
        assert_eq!(c.motore.turni_in_memoria, 5);
        assert_eq!(c.motore.lunghezza_max_memoria, 2000);
        assert_eq!(c.interfaccia.larghezza_testo, 90);
        assert_eq!(c.ai.fornitore, Fornitore::Openai);
        assert_eq!(c.ai.base_url, "https://api.openai.com/v1");
        assert_eq!(c.ai.ragionamento, Ragionamento::Predefinito);
        assert!(c.ai.json_mode);
        assert_eq!(c.telegram.token_env, "TELEGRAM_BOT_TOKEN");
        assert_eq!(c.telegram.max_turni_paralleli, 1);
        assert_eq!(c.telegram.inattivita_minuti, 30);
        assert_eq!(c.ai.locale.gpu_layers, -1);
    }

    #[test]
    fn precedenza_del_timeout() {
        let c = da_file("[ai]\ntimeout_secondi = 120\n", &[], None).unwrap();
        assert_eq!(c.ai.timeout_secondi, 120);
        let c = da_file(
            "[ai]\ntimeout_secondi = 120\n",
            &[("AVVENTURA_TIMEOUT", "300")],
            None,
        )
        .unwrap();
        assert_eq!(c.ai.timeout_secondi, 300);
        let c = da_file(
            "[ai]\ntimeout_secondi = 120\n",
            &[("AVVENTURA_TIMEOUT", "300")],
            Some(45),
        )
        .unwrap();
        assert_eq!(c.ai.timeout_secondi, 45);
    }

    #[test]
    fn timeout_non_validi() {
        for testo in [
            "[ai]\ntimeout_secondi = 0\n",
            "[ai]\ntimeout_secondi = 86401\n",
            "[ai]\ntimeout_connessione_secondi = 0\n",
        ] {
            let e = format!("{:#}", da_file(testo, &[], None).unwrap_err());
            assert!(e.contains("non valido"), "{e}");
        }
        assert!(da_file("", &[("AVVENTURA_TIMEOUT", "abc")], None).is_err());
        assert!(da_file("", &[], Some(0)).is_err());
        assert!(da_file("", &[], Some(86_400)).is_ok());
    }

    #[test]
    fn server_locali() {
        for url in [
            "http://localhost:11434",
            "http://127.0.0.1:1234/v1",
            "http://127.5.6.7/v1",
            "http://[::1]:8080/v1",
            "http://host.docker.internal:11434",
        ] {
            assert!(url_locale(url), "{url}");
        }
        for url in [
            "https://api.openai.com/v1",
            "http://192.168.1.20:11434",
            "nonunurl",
        ] {
            assert!(!url_locale(url), "{url}");
        }
    }

    #[test]
    fn chiave_richiesta_automatica_ed_esplicita() {
        let locale = da_file("[ai]\nbase_url = \"http://localhost:8080/v1\"\n", &[], None).unwrap();
        assert!(!locale.ai.chiave_richiesta_effettiva());
        assert!(locale.ai.verifica_chiave().is_ok());

        let remoto = da_file("", &[], None).unwrap();
        assert!(remoto.ai.chiave_richiesta_effettiva());
        let e = remoto.ai.verifica_chiave().unwrap_err().to_string();
        assert!(e.contains("OPENAI_API_KEY"), "{e}");
        assert!(e.contains("chiave_richiesta = false"), "{e}");
        assert!(e.contains("setx"), "{e}");

        for (f, var) in [
            ("anthropic", "ANTHROPIC_API_KEY"),
            ("gemini", "GEMINI_API_KEY"),
        ] {
            let c = da_file(&format!("[ai]\nfornitore = \"{f}\"\n"), &[], None).unwrap();
            let e = c.ai.verifica_chiave().unwrap_err().to_string();
            assert!(e.contains(var), "{e}");
        }
        for f in ["ollama", "lmstudio", "finto", "locale"] {
            let c = da_file(&format!("[ai]\nfornitore = \"{f}\"\n"), &[], None).unwrap();
            assert!(c.ai.verifica_chiave().is_ok(), "{f}");
        }

        let rete = da_file(
            "[ai]\nfornitore = \"ollama\"\nbase_url = \"http://192.168.1.5:11434\"\nchiave_richiesta = false\n",
            &[],
            None,
        )
        .unwrap();
        assert!(rete.ai.verifica_chiave().is_ok());

        let forzata = da_file(
            "[ai]\nbase_url = \"http://localhost:1234/v1\"\nchiave_richiesta = true\n",
            &[],
            None,
        )
        .unwrap();
        assert!(forzata.ai.verifica_chiave().is_err());
    }

    #[test]
    fn risoluzione_della_chiave() {
        let c = da_file("", &[("OPENAI_API_KEY", "sk-env")], None).unwrap();
        assert_eq!(c.ai.chiave.as_deref(), Some("sk-env"));
        let c = da_file(
            "[ai]\napi_key = \"sk-file\"\n",
            &[
                ("OPENAI_API_KEY", "sk-env"),
                ("AVVENTURA_API_KEY", "sk-forte"),
            ],
            None,
        )
        .unwrap();
        assert_eq!(c.ai.chiave.as_deref(), Some("sk-forte"));
        let c = da_file("[ai]\napi_key = \"sk-file\"\n", &[], None).unwrap();
        assert_eq!(c.ai.chiave.as_deref(), Some("sk-file"));
        assert!(c.ai.verifica_chiave().is_ok());
    }

    #[test]
    fn debug_maschera_i_segreti() {
        let c = da_file(
            "[ai]\napi_key = \"sk-SEGRETISSIMA\"\n[ai.header_extra]\n\"api-key\" = \"az-SEGRETA\"\n\"HTTP-Referer\" = \"https://example.org\"\n\
             [profili.x]\napi_key = \"sk-SEGRETA-PROFILO\"\n[telegram]\ncodice_invito = \"SEGRETO-INVITO\"\n",
            &[],
            None,
        )
        .unwrap();
        let testo = format!("{c:?}");
        assert!(!testo.contains("SEGRET"), "{testo}");
        assert!(testo.contains("https://example.org"));
    }

    #[test]
    fn file_indicato_inesistente() {
        let opzioni = OpzioniConfig {
            file: Some(PathBuf::from("non/esiste/config.toml")),
            ..Default::default()
        };
        assert!(Config::carica_con_ambiente(&opzioni, &|_| None).is_err());
    }

    #[test]
    fn chiavi_sconosciute_rifiutate() {
        let e = format!(
            "{:#}",
            da_file("[ai]\nmodelo = \"x\"\n", &[], None).unwrap_err()
        );
        assert!(e.contains("modelo"), "{e}");
        assert!(da_file("[motore]\nturni = 3\n", &[], None).is_err());
    }

    #[test]
    fn file_di_esempio_valido() {
        let p = Path::new(env!("CARGO_MANIFEST_DIR")).join("../config/config.esempio.toml");
        let mut c = Config::da_toml(&fs::read_to_string(p).unwrap()).unwrap();
        c.ai = c.risolvi_ai(None, None, &|_| None).unwrap();
        c.valida().unwrap();
        assert_eq!(c.ai.timeout_secondi, 900);
        assert_eq!(c.ai.fornitore, Fornitore::Openai);
        assert_eq!(
            c.motore.cartella_avventure,
            Some(PathBuf::from("avventure"))
        );
        for nome in [
            "openai",
            "openrouter",
            "claude",
            "gemini",
            "ollama",
            "ollama-rete",
            "lmstudio",
            "gguf",
            "prova",
        ] {
            let ai = c
                .risolvi_ai(Some(nome), None, &|_| None)
                .unwrap_or_else(|e| panic!("profilo {nome}: {e:#}"));
            ai.valida().unwrap();
        }
        assert_eq!(
            c.risolvi_ai(Some("ollama"), None, &|_| None)
                .unwrap()
                .contesto_token,
            16384
        );
    }

    #[test]
    fn host_e_porta() {
        let c = da_file(
            "[ai]\nbase_url = \"http://localhost:11434/v1\"\n",
            &[],
            None,
        )
        .unwrap();
        assert_eq!(c.ai.host_porta(), "localhost:11434");
    }
}
