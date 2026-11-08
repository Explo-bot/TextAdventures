//! AI simulata offline (`base_url = "finto://"`, §9).
//!
//! Deterministica e senza rete: interpreta poche azioni in italiano, inglese,
//! francese, spagnolo o tedesco (la lingua dell'interfaccia, §7.6) usando il [`ContestoTurno`] fornito
//! dal motore e risponde con un JSON conforme allo schema, che passa per la
//! stessa estrazione e validazione delle risposte reali.
//! Serve per i test, per le demo e per provare un'avventura senza modello.

use serde_json::{Value, json};

use super::{ContestoTurno, ErroreAi, Narratore, OggettoNominato, RichiestaAi, RispostaGrezza};
use crate::lingua::{self, Lingua};
use crate::tr_in;

/// Lunghezza massima della memoria scritta dal finto.
const MEMORIA_MAX: usize = 1500;

/// Marcatori di inizio e ripresa partita (minuscoli) nelle due lingue del prompt.
const MARCATORI: [&str; 4] = [
    "[inizio partita]",
    "[ripresa partita]",
    "[game start]",
    "[game resumed]",
];

/// Parole riconosciute in una lingua: verbi per prendere, lasciare e guardare,
/// articoli da togliere, verbi di movimento (con la direzione implicita,
/// inglese, per quelli che ne hanno una: `sali` = `up`).
struct Vocabolario {
    prendi: &'static [&'static str],
    lascia: &'static [&'static str],
    guarda: &'static [&'static str],
    articoli: &'static [&'static str],
    movimento: &'static [(&'static str, Option<&'static str>)],
}

fn vocabolario(l: Lingua) -> Vocabolario {
    match l {
        Lingua::It => Vocabolario {
            prendi: &["prendi", "raccogli"],
            lascia: &["lascia", "posa"],
            guarda: &["guarda", "esamina"],
            articoli: &[
                "il ", "lo ", "la ", "i ", "gli ", "le ", "un ", "una ", "uno ", "l'",
            ],
            movimento: &[
                ("vai", None),
                ("va", None),
                ("andiamo", None),
                ("cammina", None),
                ("sali", Some("up")),
                ("scendi", Some("down")),
                ("entra", Some("in")),
                ("esci", Some("out")),
            ],
        },
        Lingua::En => Vocabolario {
            prendi: &["take", "get", "grab", "pick"],
            lascia: &["drop", "leave"],
            guarda: &["look", "examine", "x", "l"],
            articoli: &["the ", "a ", "an ", "some "],
            movimento: &[
                ("go", None),
                ("walk", None),
                ("run", None),
                ("climb", None),
                ("head", None),
                ("move", None),
                ("enter", Some("in")),
                ("exit", Some("out")),
                ("leave", Some("out")),
            ],
        },
        Lingua::Fr => Vocabolario {
            prendi: &["prends", "prendre", "ramasse"],
            lascia: &["pose", "lâche", "lache", "laisse"],
            guarda: &["regarde", "examine", "observe"],
            articoli: &["le ", "la ", "les ", "un ", "une ", "des ", "du ", "l'"],
            movimento: &[
                ("va", None),
                ("vais", None),
                ("aller", None),
                ("allez", None),
                ("marche", None),
                ("monte", Some("up")),
                ("descends", Some("down")),
                ("entre", Some("in")),
                ("sors", Some("out")),
            ],
        },
        Lingua::Es => Vocabolario {
            prendi: &["coge", "toma", "agarra", "recoge"],
            lascia: &["deja", "suelta"],
            guarda: &["mira", "examina", "observa"],
            articoli: &[
                "el ", "la ", "los ", "las ", "un ", "una ", "unos ", "unas ",
            ],
            movimento: &[
                ("ve", None),
                ("voy", None),
                ("ir", None),
                ("anda", None),
                ("camina", None),
                ("sube", Some("up")),
                ("baja", Some("down")),
                ("entra", Some("in")),
                ("sal", Some("out")),
            ],
        },
        Lingua::De => Vocabolario {
            prendi: &["nimm", "nehme", "hebe"],
            lascia: &["lass", "lege", "leg", "wirf"],
            guarda: &["schau", "untersuche", "betrachte", "sieh"],
            articoli: &[
                "den ", "die ", "das ", "der ", "dem ", "einen ", "eine ", "ein ", "einem ",
            ],
            movimento: &[
                ("geh", None),
                ("gehe", None),
                ("lauf", None),
                ("laufe", None),
                ("steig", None),
                ("klettere", None),
                ("betritt", Some("in")),
                ("verlasse", Some("out")),
            ],
        },
    }
}

