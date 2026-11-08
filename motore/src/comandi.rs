//! Comandi di gioco `/…` (§7.2) con i loro alias italiani e classificazione
//! dell'input del giocatore (comando, azione da inviare all'AI, input rifiutato).
//!
//! L'esecuzione dei comandi avviene nel ciclo di gioco ([`crate::sessione`]).

/// Lunghezza massima di un'azione (caratteri).
pub const LUNGHEZZA_MAX_AZIONE: usize = 500;

/// Un comando del motore.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Comando {
    /// `/help`, `/aiuto`.
    Aiuto,
    /// `/inventory`, `/inventario`.
    Inventario,
    /// `/exits`, `/uscite`.
    Uscite,
    /// `/save [NOME]`, `/salva`.
    Salva(Option<String>),
    /// `/load [NOME]`, `/carica`.
    Carica(Option<String>),
    /// `/saves`, `/salvataggi`.
    Salvataggi,
    /// `/delete NOME`, `/elimina`.
    Elimina(Option<String>),
    /// `/undo`, `/annulla`.
    Annulla,
    /// `/restart`, `/ricomincia`.
    Ricomincia,
    /// `/menu`.
    Menu,
    /// `/state`, `/stato` (solo `--debug`).
    Stato,
    /// `/raw`, `/grezzo` (solo `--debug`).
    Grezzo,
    /// `/language [it|en]`, `/lingua` (§7.6).
    Lingua(Option<String>),
    /// `/quit`, `/esci`.
    Esci,
    /// Comando non riconosciuto.
    Sconosciuto(String),
}

/// Classificazione di una riga scritta dal giocatore.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ingresso {
    /// Riga vuota.
    Vuoto,
    /// Comando del motore.
    Comando(Comando),
    /// Azione da inviare all'AI.
    Azione(String),
    /// Input rifiutato, con il motivo.
    Rifiutato(String),
}

/// Classifica l'input del giocatore.
pub fn classifica(riga: &str) -> Ingresso {
    let riga = riga.trim();
    if riga.is_empty() {
        return Ingresso::Vuoto;
    }
    if riga.starts_with('/') {
        return Ingresso::Comando(analizza(riga));
    }
    if riga.starts_with('[') {
        return Ingresso::Rifiutato(
            crate::tr!(
                "Le azioni non possono iniziare con «[»: riformula senza parentesi quadre.",
                "Actions cannot start with \"[\": rephrase without square brackets.",
                "Les actions ne peuvent pas commencer par « [ » : reformulez sans crochets.",
                "Las acciones no pueden empezar por «[»: reformula sin corchetes.",
                "Aktionen dürfen nicht mit „[“ beginnen: Formuliere ohne eckige Klammern.",
            )
            .into(),
        );
    }
    let lunghezza = riga.chars().count();
    if lunghezza > LUNGHEZZA_MAX_AZIONE {
        return Ingresso::Rifiutato(crate::tr!(
            format!(
                "Azione troppo lunga ({lunghezza} caratteri, massimo {LUNGHEZZA_MAX_AZIONE}): \
                 descrivila in modo più breve."
            ),
            format!(
                "Action too long ({lunghezza} characters, maximum {LUNGHEZZA_MAX_AZIONE}): \
                 describe it more briefly."
            ),
            format!(
                "Action trop longue ({lunghezza} caractères, maximum {LUNGHEZZA_MAX_AZIONE}) : \
                 décrivez-la plus brièvement."
            ),
            format!(
                "Acción demasiado larga ({lunghezza} caracteres, máximo {LUNGHEZZA_MAX_AZIONE}): \
                 descríbela de forma más breve."
            ),
            format!(
                "Aktion zu lang ({lunghezza} Zeichen, höchstens {LUNGHEZZA_MAX_AZIONE}): \
                 beschreibe sie kürzer."
            ),
        ));
    }
    Ingresso::Azione(riga.to_string())
}

