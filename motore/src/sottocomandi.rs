//! Esecuzione dei sottocomandi (`gioca`, `elenca`, `salvataggi`, `valida`,
//! `nuova`, `profili`, `prova-ai`, `telegram`) e inizializzazione del log su
//! `partite/avventura.log`.
//!
//! L'output dei sottocomandi non interattivi passa per
//! [`crate::canale::terminale::scrivi`]: nessun modulo fuori dal canale
//! terminale scrive direttamente su stdout.

use std::fs;
use std::net::{TcpStream, ToSocketAddrs};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use anyhow::{Context, Result, bail};
use tracing::info;

use crate::ai::{self, ContestoTurno, Messaggio, RichiestaAi, profili, token};
use crate::avventura::{Avventura, Gravita};
use crate::canale::terminale::{Terminale, prepara_console, scrivi, scrivi_errore};
use crate::catalogo_avventure::{
    cartella_avventure, cartella_partite, riepiloga, risolvi_avventura, scansiona,
};
use crate::cli::{
    ArgAvventura, ArgElenca, ArgGioca, ArgProfili, ArgProvaAi, ArgTelegram, ArgValida, Cli, Comando,
};
use crate::config::{Config, ConfigAi, Fornitore, OpzioniConfig};
use crate::menu::formatta_salvataggi;
use crate::salvataggi::GestoreSalvataggi;
use crate::sessione::{self, Ambiente, OpzioniGioco};
use crate::stato::{PercorsiPartita, Stato, archivia_nello_storico};
use crate::telegram::api::Api as TelegramApi;
use crate::telegram::bot;
use crate::validazione_statica::valida_cartella;

/// Esegue il sottocomando richiesto.
pub fn esegui(cli: Cli) -> Result<ExitCode> {
    match cli.comando {
        None => gioca(ArgGioca::default()),
        Some(Comando::Gioca(a)) => gioca(a),
        Some(Comando::Elenca(a)) => elenca(a),
        Some(Comando::Salvataggi(a)) => salvataggi(a),
        Some(Comando::Valida(a)) => valida(a),
        Some(Comando::Nuova(a)) => nuova(a),
        Some(Comando::ProvaAi(a)) => prova_ai(a),
        Some(Comando::Profili(a)) => elenca_profili(a),
        Some(Comando::Telegram(a)) => telegram(a),
    }
}

fn carica_config(
    file: Option<PathBuf>,
    timeout: Option<u64>,
    profilo: Option<String>,
) -> Result<Config> {
    Config::carica(&OpzioniConfig {
        file,
        timeout,
        profilo,
    })
}

/// Cartelle effettive: avventure, partite e (se indicata) l'avventura scelta.
fn cartelle(
    da_cli: Option<&Path>,
    avventura: Option<&str>,
    config: &Config,
) -> Result<(PathBuf, PathBuf, Option<PathBuf>)> {
    let base = cartella_avventure(da_cli, config);
    let Some(arg) = avventura else {
        let partite = cartella_partite(&base, config);
        return Ok((base, partite, None));
    };
    let cartella = risolvi_avventura(arg, &base)?;
    let effettiva = cartella
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| base.clone());
    let partite = cartella_partite(&effettiva, config);
    Ok((effettiva, partite, Some(cartella)))
}

/// Apre il log in `<cartella_partite>/avventura.log` (livello info, debug con `--debug`).
fn inizializza_log(cartella_partite: &Path, debug: bool) {
    if fs::create_dir_all(cartella_partite).is_err() {
        return;
    }
    let Some(log) = LogRotante::apri(cartella_partite.join("avventura.log"), LOG_MAX_BYTE) else {
        return;
    };
    let log = Arc::new(log);
    let livello = if debug {
        tracing::Level::DEBUG
    } else {
        tracing::Level::INFO
    };
    let _ = tracing_subscriber::fmt()
        .with_writer(move || ScrittoreLog(Arc::clone(&log)))
        .with_ansi(false)
        .with_target(false)
        .with_max_level(livello)
        .try_init();
}

