//! Fornitore `locale`: modello GGUF letto dal disco ed eseguito nel programma
//! con llama.cpp (§5.5), senza Ollama, LM Studio o altri server.
//!
//! Questo file contiene la parte comune, compilata sempre:
//! - i controlli di configurazione e del file GGUF;
//! - le funzioni pure usate dall'implementazione (riduzione del prompt quando
//!   non entra nel contesto, chiusura del ragionamento nei template che lo
//!   prevedono, riconoscimento della fine del JSON, grammatica JSON generica);
//! - lo stub per gli eseguibili compilati **senza** la feature `locale`, che
//!   spiega come ottenere la versione con llama.cpp integrato.
//!
//! L'implementazione vera (crate `llama-cpp-2`) è nel sottomodulo `llama`,
//! compilato solo con la feature Cargo `locale` (o `cuda`/`vulkan`).

use std::fs::File;
use std::io::Read;
use std::path::Path;

use anyhow::Result;
use serde_json::Value;

use super::ragionamento::Ragionamento;
use super::{ErroreAi, Messaggio, Narratore, Ruolo};
use crate::config::ConfigAi;

#[cfg(feature = "locale")]
mod llama;

/// Contesto usato quando `contesto_token = 0` (§5.2).
pub const CONTESTO_PREDEFINITO: u32 = 8192;

/// Messaggio per un eseguibile compilato senza la feature `locale`.
pub const SENZA_FEATURE: &str = "il fornitore «locale» (modello GGUF dal disco) non è incluso in \
questo eseguibile. Usa la versione con llama.cpp integrato (avventura-cuda.exe per le GPU \
NVIDIA), oppure ricompila con `cargo build --release --features locale` (solo CPU) o \
`--features cuda` (servono CMake, LLVM/Clang e, per cuda, il CUDA Toolkit: vedi il manuale). \
In alternativa usa Ollama o LM Studio (fornitore = \"ollama\" o \"lmstudio\").";

/// Grammatica GBNF generica che ammette solo un oggetto JSON (con spazi e a capo
/// limitati). Usata se la conversione dello schema della risposta non riesce.
pub const GRAMMATICA_JSON: &str = r#"root   ::= object
value  ::= object | array | string | number | ("true" | "false" | "null") ws
object ::= "{" ws ( string ":" ws value ("," ws string ":" ws value)* )? "}" ws
array  ::= "[" ws ( value ("," ws value)* )? "]" ws
string ::= "\"" ( [^"\\\x7F\x00-\x1F] | "\\" (["\\/bfnrt] | "u" [0-9a-fA-F]{4}) )* "\"" ws
number ::= ("-"? ([0-9] | [1-9] [0-9]{0,15})) ("." [0-9]+)? ([eE] [-+]? [0-9] [1-9]{0,15})? ws
ws     ::= | " " | "\n" [ \t]{0,20}
"#;

/// Controlli di configurazione comuni (con o senza la feature).
pub fn verifica_configurazione(config: &ConfigAi) -> Result<(), ErroreAi> {
    let Some(percorso) = &config.locale.percorso_modello else {
        return Err(ErroreAi::Configurazione(
            "il fornitore «locale» richiede il file del modello: imposta \
             percorso_modello nella sezione [ai.locale] (o [profili.NOME.locale]) di \
             config.toml, oppure la variabile AVVENTURA_MODELLO_GGUF"
                .to_string(),
        ));
    };
    let estensione_gguf = percorso
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("gguf"));
    if !estensione_gguf {
        return Err(ErroreAi::Configurazione(format!(
            "il file del modello «{}» non ha estensione .gguf",
            percorso.display()
        )));
    }
    if config.locale.gpu_layers < -1 {
        return Err(ErroreAi::Configurazione(format!(
            "gpu_layers = {} non è valido: usa -1 (tutti gli strati sulla GPU), 0 (solo CPU) \
             o un numero positivo",
            config.locale.gpu_layers
        )));
    }
    let contesto = contesto_effettivo(config.contesto_token).0;
    if config.max_token >= contesto {
        return Err(ErroreAi::Configurazione(format!(
            "max_token ({}) deve essere minore del contesto ({contesto} token): \
             aumenta contesto_token",
            config.max_token
        )));
    }
    Ok(())
}