/// Narratore simulato.
#[derive(Debug, Default)]
pub struct NarratoreFinto;

impl NarratoreFinto {
    /// Nuovo narratore simulato.
    pub fn new() -> Self {
        NarratoreFinto
    }
}

impl Narratore for NarratoreFinto {
    fn turno(&self, richiesta: &RichiestaAi) -> Result<RispostaGrezza, ErroreAi> {
        let risposta = rispondi(&richiesta.contesto);
        Ok(RispostaGrezza {
            testo: serde_json::to_string_pretty(&risposta).unwrap_or_default(),
            ..Default::default()
        })
    }

    /// Simula lo streaming: il JSON arriva a pezzetti, con una breve pausa,
    /// così anche senza modello si vede il testo progressivo.
    fn turno_in_flusso(
        &self,
        richiesta: &RichiestaAi,
        pezzo: &mut dyn FnMut(&str),
    ) -> Result<RispostaGrezza, ErroreAi> {
        let risposta = self.turno(richiesta)?;
        let caratteri: Vec<char> = risposta.testo.chars().collect();
        for gruppo in caratteri.chunks(PEZZO_SIMULATO) {
            pezzo(&gruppo.iter().collect::<String>());
            std::thread::sleep(PAUSA_SIMULATA);
        }
        Ok(risposta)
    }

    fn descrizione(&self) -> String {
        "AI simulata (finto)".to_string()
    }
}

/// Caratteri per pezzo nello streaming simulato.
const PEZZO_SIMULATO: usize = 4;
/// Pausa tra un pezzo e l'altro nello streaming simulato.
const PAUSA_SIMULATA: std::time::Duration = std::time::Duration::from_millis(12);

