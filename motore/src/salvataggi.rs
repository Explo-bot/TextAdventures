//! Salvataggi con nome (§4.1): validazione dei nomi, elenco, salvataggio,
//! caricamento, eliminazione e controllo dei progressi non salvati.
//!
//! Ogni avventura ha i propri salvataggi in `partite/<id>/salvataggi/<nome>.json`.
//! Il confronto tra nomi non distingue maiuscole/minuscole; gli spazi del nome
//! diventano `-` nel nome del file.

use crate::tr;
use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};

use crate::stato::{
    Esito, PercorsiPartita, Stato, adesso, archivia_nello_storico, scrivi_json_atomico,
};

/// Lunghezza massima di un nome di salvataggio (in caratteri).
pub const LUNGHEZZA_MAX_NOME: usize = 40;

const NOMI_RISERVATI_WINDOWS: [&str; 22] = [
    "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
    "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
];

/// Riepilogo di un salvataggio, mostrato negli elenchi.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Riepilogo {
    /// Turno salvato.
    pub turno: u32,
    /// Id della stanza.
    pub stanza: String,
    /// Nome leggibile della stanza.
    pub nome_stanza: String,
    /// Punteggio.
    pub punteggio: i64,
    /// Esito.
    pub esito: Esito,
}

/// Contenuto di un file di salvataggio.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Salvataggio {
    /// Nome scelto dal giocatore.
    pub nome: String,
    /// Prima creazione.
    pub creato_il: DateTime<FixedOffset>,
    /// Ultima sovrascrittura.
    pub aggiornato_il: DateTime<FixedOffset>,
    /// Riepilogo per gli elenchi.
    pub riepilogo: Riepilogo,
    /// Copia completa dello stato.
    pub stato: Stato,
}

/// Voce dell'elenco dei salvataggi.
#[derive(Debug, Clone, PartialEq)]
pub struct InfoSalvataggio {
    /// Nome (dal file o, se danneggiato, dal nome del file).
    pub nome: String,
    /// Percorso del file.
    pub file: PathBuf,
    /// Data dell'ultimo aggiornamento (assente se danneggiato).
    pub aggiornato_il: Option<DateTime<FixedOffset>>,
    /// Riepilogo (assente se danneggiato).
    pub riepilogo: Option<Riepilogo>,
}

impl InfoSalvataggio {
    /// Vero se il file è illeggibile o corrotto.
    pub fn danneggiato(&self) -> bool {
        self.riepilogo.is_none()
    }
}