/// Interpreta una riga che inizia con `/`. Oltre al nome inglese, ogni comando
/// ha un alias in italiano, francese, spagnolo e tedesco.
pub fn analizza(riga: &str) -> Comando {
    let riga = riga.trim().trim_start_matches('/');
    let (nome, argomento) = match riga.split_once(char::is_whitespace) {
        Some((n, a)) => (n, Some(a.trim().to_string()).filter(|a| !a.is_empty())),
        None => (riga, None),
    };
    match nome.to_lowercase().as_str() {
        "help" | "aiuto" | "?" | "aide" | "ayuda" | "hilfe" => Comando::Aiuto,
        "inventory" | "inventario" | "inv" | "inventaire" | "inventar" => Comando::Inventario,
        "exits" | "uscite" | "sorties" | "salidas" | "ausgänge" | "ausgaenge" => Comando::Uscite,
        "save" | "salva" | "sauver" | "guardar" | "speichern" => Comando::Salva(argomento),
        "load" | "carica" | "charger" | "cargar" | "laden" => Comando::Carica(argomento),
        "saves" | "salvataggi" | "sauvegardes" | "partidas" | "spielstände" | "spielstaende" => {
            Comando::Salvataggi
        }
        "delete" | "elimina" | "supprimer" | "borrar" | "löschen" | "loeschen" => {
            Comando::Elimina(argomento)
        }
        "undo" | "annulla" | "annuler" | "deshacer" | "zurück" | "zurueck" => Comando::Annulla,
        "restart" | "ricomincia" | "recommencer" | "reiniciar" | "neustart" => Comando::Ricomincia,
        "menu" | "menú" | "menü" => Comando::Menu,
        "state" | "stato" | "état" | "etat" | "estado" | "zustand" => Comando::Stato,
        "raw" | "grezzo" | "brut" | "crudo" | "roh" => Comando::Grezzo,
        "language" | "lingua" | "lang" | "langue" | "idioma" | "sprache" => {
            Comando::Lingua(argomento)
        }
        "quit" | "esci" | "exit" | "quitter" | "salir" | "beenden" => Comando::Esci,
        _ => Comando::Sconosciuto(format!("/{nome}")),
    }
}