/// Costruisce la risposta JSON per il contesto dato.
pub fn rispondi(c: &ContestoTurno) -> Value {
    let l = c.lingua;
    let voc = vocabolario(l);
    let azione = c.azione.trim().to_lowercase();
    let mut aggiornamenti = json!({});
    let narrazione = if MARCATORI.iter().any(|m| azione.starts_with(m)) {
        descrivi_stanza(c)
    } else if let Some(direzione) = direzione_richiesta(&azione, l) {
        // Le uscite possono essere scritte in un'altra lingua (avventura in
        // inglese giocata in italiano): `nord` trova anche `north`.
        let trovata = c
            .uscite
            .iter()
            .find(|u| u.visibile && lingua::stessa_direzione(&u.direzione, direzione));
        let direzione = lingua::traduci_direzione(direzione, l);
        match trovata {
            Some(u) if u.disponibile => {
                aggiornamenti["stanza_corrente"] = json!(u.verso);
                tr_in!(
                    l,
                    format!("Ti dirigi verso {direzione} e raggiungi {}.", u.nome_verso),
                    format!("You head {direzione} and reach {}.", u.nome_verso),
                    format!(
                        "Vous partez vers {direzione} et arrivez : {}.",
                        u.nome_verso
                    ),
                    format!("Vas hacia {direzione} y llegas a: {}.", u.nome_verso),
                    format!(
                        "Du gehst Richtung {direzione} und erreichst: {}.",
                        u.nome_verso
                    )
                )
            }
            Some(u) => tr_in!(
                l,
                format!(
                    "Provi ad andare verso {direzione}, ma non puoi: {}.",
                    u.motivo
                ),
                format!("You try to go {direzione}, but you can't: {}.", u.motivo),
                format!(
                    "Vous essayez d'aller vers {direzione}, mais c'est impossible : {}.",
                    u.motivo
                ),
                format!(
                    "Intentas ir hacia {direzione}, pero no puedes: {}.",
                    u.motivo
                ),
                format!(
                    "Du versuchst, Richtung {direzione} zu gehen, aber es geht nicht: {}.",
                    u.motivo
                )
            ),
            None => tr_in!(
                l,
                format!("Non c'è nessuna uscita verso {direzione}."),
                format!("There is no exit {direzione}."),
                format!("Il n'y a aucune sortie vers {direzione}."),
                format!("No hay ninguna salida hacia {direzione}."),
                format!("Es gibt keinen Ausgang Richtung {direzione}.")
            ),
        }
    } else if let Some(cosa) = argomento(&azione, voc.prendi, &voc) {
        match trova(&c.oggetti_stanza, cosa) {
            Some(o) => {
                aggiornamenti["inventario_aggiungi"] = json!([o.id]);
                tr_in!(
                    l,
                    format!("Prendi {} e lo metti con le tue cose.", o.nome),
                    format!("You take {} and put it with your things.", o.nome),
                    format!("Vous prenez : {}, et le rangez avec vos affaires.", o.nome),
                    format!("Recoges y guardas con tus cosas: {}.", o.nome),
                    format!("Du nimmst und steckst zu deinen Sachen: {}.", o.nome)
                )
            }
            None => tr_in!(
                l,
                format!("Non vedi niente del genere qui ({cosa})."),
                format!("You see nothing like that here ({cosa})."),
                format!("Vous ne voyez rien de tel ici ({cosa})."),
                format!("No ves nada parecido aquí ({cosa})."),
                format!("Du siehst hier nichts dergleichen ({cosa}).")
            ),
        }
    } else if let Some(cosa) = argomento(&azione, voc.lascia, &voc) {
        match trova(&c.inventario, cosa) {
            Some(o) => {
                aggiornamenti["inventario_rimuovi"] = json!([o.id]);
                aggiornamenti["oggetti_stanza_aggiungi"] = json!({ c.stanza.clone(): [o.id] });
                tr_in!(
                    l,
                    format!("Lasci {} qui, a terra.", o.nome),
                    format!("You leave {} here, on the ground.", o.nome),
                    format!("Vous posez ici, par terre : {}.", o.nome),
                    format!("Dejas aquí, en el suelo: {}.", o.nome),
                    format!("Du legst hier auf den Boden: {}.", o.nome)
                )
            }
            None => tr_in!(
                l,
                format!("Non hai niente del genere con te ({cosa})."),
                format!("You have nothing like that with you ({cosa})."),
                format!("Vous n'avez rien de tel sur vous ({cosa})."),
                format!("No llevas nada parecido ({cosa})."),
                format!("Du hast nichts dergleichen bei dir ({cosa}).")
            ),
        }
    } else if voc.guarda.iter().any(|g| {
        azione == *g
            || azione.starts_with(&format!("{g} "))
            || (g.len() > 1 && azione.starts_with(g))
    }) {
        descrivi_stanza(c)
    } else {
        let a = c.azione.trim().trim_end_matches('.');
        tr_in!(
            l,
            format!("Provi a {a} ma non succede nulla di rilevante."),
            format!("You try to {a} but nothing relevant happens."),
            format!("Vous tentez : « {a} », mais rien d'important ne se passe."),
            format!("Intentas «{a}», pero no pasa nada relevante."),
            format!("Du versuchst „{a}“, aber nichts Wichtiges passiert.")
        )
    };
    let mut memoria = if c.memoria.is_empty() {
        c.azione.trim().to_string()
    } else {
        format!("{} | {}", c.memoria, c.azione.trim())
    };
    let lunghezza = memoria.chars().count();
    if lunghezza > MEMORIA_MAX {
        memoria = memoria.chars().skip(lunghezza - MEMORIA_MAX).collect();
    }
    json!({
        "narrazione": narrazione,
        "aggiornamenti": aggiornamenti,
        "memoria": memoria,
    })
}

