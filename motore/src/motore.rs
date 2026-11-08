//! Ciclo di turno (§6.6): rilettura dello stato dal disco, costruzione dei
//! messaggi, chiamata al narratore (con indicatore di attesa sul [`Canale`] e,
//! se il canale lo chiede, narrazione mostrata man mano che arriva) e
//! tentativi di correzione, validazione, applicazione degli aggiornamenti e
//! salvataggio atomico con storico.
//!
//! Un turno fallito non modifica mai lo stato. L'applicazione degli
//! aggiornamenti ([`applica_risposta`]) è una funzione pura e testabile.

use std::collections::HashSet;
use std::path::Path;

use anyhow::Result;
use tracing::{info, warn};

use crate::ai::flusso::EstrattoreNarrazione;
use crate::ai::token;
use crate::ai::{ErroreAi, Messaggio, Narratore, RichiestaAi};
use crate::avventura::Avventura;
use crate::canale::{Canale, Stile, con_attesa, con_attesa_e_frammenti};
use crate::config::{Config, ConfigMotore};
use crate::prompt::{self, AzioneTurno};
use crate::risposta::{self, RispostaAi};
use crate::salvataggi::GestoreSalvataggi;
use crate::stato::{self, Esito, PercorsiPartita, Stato, TurnoRecente, adesso};

/// Messaggio di cortesia quando tutti i tentativi falliscono.
pub fn messaggio_esitazione() -> &'static str {
    crate::tr!(
        "Il narratore esita… riprova o riformula l'azione.",
        "The narrator hesitates… try again or rephrase the action.",
        "Le narrateur hésite… réessayez ou reformulez l'action.",
        "El narrador duda… inténtalo de nuevo o reformula la acción.",
        "Der Erzähler zögert … versuch es noch einmal oder formuliere die Aktion anders."
    )
}

/// Ogni quanti turni al massimo ripetere l'avviso di troncamento.
const INTERVALLO_AVVISO_TRONCAMENTO: u32 = 10;

/// Esito di un turno.
#[derive(Debug, Clone, PartialEq)]
pub enum EsitoTurno {
    /// Risposta valida applicata e salvata.
    Applicato {
        /// Testo da mostrare.
        narrazione: String,
        /// Il giocatore è entrato in un'altra stanza.
        cambio_stanza: bool,
        /// Esito della partita dopo il turno.
        esito: Esito,
        /// La sconfitta è dovuta al superamento di `max_turni`.
        sconfitta_per_turni: bool,
    },
    /// Turno di ripresa (non incrementa il turno).
    Ripresa {
        /// Testo da mostrare.
        narrazione: String,
        /// Falso se l'AI non ha risposto e il testo è il riepilogo del motore.
        da_ai: bool,
    },
    /// Turno non riuscito: stato invariato.
    NonRiuscito {
        /// Messaggio per il giocatore.
        messaggio: String,
    },
    /// Turno annullato dal giocatore dopo un timeout: stato invariato.
    Annullato,
}

/// Effetti dell'applicazione di una risposta.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EffettiApplicazione {
    /// La stanza corrente è cambiata.
    pub cambio_stanza: bool,
    /// Sconfitta per superamento di `max_turni`.
    pub sconfitta_per_turni: bool,
}