/// Controlla che il file esista e sia un GGUF; restituisce la dimensione in byte.
pub fn verifica_file(percorso: &Path) -> Result<u64, ErroreAi> {
    let metadati = std::fs::metadata(percorso).map_err(|_| {
        ErroreAi::Configurazione(format!(
            "il file del modello «{}» non esiste. Controlla percorso_modello in [ai.locale] \
             (un percorso relativo è riferito alla cartella del file di configurazione) o la \
             variabile AVVENTURA_MODELLO_GGUF",
            percorso.display()
        ))
    })?;
    if !metadati.is_file() {
        return Err(ErroreAi::Configurazione(format!(
            "«{}» non è un file",
            percorso.display()
        )));
    }
    let mut firma = [0u8; 4];
    let letto = File::open(percorso).and_then(|mut f| f.read_exact(&mut firma));
    if letto.is_err() || &firma != b"GGUF" {
        return Err(ErroreAi::Configurazione(format!(
            "«{}» non è un file GGUF valido (firma iniziale assente): scarica di nuovo il \
             modello in formato GGUF",
            percorso.display()
        )));
    }
    Ok(metadati.len())
}

/// Contesto da usare e avviso se `contesto_token` non è impostato.
pub fn contesto_effettivo(contesto_token: u32) -> (u32, Option<String>) {
    if contesto_token == 0 {
        (
            CONTESTO_PREDEFINITO,
            Some(format!(
                "contesto_token non è impostato: il modello locale usa un contesto di {} token. \
                 Per le avventure grandi imposta contesto_token (es. 16384): `avventura \
                 prova-ai --avventura ID` consiglia il valore adatto.",
                super::token::migliaia(u64::from(CONTESTO_PREDEFINITO))
            )),
        )
    } else {
        (contesto_token, None)
    }
}

/// Esito della riduzione del prompt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EsitoRiduzione {
    /// Il prompt entra nel limite (eventualmente dopo la riduzione).
    Entra {
        /// Messaggi da inviare.
        messaggi: Vec<Messaggio>,
        /// Token del prompt.
        token: usize,
        /// Turni recenti tolti (i più vecchi).
        turni_tolti: usize,
        /// Vero se la memoria è stata accorciata.
        memoria_troncata: bool,
    },
    /// Il prompt non entra nemmeno senza turni recenti e memoria.
    NonEntra {
        /// Token del prompt più piccolo ottenuto.
        token: usize,
    },
}

/// Riduce il prompt finché entra in `limite` token (§5.2 punto 3): toglie prima i
/// turni di `ultimi_turni` (dal più vecchio), poi accorcia `memoria` tenendone la
/// parte finale (la più recente). `conta` restituisce i token di un elenco di
/// messaggi (con il tokenizzatore del modello; nei test una funzione qualsiasi).
///
/// Lo stato si trova nel primo messaggio utente in forma JSON
/// `{"stato": {...}, "azione_giocatore": "..."}` (vedi `prompt::messaggio_utente`).
pub fn riduci_prompt<E>(
    messaggi: &[Messaggio],
    limite: usize,
    mut conta: impl FnMut(&[Messaggio]) -> Result<usize, E>,
) -> Result<EsitoRiduzione, E> {
    let mut token = conta(messaggi)?;
    if token <= limite {
        return Ok(EsitoRiduzione::Entra {
            messaggi: messaggi.to_vec(),
            token,
            turni_tolti: 0,
            memoria_troncata: false,
        });
    }
    let Some((indice, mut valore)) = messaggi.iter().enumerate().find_map(|(i, m)| {
        if m.ruolo != Ruolo::User {
            return None;
        }
        let v: Value = serde_json::from_str(&m.contenuto).ok()?;
        v.get("stato")?.is_object().then_some((i, v))
    }) else {
        return Ok(EsitoRiduzione::NonEntra { token });
    };

    let mut lavoro = messaggi.to_vec();
    let mut turni_tolti = 0;
    let mut memoria_troncata = false;
    let mut ricostruisci = |valore: &Value, lavoro: &mut Vec<Messaggio>| -> Result<usize, E> {
        lavoro[indice].contenuto = serde_json::to_string_pretty(valore).unwrap_or_default();
        conta(lavoro)
    };

    // 1. Turni recenti, dal più vecchio.
    while let Some(turni) = valore["stato"]["ultimi_turni"]
        .as_array_mut()
        .filter(|t| !t.is_empty())
    {
        turni.remove(0);
        turni_tolti += 1;
        token = ricostruisci(&valore, &mut lavoro)?;
        if token <= limite {
            return Ok(EsitoRiduzione::Entra {
                messaggi: lavoro,
                token,
                turni_tolti,
                memoria_troncata,
            });
        }
    }

    // 2. Memoria: si tengono 3/4, 1/2, 1/4 e infine nulla della parte finale.
    let memoria: Vec<char> = valore["stato"]["memoria"]
        .as_str()
        .unwrap_or_default()
        .chars()
        .collect();
    if !memoria.is_empty() {
        for quarti in [3usize, 2, 1, 0] {
            let tenuti = memoria.len() * quarti / 4;
            let coda: String = memoria[memoria.len() - tenuti..].iter().collect();
            valore["stato"]["memoria"] = Value::String(if tenuti == 0 {
                String::new()
            } else {
                format!("…{coda}")
            });
            memoria_troncata = true;
            token = ricostruisci(&valore, &mut lavoro)?;
            if token <= limite {
                return Ok(EsitoRiduzione::Entra {
                    messaggi: lavoro,
                    token,
                    turni_tolti,
                    memoria_troncata,
                });
            }
        }
    }
    Ok(EsitoRiduzione::NonEntra { token })
}

