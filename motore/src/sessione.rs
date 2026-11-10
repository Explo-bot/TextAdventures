//! Una sessione di gioco per un giocatore: menu di avvio (scelta
//! dell'avventura e della partita, §7.1), turno di apertura o di ripresa,
//! ciclo di lettura dell'input con esecuzione dei comandi `/…` (§7.2) e delle
//! azioni, riquadro finale.
//!
//! La sessione è indipendente dal canale: tutto passa per un [`Canale`] e le
//! cartelle arrivano dall'[`Ambiente`]. Il terminale usa `partite/<id>/…`; il bot
//! Telegram (script 03) userà la stessa sessione con
//! `partite/telegram/<id_utente>/` come cartella delle partite.

use std::path::PathBuf;

use anyhow::Result;

use crate::ai::Narratore;
use crate::avventura::Avventura;
use crate::canale::{Canale, Input, Stile};
use crate::catalogo_avventure::{InfoPartita, risolvi_avventura, scansiona};
use crate::comandi::{Comando, testo_aiuto};
use crate::config::Config;
use crate::lingua::{self, Lingua};
use crate::menu::{
    self, Risposta3, SceltaAvventura, SceltaPartita, domanda_sostituzione, domanda_tre_vie,
    testo_turni_non_salvati,
};
use crate::motore::{EsitoTurno, Partita};
use crate::prompt::AzioneTurno;
use crate::salvataggi::{GestoreSalvataggi, progressi_non_salvati};
use crate::stato::{self, Esito, PercorsiPartita, Stato};
use crate::tr;
use crate::uscite;

/// Indirizzo del progetto su GitHub.
pub const URL_PROGETTO: &str = "https://github.com/Explo-bot/TextAdventures";

/// Sito dove si legge gratuitamente il racconto da cui è tratta l'avventura
/// `vinny-falco`.
pub const URL_RACCONTO: &str = "https://gronk.it";

/// Avviso mostrato a ogni avvio (e a ogni `/start` del bot): il programma è
/// open source e il codice è su GitHub; il racconto «Vinny Falco Detective»,
/// da cui è tratta un'avventura, è gratuito su gronk.it.
pub fn avviso_open_source() -> String {
    tr!(
        format!(
            "Avventura è un programma open source: il codice è disponibile su GitHub all'indirizzo {URL_PROGETTO}\n\
             Il racconto «Vinny Falco Detective» di Peter A. Ray, da cui è tratta l'avventura di Vinny Falco, è disponibile gratuitamente su {URL_RACCONTO}"
        ),
        format!(
            "Avventura is open source software: the code is available on GitHub at {URL_PROGETTO}\n\
             The story \"Vinny Falco Detective\" by Peter A. Ray, on which the Vinny Falco adventure is based, is available for free at {URL_RACCONTO}"
        ),
        format!(
            "Avventura est un logiciel libre (open source) : le code est disponible sur GitHub à l'adresse {URL_PROGETTO}\n\
             Le récit « Vinny Falco Detective » de Peter A. Ray, dont est tirée l'aventure de Vinny Falco, est disponible gratuitement sur {URL_RACCONTO}"
        ),
        format!(
            "Avventura es un programa de código abierto: el código está disponible en GitHub en {URL_PROGETTO}\n\
             El relato «Vinny Falco Detective» de Peter A. Ray, en el que se basa la aventura de Vinny Falco, está disponible gratis en {URL_RACCONTO}"
        ),
        format!(
            "Avventura ist Open-Source-Software: Der Code ist auf GitHub verfügbar unter {URL_PROGETTO}\n\
             Die Erzählung „Vinny Falco Detective“ von Peter A. Ray, auf der das Vinny-Falco-Abenteuer beruht, gibt es kostenlos unter {URL_RACCONTO}"
        )
    )
}

/// Opzioni di avvio della sessione (`avventura gioca`).
#[derive(Debug, Clone, Default)]
pub struct OpzioniGioco {
    /// `--avventura ID|DIR`.
    pub avventura: Option<String>,
    /// `--carica NOME`.
    pub carica: Option<String>,
    /// `--nuova`.
    pub nuova: bool,
    /// `--debug` (abilita `/state` e `/raw`).
    pub debug: bool,
    /// `--lingua`: lingua dell'interfaccia imposta dalla riga di comando (§7.6).
    pub lingua: Option<Lingua>,
    /// Lingua rilevata dall'ambiente (sistema operativo o app Telegram): è la
    /// proposta della domanda al primo utilizzo. `None` = inglese.
    pub lingua_rilevata: Option<Lingua>,
}

/// Cartelle e configurazione della sessione.
#[derive(Debug, Clone)]
pub struct Ambiente<'a> {
    /// Configurazione.
    pub config: &'a Config,
    /// Cartella con una sottocartella per avventura.
    pub cartella_avventure: PathBuf,
    /// Cartella delle partite (una sottocartella per avventura).
    pub cartella_partite: PathBuf,
}