/// Applica una risposta già validata allo stato (funzione pura, §6.6).
pub fn applica_risposta(
    stato: &mut Stato,
    risposta: &RispostaAi,
    azione: &str,
    avventura: &Avventura,
    config: &ConfigMotore,
) -> EffettiApplicazione {
    let stanza_prima = stato.stanza_corrente.clone();
    if let Some(agg) = &risposta.aggiornamenti {
        if let Some(s) = &agg.stanza_corrente {
            stato.stanza_corrente = s.clone();
        }
        for oggetto in agg.inventario_aggiungi.iter().flatten() {
            if !stato.inventario.contains(oggetto) {
                stato.inventario.push(oggetto.clone());
            }
            for stanza in stato.stanze.values_mut() {
                stanza.oggetti.retain(|o| o != oggetto);
            }
        }
        for oggetto in agg.inventario_rimuovi.iter().flatten() {
            stato.inventario.retain(|o| o != oggetto);
        }
        for (stanza, oggetti) in agg.oggetti_stanza_aggiungi.iter().flatten() {
            let s = stato.stanza_mut(stanza);
            for o in oggetti {
                if !s.oggetti.contains(o) {
                    s.oggetti.push(o.clone());
                }
            }
        }
        for (stanza, oggetti) in agg.oggetti_stanza_rimuovi.iter().flatten() {
            stato
                .stanza_mut(stanza)
                .oggetti
                .retain(|o| !oggetti.contains(o));
        }
        for (nome, valore) in agg.flag.iter().flatten() {
            stato.flag.insert(nome.clone(), valore.clone());
        }
        for (stanza, nota) in agg.note_stanza.iter().flatten() {
            let s = stato.stanza_mut(stanza);
            for n in nota.elenco() {
                let n = n.trim().to_string();
                if !n.is_empty() && !s.note.contains(&n) {
                    s.note.push(n);
                }
            }
            // Le note finiscono tutte nel prompt: si tengono solo le più recenti.
            let eccesso = s.note.len().saturating_sub(config.max_note_stanza.max(1));
            s.note.drain(..eccesso);
        }
        if let Some(delta) = agg.punteggio_delta {
            // `as` satura i valori enormi (e NaN → 0); la somma non deve traboccare.
            stato.punteggio = stato.punteggio.saturating_add(delta.round() as i64);
        }
        if let Some(esito) = agg.esito.as_deref().and_then(Esito::da_testo) {
            stato.esito = esito;
        }
    }
    stato.turno += 1;
    let corrente = stato.stanza_corrente.clone();
    stato.stanza_mut(&corrente).visitata = true;
    stato.ultimi_turni.push(TurnoRecente {
        turno: stato.turno,
        azione: azione.to_string(),
        narrazione: risposta.narrazione.trim().to_string(),
    });
    let eccesso = stato
        .ultimi_turni
        .len()
        .saturating_sub(config.turni_in_memoria);
    stato.ultimi_turni.drain(..eccesso);
    if let Some(memoria) = &risposta.memoria {
        stato.memoria = tronca(memoria.trim(), config.lunghezza_max_memoria);
    }
    let mut sconfitta_per_turni = false;
    if let Some(max) = avventura.gioco.max_turni
        && stato.turno >= max
        && stato.esito == Esito::InCorso
    {
        stato.esito = Esito::Sconfitta;
        sconfitta_per_turni = true;
    }
    stato.aggiornato_il = adesso();
    EffettiApplicazione {
        cambio_stanza: stato.stanza_corrente != stanza_prima,
        sconfitta_per_turni,
    }
}

/// Primi `massimo` caratteri del testo.
pub fn tronca(testo: &str, massimo: usize) -> String {
    testo.chars().take(massimo).collect()
}

/// Una partita in corso di un'avventura.
pub struct Partita<'a> {
    /// Avventura giocata.
    pub avventura: &'a Avventura,
    /// Configurazione.
    pub config: &'a Config,
    /// Percorsi dei file di partita.
    pub percorsi: PercorsiPartita,
    /// Stato attuale (copia in memoria di `stato.json`).
    pub stato: Stato,
    /// Ultima risposta grezza dell'AI (per `/raw`).
    pub ultima_risposta_grezza: Option<String>,
    stanze_avvisate: HashSet<String>,
    ultimo_avviso_troncamento: Option<u32>,
    avvisi_mostrati: HashSet<String>,
}

impl<'a> Partita<'a> {
    /// Partita con lo stato dato (non scrive nulla).
    pub fn con_stato(
        avventura: &'a Avventura,
        config: &'a Config,
        cartella_partite: &Path,
        stato: Stato,
    ) -> Self {
        Partita {
            avventura,
            config,
            percorsi: PercorsiPartita::nuovi(cartella_partite, &avventura.id),
            stato,
            ultima_risposta_grezza: None,
            stanze_avvisate: HashSet::new(),
            ultimo_avviso_troncamento: None,
            avvisi_mostrati: HashSet::new(),
        }
    }