/// Verifica un nome di salvataggio; restituisce il nome ripulito dagli spazi esterni
/// oppure il motivo del rifiuto.
pub fn valida_nome(nome: &str) -> std::result::Result<String, String> {
    let nome = nome.trim();
    if nome.is_empty() {
        return Err(tr!(
            "il nome non può essere vuoto",
            "the name cannot be empty",
            "le nom ne peut pas être vide",
            "el nombre no puede estar vacío",
            "der Name darf nicht leer sein"
        )
        .to_string());
    }
    if nome.starts_with('.') {
        return Err(tr!(
            "il nome non può iniziare con un punto",
            "the name cannot start with a dot",
            "le nom ne peut pas commencer par un point",
            "el nombre no puede empezar por un punto",
            "der Name darf nicht mit einem Punkt beginnen"
        )
        .to_string());
    }
    let lunghezza = nome.chars().count();
    if lunghezza > LUNGHEZZA_MAX_NOME {
        return Err(tr!(
            format!("il nome è troppo lungo ({lunghezza} caratteri, massimo {LUNGHEZZA_MAX_NOME})"),
            format!("the name is too long ({lunghezza} characters, maximum {LUNGHEZZA_MAX_NOME})"),
            format!("le nom est trop long ({lunghezza} caractères, maximum {LUNGHEZZA_MAX_NOME})"),
            format!(
                "el nombre es demasiado largo ({lunghezza} caracteres, máximo {LUNGHEZZA_MAX_NOME})"
            ),
            format!("der Name ist zu lang ({lunghezza} Zeichen, höchstens {LUNGHEZZA_MAX_NOME})")
        ));
    }
    if let Some(c) = nome.chars().find(|c| "/\\:*?\"<>|".contains(*c)) {
        return Err(tr!(
            format!("il carattere «{c}» non è ammesso nei nomi"),
            format!("the character \"{c}\" is not allowed in names"),
            format!("le caractère « {c} » n'est pas autorisé dans les noms"),
            format!("el carácter «{c}» no está permitido en los nombres"),
            format!("das Zeichen „{c}“ ist in Namen nicht erlaubt")
        ));
    }
    if let Some(c) = nome
        .chars()
        .find(|c| !(c.is_alphanumeric() || *c == ' ' || *c == '-' || *c == '_'))
    {
        return Err(tr!(
            format!("il carattere «{c}» non è ammesso (usa lettere, cifre, spazi, - e _)"),
            format!("the character \"{c}\" is not allowed (use letters, digits, spaces, - and _)"),
            format!(
                "le caractère « {c} » n'est pas autorisé (utilisez des lettres, des chiffres, des espaces, - et _)"
            ),
            format!("el carácter «{c}» no está permitido (usa letras, números, espacios, - y _)"),
            format!(
                "das Zeichen „{c}“ ist nicht erlaubt (verwende Buchstaben, Ziffern, Leerzeichen, - und _)"
            )
        ));
    }
    let maiuscolo = nome_file(nome).to_uppercase();
    if NOMI_RISERVATI_WINDOWS.contains(&maiuscolo.as_str()) {
        return Err(tr!(
            format!("«{nome}» è un nome riservato di Windows e non può essere usato"),
            format!("\"{nome}\" is a reserved Windows name and cannot be used"),
            format!("« {nome} » est un nom réservé de Windows et ne peut pas être utilisé"),
            format!("«{nome}» es un nombre reservado de Windows y no se puede usar"),
            format!("„{nome}“ ist ein reservierter Windows-Name und kann nicht verwendet werden")
        ));
    }
    Ok(nome.to_string())
}

/// Nome del file (senza estensione) per un nome di salvataggio: spazi → `-`.
pub fn nome_file(nome: &str) -> String {
    nome.trim().replace(' ', "-")
}

/// Nome predefinito proposto da `/save`: `<stanza>-turno-<n>`, reso valido.
pub fn nome_predefinito(stanza: &str, turno: u32) -> String {
    let base: String = stanza
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '-'
            }
        })
        .collect();
    let suffisso = tr!(
        format!("-turno-{turno}"),
        format!("-turn-{turno}"),
        format!("-tour-{turno}"),
        format!("-turno-{turno}"),
        format!("-zug-{turno}")
    );
    let massimo = LUNGHEZZA_MAX_NOME.saturating_sub(suffisso.chars().count());
    let base: String = base.chars().take(massimo).collect();
    let base = base.trim_start_matches('.');
    let base = if base.is_empty() { "partita" } else { base };
    format!("{base}{suffisso}")
}

/// `progressi_non_salvati`: vero se ci sono turni giocati dopo l'ultimo `/save` o `/load`.
pub fn progressi_non_salvati(stato: &Stato) -> bool {
    stato.turno > stato.turno_ultimo_salvataggio
}

/// Operazioni sui salvataggi di una singola avventura.
#[derive(Debug, Clone)]
pub struct GestoreSalvataggi {
    percorsi: PercorsiPartita,
    id_avventura: String,
    /// Id delle stanze dell'avventura, se noti: un salvataggio la cui stanza
    /// corrente non esiste più è di una versione precedente e non si carica.
    stanze: Option<std::collections::BTreeSet<String>>,
}

/// Messaggio per uno stato scritto da una versione precedente dell'avventura
/// (la stanza corrente non esiste più, per esempio dopo il passaggio agli id inglesi).
pub fn versione_precedente(stanza: &str) -> String {
    tr!(
        format!(
            "è di una versione precedente dell'avventura (la stanza «{stanza}» non esiste più)"
        ),
        format!(
            "it comes from an earlier version of the adventure (the room \"{stanza}\" no longer exists)"
        ),
        format!(
            "elle provient d'une version précédente de l'aventure (le lieu « {stanza} » n'existe plus)"
        ),
        format!("es de una versión anterior de la aventura (el lugar «{stanza}» ya no existe)"),
        format!(
            "er stammt aus einer früheren Version des Abenteuers (der Raum „{stanza}“ existiert nicht mehr)"
        )
    )
}