/// Dimensione oltre la quale `avventura.log` diventa `avventura.log.1`.
const LOG_MAX_BYTE: u64 = 5 * 1024 * 1024;

/// File di log a rotazione: superato il limite, il file attuale diventa
/// `<nome>.1` (sostituendo il precedente) e se ne apre uno nuovo. Così il log
/// occupa al massimo circa il doppio del limite, anche con il bot sempre attivo.
pub struct LogRotante {
    percorso: PathBuf,
    massimo: u64,
    stato: Mutex<(Option<fs::File>, u64)>,
}

impl LogRotante {
    /// Apre (o crea) il file di log; `None` se non si può aprire.
    pub fn apri(percorso: PathBuf, massimo: u64) -> Option<Self> {
        let file = Self::apri_file(&percorso).ok()?;
        let dimensione = file.metadata().map(|m| m.len()).unwrap_or(0);
        Some(LogRotante {
            percorso,
            massimo,
            stato: Mutex::new((Some(file), dimensione)),
        })
    }

    fn apri_file(percorso: &Path) -> std::io::Result<fs::File> {
        fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(percorso)
    }

    /// Scrive un evento, ruotando prima il file se il limite verrebbe superato.
    pub fn scrivi(&self, dati: &[u8]) -> std::io::Result<()> {
        use std::io::Write as _;
        let mut stato = self.stato.lock().unwrap_or_else(|e| e.into_inner());
        if stato.1 > 0 && stato.1 + dati.len() as u64 > self.massimo {
            stato.0 = None; // su Windows un file aperto non si può rinominare
            let mut vecchio = self.percorso.clone().into_os_string();
            vecchio.push(".1");
            let _ = fs::rename(&self.percorso, PathBuf::from(vecchio));
            stato.0 = Some(Self::apri_file(&self.percorso)?);
            stato.1 = 0;
        }
        if stato.0.is_none() {
            stato.0 = Some(Self::apri_file(&self.percorso)?);
        }
        if let Some(file) = stato.0.as_mut() {
            file.write_all(dati)?;
        }
        stato.1 += dati.len() as u64;
        Ok(())
    }
}

/// Scrittore passato a `tracing` per ogni evento.
struct ScrittoreLog(Arc<LogRotante>);