    /// Nuova partita: lo stato precedente va nello storico, il nuovo viene salvato.
    pub fn nuova(
        avventura: &'a Avventura,
        config: &'a Config,
        cartella_partite: &Path,
    ) -> Result<Self> {
        let mut p =
            Partita::con_stato(avventura, config, cartella_partite, Stato::nuovo(avventura));
        p.ricomincia()?;
        Ok(p)
    }

    /// Partita ripresa da `stato.json`.
    pub fn riprendi(
        avventura: &'a Avventura,
        config: &'a Config,
        cartella_partite: &Path,
    ) -> Result<Self> {
        let percorsi = PercorsiPartita::nuovi(cartella_partite, &avventura.id);
        let stato = Stato::carica(&percorsi.stato())?;
        Ok(Partita::con_stato(
            avventura,
            config,
            cartella_partite,
            stato,
        ))
    }

    /// Sostituisce la partita con una nuova (lo stato precedente resta nello storico).
    pub fn ricomincia(&mut self) -> Result<()> {
        self.stato = Stato::nuovo(self.avventura);
        stato::salva_con_storico(&self.percorsi, &self.stato)?;
        self.stanze_avvisate.clear();
        Ok(())
    }

    /// Gestore dei salvataggi con nome di questa avventura.
    pub fn salvataggi(&self) -> GestoreSalvataggi {
        GestoreSalvataggi::per_avventura(self.percorsi.clone(), self.avventura)
    }

    /// Nome della stanza corrente.
    pub fn nome_stanza_corrente(&self) -> String {
        self.avventura.nome_stanza(&self.stato.stanza_corrente)
    }