impl GestoreSalvataggi {
    /// Gestore per l'avventura `id_avventura`.
    pub fn nuovo(percorsi: PercorsiPartita, id_avventura: &str) -> Self {
        GestoreSalvataggi {
            percorsi,
            id_avventura: id_avventura.to_string(),
            stanze: None,
        }
    }

    /// Gestore che controlla anche che i salvataggi siano compatibili con
    /// le stanze dell'avventura.
    pub fn per_avventura(
        percorsi: PercorsiPartita,
        avventura: &crate::avventura::Avventura,
    ) -> Self {
        GestoreSalvataggi {
            stanze: Some(avventura.stanze.keys().cloned().collect()),
            ..GestoreSalvataggi::nuovo(percorsi, &avventura.id)
        }
    }

    /// Percorsi della partita.
    pub fn percorsi(&self) -> &PercorsiPartita {
        &self.percorsi
    }

    /// Elenco dei salvataggi, dal più recente. I file danneggiati compaiono in fondo.
    pub fn elenca(&self) -> Vec<InfoSalvataggio> {
        let Ok(voci) = fs::read_dir(self.percorsi.salvataggi()) else {
            return Vec::new();
        };
        let mut elenco: Vec<InfoSalvataggio> = voci
            .filter_map(|v| v.ok().map(|v| v.path()))
            .filter(|p| p.is_file() && p.extension().is_some_and(|e| e == "json"))
            .map(|file| {
                let stem = file
                    .file_stem()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_default();
                match leggi_salvataggio(&file) {
                    Ok(s) => InfoSalvataggio {
                        nome: s.nome,
                        file,
                        aggiornato_il: Some(s.aggiornato_il),
                        riepilogo: Some(s.riepilogo),
                    },
                    Err(_) => InfoSalvataggio {
                        nome: stem,
                        file,
                        aggiornato_il: None,
                        riepilogo: None,
                    },
                }
            })
            .collect();
        elenco.sort_by(|a, b| {
            b.aggiornato_il
                .cmp(&a.aggiornato_il)
                .then_with(|| a.nome.to_lowercase().cmp(&b.nome.to_lowercase()))
        });
        elenco
    }

    /// Cerca un salvataggio per nome (senza distinzione maiuscole/minuscole).
    pub fn trova(&self, nome: &str) -> Option<InfoSalvataggio> {
        let cercato = nome_file(nome).to_lowercase();
        self.elenca().into_iter().find(|s| {
            nome_file(&s.nome).to_lowercase() == cercato
                || s.file
                    .file_stem()
                    .is_some_and(|f| f.to_string_lossy().to_lowercase() == cercato)
        })
    }

    /// Vero se il salvataggio esiste.
    pub fn esiste(&self, nome: &str) -> bool {
        self.trova(nome).is_some()
    }

    /// Crea o sovrascrive un salvataggio. Aggiorna `salvataggio_corrente` e
    /// `turno_ultimo_salvataggio` nello stato e riscrive `stato.json`.
    pub fn salva(
        &self,
        nome: &str,
        stato: &mut Stato,
        nome_stanza: &str,
        sovrascrivi: bool,
    ) -> Result<()> {
        let nome = valida_nome(nome).map_err(|m| anyhow::anyhow!("nome non valido: {m}"))?;
        let esistente = self.trova(&nome);
        if esistente.is_some() && !sovrascrivi {
            bail!("esiste già un salvataggio «{nome}»");
        }
        let (file, creato_il) = match &esistente {
            Some(info) => {
                let creato = leggi_salvataggio(&info.file).ok().map(|s| s.creato_il);
                (info.file.clone(), creato)
            }
            None => (
                self.percorsi
                    .salvataggi()
                    .join(format!("{}.json", nome_file(&nome))),
                None,
            ),
        };
        stato.salvataggio_corrente = Some(nome.clone());
        stato.turno_ultimo_salvataggio = stato.turno;
        let ora = adesso();
        let salvataggio = Salvataggio {
            nome: nome.clone(),
            creato_il: creato_il.unwrap_or(ora),
            aggiornato_il: ora,
            riepilogo: Riepilogo {
                turno: stato.turno,
                stanza: stato.stanza_corrente.clone(),
                nome_stanza: nome_stanza.to_string(),
                punteggio: stato.punteggio,
                esito: stato.esito,
            },
            stato: stato.clone(),
        };
        scrivi_json_atomico(&file, &salvataggio)?;
        stato.salva_atomico(&self.percorsi.stato())
    }

