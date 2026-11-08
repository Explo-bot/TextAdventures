//! Risoluzione della configurazione AI (§5): profilo scelto, fornitore,
//! predefiniti per fornitore, variabili d'ambiente e opzioni da riga di comando.
//!
//! Precedenza (dalla più forte): `--profilo`/`--timeout` → variabili
//! d'ambiente (`AVVENTURA_*`) → profilo `[profili.NOME]` → sezione `[ai]` →
//! predefiniti del fornitore.
//!
//! Regola di unione dei profili: ogni chiave del profilo sostituisce quella di
//! `[ai]`; le sotto-sezioni `locale` e `header_extra` vengono unite chiave per
//! chiave. Se il profilo cambia **fornitore**, le chiavi di `[ai]` legate al
//! vecchio fornitore (`base_url`, `api_key_env`, `api_key`, `chiave_richiesta`,
//! `modello`, `ragionamento`, `header_extra`) non vengono ereditate: altrimenti
//! un profilo `claude` partirebbe con il `base_url` di OpenAI.
//!
//! Compatibilità: `protocollo = "openai" | "ollama"` è sinonimo di `fornitore`
//! (con un avviso); `base_url = "finto://"` equivale a `fornitore = "finto"`.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use serde::Deserialize;

use crate::ai::ragionamento::Ragionamento;
use crate::config::{ConfigAi, ConfigLocale, Fornitore, LettoreAmbiente, SorgenteAi};

/// Chiavi di `[ai]` legate al fornitore, non ereditate da un profilo che cambia fornitore.
const CHIAVI_DEL_FORNITORE: [&str; 9] = [
    "fornitore",
    "protocollo",
    "base_url",
    "api_key_env",
    "api_key",
    "chiave_richiesta",
    "modello",
    "ragionamento",
    "header_extra",
];

/// Sezione `[ai.locale]` come scritta nel file.
#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct FileLocale {
    percorso_modello: Option<String>,
    gpu_layers: Option<i32>,
    thread: Option<u32>,
    template_chat: Option<String>,
}

/// Sezione `[ai]` (o profilo già unito) come scritta nel file: tutto facoltativo.
#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct FileAi {
    fornitore: Option<String>,
    protocollo: Option<String>,
    base_url: Option<String>,
    api_key_env: Option<String>,
    api_key: Option<String>,
    chiave_richiesta: Option<bool>,
    modello: Option<String>,
    temperatura: Option<f32>,
    max_token: Option<u32>,
    contesto_token: Option<u32>,
    ragionamento: Option<String>,
    json_mode: Option<bool>,
    timeout_secondi: Option<u64>,
    timeout_connessione_secondi: Option<u64>,
    max_tentativi: Option<u32>,
    header_extra: Option<BTreeMap<String, String>>,
    locale: Option<FileLocale>,
}

/// Fornitore dichiarato da una tabella (fornitore, protocollo o `finto://`).
fn fornitore_della_tabella(t: &toml::Table) -> Option<Fornitore> {
    if t.get("base_url")
        .and_then(toml::Value::as_str)
        .is_some_and(e_finto)
    {
        return Some(Fornitore::Finto);
    }
    t.get("fornitore")
        .or_else(|| t.get("protocollo"))
        .and_then(toml::Value::as_str)
        .and_then(Fornitore::da_testo)
}

fn e_finto(url: &str) -> bool {
    url.trim().to_lowercase().starts_with("finto://")
}

/// Unisce `[ai]` con un profilo secondo le regole descritte nel modulo.
pub fn unisci(base: &toml::Table, profilo: &toml::Table) -> toml::Table {
    let mut risultato = base.clone();
    let cambia_fornitore = match fornitore_della_tabella(profilo) {
        Some(nuovo) => fornitore_della_tabella(base).unwrap_or_default() != nuovo,
        None => false,
    };
    if cambia_fornitore {
        for chiave in CHIAVI_DEL_FORNITORE {
            risultato.remove(chiave);
        }
    }
    for (chiave, valore) in profilo {
        match (risultato.get_mut(chiave), valore) {
            (Some(toml::Value::Table(esistente)), toml::Value::Table(nuova))
                if chiave == "locale" || chiave == "header_extra" =>
            {
                for (k, v) in nuova {
                    esistente.insert(k.clone(), v.clone());
                }
            }
            _ => {
                risultato.insert(chiave.clone(), valore.clone());
            }
        }
    }
    risultato
}