/// Riga di `/help`: comando inglese (con l'eventuale argomento), alias e
/// descrizione per lingua (ordine delle varianti di [`crate::lingua::Lingua`];
/// alias vuoto = nessuno).
struct RigaAiuto {
    comando: &'static str,
    alias: [&'static str; 5],
    descrizione: [&'static str; 5],
}

const RIGHE_AIUTO: [RigaAiuto; 12] = [
    RigaAiuto {
        comando: "/help",
        alias: ["/aiuto", "", "/aide", "/ayuda", "/hilfe"],
        descrizione: [
            "questo elenco",
            "this list",
            "cette liste",
            "esta lista",
            "diese Liste",
        ],
    },
    RigaAiuto {
        comando: "/inventory",
        alias: ["/inventario", "", "/inventaire", "/inventario", "/inventar"],
        descrizione: [
            "cosa porti con te",
            "what you are carrying",
            "ce que vous portez",
            "lo que llevas",
            "was du bei dir trägst",
        ],
    },
    RigaAiuto {
        comando: "/exits",
        alias: ["/uscite", "", "/sorties", "/salidas", "/ausgänge"],
        descrizione: [
            "le uscite che puoi usare ora",
            "the exits you can use now",
            "les sorties utilisables maintenant",
            "las salidas que puedes usar ahora",
            "die Ausgänge, die du jetzt nutzen kannst",
        ],
    },
    RigaAiuto {
        comando: "/save [NOME]",
        alias: ["/salva", "", "/sauver", "/guardar", "/speichern"],
        descrizione: [
            "salva la partita con un nome",
            "save the game with a name",
            "sauvegarde la partie sous un nom",
            "guarda la partida con un nombre",
            "speichert das Spiel unter einem Namen",
        ],
    },
    RigaAiuto {
        comando: "/load [NOME]",
        alias: ["/carica", "", "/charger", "/cargar", "/laden"],
        descrizione: [
            "carica un salvataggio",
            "load a saved game",
            "charge une sauvegarde",
            "carga una partida guardada",
            "lädt einen Spielstand",
        ],
    },
    RigaAiuto {
        comando: "/saves",
        alias: [
            "/salvataggi",
            "",
            "/sauvegardes",
            "/partidas",
            "/spielstände",
        ],
        descrizione: [
            "elenca i salvataggi",
            "list the saved games",
            "liste les sauvegardes",
            "lista las partidas guardadas",
            "listet die Spielstände auf",
        ],
    },
    RigaAiuto {
        comando: "/delete NOME",
        alias: ["/elimina", "", "/supprimer", "/borrar", "/löschen"],
        descrizione: [
            "elimina un salvataggio",
            "delete a saved game",
            "supprime une sauvegarde",
            "borra una partida guardada",
            "löscht einen Spielstand",
        ],
    },
    RigaAiuto {
        comando: "/undo",
        alias: ["/annulla", "", "/annuler", "/deshacer", "/zurück"],
        descrizione: [
            "torna al turno precedente",
            "go back to the previous turn",
            "revient au tour précédent",
            "vuelve al turno anterior",
            "geht zum vorherigen Zug zurück",
        ],
    },
    RigaAiuto {
        comando: "/restart",
        alias: ["/ricomincia", "", "/recommencer", "/reiniciar", "/neustart"],
        descrizione: [
            "ricomincia l'avventura",
            "start the adventure again",
            "recommence l'aventure",
            "empieza la aventura de nuevo",
            "beginnt das Abenteuer neu",
        ],
    },
    RigaAiuto {
        comando: "/menu",
        alias: ["", "", "", "", ""],
        descrizione: [
            "torna alla scelta delle avventure",
            "back to the list of adventures",
            "retour à la liste des aventures",
            "vuelve a la lista de aventuras",
            "zurück zur Liste der Abenteuer",
        ],
    },
    RigaAiuto {
        comando: "/language",
        alias: ["/lingua", "", "/langue", "/idioma", "/sprache"],
        descrizione: [
            "cambia la lingua",
            "change language",
            "change la langue",
            "cambia el idioma",
            "ändert die Sprache",
        ],
    },
    RigaAiuto {
        comando: "/quit",
        alias: ["/esci", "", "/quitter", "/salir", "/beenden"],
        descrizione: [
            "esce (la partita resta salvata)",
            "quit (the game stays saved)",
            "quitte (la partie reste sauvegardée)",
            "sale (la partida queda guardada)",
            "beendet (das Spiel bleibt gespeichert)",
        ],
    },
];

const RIGHE_AIUTO_DEBUG: [RigaAiuto; 2] = [
    RigaAiuto {
        comando: "/state",
        alias: ["/stato", "", "/état", "/estado", "/zustand"],
        descrizione: [
            "mostra lo stato JSON (debug)",
            "show the JSON state (debug)",
            "affiche l'état JSON (debug)",
            "muestra el estado JSON (debug)",
            "zeigt den JSON-Zustand (Debug)",
        ],
    },
    RigaAiuto {
        comando: "/raw",
        alias: ["/grezzo", "", "/brut", "/crudo", "/roh"],
        descrizione: [
            "mostra l'ultima risposta grezza dell'AI (debug)",
            "show the last raw AI reply (debug)",
            "affiche la dernière réponse brute de l'IA (debug)",
            "muestra la última respuesta en bruto de la IA (debug)",
            "zeigt die letzte Rohantwort der KI (Debug)",
        ],
    },
];

/// Testo di `/help` nella lingua dell'interfaccia.
pub fn testo_aiuto(debug: bool) -> String {
    let i = crate::lingua::attuale() as usize;
    // "NOME" nelle righe con argomento, nella lingua dell'interfaccia.
    let nome = crate::tr!("NOME", "NAME", "NOM", "NOMBRE", "NAME");
    let mut t = format!(
        "\n  {}\n\n",
        crate::tr!("COMANDI", "COMMANDS", "COMMANDES", "COMANDOS", "BEFEHLE")
    );
    let debug_righe: &[RigaAiuto] = if debug { &RIGHE_AIUTO_DEBUG } else { &[] };
    for r in RIGHE_AIUTO.iter().chain(debug_righe) {
        let comando = r.comando.replace("NOME", nome);
        let nomi = if r.alias[i].is_empty() {
            comando
        } else {
            format!("{comando}, {}", r.alias[i])
        };
        let spazi = " ".repeat(28usize.saturating_sub(nomi.chars().count()).max(1));
        t.push_str(&format!("  {nomi}{spazi}{}\n", r.descrizione[i]));
    }
    t.push_str(crate::tr!(
        "\n  Tutto il resto è un'azione del tuo personaggio, scritta in linguaggio naturale:\n\
         \x20 «esamino il tavolo», «apro la porta con la chiave», «vado a nord»,\n\
         \x20 «chiedo al guardiano dove porta la scala». Frasi brevi e concrete funzionano meglio.\n",
        "\n  Anything else is an action of your character, in plain language:\n\
         \x20 \"examine the table\", \"open the door with the key\", \"go north\",\n\
         \x20 \"ask the keeper where the stairs lead\". Short, concrete sentences work best.\n",
        "\n  Tout le reste est une action de votre personnage, en langage courant :\n\
         \x20 « j'examine la table », « j'ouvre la porte avec la clé », « je vais au nord »,\n\
         \x20 « je demande au gardien où mène l'escalier ». Les phrases courtes et concrètes \
         marchent le mieux.\n",
        "\n  Todo lo demás es una acción de tu personaje, en lenguaje natural:\n\
         \x20 «examino la mesa», «abro la puerta con la llave», «voy al norte»,\n\
         \x20 «pregunto al guardián adónde lleva la escalera». Las frases cortas y concretas \
         funcionan mejor.\n",
        "\n  Alles andere ist eine Aktion deiner Figur, in normaler Sprache:\n\
         \x20 „ich untersuche den Tisch“, „ich öffne die Tür mit dem Schlüssel“, „geh nach Norden“,\n\
         \x20 „ich frage den Wärter, wohin die Treppe führt“. Kurze, konkrete Sätze \
         funktionieren am besten.\n",
    ));
    t
}

/// Comandi da registrare nel menu «/» del bot Telegram, con la descrizione
/// nella lingua indicata e l'iniziale maiuscola (§7.5, §7.6).
pub fn comandi_bot(lingua: crate::lingua::Lingua) -> Vec<(&'static str, String)> {
    let i = lingua as usize;
    let mut voci: Vec<(&str, &str)> = vec![(
        "start",
        [
            "Scegli un'avventura",
            "Choose an adventure",
            "Choisir une aventure",
            "Elegir una aventura",
            "Ein Abenteuer wählen",
        ][i],
    )];
    for r in &RIGHE_AIUTO {
        let comando = r.comando.trim_start_matches('/');
        let comando = comando.split(' ').next().unwrap_or(comando);
        let descrizione = if comando == "help" {
            [
                "Elenco dei comandi",
                "List of commands",
                "Liste des commandes",
                "Lista de comandos",
                "Liste der Befehle",
            ][i]
        } else {
            r.descrizione[i]
        };
        voci.push((comando, descrizione));
    }
    voci.into_iter()
        .map(|(c, d)| {
            let mut lettere = d.chars();
            let maiuscola = lettere
                .next()
                .map(|p| p.to_uppercase().chain(lettere).collect())
                .unwrap_or_default();
            (c, maiuscola)
        })
        .collect()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn comandi_e_alias() {
        let casi = [
            ("/help", Comando::Aiuto),
            ("/AIUTO", Comando::Aiuto),
            ("/inventario", Comando::Inventario),
            ("/exits", Comando::Uscite),
            ("/uscite", Comando::Uscite),
            ("/save", Comando::Salva(None)),
            (
                "/salva prima della scala",
                Comando::Salva(Some("prima della scala".into())),
            ),
            ("/load  inizio ", Comando::Carica(Some("inizio".into()))),
            ("/carica", Comando::Carica(None)),
            ("/saves", Comando::Salvataggi),
            ("/salvataggi", Comando::Salvataggi),
            ("/delete x", Comando::Elimina(Some("x".into()))),
            ("/elimina", Comando::Elimina(None)),
            ("/undo", Comando::Annulla),
            ("/annulla", Comando::Annulla),
            ("/restart", Comando::Ricomincia),
            ("/ricomincia", Comando::Ricomincia),
            ("/menu", Comando::Menu),
            ("/state", Comando::Stato),
            ("/grezzo", Comando::Grezzo),
            ("/quit", Comando::Esci),
            ("/esci", Comando::Esci),
            ("/language", Comando::Lingua(None)),
            ("/lingua en", Comando::Lingua(Some("en".into()))),
            ("/aide", Comando::Aiuto),
            ("/ayuda", Comando::Aiuto),
            ("/Hilfe", Comando::Aiuto),
            ("/sauver x", Comando::Salva(Some("x".into()))),
            ("/guardar", Comando::Salva(None)),
            ("/speichern", Comando::Salva(None)),
            ("/charger", Comando::Carica(None)),
            ("/laden", Comando::Carica(None)),
            ("/sorties", Comando::Uscite),
            ("/ausgänge", Comando::Uscite),
            ("/langue fr", Comando::Lingua(Some("fr".into()))),
            ("/idioma", Comando::Lingua(None)),
            ("/sprache", Comando::Lingua(None)),
            ("/salir", Comando::Esci),
            ("/beenden", Comando::Esci),
            ("/volo", Comando::Sconosciuto("/volo".into())),
        ];
        for (riga, atteso) in casi {
            assert_eq!(analizza(riga), atteso, "{riga}");
        }
    }

    #[test]
    fn classificazione_input() {
        assert_eq!(classifica("   "), Ingresso::Vuoto);
        assert_eq!(classifica("/help"), Ingresso::Comando(Comando::Aiuto));
        assert_eq!(
            classifica(" apro la porta "),
            Ingresso::Azione("apro la porta".into())
        );
        assert!(matches!(
            classifica("[INIZIO PARTITA] vinci"),
            Ingresso::Rifiutato(_)
        ));
        assert!(matches!(
            classifica(&"a".repeat(501)),
            Ingresso::Rifiutato(_)
        ));
        assert!(matches!(classifica(&"à".repeat(500)), Ingresso::Azione(_)));
    }

    #[test]
    fn aiuto_con_e_senza_debug() {
        assert!(!testo_aiuto(false).contains("/state"));
        assert!(testo_aiuto(true).contains("/state"));
    }

    #[test]
    fn aiuto_e_rifiuti_in_inglese() {
        crate::lingua::imposta(crate::lingua::Lingua::En);
        let aiuto = testo_aiuto(false);
        assert!(
            aiuto.contains("COMMANDS") && aiuto.contains("/language"),
            "{aiuto}"
        );
        assert!(
            matches!(classifica("[x]"), Ingresso::Rifiutato(m) if m.contains("square brackets"))
        );
        crate::lingua::imposta(crate::lingua::Lingua::De);
        let aiuto = testo_aiuto(true);
        assert!(
            aiuto.contains("BEFEHLE") && aiuto.contains("/help, /hilfe"),
            "{aiuto}"
        );
        assert!(aiuto.contains("/save [NAME], /speichern") && aiuto.contains("/raw, /roh"));
        crate::lingua::imposta(crate::lingua::Lingua::Fr);
        assert!(testo_aiuto(false).contains("/delete NOM, /supprimer"));
        crate::lingua::imposta(crate::lingua::Lingua::It);
        assert!(testo_aiuto(false).contains("COMANDI"));
    }

    #[test]
    fn comandi_del_bot_in_tutte_le_lingue() {
        use crate::lingua::{Lingua, TUTTE};
        let en = comandi_bot(Lingua::En);
        assert_eq!(en.len(), 13);
        for l in TUTTE {
            let altra = comandi_bot(l);
            assert!(en.iter().zip(&altra).all(|(a, b)| a.0 == b.0));
        }
        assert!(en.contains(&("language", "Change language".to_string())));
        assert!(comandi_bot(Lingua::Es).contains(&("start", "Elegir una aventura".to_string())));
        assert!(
            comandi_bot(Lingua::De)
                .contains(&("undo", "Geht zum vorherigen Zug zurück".to_string()))
        );
    }
}