    /// Esegue un turno completo.
    pub fn esegui_turno(
        &mut self,
        narratore: &dyn Narratore,
        azione: AzioneTurno,
        canale: &mut dyn Canale,
    ) -> Result<EsitoTurno> {
        // Lo stato su disco è l'unica fonte di verità.
        match Stato::carica_se_esiste(&self.percorsi.stato()) {
            Ok(Some(s)) => self.stato = s,
            Ok(None) => {}
            Err(e) => {
                return Ok(EsitoTurno::NonRiuscito {
                    messaggio: format!("{e:#}"),
                });
            }
        }
        let ripresa = azione == AzioneTurno::RipresaPartita;
        // L'AI narra nella lingua dell'interfaccia del giocatore (§7.6).
        let narrazione = crate::lingua::attuale();
        let base = prompt::costruisci_messaggi(self.avventura, &self.stato, &azione, narrazione);
        let contesto = prompt::contesto_turno(self.avventura, &self.stato, &azione, narrazione);
        self.controlla_contesto(&base, canale)?;

        let max_tentativi = self.config.ai.max_tentativi.max(1);
        let mut messaggi = base.clone();
        let mut tentativo = 1;
        let mut ultimi_errori: Vec<String> = Vec::new();
        let valida = loop {
            if tentativo > max_tentativi {
                break None;
            }
            let richiesta = RichiestaAi {
                messaggi: messaggi.clone(),
                contesto: contesto.clone(),
            };
            let esito = if canale.testo_progressivo() {
                // Testo progressivo: dal flusso grezzo si estrae la sola narrazione.
                con_attesa_e_frammenti(canale, |invia| {
                    let mut estrattore = EstrattoreNarrazione::default();
                    narratore.turno_in_flusso(&richiesta, &mut |pezzo| {
                        let nuovo = estrattore.aggiungi(pezzo);
                        if !nuovo.is_empty() {
                            invia(nuovo);
                        }
                    })
                })?
            } else {
                con_attesa(canale, || narratore.turno(&richiesta))?
            };
            let grezza = match esito {
                Ok(g) => g,
                Err(ErroreAi::TimeoutLettura { secondi }) => {
                    warn!(secondi, "timeout di lettura");
                    if canale.conferma(&crate::tr!(
                        format!("Nessuna risposta entro {secondi} s. Riprovare?"),
                        format!("No reply within {secondi} s. Try again?"),
                        format!("Pas de réponse en {secondi} s. Réessayer ?"),
                        format!("Sin respuesta en {secondi} s. ¿Reintentar?"),
                        format!("Keine Antwort innerhalb von {secondi} s. Noch einmal versuchen?")
                    ))? {
                        continue;
                    }
                    return Ok(if ripresa {
                        self.ripresa_senza_ai()
                    } else {
                        EsitoTurno::Annullato
                    });
                }
                Err(e) => {
                    warn!(errore = %e, "errore del narratore");
                    return Ok(if ripresa {
                        canale.mostra(
                            &crate::tr!(
                                format!("Il narratore non è raggiungibile: {e}"),
                                format!("The narrator cannot be reached: {e}"),
                                format!("Le narrateur est injoignable : {e}"),
                                format!("No se puede contactar con el narrador: {e}"),
                                format!("Der Erzähler ist nicht erreichbar: {e}")
                            ),
                            Stile::Avviso,
                        )?;
                        self.ripresa_senza_ai()
                    } else {
                        EsitoTurno::NonRiuscito {
                            messaggio: crate::tr!(
                                format!("Errore del narratore: {e}"),
                                format!("Narrator error: {e}"),
                                format!("Erreur du narrateur : {e}"),
                                format!("Error del narrador: {e}"),
                                format!("Fehler des Erzählers: {e}")
                            ),
                        }
                    });
                }
            };
            info!(
                durata_ms = grezza.durata.as_millis() as u64,
                "risposta del narratore ricevuta"
            );
            for avviso in &grezza.avvisi {
                if self.avvisi_mostrati.insert(avviso.clone()) {
                    canale.mostra(avviso, Stile::Avviso)?;
                }
            }
            self.ultima_risposta_grezza = Some(grezza.testo.clone());
            if let Some(dichiarati) = grezza.token_prompt {
                self.controlla_troncamento(dichiarati, &messaggi, canale)?;
            }
            let esito_validazione = risposta::analizza_in(&grezza.testo, self.avventura.lingua())
                .and_then(|mut r| {
                    risposta::valida(
                        &mut r,
                        self.avventura,
                        &self.stato,
                        self.config.motore.validazione,
                    )
                    .map(|avvisi| (r, avvisi))
                });
            match esito_validazione {
                Ok((r, avvisi)) => {
                    for a in avvisi {
                        warn!(avviso = %a, "parte della risposta scartata (validazione permissiva)");
                    }
                    break Some(r);
                }
                Err(errori) => {
                    warn!(tentativo, errori = ?errori, "risposta non valida");
                    // Correzione (§6.1): la risposta sbagliata come messaggio
                    // dell'assistente, poi l'elenco degli errori come messaggio utente.
                    messaggi = base.clone();
                    let sbagliata = if grezza.testo.trim().is_empty() {
                        crate::tr_in!(self.avventura.lingua(), "(risposta vuota)", "(empty reply)")
                            .to_string()
                    } else {
                        grezza.testo.clone()
                    };
                    messaggi.push(Messaggio::assistente(sbagliata));
                    messaggi.push(prompt::messaggio_correzione(
                        &errori,
                        self.avventura.lingua(),
                    ));
                    ultimi_errori = errori;
                    tentativo += 1;
                }
            }
        };

        let Some(risposta) = valida else {
            info!(errori = ?ultimi_errori, "tutti i tentativi falliti");
            return Ok(if ripresa {
                self.ripresa_senza_ai()
            } else {
                EsitoTurno::NonRiuscito {
                    messaggio: messaggio_esitazione().to_string(),
                }
            });
        };

        if ripresa {
            // La ripresa aggiorna solo la memoria e non passa per lo storico.
            if let Some(m) = &risposta.memoria {
                self.stato.memoria = tronca(m.trim(), self.config.motore.lunghezza_max_memoria);
                self.stato.salva_atomico(&self.percorsi.stato())?;
            }
            return Ok(EsitoTurno::Ripresa {
                narrazione: risposta.narrazione.trim().to_string(),
                da_ai: true,
            });
        }

        let mut nuovo = self.stato.clone();
        let effetti = applica_risposta(
            &mut nuovo,
            &risposta,
            &azione.per_storico(self.avventura.lingua()),
            self.avventura,
            &self.config.motore,
        );
        stato::salva_con_storico(&self.percorsi, &nuovo)?;
        self.stato = nuovo;
        info!(
            turno = self.stato.turno,
            stanza = %self.stato.stanza_corrente,
            esito = self.stato.esito.come_testo(),
            "turno applicato"
        );
        Ok(EsitoTurno::Applicato {
            narrazione: risposta.narrazione.trim().to_string(),
            cambio_stanza: effetti.cambio_stanza,
            esito: self.stato.esito,
            sconfitta_per_turni: effetti.sconfitta_per_turni,
        })
    }