/// Come termina il ciclo di gioco.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FineCiclo {
    /// Torna al menu delle avventure.
    Menu,
    /// Esce dal programma (o chiude la partita, nel bot).
    Esci,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Avvio {
    Nuova,
    Ripresa,
}

fn arrivederci() -> &'static str {
    tr!(
        "Arrivederci! La partita in corso è salvata automaticamente.",
        "Goodbye! The game in progress is saved automatically.",
        "Au revoir ! La partie en cours est sauvegardée automatiquement.",
        "¡Adiós! La partida en curso se guarda automáticamente.",
        "Auf Wiedersehen! Das laufende Spiel wird automatisch gespeichert."
    )
}

fn si_continua() -> &'static str {
    tr!(
        "Ok, si continua.",
        "Ok, let's go on.",
        "D'accord, on continue.",
        "Vale, seguimos.",
        "Gut, weiter geht's."
    )
}

/// Sceglie e imposta la lingua dell'interfaccia all'avvio della sessione (§7.6):
/// `--lingua` → preferenza salvata → `AVVENTURA_LINGUA` o `interfaccia.lingua` →
/// domanda al primo utilizzo (solo con un canale interattivo, proponendo la
/// lingua rilevata) → lingua rilevata → inglese.
pub fn prepara_lingua(
    ambiente: &Ambiente<'_>,
    opzioni: &OpzioniGioco,
    canale: &mut dyn Canale,
) -> Result<Lingua> {
    let fissata = || {
        lingua::da_variabile().or_else(|| Lingua::da_codice(&ambiente.config.interfaccia.lingua))
    };
    let scelta = match opzioni
        .lingua
        .or_else(|| lingua::preferita(&ambiente.cartella_partite))
        .or_else(fissata)
    {
        Some(l) => l,
        None => {
            let proposta = opzioni.lingua_rilevata.unwrap_or(Lingua::En);
            lingua::imposta(proposta);
            if canale.interattivo() {
                // Annullare la domanda vale come accettare la proposta: la si
                // salva comunque, così non viene ripetuta.
                let l = menu::scegli_lingua(canale, proposta)?.unwrap_or(proposta);
                salva_lingua(ambiente, l, canale)?;
                l
            } else {
                proposta
            }
        }
    };
    lingua::imposta(scelta);
    Ok(scelta)
}

/// Salva la lingua nelle preferenze del giocatore (un errore è solo un avviso).
fn salva_lingua(ambiente: &Ambiente<'_>, l: Lingua, canale: &mut dyn Canale) -> Result<()> {
    if let Err(e) = lingua::salva_preferita(&ambiente.cartella_partite, l) {
        canale.mostra(
            &crate::tr_in!(
                l,
                format!("Preferenza della lingua non salvata: {e:#}"),
                format!("Language preference not saved: {e:#}"),
                format!("Préférence de langue non enregistrée : {e:#}"),
                format!("No se ha guardado la preferencia de idioma: {e:#}"),
                format!("Spracheinstellung nicht gespeichert: {e:#}")
            ),
            Stile::Avviso,
        )?;
    }
    Ok(())
}

/// Menu della lingua: imposta e salva la scelta. `None` = annullato.
fn cambia_lingua(ambiente: &Ambiente<'_>, canale: &mut dyn Canale) -> Result<Option<Lingua>> {
    let Some(l) = menu::scegli_lingua(canale, lingua::attuale())? else {
        return Ok(None);
    };
    imposta_e_salva(ambiente, l, canale)?;
    Ok(Some(l))
}

fn imposta_e_salva(ambiente: &Ambiente<'_>, l: Lingua, canale: &mut dyn Canale) -> Result<()> {
    lingua::imposta(l);
    salva_lingua(ambiente, l, canale)?;
    canale.mostra(
        tr!(
            "Lingua impostata: Italiano.",
            "Language set to English.",
            "Langue choisie : Français.",
            "Idioma elegido: Español.",
            "Sprache eingestellt: Deutsch."
        ),
        Stile::Motore,
    )
}

fn caricato(nome: &str, turno: u32) -> String {
    tr!(
        format!("Caricato «{nome}» (turno {turno})."),
        format!("Loaded \"{nome}\" (turn {turno})."),
        format!("« {nome} » chargée (tour {turno})."),
        format!("Cargada «{nome}» (turno {turno})."),
        format!("„{nome}“ geladen (Zug {turno}).")
    )
}