impl std::io::Write for ScrittoreLog {
    fn write(&mut self, dati: &[u8]) -> std::io::Result<usize> {
        self.0.scrivi(dati)?;
        Ok(dati.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

fn gioca(arg: ArgGioca) -> Result<ExitCode> {
    let lingua = match arg.lingua.as_deref() {
        None => None,
        Some(l) => match crate::lingua::Lingua::da_codice(l) {
            Some(l) => Some(l),
            None => anyhow::bail!("--lingua accetta «it», «en», «fr», «es» o «de» (trovato «{l}»)"),
        },
    };
    let config = carica_config(arg.config.clone(), arg.timeout, arg.profilo.clone())?;
    config.ai.verifica_chiave()?;
    let (avventure, partite, scelta) = cartelle(
        arg.cartella_avventure.as_deref(),
        arg.avventura.as_deref(),
        &config,
    )?;
    inizializza_log(&partite, arg.debug);
    info!(config = ?config, avventure = %avventure.display(), "avvio di avventura gioca");
    let narratore = ai::crea_narratore(&config.ai)?;
    info!(narratore = %narratore.descrizione(), "narratore pronto");
    let mut canale = Terminale::nuovo(&config.interfaccia, arg.trascrizione.as_deref())?;
    let ambiente = Ambiente {
        config: &config,
        cartella_avventure: avventure,
        cartella_partite: partite,
    };
    let opzioni = OpzioniGioco {
        avventura: scelta.map(|p| p.to_string_lossy().to_string()),
        carica: arg.carica,
        nuova: arg.nuova,
        debug: arg.debug,
        lingua,
        lingua_rilevata: crate::lingua::rileva_sistema(),
    };
    sessione::gioca(&ambiente, &opzioni, &mut canale, narratore.as_ref())?;
    Ok(ExitCode::SUCCESS)
}

fn elenca(arg: ArgElenca) -> Result<ExitCode> {
    prepara_console();
    let config = carica_config(arg.config, None, None)?;
    let (avventure, partite, _) = cartelle(arg.cartella_avventure.as_deref(), None, &config)?;
    let voci = scansiona(&avventure, &partite);
    if voci.is_empty() {
        scrivi(&format!(
            "Nessuna avventura trovata in «{}». Ogni avventura è una sottocartella con un file \
             gioco.md (vedi docs/MANUALE.md).",
            avventure.display()
        ));
        return Ok(ExitCode::SUCCESS);
    }
    scrivi(&format!("Avventure in «{}»:\n", avventure.display()));
    for v in &voci {
        match &v.esito {
            Ok(r) => {
                scrivi(&format!("  {} — {}  {}", v.id, r.titolo, r.etichetta()));
                if !r.descrizione_breve.is_empty() {
                    scrivi(&format!("     {}", r.descrizione_breve));
                }
                let partita = match &r.partita {
                    Some(p) => format!(
                        "partita: turno {}, {}, {}",
                        p.turno,
                        p.nome_stanza,
                        p.aggiornato_il.format("%d/%m %H:%M")
                    ),
                    None => "nessuna partita in corso".to_string(),
                };
                scrivi(&format!(
                    "     {} stanze · {partita} · {} salvataggi",
                    r.numero_stanze, r.numero_salvataggi
                ));
            }
            Err(motivo) => scrivi(&format!("  {} — NON VALIDA: {motivo}", v.id)),
        }
    }
    Ok(ExitCode::SUCCESS)
}

fn salvataggi(arg: ArgAvventura) -> Result<ExitCode> {
    prepara_console();
    let config = carica_config(arg.config, None, None)?;
    let (_, partite, cartella) = cartelle(
        arg.cartella_avventure.as_deref(),
        Some(&arg.avventura),
        &config,
    )?;
    let cartella = cartella.context("avventura non indicata")?;
    let voce = riepiloga(&cartella, &partite);
    let titolo = voce
        .esito
        .as_ref()
        .map(|r| r.titolo.clone())
        .unwrap_or_else(|_| voce.id.clone());
    let gestore = GestoreSalvataggi::nuovo(PercorsiPartita::nuovi(&partite, &voce.id), &voce.id);
    let elenco = gestore.elenca();
    if elenco.is_empty() {
        scrivi(&format!("Nessun salvataggio per «{titolo}»."));
    } else {
        let corrente = Stato::carica(&gestore.percorsi().stato())
            .ok()
            .and_then(|s| s.salvataggio_corrente);
        scrivi(&formatta_salvataggi(&titolo, &elenco, corrente.as_deref()));
    }
    Ok(ExitCode::SUCCESS)
}

fn valida(arg: ArgValida) -> Result<ExitCode> {
    prepara_console();
    // La validazione funziona anche con una configurazione assente o errata.
    let config = carica_config(arg.config.clone(), None, None).unwrap_or_else(|e| {
        scrivi_errore(&format!(
            "Avviso: configurazione non valida ({e:#}); uso i valori predefiniti."
        ));
        Config::default()
    });
    let (avventure, _, scelta) = cartelle(
        arg.cartella_avventure.as_deref(),
        arg.avventura.as_deref(),
        &config,
    )?;
    let cartelle_da_validare: Vec<PathBuf> = match scelta {
        Some(c) => vec![c],
        None => {
            let mut c: Vec<PathBuf> = fs::read_dir(&avventure)
                .with_context(|| format!("impossibile leggere «{}»", avventure.display()))?
                .filter_map(|v| v.ok().map(|v| v.path()))
                .filter(|p| p.join("gioco.md").is_file())
                .collect();
            c.sort();
            c
        }
    };
    if cartelle_da_validare.is_empty() {
        bail!("nessuna avventura trovata in «{}»", avventure.display());
    }
    let mut errori = false;
    for cartella in cartelle_da_validare {
        let rapporto = valida_cartella(&cartella);
        scrivi(&format!("Avventura «{}»:", cartella.display()));
        scrivi(&rapporto.formatta());
        if let Some(a) = &rapporto.avventura
            && let Some(stima) = token::stima_avventura(a, &config)
        {
            scrivi(&format!(
                "  Contesto consigliato per questa avventura: {} token (prompt più grande ≈ {} \
                 token, stanza «{}»). Serve con i modelli locali (contesto_token).",
                token::migliaia(stima.consigliato),
                token::migliaia(stima.token_messaggi),
                stima.stanza
            ));
        }
        scrivi("");
        errori |= rapporto.numero(Gravita::Errore) > 0;
    }
    Ok(if errori {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    })
}

fn nuova(arg: ArgAvventura) -> Result<ExitCode> {
    prepara_console();
    let config = carica_config(arg.config, None, None)?;
    let (_, partite, cartella) = cartelle(
        arg.cartella_avventure.as_deref(),
        Some(&arg.avventura),
        &config,
    )?;
    let cartella = cartella.context("avventura non indicata")?;
    let avventura = Avventura::carica(&cartella)?;
    let percorsi = PercorsiPartita::nuovi(&partite, &avventura.id);
    if !percorsi.stato().exists() {
        scrivi(&format!(
            "Nessuna partita in corso per «{}».",
            avventura.gioco.titolo
        ));
        return Ok(ExitCode::SUCCESS);
    }
    archivia_nello_storico(&percorsi)?;
    fs::remove_file(percorsi.stato())
        .with_context(|| format!("impossibile eliminare «{}»", percorsi.stato().display()))?;
    scrivi(&format!(
        "Partita in corso di «{}» azzerata. I salvataggi con nome restano disponibili.",
        avventura.gioco.titolo
    ));
    Ok(ExitCode::SUCCESS)
}

fn elenca_profili(arg: ArgProfili) -> Result<ExitCode> {
    prepara_console();
    let config = carica_config(arg.config, None, arg.profilo.clone())?;
    scrivi(&format!(
        "Configurazione: {}\n",
        match &config.file {
            Some(f) => f.display().to_string(),
            None => "valori predefiniti (nessun config.toml trovato)".to_string(),
        }
    ));
    let attivo = config.ai.profilo.clone();
    let ambiente = |k: &str| std::env::var(k).ok();
    for p in profili::elenca(&config.sorgente, attivo.as_deref(), &ambiente) {
        let segno = if p.attivo { "*" } else { " " };
        let nome = p
            .nome
            .clone()
            .unwrap_or_else(|| "[ai] (predefinito)".to_string());
        match &p.descrizione {
            Ok(d) => scrivi(&format!(" {segno} {nome:<20} {d}")),
            Err(e) => scrivi(&format!(" {segno} {nome:<20} NON VALIDO: {e}")),
        }
    }
    scrivi(&format!(
        "\n* = attivo{}. Scegli un profilo con --profilo NOME o la variabile AVVENTURA_PROFILO.",
        if attivo.is_none() {
            " (sezione [ai])"
        } else {
            ""
        }
    ));
    Ok(ExitCode::SUCCESS)
}

/// `avventura telegram`: avvia il bot (§7.5) o, con `--verifica`, controlla
/// token, AI e cartelle ed esce.
fn telegram(arg: ArgTelegram) -> Result<ExitCode> {
    prepara_console();
    let ambiente = |k: &str| std::env::var(k).ok();
    let mut config = carica_config(arg.config.clone(), arg.timeout, arg.profilo.clone())?;
    // Profilo del bot (`telegram.profilo`) se non ne è stato scelto uno da CLI o ambiente.
    if arg.profilo.is_none()
        && ambiente("AVVENTURA_PROFILO").is_none()
        && let Some(p) = config.telegram.profilo.clone()
    {
        config.ai = config
            .risolvi_ai(Some(&p), arg.timeout, &ambiente)
            .with_context(|| format!("profilo «{p}» indicato in [telegram] non valido"))?;
        config.valida()?;
    }
    // Il token si controlla per primo: senza, il bot non può fare nulla.
    let token = bot::leggi_token(&config, &ambiente)?;
    let avventure = cartella_avventure(None, &config);
    let partite = cartella_partite(&avventure, &config);
    let partite_tg = bot::cartella_partite_telegram(&avventure, &config);
    inizializza_log(&partite, arg.debug);
    info!(config = ?config, avventure = %avventure.display(), "avvio di avventura telegram");
    // AVVENTURA_TELEGRAM_API (non documentata all'utente) sposta l'API su un server di prova:
    // serve solo al collaudo con un finto server Telegram.
    let api = match ambiente("AVVENTURA_TELEGRAM_API").filter(|u| !u.trim().is_empty()) {
        Some(url) => TelegramApi::con_base_url(&token, url.trim()),
        None => TelegramApi::nuovo(&token),
    }
    .map_err(|e| anyhow::anyhow!("{e}"))?;

    if arg.verifica {
        return verifica_telegram(&config, &api, &avventure, &partite_tg);
    }

    config.ai.verifica_chiave()?;
    let narratore: Arc<dyn ai::Narratore> = Arc::from(ai::crea_narratore(&config.ai)?);
    info!(narratore = %narratore.descrizione(), "narratore pronto");

    let ferma = Arc::new(AtomicBool::new(false));
    {
        let ferma = Arc::clone(&ferma);
        let _ = ctrlc::set_handler(move || {
            if ferma.swap(true, Ordering::SeqCst) {
                // Secondo Ctrl+C: uscita immediata.
                std::process::exit(130);
            }
            scrivi(
                "\nArresto richiesto: chiusura ordinata in corso (di nuovo Ctrl+C per forzare)…",
            );
        });
    }
    let opzioni = bot::OpzioniBot::da_config(&config, ferma);
    bot::esegui(
        bot::DatiBot {
            config,
            cartella_avventure: avventure,
            cartella_partite: partite_tg,
            narratore,
            api,
        },
        opzioni,
    )?;
    Ok(ExitCode::SUCCESS)
}

/// `avventura telegram --verifica`: token (`getMe`), AI (senza inviare prompt) e cartelle.
fn verifica_telegram(
    config: &Config,
    api: &TelegramApi,
    avventure: &Path,
    partite_tg: &Path,
) -> Result<ExitCode> {
    let mut ok = true;
    scrivi(&format!(
        "Token:          presente (variabile {})",
        config.telegram.token_env
    ));
    match api.get_me() {
        Ok(io) => scrivi(&format!(
            "Bot:            @{} (id {}) — token valido",
            io.username.as_deref().unwrap_or(&io.first_name),
            io.id
        )),
        Err(e) => {
            ok = false;
            scrivi(&format!("Bot:            ERRORE: {e}"));
        }
    }
    scrivi("");
    riepilogo_ai(config);
    let ai_cfg = &config.ai;
    let esito_ai = match ai_cfg.verifica_chiave() {
        Err(e) => Err(format!("{e:#}")),
        Ok(()) => match ai_cfg.fornitore {
            Fornitore::Finto => Ok("AI simulata: nessun server da contattare".to_string()),
            Fornitore::Locale => ai::crea_narratore(ai_cfg)
                .map(|_| "modello locale caricato".to_string())
                .map_err(|e| format!("{e:#}")),
            _ => raggiungibile(ai_cfg)
                .map(|()| format!("il server {} accetta connessioni", ai_cfg.host_porta()))
                .map_err(|e| format!("il server {} non è raggiungibile: {e}", ai_cfg.host_porta())),
        },
    };
    match esito_ai {
        Ok(t) => scrivi(&format!("AI:             {t} (nessun prompt inviato)")),
        Err(e) => {
            ok = false;
            scrivi(&format!("AI:             ERRORE: {e}"));
        }
    }
    scrivi("");
    let voci = scansiona(avventure, partite_tg);
    let valide = voci.iter().filter(|v| v.valida()).count();
    if valide == 0 {
        ok = false;
        scrivi(&format!(
            "Avventure:      ERRORE: nessuna avventura valida in «{}»",
            avventure.display()
        ));
    } else {
        scrivi(&format!(
            "Avventure:      {valide} valide in «{}»",
            avventure.display()
        ));
    }
    match fs::create_dir_all(partite_tg) {
        Ok(()) => scrivi(&format!("Partite:        «{}»", partite_tg.display())),
        Err(e) => {
            ok = false;
            scrivi(&format!(
                "Partite:        ERRORE: impossibile creare «{}»: {e}",
                partite_tg.display()
            ));
        }
    }
    match bot::Accesso::carica(config, partite_tg.join("utenti.json")) {
        Ok(a) => {
            let n = a.elenco().len();
            scrivi(&format!(
                "Utenti ammessi: {n} (amministratori: {}){}",
                config.telegram.amministratori.len(),
                if n == 0 {
                    " — nessuno può giocare: aggiungi il tuo ID a telegram.utenti_ammessi"
                } else {
                    ""
                }
            ));
        }
        Err(e) => {
            ok = false;
            scrivi(&format!("Utenti ammessi: ERRORE: {e:#}"));
        }
    }
    scrivi(if ok {
        "\nTutto pronto: avvia il bot con `avventura telegram`."
    } else {
        "\nCi sono problemi da risolvere (vedi sopra)."
    });
    Ok(if ok {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    })
}

fn si_no(v: bool) -> &'static str {
    if v { "sì" } else { "no" }
}

fn riepilogo_ai(config: &Config) {
    let ai_cfg = &config.ai;
    scrivi(&format!(
        "Configurazione: {}",
        match &config.file {
            Some(f) => f.display().to_string(),
            None => "valori predefiniti (nessun config.toml trovato)".to_string(),
        }
    ));
    scrivi(&format!(
        "Profilo:        {}",
        ai_cfg.profilo.as_deref().unwrap_or("[ai] (predefinito)")
    ));
    scrivi(&format!("Fornitore:      {}", ai_cfg.fornitore));
    if ai_cfg.fornitore.usa_http() {
        scrivi(&format!("Endpoint:       {}", ai_cfg.base_url));
        scrivi(&format!("Modello:        {}", ai_cfg.modello));
    }
    if ai_cfg.fornitore == Fornitore::Locale {
        scrivi(&format!(
            "File GGUF:      {}",
            ai_cfg
                .locale
                .percorso_modello
                .as_ref()
                .map_or_else(|| "non indicato".to_string(), |p| p.display().to_string())
        ));
    }
    scrivi(&format!(
        "Server locale:  {}",
        si_no(ai_cfg.server_locale())
    ));
    scrivi(&format!(
        "Chiave API:     {}",
        if ai_cfg.chiave.is_some() {
            "usata (valore non mostrato)".to_string()
        } else if ai_cfg.chiave_richiesta_effettiva() {
            format!("MANCANTE (variabile {})", ai_cfg.api_key_env)
        } else {
            "non usata (non richiesta)".to_string()
        }
    ));
    scrivi(&format!("Ragionamento:   {}", ai_cfg.ragionamento));
    scrivi(&format!(
        "Timeout:        risposta {} s, connessione {} s",
        ai_cfg.timeout_secondi, ai_cfg.timeout_connessione_secondi
    ));
    scrivi(&format!(
        "Contesto:       {}",
        if ai_cfg.contesto_token == 0 {
            "non configurato (0 = non controllato)".to_string()
        } else {
            format!(
                "{} token",
                token::migliaia(u64::from(ai_cfg.contesto_token))
            )
        }
    ));
    for a in &ai_cfg.avvisi {
        scrivi(&format!("Avviso: {a}"));
    }
}

/// Verifica che il server accetti connessioni (nessun prompt inviato).
fn raggiungibile(ai_cfg: &ConfigAi) -> std::result::Result<(), String> {
    let url = reqwest::Url::parse(&ai_cfg.base_url).map_err(|e| e.to_string())?;
    let host = url.host_str().ok_or("URL senza host")?.to_string();
    let porta = url.port_or_known_default().ok_or("porta sconosciuta")?;
    let indirizzi: Vec<_> = (host.trim_matches(['[', ']']), porta)
        .to_socket_addrs()
        .map_err(|e| format!("nome «{host}» non risolto: {e}"))?
        .collect();
    let attesa = Duration::from_secs(ai_cfg.timeout_connessione_secondi);
    let mut ultimo = String::from("nessun indirizzo");
    for indirizzo in indirizzi {
        match TcpStream::connect_timeout(&indirizzo, attesa) {
            Ok(_) => return Ok(()),
            Err(e) => ultimo = e.to_string(),
        }
    }
    Err(ultimo)
}

fn prova_ai(arg: ArgProvaAi) -> Result<ExitCode> {
    prepara_console();
    let config = carica_config(arg.config.clone(), arg.timeout, arg.profilo.clone())?;
    let ai_cfg = &config.ai;
    let (_, partite, scelta) = cartelle(
        arg.cartella_avventure.as_deref(),
        arg.avventura.as_deref(),
        &config,
    )?;
    inizializza_log(&partite, false);
    riepilogo_ai(&config);
    ai_cfg.verifica_chiave()?;

    if arg.solo_config {
        match ai_cfg.fornitore {
            Fornitore::Finto => scrivi("\nAI simulata: nessun server da contattare. Tutto pronto."),
            Fornitore::Locale => {
                let narratore = match ai::crea_narratore(ai_cfg) {
                    Ok(n) => n,
                    Err(e) => {
                        scrivi(&format!("\nERRORE: {e:#}"));
                        return Ok(ExitCode::FAILURE);
                    }
                };
                scrivi("\nModello locale caricato correttamente. Nessun prompt inviato.");
                for riga in narratore.diagnostica() {
                    scrivi(&riga);
                }
            }
            _ => match raggiungibile(ai_cfg) {
                Ok(()) => scrivi(&format!(
                    "\nIl server {} accetta connessioni. Nessun prompt inviato.",
                    ai_cfg.host_porta()
                )),
                Err(e) => {
                    scrivi(&format!(
                        "\nERRORE: il server {} non è raggiungibile: {e}",
                        ai_cfg.host_porta()
                    ));
                    if ai_cfg.server_locale() {
                        scrivi(
                            "È avviato? (Ollama: `ollama serve`; LM Studio: scheda Developer → Start Server)",
                        );
                    }
                    return Ok(ExitCode::FAILURE);
                }
            },
        }
        return Ok(ExitCode::SUCCESS);
    }

    let narratore = ai::crea_narratore(ai_cfg)?;
    let richiesta = RichiestaAi {
        messaggi: vec![
            Messaggio::sistema(
                "Sei un servizio di prova. Rispondi solo con questo oggetto JSON: \
                 {\"narrazione\": \"pronto\"}",
            ),
            Messaggio::utente("ping"),
        ],
        contesto: ContestoTurno {
            nome_stanza: "prova".into(),
            azione: "guarda".into(),
            ..Default::default()
        },
    };
    scrivi("\nRichiesta minima in corso…");
    let inizio = Instant::now();
    let risposta = match narratore.turno(&richiesta) {
        Ok(r) => r,
        Err(e) => {
            scrivi(&format!("ERRORE: {e}"));
            return Ok(ExitCode::FAILURE);
        }
    };
    let durata = inizio.elapsed();
    for a in &risposta.avvisi {
        scrivi(&format!("Avviso: {a}"));
    }
    let json_valido = crate::risposta::analizza(&risposta.testo).is_ok();
    scrivi(&format!("Tempo di risposta: {:.1} s", durata.as_secs_f64()));
    scrivi(&format!("Risposta JSON valida: {}", si_no(json_valido)));
    if let Some(j) = narratore.json_mode_attivo() {
        scrivi(&format!(
            "JSON garantito (json_mode): {}",
            if j {
                "attivo e accettato dal server"
            } else {
                "disattivato"
            }
        ));
    }
    scrivi(&format!(
        "Token di prompt dichiarati dal server: {}",
        risposta
            .token_prompt
            .map_or_else(|| "non dichiarati".to_string(), |t| t.to_string())
    ));
    for riga in narratore.diagnostica() {
        scrivi(&riga);
    }
    suggerimento_tempo(durata.as_secs_f64(), ai_cfg.timeout_secondi);

    if let Some(cartella) = scelta {
        let avventura = Avventura::carica(&cartella)?;
        let Some(stima) = token::stima_avventura(&avventura, &config) else {
            bail!("l'avventura non ha stanze");
        };
        scrivi(&format!(
            "\nPrompt reale più grande di «{}»: stanza «{}», ≈ {} token stimati. Invio in corso…",
            avventura.gioco.titolo,
            stima.stanza,
            token::migliaia(stima.token_messaggi)
        ));
        let richiesta = RichiestaAi {
            messaggi: stima.messaggi.clone(),
            contesto: richiesta.contesto.clone(),
        };
        let inizio = Instant::now();
        match narratore.turno(&richiesta) {
            Ok(r) => {
                let durata = inizio.elapsed();
                scrivi(&format!("Tempo di risposta: {:.1} s", durata.as_secs_f64()));
                match r.token_prompt {
                    Some(d) => {
                        scrivi(&format!(
                            "Token di prompt dichiarati: {}",
                            token::migliaia(d)
                        ));
                        if token::troncamento_probabile(d, stima.token_messaggi) {
                            scrivi(&token::avviso_troncamento(d, stima.token_messaggi));
                        } else {
                            scrivi("Nessun segno di troncamento del contesto.");
                        }
                    }
                    None => scrivi("Il server non dichiara i token di prompt."),
                }
                suggerimento_tempo(durata.as_secs_f64(), ai_cfg.timeout_secondi);
            }
            Err(e) => scrivi(&format!("ERRORE con il prompt reale: {e}")),
        }
        scrivi(&format!(
            "Contesto consigliato: contesto_token = {} (stima + max_token + 25%).",
            stima.consigliato
        ));
        if ai_cfg.contesto_token > 0 && u64::from(ai_cfg.contesto_token) < stima.consigliato {
            scrivi(&format!(
                "Il valore attuale ({}) è più basso del consigliato.",
                ai_cfg.contesto_token
            ));
        }
    }
    Ok(ExitCode::SUCCESS)
}

fn suggerimento_tempo(secondi: f64, timeout: u64) {
    if secondi > timeout as f64 * 0.5 {
        scrivi(&format!(
            "Attenzione: la risposta ha richiesto più della metà del timeout ({timeout} s). \
             Aumenta timeout_secondi (o --timeout) oppure usa un modello più piccolo."
        ));
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn log_a_rotazione() {
        let dir = tempfile::tempdir().unwrap();
        let percorso = dir.path().join("avventura.log");
        let log = LogRotante::apri(percorso.clone(), 100).unwrap();
        for _ in 0..3 {
            log.scrivi(&[b'a'; 40]).unwrap();
        }
        // 120 byte > 100: i primi 80 sono passati in avventura.log.1.
        assert_eq!(
            fs::metadata(dir.path().join("avventura.log.1"))
                .unwrap()
                .len(),
            80
        );
        assert_eq!(fs::metadata(&percorso).unwrap().len(), 40);
        drop(log);
        // Alla riapertura riparte dalla dimensione attuale.
        let log = LogRotante::apri(percorso.clone(), 100).unwrap();
        log.scrivi(&[b'b'; 70]).unwrap();
        assert_eq!(
            fs::metadata(dir.path().join("avventura.log.1"))
                .unwrap()
                .len(),
            40
        );
        assert_eq!(fs::metadata(&percorso).unwrap().len(), 70);
    }
}