    /// Testo di ripresa prodotto dal motore se l'AI non risponde.
    fn ripresa_senza_ai(&self) -> EsitoTurno {
        let descrizione = self
            .avventura
            .stanza(&self.stato.stanza_corrente)
            .map(|s| s.descrizione().to_string())
            .unwrap_or_default();
        let mut testo = descrizione;
        if !self.stato.memoria.trim().is_empty() {
            let finora = crate::tr_in!(
                self.avventura.lingua(),
                "Finora",
                "So far",
                "Jusqu'ici",
                "Hasta ahora",
                "Bisher"
            );
            testo.push_str(&format!("\n\n{finora}: {}", self.stato.memoria.trim()));
        }
        EsitoTurno::Ripresa {
            narrazione: testo,
            da_ai: false,
        }
    }

    fn controlla_contesto(
        &mut self,
        messaggi: &[Messaggio],
        canale: &mut dyn Canale,
    ) -> Result<()> {
        let contesto = self.config.ai.contesto_token;
        let stima = token::stima_richiesta(messaggi, self.config.ai.max_token);
        if token::supera_contesto(stima, contesto) {
            let avviso = token::avviso_contesto(stima, contesto);
            warn!(stima, contesto, "{avviso}");
            if self
                .stanze_avvisate
                .insert(self.stato.stanza_corrente.clone())
            {
                canale.mostra(&avviso, Stile::Avviso)?;
            }
        }
        Ok(())
    }