/// Avvia la sessione: menu, gioco, ritorno al menu finché il giocatore non esce.
pub fn gioca(
    ambiente: &Ambiente<'_>,
    opzioni: &OpzioniGioco,
    canale: &mut dyn Canale,
    narratore: &dyn Narratore,
) -> Result<()> {
    prepara_lingua(ambiente, opzioni, canale)?;
    canale.mostra(&avviso_open_source(), Stile::Motore)?;
    for avviso in &ambiente.config.ai.avvisi {
        canale.mostra(avviso, Stile::Avviso)?;
    }
    let mut diretta = opzioni.avventura.clone();
    let mut carica_subito = opzioni.carica.clone();
    let mut nuova_subito = opzioni.nuova;
    loop {
        // Passo 1: scelta dell'avventura.
        let da_opzione = diretta.is_some();
        let cartella = match diretta.take() {
            Some(arg) => risolvi_avventura(&arg, &ambiente.cartella_avventure)?,
            None => {
                let voci = scansiona(&ambiente.cartella_avventure, &ambiente.cartella_partite);
                match menu::scegli_avventura(canale, &voci, &ambiente.cartella_avventure)? {
                    SceltaAvventura::Esci => return Ok(()),
                    SceltaAvventura::Lingua => {
                        cambia_lingua(ambiente, canale)?;
                        continue;
                    }
                    SceltaAvventura::Avventura(i) => voci[i].cartella.clone(),
                }
            }
        };
        let avventura = match Avventura::carica(&cartella) {
            Ok(a) => a,
            Err(e) if da_opzione => return Err(e),
            Err(e) => {
                canale.mostra(&format!("{e:#}"), Stile::Errore)?;
                continue;
            }
        };

        // Passo 2: scelta della partita.
        let percorsi = PercorsiPartita::nuovi(&ambiente.cartella_partite, &avventura.id);
        let gestore = GestoreSalvataggi::per_avventura(percorsi.clone(), &avventura);
        let da_cli = carica_subito.is_some() || nuova_subito;
        let scelta = if let Some(nome) = carica_subito.take() {
            SceltaPartita::Carica(nome)
        } else if std::mem::take(&mut nuova_subito) {
            // `--nuova` con progressi non salvati: stessa conferma del menu.
            if let Ok(Some(stato)) = Stato::carica_se_esiste(&percorsi.stato())
                && stato.esito == Esito::InCorso
                && progressi_non_salvati(&stato)
                && !canale.conferma(&domanda_sostituzione(&stato))?
            {
                canale.mostra(
                    tr!(
                        "Operazione annullata.",
                        "Operation cancelled.",
                        "Opération annulée.",
                        "Operación cancelada.",
                        "Vorgang abgebrochen."
                    ),
                    Stile::Motore,
                )?;
                return Ok(());
            }
            SceltaPartita::Nuova
        } else {
            // Una partita di una versione precedente dell'avventura (stanza che
            // non esiste più) si tratta come illeggibile.
            let letta = Stato::carica_se_esiste(&percorsi.stato()).and_then(|s| match s {
                Some(s) if avventura.stanza(&s.stanza_corrente).is_none() => Err(anyhow::anyhow!(
                    crate::salvataggi::versione_precedente(&s.stanza_corrente)
                )),
                altro => Ok(altro),
            });
            let in_corso = match letta {
                Ok(s) => s,
                Err(e) => {
                    canale.mostra(
                        &tr!(
format!(
                                "La partita in corso non è leggibile ({e:#}). Puoi caricare un \
                                 salvataggio o iniziare una nuova partita."
                            ),
format!(
                                "The game in progress cannot be read ({e:#}). You can load a \
                                 save or start a new game."
                            ),
format!("La partie en cours est illisible ({e:#}). Vous pouvez charger une sauvegarde ou commencer une nouvelle partie."),
format!("No se puede leer la partida en curso ({e:#}). Puedes cargar una partida guardada o empezar una nueva."),
format!("Das laufende Spiel kann nicht gelesen werden ({e:#}). Du kannst einen Spielstand laden oder ein neues Spiel beginnen.")
),
                        Stile::Errore,
                    )?;
                    None
                }
            };
            let info = in_corso.as_ref().map(|s| InfoPartita {
                turno: s.turno,
                nome_stanza: avventura.nome_stanza(&s.stanza_corrente),
                aggiornato_il: s.aggiornato_il,
                esito: s.esito,
            });
            menu::scegli_partita(
                canale,
                &avventura.titolo(),
                info.as_ref().zip(in_corso.as_ref()),
                &gestore.elenca(),
            )?
        };

        let (partita, avvio) = match scelta {
            SceltaPartita::Indietro => continue,
            SceltaPartita::Nuova => (
                Partita::nuova(&avventura, ambiente.config, &ambiente.cartella_partite)?,
                Avvio::Nuova,
            ),
            SceltaPartita::Continua => (
                Partita::riprendi(&avventura, ambiente.config, &ambiente.cartella_partite)?,
                Avvio::Ripresa,
            ),
            SceltaPartita::Carica(nome) => match gestore.carica(&nome) {
                Ok(stato) => {
                    canale.mostra(
                        &caricato(
                            stato.salvataggio_corrente.as_deref().unwrap_or(&nome),
                            stato.turno,
                        ),
                        Stile::Motore,
                    )?;
                    (
                        Partita::con_stato(
                            &avventura,
                            ambiente.config,
                            &ambiente.cartella_partite,
                            stato,
                        ),
                        Avvio::Ripresa,
                    )
                }
                Err(e) if da_cli => return Err(e),
                Err(e) => {
                    canale.mostra(&format!("{e:#}"), Stile::Errore)?;
                    continue;
                }
            },
        };

        // Passo 3: gioco.
        match ciclo_di_gioco(ambiente, partita, avvio, canale, narratore, opzioni.debug)? {
            FineCiclo::Menu => continue,
            FineCiclo::Esci => return Ok(()),
        }
    }
}