fn descrivi_stanza(c: &ContestoTurno) -> String {
    let l = c.lingua;
    let mut testo = if c.descrizione.trim().is_empty() {
        tr_in!(
            l,
            format!("Ti trovi in {}.", c.nome_stanza),
            format!("You are in {}.", c.nome_stanza),
            format!("Vous êtes ici : {}.", c.nome_stanza),
            format!("Estás en: {}.", c.nome_stanza),
            format!("Du befindest dich hier: {}.", c.nome_stanza)
        )
    } else {
        c.descrizione.trim().to_string()
    };
    if !c.oggetti_stanza.is_empty() {
        let nomi: Vec<&str> = c.oggetti_stanza.iter().map(|o| o.nome.as_str()).collect();
        let qui = tr_in!(
            l,
            "Qui vedi:",
            "Here you see:",
            "Ici, vous voyez :",
            "Aquí ves:",
            "Hier siehst du:"
        );
        testo.push_str(&format!(" {qui} {}.", nomi.join(", ")));
    }
    let uscite: Vec<String> = c
        .uscite
        .iter()
        .filter(|u| u.visibile)
        .map(|u| lingua::traduci_direzione(&u.direzione, l))
        .collect();
    if !uscite.is_empty() {
        let etichetta = tr_in!(l, "Uscite:", "Exits:", "Sorties :", "Salidas:", "Ausgänge:");
        testo.push_str(&format!(" {etichetta} {}.", uscite.join(", ")));
    }
    testo
}

/// Riconosce una direzione (nella lingua `l` o in inglese), da sola o dopo un
/// verbo di movimento, e la restituisce in forma inglese (`north`).
fn direzione_richiesta(azione: &str, l: Lingua) -> Option<&'static str> {
    let parole: Vec<&str> = azione
        .split(|c: char| !c.is_alphanumeric())
        .filter(|p| !p.is_empty())
        .collect();
    let (primo, resto) = parole.split_first()?;
    if parole.len() == 1 {
        if let Some(d) = lingua::direzione_canonica(primo, l) {
            return Some(d);
        }
        // Abbreviazioni inglesi: n, s, e, w, u, d.
        if l == Lingua::En {
            let abbreviazioni = [
                ("n", "north"),
                ("s", "south"),
                ("e", "east"),
                ("w", "west"),
                ("u", "up"),
                ("d", "down"),
            ];
            if let Some((_, d)) = abbreviazioni.iter().find(|(a, _)| a == primo) {
                return Some(d);
            }
        }
    }
    let (_, implicita) = vocabolario(l)
        .movimento
        .iter()
        .find(|(v, _)| v == primo)
        .copied()?;
    resto
        .iter()
        .find_map(|p| lingua::direzione_canonica(p, l))
        .or(implicita)
}

/// Se l'azione inizia con uno dei verbi restituisce l'argomento (senza articoli).
fn argomento<'a>(azione: &'a str, verbi: &[&str], voc: &Vocabolario) -> Option<&'a str> {
    let (verbo, resto) = azione.split_once(char::is_whitespace)?;
    if !verbi.contains(&verbo) {
        return None;
    }
    let mut resto = resto.trim();
    // "pick up the key", "pick the key up"
    if let Some(r) = resto.strip_prefix("up ") {
        resto = r.trim();
    }
    for articolo in voc.articoli {
        if let Some(r) = resto.strip_prefix(articolo) {
            resto = r.trim();
            break;
        }
    }
    (!resto.is_empty()).then_some(resto)
}