    /// Carica un salvataggio in `stato.json` (lo stato precedente va nello storico).
    pub fn carica(&self, nome: &str) -> Result<Stato> {
        let info = self
            .trova(nome)
            .with_context(|| crate::menu::non_esiste(nome))?;
        let salvataggio = leggi_salvataggio(&info.file).with_context(|| {
            tr!(
                format!("il salvataggio «{}» è danneggiato", info.nome),
                format!("the save \"{}\" is damaged", info.nome),
                format!("la sauvegarde « {} » est endommagée", info.nome),
                format!("la partida guardada «{}» está dañada", info.nome),
                format!("der Spielstand „{}“ ist beschädigt", info.nome)
            )
        })?;
        if salvataggio.stato.avventura != self.id_avventura {
            bail!(
                "il salvataggio «{}» appartiene all'avventura «{}», non a «{}»",
                info.nome,
                salvataggio.stato.avventura,
                self.id_avventura
            );
        }
        if let Some(stanze) = &self.stanze
            && !stanze.contains(&salvataggio.stato.stanza_corrente)
        {
            bail!(
                "{}",
                tr!(
                    format!(
                        "il salvataggio «{}» {}",
                        info.nome,
                        versione_precedente(&salvataggio.stato.stanza_corrente)
                    ),
                    format!(
                        "the save \"{}\" {}",
                        info.nome,
                        versione_precedente(&salvataggio.stato.stanza_corrente)
                    ),
                    format!(
                        "la sauvegarde « {} » : {}",
                        info.nome,
                        versione_precedente(&salvataggio.stato.stanza_corrente)
                    ),
                    format!(
                        "la partida guardada «{}»: {}",
                        info.nome,
                        versione_precedente(&salvataggio.stato.stanza_corrente)
                    ),
                    format!(
                        "der Spielstand „{}“: {}",
                        info.nome,
                        versione_precedente(&salvataggio.stato.stanza_corrente)
                    )
                )
            );
        }
        archivia_nello_storico(&self.percorsi)?;
        let mut stato = salvataggio.stato;
        stato.salvataggio_corrente = Some(salvataggio.nome);
        stato.turno_ultimo_salvataggio = stato.turno;
        stato.salva_atomico(&self.percorsi.stato())?;
        Ok(stato)
    }

    /// Elimina un salvataggio.
    pub fn elimina(&self, nome: &str) -> Result<String> {
        let info = self
            .trova(nome)
            .with_context(|| crate::menu::non_esiste(nome))?;
        fs::remove_file(&info.file)
            .with_context(|| format!("impossibile eliminare «{}»", info.file.display()))?;
        Ok(info.nome)
    }
}