    fn controlla_troncamento(
        &mut self,
        dichiarati: u64,
        messaggi: &[Messaggio],
        canale: &mut dyn Canale,
    ) -> Result<()> {
        let stima = token::stima_messaggi(messaggi);
        if !token::troncamento_probabile(dichiarati, stima) {
            return Ok(());
        }
        let avviso = token::avviso_troncamento(dichiarati, stima);
        warn!(dichiarati, stima, "{avviso}");
        let turno = self.stato.turno;
        let da_mostrare = self
            .ultimo_avviso_troncamento
            .is_none_or(|t| turno >= t + INTERVALLO_AVVISO_TRONCAMENTO);
        if da_mostrare {
            self.ultimo_avviso_troncamento = Some(turno);
            canale.mostra(&avviso, Stile::Avviso)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::ai::finto::NarratoreFinto;
    use crate::ai::{RispostaGrezza, Ruolo};
    use crate::canale::prova::CanaleProva;
    use crate::risposta::analizza;
    use crate::stato::test::avventura_di_prova;
    use std::sync::Mutex;

    /// Narratore che restituisce risposte preparate e registra i messaggi ricevuti.
    struct NarratoreScriptato {
        risposte: Mutex<Vec<Result<RispostaGrezza, ErroreAi>>>,
        ricevuti: Mutex<Vec<Vec<Messaggio>>>,
    }

    impl NarratoreScriptato {
        fn nuovo(mut risposte: Vec<Result<RispostaGrezza, ErroreAi>>) -> Self {
            risposte.reverse();
            NarratoreScriptato {
                risposte: Mutex::new(risposte),
                ricevuti: Mutex::new(Vec::new()),
            }
        }

        fn ricevuti(&self) -> Vec<Vec<Messaggio>> {
            self.ricevuti.lock().unwrap().clone()
        }
    }

    fn testo(t: &str) -> Result<RispostaGrezza, ErroreAi> {
        Ok(RispostaGrezza {
            testo: t.to_string(),
            ..Default::default()
        })
    }

    impl Narratore for NarratoreScriptato {
        fn turno(&self, richiesta: &RichiestaAi) -> Result<RispostaGrezza, ErroreAi> {
            self.ricevuti
                .lock()
                .unwrap()
                .push(richiesta.messaggi.clone());
            self.risposte
                .lock()
                .unwrap()
                .pop()
                .unwrap_or_else(|| Err(ErroreAi::Formato("finite".into())))
        }
        fn descrizione(&self) -> String {
            "scriptato".into()
        }
    }

    fn config() -> Config {
        Config::default()
    }

    #[test]
    fn applicazione_degli_aggiornamenti() {
        let a = avventura_di_prova();
        let mut s = Stato::nuovo(&a);
        let r = analizza(
            r#"{"narrazione":"Prendi la chiave.","aggiornamenti":{
                "inventario_aggiungi":["chiave"],
                "oggetti_stanza_aggiungi":{"b":["gemma"]},
                "flag":{"botola_aperta":true,"conta":2},
                "note_stanza":{"a":"Polvere smossa."},
                "punteggio_delta":5,
                "stanza_corrente":"c"},
              "memoria":"Ho la chiave."}"#,
        )
        .unwrap();
        let e = applica_risposta(&mut s, &r, "prendo la chiave", &a, &config().motore);
        assert!(e.cambio_stanza);
        assert_eq!(s.turno, 1);
        assert_eq!(s.stanza_corrente, "c");
        assert!(s.stanze["c"].visitata);
        assert_eq!(s.inventario, vec!["moneta", "chiave"]);
        assert!(s.stanze["a"].oggetti.is_empty(), "tolta dalla stanza");
        assert_eq!(s.stanze["b"].oggetti, vec!["gemma"]);
        assert_eq!(s.flag["conta"], 2);
        assert_eq!(s.stanze["a"].note, vec!["Polvere smossa."]);
        assert_eq!(s.punteggio, 5);
        assert_eq!(s.memoria, "Ho la chiave.");
        assert_eq!(s.ultimi_turni[0].azione, "prendo la chiave");
    }

    #[test]
    fn buffer_turni_e_memoria_troncati() {
        let a = avventura_di_prova();
        let mut s = Stato::nuovo(&a);
        let mut motore = config().motore;
        motore.turni_in_memoria = 3;
        motore.lunghezza_max_memoria = 10;
        let r = analizza(&format!(
            r#"{{"narrazione":"n","memoria":"{}"}}"#,
            "è".repeat(50)
        ))
        .unwrap();
        for i in 0..5 {
            applica_risposta(&mut s, &r, &format!("azione {i}"), &a, &motore);
        }
        assert_eq!(s.ultimi_turni.len(), 3);
        assert_eq!(s.ultimi_turni[0].turno, 3);
        assert_eq!(s.ultimi_turni[2].azione, "azione 4");
        assert_eq!(s.memoria.chars().count(), 10);
    }

    #[test]
    fn note_della_stanza_limitate() {
        let a = avventura_di_prova();
        let mut s = Stato::nuovo(&a);
        let mut motore = config().motore;
        motore.max_note_stanza = 3;
        for i in 0..5 {
            let r = analizza(&format!(
                r#"{{"narrazione":"n","aggiornamenti":{{"note_stanza":{{"a":"nota {i}"}}}}}}"#
            ))
            .unwrap();
            applica_risposta(&mut s, &r, "x", &a, &motore);
        }
        assert_eq!(s.stanze["a"].note, vec!["nota 2", "nota 3", "nota 4"]);
    }