fn trova<'a>(oggetti: &'a [OggettoNominato], cosa: &str) -> Option<&'a OggettoNominato> {
    let cosa = cosa.to_lowercase();
    oggetti.iter().find(|o| {
        o.id.to_lowercase().contains(&cosa)
            || o.nome.to_lowercase().contains(&cosa)
            || cosa.contains(&o.id.to_lowercase())
    })
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::uscite::InfoUscita;

    fn contesto(azione: &str) -> ContestoTurno {
        let uscita = |dir: &str, verso: &str, disponibile: bool| InfoUscita {
            direzione: dir.into(),
            verso: verso.into(),
            nome_verso: format!("Stanza {verso}"),
            descrizione: String::new(),
            disponibile,
            motivo: if disponibile {
                "ok".into()
            } else {
                "richiede la chiave".into()
            },
            nascosta: false,
            visibile: true,
        };
        ContestoTurno {
            stanza: "a".into(),
            nome_stanza: "Stanza A".into(),
            descrizione: "Una stanza spoglia.".into(),
            uscite: vec![uscita("nord", "b", false), uscita("giu", "c", true)],
            oggetti_stanza: vec![OggettoNominato {
                id: "chiave_ottone".into(),
                nome: "una chiave d'ottone".into(),
            }],
            inventario: vec![OggettoNominato {
                id: "moneta".into(),
                nome: "una moneta".into(),
            }],
            azione: azione.into(),
            memoria: String::new(),
            lingua: crate::lingua::Lingua::It,
        }
    }

    #[test]
    fn movimento() {
        let r = rispondi(&contesto("scendi"));
        assert_eq!(r["aggiornamenti"]["stanza_corrente"], "c");
        let r = rispondi(&contesto("vai a giù"));
        assert_eq!(r["aggiornamenti"]["stanza_corrente"], "c");
        let r = rispondi(&contesto("Nord"));
        assert!(r["aggiornamenti"].get("stanza_corrente").is_none());
        assert!(r["narrazione"].as_str().unwrap().contains("chiave"));
        let r = rispondi(&contesto("vai a est"));
        assert!(r["narrazione"].as_str().unwrap().contains("Non c'è"));
    }

    #[test]
    fn prendi_e_lascia() {
        let r = rispondi(&contesto("prendi la chiave"));
        assert_eq!(
            r["aggiornamenti"]["inventario_aggiungi"][0],
            "chiave_ottone"
        );
        let r = rispondi(&contesto("raccogli ottone"));
        assert_eq!(
            r["aggiornamenti"]["inventario_aggiungi"][0],
            "chiave_ottone"
        );
        let r = rispondi(&contesto("lascia la moneta"));
        assert_eq!(r["aggiornamenti"]["inventario_rimuovi"][0], "moneta");
        assert_eq!(
            r["aggiornamenti"]["oggetti_stanza_aggiungi"]["a"][0],
            "moneta"
        );
        let r = rispondi(&contesto("prendi spada"));
        assert!(r["aggiornamenti"].as_object().unwrap().is_empty());
    }

    #[test]
    fn guarda_inizio_e_generico() {
        let r = rispondi(&contesto("[INIZIO PARTITA] Descrivi la stanza iniziale."));
        let n = r["narrazione"].as_str().unwrap();
        assert!(n.contains("spoglia") && n.contains("chiave") && n.contains("giù"));
        assert!(
            rispondi(&contesto("guarda"))["narrazione"]
                .as_str()
                .unwrap()
                .contains("spoglia")
        );
        let r = rispondi(&contesto("canto una canzone"));
        assert!(
            r["narrazione"]
                .as_str()
                .unwrap()
                .contains("non succede nulla")
        );
        assert_eq!(r["memoria"], "canto una canzone");
    }

    #[test]
    fn in_italiano_con_uscite_inglesi() {
        let italiano = |azione: &str| {
            let mut c = contesto(azione);
            c.lingua = Lingua::It;
            c.uscite[0].direzione = "north".into();
            c.uscite[1].direzione = "down".into();
            rispondi(&c)
        };
        assert_eq!(italiano("vai giù")["aggiornamenti"]["stanza_corrente"], "c");
        let r = italiano("nord");
        assert!(
            r["narrazione"]
                .as_str()
                .unwrap()
                .starts_with("Provi ad andare verso nord")
        );
        let r = italiano("[GAME START] Describe the starting room.");
        assert!(
            r["narrazione"]
                .as_str()
                .unwrap()
                .contains("Uscite: nord, giù")
        );
    }

    #[test]
    fn in_francese_spagnolo_e_tedesco() {
        let in_lingua = |l: Lingua, azione: &str| {
            let mut c = contesto(azione);
            c.lingua = l;
            c.uscite[0].direzione = "north".into();
            c.uscite[1].direzione = "down".into();
            rispondi(&c)
        };
        let stanza = |l, a| in_lingua(l, a)["aggiornamenti"]["stanza_corrente"].clone();
        assert_eq!(stanza(Lingua::Fr, "descends"), "c");
        assert_eq!(stanza(Lingua::Fr, "va en bas"), "c");
        assert_eq!(stanza(Lingua::Es, "baja"), "c");
        assert_eq!(stanza(Lingua::Es, "ve abajo"), "c");
        assert_eq!(stanza(Lingua::De, "geh nach unten"), "c");
        assert_eq!(stanza(Lingua::De, "runter"), "c");
        let testo = |l, a| in_lingua(l, a)["narrazione"].as_str().unwrap().to_string();
        assert!(testo(Lingua::Fr, "nord").contains("vers nord"));
        assert!(testo(Lingua::Es, "norte").contains("hacia norte"));
        assert!(testo(Lingua::De, "norden").contains("Richtung Norden"));
        assert!(testo(Lingua::De, "[GAME START]").contains("Ausgänge: Norden, runter"));
        let r = in_lingua(Lingua::Fr, "prends la chiave");
        assert_eq!(
            r["aggiornamenti"]["inventario_aggiungi"][0],
            "chiave_ottone"
        );
        let r = in_lingua(Lingua::Es, "coge la chiave");
        assert_eq!(
            r["aggiornamenti"]["inventario_aggiungi"][0],
            "chiave_ottone"
        );
        let r = in_lingua(Lingua::De, "nimm die chiave");
        assert_eq!(
            r["aggiornamenti"]["inventario_aggiungi"][0],
            "chiave_ottone"
        );
        assert!(testo(Lingua::Es, "canto").contains("no pasa nada"));
    }

    #[test]
    fn in_inglese() {
        let inglese = |azione: &str| {
            let mut c = contesto(azione);
            c.lingua = Lingua::En;
            c.uscite[0].direzione = "north".into();
            c.uscite[1].direzione = "down".into();
            c.oggetti_stanza[0].nome = "a brass key".into();
            c.inventario[0].nome = "a coin".into();
            rispondi(&c)
        };
        assert_eq!(inglese("go down")["aggiornamenti"]["stanza_corrente"], "c");
        assert_eq!(inglese("d")["aggiornamenti"]["stanza_corrente"], "c");
        let r = inglese("north");
        assert!(
            r["narrazione"]
                .as_str()
                .unwrap()
                .starts_with("You try to go north")
        );
        let r = inglese("pick up the key");
        assert_eq!(
            r["aggiornamenti"]["inventario_aggiungi"][0],
            "chiave_ottone"
        );
        let r = inglese("drop the coin");
        assert_eq!(r["aggiornamenti"]["inventario_rimuovi"][0], "moneta");
        let r = inglese("[GAME START] Describe the starting room.");
        assert!(
            r["narrazione"]
                .as_str()
                .unwrap()
                .contains("Exits: north, down")
        );
        let r = inglese("sing a song");
        assert!(
            r["narrazione"]
                .as_str()
                .unwrap()
                .contains("nothing relevant happens")
        );
    }

    #[test]
    fn memoria_troncata() {
        let mut c = contesto("salto");
        c.memoria = "x".repeat(2000);
        let r = rispondi(&c);
        let m = r["memoria"].as_str().unwrap();
        assert_eq!(m.chars().count(), MEMORIA_MAX);
        assert!(m.ends_with("| salto"));
    }
}