/// Nome del profilo effettivo: opzione da riga di comando, poi `AVVENTURA_PROFILO`.
pub fn profilo_scelto(da_cli: Option<&str>, ambiente: LettoreAmbiente<'_>) -> Option<String> {
    da_cli
        .map(str::to_string)
        .or_else(|| ambiente("AVVENTURA_PROFILO"))
        .map(|p| p.trim().to_string())
        .filter(|p| !p.is_empty())
}

/// Risolve la configurazione AI completa.
pub fn risolvi(
    sorgente: &SorgenteAi,
    profilo: Option<&str>,
    timeout_cli: Option<u64>,
    ambiente: LettoreAmbiente<'_>,
) -> Result<ConfigAi> {
    let leggi = |k: &str| {
        ambiente(k)
            .map(|v| v.trim().to_string())
            .filter(|v| !v.is_empty())
    };
    let nome_profilo = profilo_scelto(profilo, ambiente);
    let tabella = match &nome_profilo {
        None => sorgente.ai.clone(),
        Some(nome) => {
            let Some(p) = sorgente.profili.get(nome) else {
                let disponibili = if sorgente.profili.is_empty() {
                    "nessun profilo è definito nel file di configurazione".to_string()
                } else {
                    format!(
                        "profili disponibili: {}",
                        sorgente
                            .profili
                            .keys()
                            .cloned()
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                };
                bail!(
                    "il profilo «{nome}» non esiste ({disponibili}). Usa `avventura profili` per l'elenco."
                );
            };
            unisci(&sorgente.ai, p)
        }
    };
    let dove = match &nome_profilo {
        Some(n) => format!("[profili.{n}] (unito ad [ai])"),
        None => "[ai]".to_string(),
    };
    let file: FileAi = toml::Value::Table(tabella)
        .try_into()
        .map_err(|e: toml::de::Error| anyhow::anyhow!("{dove}: {}", e.message().trim()))?;

    let mut avvisi = Vec::new();
    // Fornitore.
    let base_url_env = leggi("AVVENTURA_BASE_URL");
    let fornitore = if let Some(v) = leggi("AVVENTURA_FORNITORE") {
        Fornitore::da_testo(&v).with_context(|| {
            format!(
                "AVVENTURA_FORNITORE = «{v}» non valido: usa uno tra {}",
                Fornitore::nomi_validi()
            )
        })?
    } else if base_url_env.as_deref().is_some_and(e_finto)
        || file.base_url.as_deref().is_some_and(e_finto)
    {
        Fornitore::Finto
    } else if let Some(f) = &file.fornitore {
        Fornitore::da_testo(f).with_context(|| {
            format!(
                "{dove}: fornitore = «{f}» non valido: usa uno tra {}",
                Fornitore::nomi_validi()
            )
        })?
    } else if let Some(p) = &file.protocollo {
        let f = match p.trim().to_lowercase().as_str() {
            "openai" => Fornitore::Openai,
            "ollama" => Fornitore::Ollama,
            _ => bail!(
                "{dove}: protocollo = «{p}» non valido: usa fornitore = \"openai\" o \"ollama\""
            ),
        };
        avvisi.push(format!(
            "La chiave `protocollo` è sostituita da `fornitore`: in config.toml scrivi \
             fornitore = \"{}\" (per LM Studio puoi usare fornitore = \"lmstudio\").",
            f.nome()
        ));
        f
    } else {
        Fornitore::default()
    };

    let base_url = if fornitore.usa_http() {
        base_url_env
            .or(file.base_url.clone())
            .map(|u| u.trim().to_string())
            .filter(|u| !u.is_empty())
            .unwrap_or_else(|| fornitore.base_url_predefinito().to_string())
    } else {
        String::new()
    };
    let modello = leggi("AVVENTURA_MODELLO")
        .or(file.modello.clone())
        .unwrap_or_else(|| fornitore.modello_predefinito().to_string());
    let api_key_env = file
        .api_key_env
        .clone()
        .map(|v| v.trim().to_string())
        .unwrap_or_else(|| fornitore.api_key_env_predefinita().to_string());
    let dal_file = file
        .api_key
        .clone()
        .map(|k| k.trim().to_string())
        .filter(|k| !k.is_empty());
    let chiave = if fornitore.usa_http() {
        leggi("AVVENTURA_API_KEY")
            .or_else(|| {
                if api_key_env.is_empty() {
                    None
                } else {
                    leggi(&api_key_env)
                }
            })
            .or(dal_file)
    } else {
        None
    };
    let contesto_token = match leggi("AVVENTURA_CONTESTO") {
        Some(v) => v.parse().with_context(|| {
            format!("AVVENTURA_CONTESTO = «{v}» non valido: serve un numero di token")
        })?,
        None => file.contesto_token.unwrap_or(0),
    };
    let timeout_secondi = match (timeout_cli, leggi("AVVENTURA_TIMEOUT")) {
        (Some(t), _) => t,
        (None, Some(v)) => v.parse().with_context(|| {
            format!("AVVENTURA_TIMEOUT = «{v}» non valido: serve un numero di secondi")
        })?,
        (None, None) => file.timeout_secondi.unwrap_or(900),
    };
    let ragionamento = match &file.ragionamento {
        Some(r) => Ragionamento::da_testo(r).with_context(|| {
            format!(
                "{dove}: ragionamento = «{r}» non valido: usa predefinito, spento, basso, medio o alto"
            )
        })?,
        None => fornitore.ragionamento_predefinito(),
    };
    let fl = file.locale.unwrap_or_default();
    let percorso = leggi("AVVENTURA_MODELLO_GGUF").or(fl.percorso_modello);
    let locale = ConfigLocale {
        percorso_modello: percorso.map(|p| percorso_relativo(&p, sorgente.cartella.as_deref())),
        gpu_layers: fl.gpu_layers.unwrap_or(-1),
        thread: fl.thread.unwrap_or(0),
        template_chat: fl.template_chat.filter(|t| !t.trim().is_empty()),
    };

    let ai = ConfigAi {
        fornitore,
        base_url,
        api_key_env,
        api_key: file.api_key,
        chiave_richiesta: file.chiave_richiesta,
        modello,
        temperatura: file.temperatura.unwrap_or(0.7),
        max_token: file.max_token.unwrap_or(1200),
        contesto_token,
        ragionamento,
        json_mode: file.json_mode.unwrap_or(true),
        timeout_secondi,
        timeout_connessione_secondi: file.timeout_connessione_secondi.unwrap_or(10),
        max_tentativi: file.max_tentativi.unwrap_or(3),
        header_extra: file.header_extra.unwrap_or_default(),
        locale,
        chiave,
        profilo: nome_profilo,
        avvisi,
    };
    ai.valida()?;
    Ok(ai)
}

/// Percorso relativo alla cartella del file di configurazione (o assoluto).
fn percorso_relativo(percorso: &str, cartella: Option<&Path>) -> PathBuf {
    let p = PathBuf::from(percorso.trim());
    match cartella {
        Some(c) if p.is_relative() => c.join(p),
        _ => p,
    }
}

/// Riga di riepilogo di un profilo per `avventura profili`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InfoProfilo {
    /// Nome (None = sezione `[ai]`).
    pub nome: Option<String>,
    /// Fornitore, modello, base_url, chiave… oppure il motivo dell'errore.
    pub descrizione: std::result::Result<String, String>,
    /// Vero se è il profilo attivo.
    pub attivo: bool,
}