fn ciclo_di_gioco(
    ambiente: &Ambiente<'_>,
    mut partita: Partita<'_>,
    avvio: Avvio,
    canale: &mut dyn Canale,
    narratore: &dyn Narratore,
    debug: bool,
) -> Result<FineCiclo> {
    match avvio {
        Avvio::Nuova => inizia_nuova(&mut partita, canale, narratore)?,
        Avvio::Ripresa => riprendi(&mut partita, canale, narratore)?,
    }
    loop {
        match canale.leggi_azione()? {
            Input::Fine => {
                canale.mostra(arrivederci(), Stile::Motore)?;
                return Ok(FineCiclo::Esci);
            }
            Input::Azione(azione) => {
                rileggi(&mut partita);
                if partita.stato.esito.concluso() {
                    canale.mostra(
                        tr!(
"La partita è conclusa: usa /restart per ricominciare, /load per \
                             caricare un salvataggio, /menu o /quit.",
"The game is over: use /restart to start again, /load to load a \
                             save, /menu or /quit.",
"La partie est terminée : /restart pour recommencer, /load pour charger une sauvegarde, /menu ou /quit.",
"La partida ha terminado: usa /restart para empezar de nuevo, /load para cargar una partida guardada, /menu o /quit.",
"Das Spiel ist vorbei: /restart zum Neustarten, /load zum Laden eines Spielstands, /menu oder /quit."
),
                        Stile::Motore,
                    )?;
                    continue;
                }
                let esito =
                    partita.esegui_turno(narratore, AzioneTurno::Giocatore(azione), canale)?;
                mostra_esito(&partita, esito, canale)?;
            }
            Input::Comando(comando) => {
                rileggi(&mut partita);
                if let Some(fine) =
                    esegui_comando(ambiente, comando, &mut partita, canale, narratore, debug)?
                {
                    return Ok(fine);
                }
            }
        }
    }
}

/// Riallinea lo stato in memoria con `stato.json` (potrebbe essere stato modificato a mano).
fn rileggi(partita: &mut Partita<'_>) {
    if let Ok(Some(s)) = Stato::carica_se_esiste(&partita.percorsi.stato()) {
        partita.stato = s;
    }
}

fn inizia_nuova(
    partita: &mut Partita<'_>,
    canale: &mut dyn Canale,
    narratore: &dyn Narratore,
) -> Result<()> {
    let avventura = partita.avventura;
    canale.mostra(&avventura.titolo().to_uppercase(), Stile::Titolo)?;
    let intro = avventura.introduzione();
    if !intro.trim().is_empty() {
        canale.mostra(intro.trim(), Stile::Narrazione)?;
    }
    canale.mostra(
        tr!(
            "Scrivi cosa vuoi fare; /help per i comandi.",
            "Type what you want to do; /help for the commands.",
            "Écrivez ce que vous voulez faire ; /help pour les commandes.",
            "Escribe lo que quieres hacer; /help para los comandos.",
            "Schreib, was du tun willst; /help zeigt die Befehle."
        ),
        Stile::Motore,
    )?;
    canale.mostra(&partita.nome_stanza_corrente(), Stile::Titolo)?;
    let esito = partita.esegui_turno(narratore, AzioneTurno::InizioPartita, canale)?;
    match esito {
        EsitoTurno::Applicato { .. } => mostra_esito(partita, esito, canale)?,
        altro => {
            mostra_esito(partita, altro, canale)?;
            let descrizione = partita
                .avventura
                .stanza(&partita.stato.stanza_corrente)
                .map(|s| s.descrizione().to_string())
                .unwrap_or_default();
            canale.mostra(&descrizione, Stile::Narrazione)?;
        }
    }
    Ok(())
}

fn riprendi(
    partita: &mut Partita<'_>,
    canale: &mut dyn Canale,
    narratore: &dyn Narratore,
) -> Result<()> {
    canale.mostra(&partita.nome_stanza_corrente(), Stile::Titolo)?;
    if partita.stato.esito.concluso() {
        riquadro_finale(&partita.stato, canale)?;
        return Ok(());
    }
    let esito = partita.esegui_turno(narratore, AzioneTurno::RipresaPartita, canale)?;
    mostra_esito(partita, esito, canale)
}