fn leggi_salvataggio(file: &std::path::Path) -> Result<Salvataggio> {
    let testo = fs::read_to_string(file)?;
    Ok(serde_json::from_str(&testo)?)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::stato::test::avventura_di_prova;

    #[test]
    fn nomi_validi() {
        for nome in [
            "prima-della-scala",
            "Salvo 1",
            "già_fatto",
            "a",
            &"x".repeat(40),
        ] {
            assert!(valida_nome(nome).is_ok(), "{nome}");
        }
        assert_eq!(valida_nome("  ciao  ").unwrap(), "ciao");
    }

    #[test]
    fn nomi_non_validi() {
        let casi = [
            ("", "vuoto"),
            ("   ", "vuoto"),
            (".nascosto", "punto"),
            ("a/b", "«/»"),
            ("a\\b", "«\\»"),
            ("a:b", "«:»"),
            ("a*b", "«*»"),
            ("a?b", "«?»"),
            ("a\"b", "«\"»"),
            ("a<b", "«<»"),
            ("a|b", "«|»"),
            ("a.b", "«.»"),
            ("CON", "riservato"),
            ("com1", "riservato"),
            ("Lpt9", "riservato"),
        ];
        for (nome, atteso) in casi {
            let errore = valida_nome(nome).unwrap_err();
            assert!(errore.contains(atteso), "{nome}: {errore}");
        }
        assert!(valida_nome(&"x".repeat(41)).unwrap_err().contains("lungo"));
        assert!(valida_nome("CONSOLE").is_ok());
    }

    #[test]
    fn nome_file_e_predefinito() {
        assert_eq!(nome_file("prima della scala"), "prima-della-scala");
        assert_eq!(nome_predefinito("ingresso", 12), "ingresso-turno-12");
        assert!(valida_nome(&nome_predefinito(&"z".repeat(60), 3)).is_ok());
        assert!(valida_nome(&nome_predefinito("a.b", 3)).is_ok());
    }

    #[test]
    fn progressi() {
        let mut s = Stato::nuovo(&avventura_di_prova());
        assert!(!progressi_non_salvati(&s));
        s.turno = 3;
        assert!(progressi_non_salvati(&s));
        s.turno_ultimo_salvataggio = 3;
        assert!(!progressi_non_salvati(&s));
    }

    #[test]
    fn salva_carica_elimina() {
        let dir = tempfile::tempdir().unwrap();
        let a = avventura_di_prova();
        let g = GestoreSalvataggi::nuovo(PercorsiPartita::nuovi(dir.path(), &a.id), &a.id);
        let mut s = Stato::nuovo(&a);
        s.turno = 4;
        g.salva("Primo salvo", &mut s, "Stanza A", false).unwrap();
        assert_eq!(s.salvataggio_corrente.as_deref(), Some("Primo salvo"));
        assert_eq!(s.turno_ultimo_salvataggio, 4);
        assert!(g.percorsi().salvataggi().join("Primo-salvo.json").exists());
        assert!(g.esiste("primo SALVO"));
        assert!(g.salva("primo salvo", &mut s, "Stanza A", false).is_err());

        s.turno = 9;
        g.salva("primo salvo", &mut s, "Stanza A", true).unwrap();
        let elenco = g.elenca();
        assert_eq!(elenco.len(), 1);
        assert_eq!(elenco[0].riepilogo.as_ref().unwrap().turno, 9);

        s.turno = 15;
        s.salva_atomico(&g.percorsi().stato()).unwrap();
        let caricato = g.carica("PRIMO SALVO").unwrap();
        assert_eq!(caricato.turno, 9);
        assert_eq!(caricato.turno_ultimo_salvataggio, 9);
        assert_eq!(Stato::carica(&g.percorsi().stato()).unwrap().turno, 9);
        // Lo stato precedente (turno 15) è nello storico.
        assert!(g.percorsi().storico().join("turno_0015.json").exists());

        assert_eq!(g.elimina("primo salvo").unwrap(), "primo salvo");
        assert!(g.elenca().is_empty());
        assert!(g.carica("primo salvo").is_err());
    }

    #[test]
    fn danneggiati_e_altre_avventure() {
        let dir = tempfile::tempdir().unwrap();
        let a = avventura_di_prova();
        let g = GestoreSalvataggi::nuovo(PercorsiPartita::nuovi(dir.path(), &a.id), &a.id);
        let mut s = Stato::nuovo(&a);
        g.salva("buono", &mut s, "A", false).unwrap();
        fs::write(g.percorsi().salvataggi().join("rotto.json"), "{ non json").unwrap();
        let elenco = g.elenca();
        assert_eq!(elenco.len(), 2);
        assert_eq!(elenco[1].nome, "rotto");
        assert!(elenco[1].danneggiato());
        assert!(g.carica("rotto").is_err());

        let mut estraneo = s.clone();
        estraneo.avventura = "altra".into();
        let altro = GestoreSalvataggi::nuovo(PercorsiPartita::nuovi(dir.path(), &a.id), "altra");
        altro.salva("intruso", &mut estraneo, "A", false).unwrap();
        let errore = g.carica("intruso").unwrap_err().to_string();
        assert!(
            errore.contains("appartiene all'avventura «altra»"),
            "{errore}"
        );
    }
}