/// Elenca `[ai]` e i profili configurati, indicando quello attivo.
pub fn elenca(
    sorgente: &SorgenteAi,
    attivo: Option<&str>,
    ambiente: LettoreAmbiente<'_>,
) -> Vec<InfoProfilo> {
    let nomi = std::iter::once(None).chain(sorgente.profili.keys().map(|k| Some(k.as_str())));
    nomi.map(|nome| {
        // Il profilo richiesto esplicitamente vince su AVVENTURA_PROFILO.
        let ambiente_senza_profilo = |k: &str| {
            if k == "AVVENTURA_PROFILO" {
                None
            } else {
                ambiente(k)
            }
        };
        let descrizione = risolvi(sorgente, nome, None, &ambiente_senza_profilo)
            .map(|ai| descrivi(&ai))
            .map_err(|e| format!("{e:#}"));
        InfoProfilo {
            nome: nome.map(str::to_string),
            descrizione,
            attivo: nome == attivo,
        }
    })
    .collect()
}

/// Descrizione su una riga, senza segreti.
pub fn descrivi(ai: &ConfigAi) -> String {
    let mut parti = vec![format!("fornitore {}", ai.fornitore)];
    if !ai.modello.is_empty() && ai.fornitore != Fornitore::Finto {
        parti.push(format!("modello {}", ai.modello));
    }
    if ai.fornitore.usa_http() {
        parti.push(ai.base_url.clone());
        parti.push(
            if ai.chiave.is_some() {
                "chiave: presente"
            } else if ai.chiave_richiesta_effettiva() {
                "chiave: MANCANTE"
            } else {
                "chiave: non richiesta"
            }
            .to_string(),
        );
    }
    if ai.fornitore == Fornitore::Locale {
        parti.push(match &ai.locale.percorso_modello {
            Some(p) => format!("file {}", p.display()),
            None => "file GGUF non indicato".to_string(),
        });
    }
    if ai.contesto_token > 0 {
        parti.push(format!("contesto {}", ai.contesto_token));
    }
    parti.join(" · ")
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::config::Config;

    const FILE: &str = r#"
[ai]
fornitore = "openai"
base_url = "https://gateway.example/v1"
modello = "gpt-x"
temperatura = 0.5
contesto_token = 4096
[ai.header_extra]
"HTTP-Referer" = "https://example.org"
[ai.locale]
gpu_layers = 10

[profili.claude]
fornitore = "anthropic"
modello = "claude-sonnet-5-5"

[profili.veloce]
modello = "gpt-y"
timeout_secondi = 60

[profili.gguf]
fornitore = "locale"
[profili.gguf.locale]
percorso_modello = "modelli/m.gguf"
"#;

    fn sorgente() -> SorgenteAi {
        let mut c = Config::da_toml(FILE).unwrap();
        c.sorgente.cartella = Some(PathBuf::from("base"));
        c.sorgente
    }

    fn nessuno(_: &str) -> Option<String> {
        None
    }

    #[test]
    fn profilo_con_altro_fornitore_non_eredita_le_chiavi_del_fornitore() {
        let ai = risolvi(&sorgente(), Some("claude"), None, &nessuno).unwrap();
        assert_eq!(ai.fornitore, Fornitore::Anthropic);
        assert_eq!(ai.base_url, "https://api.anthropic.com");
        assert_eq!(ai.api_key_env, "ANTHROPIC_API_KEY");
        assert_eq!(ai.modello, "claude-sonnet-5-5");
        assert!(ai.header_extra.is_empty());
        // Le chiavi generiche restano ereditate.
        assert_eq!(ai.temperatura, 0.5);
        assert_eq!(ai.contesto_token, 4096);
        assert_eq!(ai.profilo.as_deref(), Some("claude"));
    }

    #[test]
    fn profilo_con_lo_stesso_fornitore_eredita_tutto() {
        let ai = risolvi(&sorgente(), Some("veloce"), None, &nessuno).unwrap();
        assert_eq!(ai.base_url, "https://gateway.example/v1");
        assert_eq!(ai.modello, "gpt-y");
        assert_eq!(ai.timeout_secondi, 60);
        assert_eq!(ai.header_extra.len(), 1);
    }

    #[test]
    fn sezioni_annidate_unite_e_percorsi_relativi() {
        let ai = risolvi(&sorgente(), Some("gguf"), None, &nessuno).unwrap();
        assert_eq!(ai.fornitore, Fornitore::Locale);
        assert_eq!(ai.locale.gpu_layers, 10);
        assert_eq!(
            ai.locale.percorso_modello,
            Some(PathBuf::from("base").join("modelli/m.gguf"))
        );
        assert_eq!(ai.ragionamento, Ragionamento::Spento);
        assert!(ai.base_url.is_empty());
        let env = |k: &str| (k == "AVVENTURA_MODELLO_GGUF").then(|| "/abs/x.gguf".to_string());
        let ai = risolvi(&sorgente(), Some("gguf"), None, &env).unwrap();
        assert_eq!(
            ai.locale.percorso_modello,
            Some(PathBuf::from("/abs/x.gguf"))
        );
    }

    #[test]
    fn precedenze_tra_riga_di_comando_ambiente_e_profilo() {
        let env = |k: &str| match k {
            "AVVENTURA_PROFILO" => Some("veloce".to_string()),
            "AVVENTURA_MODELLO" => Some("dall-ambiente".to_string()),
            "AVVENTURA_TIMEOUT" => Some("300".to_string()),
            _ => None,
        };
        // AVVENTURA_PROFILO sceglie il profilo; l'ambiente vince sul profilo.
        let ai = risolvi(&sorgente(), None, None, &env).unwrap();
        assert_eq!(ai.profilo.as_deref(), Some("veloce"));
        assert_eq!(ai.modello, "dall-ambiente");
        assert_eq!(ai.timeout_secondi, 300);
        // --profilo e --timeout vincono sull'ambiente.
        let ai = risolvi(&sorgente(), Some("claude"), Some(30), &env).unwrap();
        assert_eq!(ai.fornitore, Fornitore::Anthropic);
        assert_eq!(ai.timeout_secondi, 30);
        // AVVENTURA_FORNITORE vince sul file.
        let env = |k: &str| (k == "AVVENTURA_FORNITORE").then(|| "ollama".to_string());
        let ai = risolvi(&sorgente(), None, None, &env).unwrap();
        assert_eq!(ai.fornitore, Fornitore::Ollama);
    }

    #[test]
    fn profilo_inesistente_e_valori_non_validi() {
        let e = risolvi(&sorgente(), Some("nessuno"), None, &nessuno)
            .unwrap_err()
            .to_string();
        assert!(e.contains("claude, gguf, veloce"), "{e}");
        let s = Config::da_toml("[ai]\nfornitore = \"boh\"\n")
            .unwrap()
            .sorgente;
        let e = risolvi(&s, None, None, &nessuno).unwrap_err().to_string();
        assert!(e.contains("anthropic"), "{e}");
        let s = Config::da_toml("[ai]\nragionamento = \"tanto\"\n")
            .unwrap()
            .sorgente;
        assert!(risolvi(&s, None, None, &nessuno).is_err());
    }

    #[test]
    fn sinonimi_di_compatibilita() {
        let s = Config::da_toml(
            "[ai]\nprotocollo = \"ollama\"\nbase_url = \"http://localhost:11434\"\n",
        )
        .unwrap()
        .sorgente;
        let ai = risolvi(&s, None, None, &nessuno).unwrap();
        assert_eq!(ai.fornitore, Fornitore::Ollama);
        assert_eq!(ai.avvisi.len(), 1);
        assert!(ai.avvisi[0].contains("fornitore"));
        let s = Config::da_toml("[ai]\nbase_url = \"finto://\"\n")
            .unwrap()
            .sorgente;
        assert_eq!(
            risolvi(&s, None, None, &nessuno).unwrap().fornitore,
            Fornitore::Finto
        );
    }

    #[test]
    fn predefiniti_per_fornitore() {
        for (f, url, chiave, rag) in [
            (
                "openai",
                "https://api.openai.com/v1",
                "OPENAI_API_KEY",
                Ragionamento::Predefinito,
            ),
            (
                "anthropic",
                "https://api.anthropic.com",
                "ANTHROPIC_API_KEY",
                Ragionamento::Predefinito,
            ),
            (
                "gemini",
                "https://generativelanguage.googleapis.com/v1beta",
                "GEMINI_API_KEY",
                Ragionamento::Predefinito,
            ),
            ("ollama", "http://localhost:11434", "", Ragionamento::Spento),
            (
                "lmstudio",
                "http://localhost:1234/v1",
                "",
                Ragionamento::Spento,
            ),
        ] {
            let s = Config::da_toml(&format!("[ai]\nfornitore = \"{f}\"\n"))
                .unwrap()
                .sorgente;
            let ai = risolvi(&s, None, None, &nessuno).unwrap();
            assert_eq!(ai.base_url, url, "{f}");
            assert_eq!(ai.api_key_env, chiave, "{f}");
            assert_eq!(ai.ragionamento, rag, "{f}");
        }
    }

    #[test]
    fn elenco_dei_profili() {
        let elenco = elenca(&sorgente(), Some("claude"), &nessuno);
        assert_eq!(elenco.len(), 4);
        assert!(elenco[0].nome.is_none());
        let claude = elenco
            .iter()
            .find(|p| p.nome.as_deref() == Some("claude"))
            .unwrap();
        assert!(claude.attivo);
        let d = claude.descrizione.as_ref().unwrap();
        assert!(
            d.contains("anthropic") && d.contains("chiave: MANCANTE"),
            "{d}"
        );
    }
}