fn mostra_esito(partita: &Partita<'_>, esito: EsitoTurno, canale: &mut dyn Canale) -> Result<()> {
    match esito {
        EsitoTurno::Applicato {
            narrazione,
            cambio_stanza,
            esito,
            sconfitta_per_turni,
        } => {
            if cambio_stanza {
                canale.mostra(&partita.nome_stanza_corrente(), Stile::Titolo)?;
            }
            canale.mostra(&narrazione, Stile::Narrazione)?;
            if sconfitta_per_turni {
                let max = partita.avventura.gioco.max_turni.unwrap_or_default();
                canale.mostra(
                    &tr!(
                        format!("Hai esaurito i turni a disposizione ({max})."),
                        format!("You have run out of turns ({max})."),
                        format!("Vous avez épuisé vos tours ({max})."),
                        format!("Se te han acabado los turnos ({max})."),
                        format!("Du hast keine Züge mehr ({max}).")
                    ),
                    Stile::Motore,
                )?;
            }
            if partita.config.interfaccia.mostra_turno && !esito.concluso() {
                let (turno, punti) = (partita.stato.turno, partita.stato.punteggio);
                canale.mostra(
                    &tr!(
                        format!("[Turno {turno} · Punti {punti}]"),
                        format!("[Turn {turno} · Score {punti}]"),
                        format!("[Tour {turno} · Score {punti}]"),
                        format!("[Turno {turno} · Puntos {punti}]"),
                        format!("[Zug {turno} · Punkte {punti}]")
                    ),
                    Stile::Motore,
                )?;
            }
            if esito.concluso() {
                riquadro_finale(&partita.stato, canale)?;
            }
        }
        EsitoTurno::Ripresa { narrazione, .. } => canale.mostra(&narrazione, Stile::Narrazione)?,
        EsitoTurno::NonRiuscito { messaggio } => canale.mostra(&messaggio, Stile::Errore)?,
        EsitoTurno::Annullato => canale.mostra(
            tr!(
                "Turno annullato: lo stato non è cambiato.",
                "Turn cancelled: the state has not changed.",
                "Tour annulé : l'état n'a pas changé.",
                "Turno cancelado: el estado no ha cambiado.",
                "Zug abgebrochen: Der Spielstand hat sich nicht verändert."
            ),
            Stile::Motore,
        )?,
    }
    Ok(())
}

/// Testo del riquadro di fine partita con turni e punteggio.
pub fn testo_riquadro_finale(stato: &Stato) -> String {
    let titolo = match stato.esito {
        Esito::Vittoria => tr!("VITTORIA!", "VICTORY!", "VICTOIRE !", "¡VICTORIA!", "SIEG!"),
        Esito::Sconfitta => tr!("SCONFITTA", "DEFEAT", "DÉFAITE", "DERROTA", "NIEDERLAGE"),
        Esito::InCorso => tr!(
            "PARTITA IN CORSO",
            "GAME IN PROGRESS",
            "PARTIE EN COURS",
            "PARTIDA EN CURSO",
            "SPIEL LÄUFT"
        ),
    };
    let righe = [
        titolo.to_string(),
        tr!(
            format!("Turni: {} · Punteggio: {}", stato.turno, stato.punteggio),
            format!("Turns: {} · Score: {}", stato.turno, stato.punteggio),
            format!("Tours : {} · Score : {}", stato.turno, stato.punteggio),
            format!("Turnos: {} · Puntos: {}", stato.turno, stato.punteggio),
            format!("Züge: {} · Punkte: {}", stato.turno, stato.punteggio)
        ),
    ];
    let larghezza = righe.iter().map(|r| r.chars().count()).max().unwrap_or(0) + 4;
    let mut t = format!("\n  ╔{}╗\n", "═".repeat(larghezza));
    for r in &righe {
        let spazio = larghezza - 2 - r.chars().count();
        t.push_str(&format!("  ║  {r}{}║\n", " ".repeat(spazio)));
    }
    t.push_str(&format!("  ╚{}╝", "═".repeat(larghezza)));
    t
}

/// Riquadro di fine partita e comandi disponibili.
pub fn riquadro_finale(stato: &Stato, canale: &mut dyn Canale) -> Result<()> {
    canale.mostra(&testo_riquadro_finale(stato), Stile::Finale)?;
    canale.mostra(
        tr!(
"Usa /restart per ricominciare, /load per caricare un salvataggio, /menu per \
             scegliere un'altra avventura o /quit per uscire.",
"Use /restart to start again, /load to load a save, /menu to choose another \
             adventure or /quit to quit.",
"Utilisez /restart pour recommencer, /load pour charger une sauvegarde, /menu pour choisir une autre aventure ou /quit pour quitter.",
"Usa /restart para empezar de nuevo, /load para cargar una partida guardada, /menu para elegir otra aventura o /quit para salir.",
"Mit /restart beginnst du neu, mit /load lädst du einen Spielstand, mit /menu wählst du ein anderes Abenteuer, mit /quit beendest du."
),
        Stile::Motore,
    )
}