/// Avviso mostrato quando il prompt è stato ridotto.
pub fn avviso_riduzione(turni_tolti: usize, memoria_troncata: bool, contesto: u32) -> String {
    let mut parti = Vec::new();
    if turni_tolti > 0 {
        parti.push(format!("tolti {turni_tolti} turni recenti"));
    }
    if memoria_troncata {
        parti.push("accorciata la memoria".to_string());
    }
    format!(
        "Il prompt non entrava nel contesto del modello locale ({} token): {}. Aumenta \
         contesto_token per dare al narratore tutto il contesto.",
        super::token::migliaia(u64::from(contesto)),
        parti.join(", ")
    )
}

/// Testo da aggiungere al prompt per chiudere subito il ragionamento, se il
/// template del modello prevede l'interruttore `enable_thinking` (es. Qwen3) e
/// `ragionamento = "spento"`: è ciò che il template stesso produce con
/// `enable_thinking = false`.
pub fn chiusura_ragionamento(
    template: &str,
    prompt: &str,
    ragionamento: Ragionamento,
) -> Option<&'static str> {
    let previsto = template.contains("enable_thinking");
    let gia_chiuso = prompt.trim_end().ends_with("</think>");
    (ragionamento == Ragionamento::Spento && previsto && !gia_chiuso)
        .then_some("<think>\n\n</think>\n\n")
}

/// Vero se il testo contiene un oggetto JSON di primo livello già chiuso
/// (graffe bilanciate fuori dalle stringhe). Serve a fermare la generazione
/// appena il JSON è completo.
pub fn json_completo(testo: &str) -> bool {
    let mut profondita = 0usize;
    let mut iniziato = false;
    let mut in_stringa = false;
    let mut escape = false;
    for c in testo.chars() {
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
            '{' | '[' => {
                profondita += 1;
                iniziato = true;
            }
            '}' | ']' => {
                profondita = profondita.saturating_sub(1);
                if iniziato && profondita == 0 {
                    return true;
                }
            }
            _ => {}
        }
    }
    false
}