    #[test]
    fn sconfitta_per_max_turni() {
        let mut a = avventura_di_prova();
        a.gioco.max_turni = Some(2);
        let mut s = Stato::nuovo(&a);
        let r = analizza(r#"{"narrazione":"n"}"#).unwrap();
        assert!(!applica_risposta(&mut s, &r, "x", &a, &config().motore).sconfitta_per_turni);
        assert!(applica_risposta(&mut s, &r, "x", &a, &config().motore).sconfitta_per_turni);
        assert_eq!(s.esito, Esito::Sconfitta);
    }

    #[test]
    fn turno_con_correzione_e_fallimento() {
        let dir = tempfile::tempdir().unwrap();
        let a = avventura_di_prova();
        let c = config();
        let mut p = Partita::nuova(&a, &c, dir.path()).unwrap();
        let mut canale = CanaleProva::nuovo(&[] as &[&str]);

        let n = NarratoreScriptato::nuovo(vec![
            testo("non è json"),
            testo(r#"{"narrazione":"ok","aggiornamenti":{"stanza_corrente":"b"}}"#),
            testo(r#"{"narrazione":"Vai a est.","aggiornamenti":{"stanza_corrente":"c"}}"#),
        ]);
        let esito = p
            .esegui_turno(&n, AzioneTurno::Giocatore("vado a est".into()), &mut canale)
            .unwrap();
        assert!(matches!(
            esito,
            EsitoTurno::Applicato {
                cambio_stanza: true,
                ..
            }
        ));
        let ricevuti = n.ricevuti();
        assert_eq!(ricevuti[0].len(), 2);
        // sistema, utente, assistente (risposta sbagliata), utente (errori)
        assert_eq!(ricevuti[1].len(), 4);
        assert_eq!(ricevuti[1][2].ruolo, Ruolo::Assistant);
        assert_eq!(ricevuti[1][2].contenuto, "non è json");
        assert!(
            ricevuti[1][3]
                .contenuto
                .contains("non contiene un oggetto JSON")
        );
        assert_eq!(ricevuti[2].len(), 4, "le correzioni non si accumulano");
        assert!(ricevuti[2][3].contenuto.contains("uscita disponibile"));
        assert_eq!(Stato::carica(&p.percorsi.stato()).unwrap().turno, 1);

        let n = NarratoreScriptato::nuovo(vec![testo("x"), testo("y"), testo("z")]);
        let esito = p
            .esegui_turno(&n, AzioneTurno::Giocatore("boh".into()), &mut canale)
            .unwrap();
        assert_eq!(
            esito,
            EsitoTurno::NonRiuscito {
                messaggio: messaggio_esitazione().into()
            }
        );
        assert_eq!(
            Stato::carica(&p.percorsi.stato()).unwrap().turno,
            1,
            "stato invariato"
        );
    }

    #[test]
    fn timeout_chiede_al_giocatore() {
        let dir = tempfile::tempdir().unwrap();
        let a = avventura_di_prova();
        let c = config();
        let mut p = Partita::nuova(&a, &c, dir.path()).unwrap();
        let n = NarratoreScriptato::nuovo(vec![
            Err(ErroreAi::TimeoutLettura { secondi: 900 }),
            Err(ErroreAi::TimeoutLettura { secondi: 900 }),
            testo(r#"{"narrazione":"finalmente"}"#),
        ]);
        let mut canale = CanaleProva::nuovo(&["s", "n"]);
        let esito = p
            .esegui_turno(&n, AzioneTurno::Giocatore("aspetto".into()), &mut canale)
            .unwrap();
        assert_eq!(esito, EsitoTurno::Annullato);
        assert_eq!(
            n.ricevuti().len(),
            2,
            "un solo nuovo tentativo, chiesto al giocatore"
        );
        assert!(
            canale
                .uscita
                .contains("Nessuna risposta entro 900 s. Riprovare? (s/n)")
        );
        assert_eq!(Stato::carica(&p.percorsi.stato()).unwrap().turno, 0);
    }

    #[test]
    fn ripresa_aggiorna_solo_la_memoria() {
        let dir = tempfile::tempdir().unwrap();
        let a = avventura_di_prova();
        let c = config();
        let mut p = Partita::nuova(&a, &c, dir.path()).unwrap();
        let n = NarratoreScriptato::nuovo(vec![testo(
            r#"{"narrazione":"Eri qui.","memoria":"ricordo","aggiornamenti":{"punteggio_delta":50}}"#,
        )]);
        let mut canale = CanaleProva::default();
        let esito = p
            .esegui_turno(&n, AzioneTurno::RipresaPartita, &mut canale)
            .unwrap();
        assert_eq!(
            esito,
            EsitoTurno::Ripresa {
                narrazione: "Eri qui.".into(),
                da_ai: true
            }
        );
        let s = Stato::carica(&p.percorsi.stato()).unwrap();
        assert_eq!(
            (s.turno, s.punteggio, s.memoria.as_str()),
            (0, 0, "ricordo")
        );

        let n = NarratoreScriptato::nuovo(vec![Err(ErroreAi::Connessione("giù".into()))]);
        match p
            .esegui_turno(&n, AzioneTurno::RipresaPartita, &mut canale)
            .unwrap()
        {
            EsitoTurno::Ripresa { da_ai, narrazione } => {
                assert!(!da_ai);
                assert!(narrazione.contains("ricordo"));
            }
            altro => panic!("{altro:?}"),
        }
    }

    #[test]
    fn narrazione_progressiva_con_il_finto() {
        let dir = tempfile::tempdir().unwrap();
        let a = avventura_di_prova();
        let c = config();
        let mut p = Partita::nuova(&a, &c, dir.path()).unwrap();
        let mut canale = CanaleProva::default();
        canale.progressivo = true;
        let esito = p
            .esegui_turno(
                &NarratoreFinto,
                AzioneTurno::Giocatore("guarda".into()),
                &mut canale,
            )
            .unwrap();
        let EsitoTurno::Applicato { narrazione, .. } = esito else {
            panic!("{esito:?}");
        };
        assert!(canale.frammenti.len() > 1, "{:?}", canale.frammenti);
        assert_eq!(canale.frammenti.concat().trim(), narrazione);
    }

    #[test]
    fn avviso_di_contesto_una_volta_per_stanza() {
        let dir = tempfile::tempdir().unwrap();
        let a = avventura_di_prova();
        let mut c = config();
        c.ai.contesto_token = 100;
        let mut p = Partita::nuova(&a, &c, dir.path()).unwrap();
        let mut canale = CanaleProva::default();
        for _ in 0..2 {
            p.esegui_turno(
                &NarratoreFinto,
                AzioneTurno::Giocatore("guarda".into()),
                &mut canale,
            )
            .unwrap();
        }
        assert_eq!(canale.avvisi.len(), 1);
        assert!(canale.avvisi[0].contains("supera il contesto configurato"));
    }

    #[test]
    fn avviso_di_troncamento() {
        let dir = tempfile::tempdir().unwrap();
        let a = avventura_di_prova();
        let c = config();
        let mut p = Partita::nuova(&a, &c, dir.path()).unwrap();
        let mut canale = CanaleProva::default();
        let risposta = || {
            Ok(RispostaGrezza {
                testo: r#"{"narrazione":"n"}"#.into(),
                token_prompt: Some(10),
                ..Default::default()
            })
        };
        let n = NarratoreScriptato::nuovo(vec![risposta(), risposta()]);
        for _ in 0..2 {
            p.esegui_turno(&n, AzioneTurno::Giocatore("x".into()), &mut canale)
                .unwrap();
        }
        assert_eq!(canale.avvisi.len(), 1, "massimo una volta ogni 10 turni");
        assert!(canale.avvisi[0].contains("probabilmente ha troncato"));
    }
}