fn esegui_comando(
    ambiente: &Ambiente<'_>,
    comando: Comando,
    partita: &mut Partita<'_>,
    canale: &mut dyn Canale,
    narratore: &dyn Narratore,
    debug: bool,
) -> Result<Option<FineCiclo>> {
    let titolo = partita.avventura.titolo();
    let gestore = partita.salvataggi();
    match comando {
        Comando::Aiuto => canale.mostra(&testo_aiuto(debug), Stile::Elenco)?,
        Comando::Inventario => {
            if partita.stato.inventario.is_empty() {
                canale.mostra(
                    tr!(
                        "Non porti niente con te.",
                        "You are not carrying anything.",
                        "Vous ne portez rien.",
                        "No llevas nada.",
                        "Du trägst nichts bei dir."
                    ),
                    Stile::Motore,
                )?;
            } else {
                let righe: Vec<String> = partita
                    .stato
                    .inventario
                    .iter()
                    .map(|o| format!("  - {}", partita.avventura.nome_oggetto(o)))
                    .collect();
                canale.mostra(
                    &format!(
                        "{}\n{}",
                        tr!(
                            "Porti con te:",
                            "You are carrying:",
                            "Vous portez :",
                            "Llevas:",
                            "Du trägst:"
                        ),
                        righe.join("\n")
                    ),
                    Stile::Elenco,
                )?;
            }
        }
        Comando::Uscite => {
            let disponibili: Vec<String> = uscite::calcola(
                partita.avventura,
                &partita.stato,
                &partita.stato.stanza_corrente,
            )
            .into_iter()
            .filter(|u| u.visibile && u.disponibile)
            .map(|u| {
                let descrizione = partita.avventura.descrizione_uscita(
                    &partita.stato.stanza_corrente,
                    &u.direzione,
                    &u.descrizione,
                );
                let descrizione = if descrizione.is_empty() {
                    String::new()
                } else {
                    format!(" — {descrizione}")
                };
                format!(
                    "  - {}: {}{descrizione}",
                    crate::lingua::traduci_direzione(&u.direzione, crate::lingua::attuale()),
                    u.nome_verso
                )
            })
            .collect();
            if disponibili.is_empty() {
                canale.mostra(
                    tr!(
                        "Nessuna uscita disponibile al momento.",
                        "No exits available right now.",
                        "Aucune sortie disponible pour le moment.",
                        "No hay salidas disponibles ahora mismo.",
                        "Im Moment gibt es keine verfügbaren Ausgänge."
                    ),
                    Stile::Motore,
                )?;
            } else {
                canale.mostra(
                    &format!(
                        "{}\n{}",
                        tr!(
                            "Uscite disponibili:",
                            "Available exits:",
                            "Sorties disponibles :",
                            "Salidas disponibles:",
                            "Verfügbare Ausgänge:"
                        ),
                        disponibili.join("\n")
                    ),
                    Stile::Elenco,
                )?;
            }
        }
        Comando::Salva(nome) => {
            let nome_stanza = partita.nome_stanza_corrente();
            match nome {
                None => {
                    menu::menu_salva(canale, &gestore, &titolo, &mut partita.stato, &nome_stanza)?;
                }
                Some(n) => {
                    menu::salva_con_nome(canale, &gestore, &mut partita.stato, &nome_stanza, &n)?;
                }
            }
        }
        Comando::Carica(nome) => {
            if let Some(scelto) =
                menu::menu_carica(canale, &gestore, &titolo, &partita.stato, nome.as_deref())?
            {
                match gestore.carica(&scelto) {
                    Ok(stato) => {
                        partita.stato = stato;
                        canale.mostra(&caricato(&scelto, partita.stato.turno), Stile::Motore)?;
                        riprendi(partita, canale, narratore)?;
                    }
                    Err(e) => canale.mostra(&format!("{e:#}"), Stile::Errore)?,
                }
            }
        }
        Comando::Salvataggi => {
            let elenco = gestore.elenca();
            if elenco.is_empty() {
                canale.mostra(menu::nessun_salvataggio(), Stile::Motore)?;
            } else {
                canale.mostra(
                    &menu::formatta_salvataggi(
                        &titolo,
                        &elenco,
                        partita.stato.salvataggio_corrente.as_deref(),
                    ),
                    Stile::Elenco,
                )?;
            }
        }
        Comando::Elimina(None) => canale.mostra(
            tr!(
                "Uso: /delete NOME (l'elenco dei salvataggi si vede con /saves).",
                "Usage: /delete NAME (/saves shows the list of saves).",
                "Usage : /delete NOM (/saves affiche la liste des sauvegardes).",
                "Uso: /delete NOMBRE (/saves muestra la lista de partidas guardadas).",
                "Verwendung: /delete NAME (/saves zeigt die Liste der Spielstände)."
            ),
            Stile::Motore,
        )?,
        Comando::Elimina(Some(nome)) => match gestore.trova(&nome) {
            None => canale.mostra(&menu::non_esiste(&nome), Stile::Motore)?,
            Some(info) => {
                if canale.conferma(&tr!(
                    format!("Eliminare definitivamente il salvataggio «{}»?", info.nome),
                    format!("Permanently delete the save \"{}\"?", info.nome),
                    format!("Supprimer définitivement la sauvegarde « {} » ?", info.nome),
                    format!(
                        "¿Borrar definitivamente la partida guardada «{}»?",
                        info.nome
                    ),
                    format!("Spielstand „{}“ endgültig löschen?", info.nome)
                ))? {
                    let eliminato = gestore.elimina(&info.nome)?;
                    canale.mostra(
                        &tr!(
                            format!("Salvataggio «{eliminato}» eliminato."),
                            format!("Save \"{eliminato}\" deleted."),
                            format!("Sauvegarde « {eliminato} » supprimée."),
                            format!("Partida guardada «{eliminato}» borrada."),
                            format!("Spielstand „{eliminato}“ gelöscht.")
                        ),
                        Stile::Motore,
                    )?;
                } else {
                    canale.mostra(
                        tr!(
                            "Eliminazione annullata.",
                            "Deletion cancelled.",
                            "Suppression annulée.",
                            "Borrado cancelado.",
                            "Löschen abgebrochen."
                        ),
                        Stile::Motore,
                    )?;
                }
            }
        },
        Comando::Annulla => match stato::annulla(&partita.percorsi)? {
            Some(s) => {
                partita.stato = s;
                let (turno, stanza) = (partita.stato.turno, partita.nome_stanza_corrente());
                canale.mostra(
                    &tr!(
                        format!("Annullato: sei tornato al turno {turno} ({stanza})."),
                        format!("Undone: you are back at turn {turno} ({stanza})."),
                        format!("Annulé : vous êtes revenu au tour {turno} ({stanza})."),
                        format!("Deshecho: vuelves al turno {turno} ({stanza})."),
                        format!("Rückgängig gemacht: Du bist wieder bei Zug {turno} ({stanza}).")
                    ),
                    Stile::Motore,
                )?;
            }
            None => canale.mostra(
                tr!(
                    "Non c'è niente da annullare.",
                    "There is nothing to undo.",
                    "Il n'y a rien à annuler.",
                    "No hay nada que deshacer.",
                    "Es gibt nichts rückgängig zu machen."
                ),
                Stile::Motore,
            )?,
        },
        Comando::Ricomincia => {
            if progressi_non_salvati(&partita.stato)
                && !canale.conferma(&tr!(
format!(
                        "Hai {} con nome. Ricominciare comunque?",
                        testo_turni_non_salvati(&partita.stato)
                    ),
format!(
                        "You have {} (no named save). Start again anyway?",
                        testo_turni_non_salvati(&partita.stato)
                    ),
format!("Vous avez {} (aucune sauvegarde nommée). Recommencer quand même ?", testo_turni_non_salvati(&partita.stato)),
format!("Tienes {} (ninguna partida guardada con nombre). ¿Empezar de nuevo de todos modos?", testo_turni_non_salvati(&partita.stato)),
format!("Du hast {} (kein benannter Spielstand). Trotzdem neu beginnen?", testo_turni_non_salvati(&partita.stato))
))?
            {
                canale.mostra(si_continua(), Stile::Motore)?;
                return Ok(None);
            }
            partita.ricomincia()?;
            inizia_nuova(partita, canale, narratore)?;
        }
        Comando::Menu => {
            if progressi_non_salvati(&partita.stato)
                && !canale.conferma(&tr!(
format!(
                        "Hai {} con nome (la partita resta comunque in stato.json). Tornare al \
                         menu?",
                        testo_turni_non_salvati(&partita.stato)
                    ),
format!(
                        "You have {} (no named save; the game is still kept in stato.json). \
                         Back to the menu?",
                        testo_turni_non_salvati(&partita.stato)
                    ),
format!("Vous avez {} (aucune sauvegarde nommée ; la partie reste conservée dans stato.json). Revenir au menu ?", testo_turni_non_salvati(&partita.stato)),
format!("Tienes {} (ninguna partida guardada con nombre; la partida sigue en stato.json). ¿Volver al menú?", testo_turni_non_salvati(&partita.stato)),
format!("Du hast {} (kein benannter Spielstand; das Spiel bleibt in stato.json erhalten). Zurück zum Menü?", testo_turni_non_salvati(&partita.stato))
))?
            {
                return Ok(None);
            }
            return Ok(Some(FineCiclo::Menu));
        }
        Comando::Stato if debug => {
            canale.mostra(
                &serde_json::to_string_pretty(&partita.stato)?,
                Stile::Elenco,
            )?;
        }
        Comando::Grezzo if debug => match &partita.ultima_risposta_grezza {
            Some(r) => canale.mostra(r, Stile::Elenco)?,
            None => canale.mostra(
                tr!(
                    "Nessuna risposta dell'AI in questa sessione.",
                    "No AI reply in this session.",
                    "Aucune réponse de l'IA dans cette session.",
                    "Ninguna respuesta de la IA en esta sesión.",
                    "Keine KI-Antwort in dieser Sitzung."
                ),
                Stile::Motore,
            )?,
        },
        Comando::Stato | Comando::Grezzo => canale.mostra(
            tr!(
                "Comando disponibile solo avviando il gioco con --debug.",
                "Command available only when the game is started with --debug.",
                "Commande disponible uniquement si le jeu est lancé avec --debug.",
                "Comando disponible solo si el juego se inicia con --debug.",
                "Befehl nur verfügbar, wenn das Spiel mit --debug gestartet wurde."
            ),
            Stile::Motore,
        )?,
        Comando::Lingua(argomento) => {
            let scelta = match argomento {
                Some(a) => match Lingua::da_codice(&a) {
                    Some(l) => {
                        imposta_e_salva(ambiente, l, canale)?;
                        Some(l)
                    }
                    None => {
                        canale.mostra(
                            tr!(
"Lingua non disponibile: usa /language seguito da en, it, fr, es o de.",
"Language not available: use /language followed by en, it, fr, es or de.",
"Langue non disponible : utilisez /language suivi de en, it, fr, es ou de.",
"Idioma no disponible: usa /language seguido de en, it, fr, es o de.",
"Sprache nicht verfügbar: verwende /language mit en, it, fr, es oder de."
),
                            Stile::Motore,
                        )?;
                        None
                    }
                },
                None => cambia_lingua(ambiente, canale)?,
            };
            if scelta.is_some() {
                canale.mostra(
                    tr!(
                        "Il narratore risponderà in italiano dal prossimo turno.",
                        "The narrator will answer in English from the next turn.",
                        "Le narrateur répondra en français à partir du prochain tour.",
                        "El narrador responderá en español a partir del próximo turno.",
                        "Der Erzähler antwortet ab dem nächsten Zug auf Deutsch."
                    ),
                    Stile::Motore,
                )?;
            }
        }
        Comando::Esci => {
            if progressi_non_salvati(&partita.stato) {
                let s = &partita.stato;
                let domanda = if s.salvataggio_corrente.is_none() && s.turno_ultimo_salvataggio == 0
                {
                    tr!(
                        format!(
                            "Non hai ancora salvato con nome, ora sei al turno {}: vuoi salvare?",
                            s.turno
                        ),
                        format!(
                            "You have not saved with a name yet, you are at turn {}: save now?",
                            s.turno
                        ),
                        format!(
                            "Vous n'avez pas encore sauvegardé avec un nom, vous êtes au tour {} : sauvegarder maintenant ?",
                            s.turno
                        ),
                        format!(
                            "Aún no has guardado con un nombre y estás en el turno {}: ¿guardar ahora?",
                            s.turno
                        ),
                        format!(
                            "Du hast noch nicht unter einem Namen gespeichert und bist bei Zug {}: jetzt speichern?",
                            s.turno
                        )
                    )
                } else {
                    tr!(
                        format!(
                            "Ultimo /save al turno {}, ora sei al turno {}: vuoi salvare?",
                            s.turno_ultimo_salvataggio, s.turno
                        ),
                        format!(
                            "Last /save at turn {}, you are now at turn {}: save now?",
                            s.turno_ultimo_salvataggio, s.turno
                        ),
                        format!(
                            "Dernier /save au tour {}, vous êtes maintenant au tour {} : sauvegarder maintenant ?",
                            s.turno_ultimo_salvataggio, s.turno
                        ),
                        format!(
                            "Último /save en el turno {}, ahora estás en el turno {}: ¿guardar ahora?",
                            s.turno_ultimo_salvataggio, s.turno
                        ),
                        format!(
                            "Letztes /save bei Zug {}, jetzt bist du bei Zug {}: jetzt speichern?",
                            s.turno_ultimo_salvataggio, s.turno
                        )
                    )
                };
                match domanda_tre_vie(
                    canale,
                    &domanda,
                    tr!(
                        "Sì, salva",
                        "Yes, save",
                        "Oui, sauvegarder",
                        "Sí, guardar",
                        "Ja, speichern"
                    ),
                    tr!(
                        "No, esci senza salvare",
                        "No, quit without saving",
                        "Non, quitter sans sauvegarder",
                        "No, salir sin guardar",
                        "Nein, ohne Speichern beenden"
                    ),
                )? {
                    Risposta3::Annulla => {
                        canale.mostra(si_continua(), Stile::Motore)?;
                        return Ok(None);
                    }
                    Risposta3::No => {}
                    Risposta3::Si => {
                        let nome_stanza = partita.nome_stanza_corrente();
                        let salvato = menu::menu_salva(
                            canale,
                            &gestore,
                            &titolo,
                            &mut partita.stato,
                            &nome_stanza,
                        )?;
                        if salvato.is_none() {
                            return Ok(None);
                        }
                    }
                }
            }
            canale.mostra(arrivederci(), Stile::Motore)?;
            return Ok(Some(FineCiclo::Esci));
        }
        Comando::Sconosciuto(_) => canale.mostra(
            tr!(
                "Comando sconosciuto, scrivi /help",
                "Unknown command, type /help",
                "Commande inconnue, tapez /help",
                "Comando desconocido, escribe /help",
                "Unbekannter Befehl, gib /help ein"
            ),
            Stile::Motore,
        )?,
    }
    Ok(None)
}