/// Crea il narratore `locale`.
pub fn crea(config: &ConfigAi) -> Result<Box<dyn Narratore>> {
    verifica_configurazione(config)?;
    #[cfg(feature = "locale")]
    {
        Ok(Box::new(llama::NarratoreLocale::nuovo(config)?))
    }
    #[cfg(not(feature = "locale"))]
    {
        Err(ErroreAi::Configurazione(SENZA_FEATURE.to_string()).into())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::config::Fornitore;
    use std::convert::Infallible;
    use std::path::PathBuf;

    fn config_locale(percorso: Option<&str>) -> ConfigAi {
        let mut c = ConfigAi {
            fornitore: Fornitore::Locale,
            ..ConfigAi::default()
        };
        c.locale.percorso_modello = percorso.map(PathBuf::from);
        c
    }

    #[test]
    fn controlli_di_configurazione() {
        let e = verifica_configurazione(&config_locale(None))
            .unwrap_err()
            .to_string();
        assert!(e.contains("percorso_modello"), "{e}");
        assert!(e.contains("AVVENTURA_MODELLO_GGUF"), "{e}");
        let e = verifica_configurazione(&config_locale(Some("modelli/x.bin")))
            .unwrap_err()
            .to_string();
        assert!(e.contains(".gguf"), "{e}");
        assert!(verifica_configurazione(&config_locale(Some("modelli/x.GGUF"))).is_ok());

        let mut c = config_locale(Some("m.gguf"));
        c.locale.gpu_layers = -2;
        let e = verifica_configurazione(&c).unwrap_err().to_string();
        assert!(e.contains("gpu_layers"), "{e}");

        let mut c = config_locale(Some("m.gguf"));
        c.max_token = 9000;
        let e = verifica_configurazione(&c).unwrap_err().to_string();
        assert!(e.contains("max_token") && e.contains("8192"), "{e}");
        c.contesto_token = 16384;
        assert!(verifica_configurazione(&c).is_ok());
    }

    #[cfg(not(feature = "locale"))]
    #[test]
    fn stub_senza_feature() {
        let e = crea(&config_locale(None)).err().unwrap().to_string();
        assert!(e.contains("percorso_modello"), "{e}");
        let e = crea(&config_locale(Some("modelli/x.gguf")))
            .err()
            .unwrap()
            .to_string();
        assert!(e.contains("--features locale"), "{e}");
        assert!(e.contains("avventura-cuda.exe"), "{e}");
    }

    #[test]
    fn verifica_del_file_gguf() {
        let dir = tempfile::tempdir().unwrap();
        let mancante = dir.path().join("manca.gguf");
        let e = verifica_file(&mancante).unwrap_err().to_string();
        assert!(e.contains("non esiste"), "{e}");
        assert!(e.contains("cartella del file di configurazione"), "{e}");

        let falso = dir.path().join("falso.gguf");
        std::fs::write(&falso, b"<html>pagina di errore</html>").unwrap();
        let e = verifica_file(&falso).unwrap_err().to_string();
        assert!(e.contains("non è un file GGUF"), "{e}");

        let corto = dir.path().join("corto.gguf");
        std::fs::write(&corto, b"GG").unwrap();
        assert!(verifica_file(&corto).is_err());

        let buono = dir.path().join("buono.gguf");
        std::fs::write(&buono, b"GGUF\x03\x00\x00\x00resto").unwrap();
        assert_eq!(verifica_file(&buono).unwrap(), 13);

        let e = verifica_file(dir.path()).unwrap_err().to_string();
        assert!(e.contains("non è un file"), "{e}");
    }

    #[test]
    fn contesto_predefinito_con_avviso() {
        let (n, avviso) = contesto_effettivo(0);
        assert_eq!(n, 8192);
        assert!(avviso.unwrap().contains("8.192"));
        assert_eq!(contesto_effettivo(16384), (16384, None));
    }

    /// Messaggi come quelli del motore: sistema + utente con lo stato in JSON.
    fn messaggi(turni: usize, memoria: &str) -> Vec<Messaggio> {
        let ultimi: Vec<Value> = (0..turni)
            .map(|i| serde_json::json!({"azione": format!("azione {i}"), "narrazione": "x".repeat(100)}))
            .collect();
        let utente = serde_json::json!({
            "stato": {"stanza_corrente": "a", "memoria": memoria, "ultimi_turni": ultimi},
            "azione_giocatore": "<<<guarda>>>",
        });
        vec![
            Messaggio::sistema("regole ".repeat(50)),
            Messaggio::utente(serde_json::to_string_pretty(&utente).unwrap()),
        ]
    }

    fn caratteri(m: &[Messaggio]) -> Result<usize, Infallible> {
        Ok(m.iter().map(|m| m.contenuto.chars().count()).sum())
    }

    fn stato(m: &[Messaggio]) -> Value {
        serde_json::from_str::<Value>(&m[1].contenuto).unwrap()["stato"].clone()
    }

    #[test]
    fn riduzione_non_necessaria() {
        let m = messaggi(3, "breve");
        let n = caratteri(&m).unwrap();
        let Ok(EsitoRiduzione::Entra {
            messaggi: r,
            token,
            turni_tolti,
            memoria_troncata,
        }) = riduci_prompt(&m, n, caratteri)
        else {
            panic!("doveva entrare");
        };
        assert_eq!((r, token, turni_tolti, memoria_troncata), (m, n, 0, false));
    }

    #[test]
    fn riduzione_dei_turni_dal_piu_vecchio() {
        let m = messaggi(5, "memoria");
        let n = caratteri(&m).unwrap();
        // Ogni turno vale circa 170 caratteri: per 250 se ne tolgono due.
        let Ok(EsitoRiduzione::Entra {
            messaggi: r,
            turni_tolti,
            memoria_troncata,
            token,
        }) = riduci_prompt(&m, n - 250, caratteri)
        else {
            panic!("doveva entrare");
        };
        assert_eq!(turni_tolti, 2);
        assert!(!memoria_troncata);
        assert!(token <= n - 250);
        let s = stato(&r);
        let turni = s["ultimi_turni"].as_array().unwrap();
        assert_eq!(turni.len(), 3);
        assert_eq!(turni[0]["azione"], "azione 2");
        assert_eq!(s["memoria"], "memoria");
        assert_eq!(r[0], m[0]);
    }

    #[test]
    fn riduzione_della_memoria_dopo_i_turni() {
        let memoria = format!("INIZIO{}FINE", "m".repeat(2000));
        let m = messaggi(2, &memoria);
        let senza_turni = caratteri(&messaggi(0, &memoria)).unwrap();
        let Ok(EsitoRiduzione::Entra {
            messaggi: r,
            turni_tolti,
            memoria_troncata,
            ..
        }) = riduci_prompt(&m, senza_turni - 300, caratteri)
        else {
            panic!("doveva entrare");
        };
        assert_eq!(turni_tolti, 2);
        assert!(memoria_troncata);
        let s = stato(&r);
        let tenuta = s["memoria"].as_str().unwrap();
        assert!(
            tenuta.starts_with('…') && tenuta.ends_with("FINE"),
            "{tenuta}"
        );
        assert!(!tenuta.contains("INIZIO"));
        assert_eq!(s["ultimi_turni"].as_array().unwrap().len(), 0);
    }

    #[test]
    fn riduzione_impossibile() {
        let m = messaggi(2, "memoria");
        let Ok(EsitoRiduzione::NonEntra { token }) = riduci_prompt(&m, 10, caratteri) else {
            panic!("non doveva entrare");
        };
        // Il prompt più piccolo: senza turni e senza memoria.
        let minimo = caratteri(&messaggi(0, "")).unwrap();
        assert_eq!(token, minimo);

        // Senza lo stato in JSON non si può ridurre nulla.
        let semplici = vec![Messaggio::sistema("s"), Messaggio::utente("ping")];
        assert_eq!(
            riduci_prompt(&semplici, 1, caratteri),
            Ok(EsitoRiduzione::NonEntra { token: 5 })
        );
    }

    #[test]
    fn testo_dell_avviso_di_riduzione() {
        let a = avviso_riduzione(3, true, 16384);
        assert!(a.contains("16.384") && a.contains("tolti 3 turni") && a.contains("memoria"));
        assert!(!avviso_riduzione(1, false, 8192).contains("memoria"));
    }

    #[test]
    fn chiusura_del_ragionamento() {
        let qwen = "{%- if enable_thinking is defined and enable_thinking is false %}";
        let p = "<|im_start|>assistant\n";
        assert_eq!(
            chiusura_ragionamento(qwen, p, Ragionamento::Spento),
            Some("<think>\n\n</think>\n\n")
        );
        assert_eq!(chiusura_ragionamento(qwen, p, Ragionamento::Alto), None);
        assert_eq!(
            chiusura_ragionamento("<start_of_turn>", p, Ragionamento::Spento),
            None
        );
        let chiuso = "<|im_start|>assistant\n<think>\n\n</think>\n";
        assert_eq!(
            chiusura_ragionamento(qwen, chiuso, Ragionamento::Spento),
            None
        );
    }

    #[test]
    fn riconoscimento_del_json_completo() {
        assert!(!json_completo(""));
        assert!(!json_completo("{\"narrazione\": \"apri {la} porta"));
        assert!(!json_completo("{\"a\": {\"b\": 1}"));
        assert!(json_completo("{\"a\": {\"b\": 1}}"));
        assert!(json_completo("  {\"a\": \"graffa } e \\\" virgolette\"}\n"));
        // Una barra rovesciata finale (escape di sé stessa) non apre un escape.
        assert!(json_completo("{\"a\": \"\\\\\"}"));
    }

    #[test]
    fn grammatica_generica_ben_formata() {
        assert!(GRAMMATICA_JSON.starts_with("root"));
        for regola in ["object", "array", "string", "number", "ws"] {
            assert!(
                GRAMMATICA_JSON.contains(&format!("\n{regola}")),
                "manca la regola {regola}"
            );
        }
    }

    /// Le funzioni pure di pulizia del ragionamento valgono anche per `locale`.
    #[test]
    fn pulizia_del_ragionamento_per_il_locale() {
        use crate::ai::ragionamento::rimuovi_ragionamento;
        assert_eq!(
            rimuovi_ragionamento("<think>\n\n</think>\n\n{\"narrazione\":\"ok\"}"),
            "{\"narrazione\":\"ok\"}"
        );
    }
}
