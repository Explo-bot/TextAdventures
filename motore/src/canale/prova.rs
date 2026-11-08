//! Canale per i test: l'input arriva da una lista di righe preparate, tutto
//! l'output (prompt e righe lette comprese) viene registrato in
//! [`CanaleProva::uscita`]. Menu, conferme e domande usano la stessa logica a
//! righe del terminale ([`super::scegli_a_righe`] e simili). A righe finite
//! si comporta come un terminale a fine input: menu e conferme = annulla/no,
//! in gioco = [`Input::Fine`].

use std::collections::VecDeque;

use anyhow::Result;

use super::{
    Canale, Input, IoRighe, Stile, Voce, chiedi_testo_a_righe, conferma_a_righe,
    leggi_azione_a_righe, scegli_a_righe,
};

/// Canale di prova.
#[derive(Debug, Default)]
pub struct CanaleProva {
    righe: VecDeque<String>,
    /// Tutto ciò che è stato «mostrato», input compreso.
    pub uscita: String,
    /// Avvisi ricevuti (stile [`Stile::Avviso`]).
    pub avvisi: Vec<String>,
    /// Errori ricevuti (stile [`Stile::Errore`]).
    pub errori: Vec<String>,
    /// Numero di chiamate a [`Canale::attesa`].
    pub attese: u64,
    /// Valore di [`Canale::interattivo`] (falso: nessuna domanda sulla lingua).
    pub interattivo: bool,
    /// Valore di [`Canale::testo_progressivo`] (falso: nessuno streaming).
    pub progressivo: bool,
    /// Pezzi di narrazione ricevuti con [`Canale::frammento`].
    pub frammenti: Vec<String>,
}

impl CanaleProva {
    /// Canale che risponderà con le righe date, nell'ordine.
    pub fn nuovo<S: AsRef<str>>(righe: &[S]) -> Self {
        CanaleProva {
            righe: righe.iter().map(|r| r.as_ref().to_string()).collect(),
            ..Default::default()
        }
    }

    /// Righe non ancora lette.
    pub fn righe_rimaste(&self) -> usize {
        self.righe.len()
    }
}

impl IoRighe for CanaleProva {
    fn leggi(&mut self, prompt: &str) -> Option<String> {
        let riga = self.righe.pop_front()?;
        self.uscita.push_str(&format!("{prompt}{riga}\n"));
        Some(riga)
    }

    fn scrivi(&mut self, testo: &str, stile: Stile) {
        match stile {
            Stile::Titolo => self.uscita.push_str(&format!("== {testo} ==\n")),
            Stile::Avviso => {
                self.avvisi.push(testo.to_string());
                self.uscita.push_str(&format!("AVVISO: {testo}\n"));
            }
            Stile::Errore => {
                self.errori.push(testo.to_string());
                self.uscita.push_str(&format!("ERRORE: {testo}\n"));
            }
            _ => self.uscita.push_str(&format!("{testo}\n")),
        }
    }
}

impl Canale for CanaleProva {
    fn mostra(&mut self, testo: &str, stile: Stile) -> Result<()> {
        self.scrivi(testo, stile);
        Ok(())
    }

    fn scegli(&mut self, titolo: &str, voci: &[Voce]) -> Result<Option<usize>> {
        Ok(scegli_a_righe(self, titolo, voci))
    }

    fn chiedi_testo(&mut self, domanda: &str, proposta: Option<&str>) -> Result<Option<String>> {
        Ok(chiedi_testo_a_righe(self, domanda, proposta))
    }

    fn conferma(&mut self, domanda: &str) -> Result<bool> {
        Ok(conferma_a_righe(self, domanda))
    }

    fn leggi_azione(&mut self) -> Result<Input> {
        Ok(leggi_azione_a_righe(self))
    }

    fn interattivo(&self) -> bool {
        self.interattivo
    }

    fn attesa(&mut self, _secondi_trascorsi: u64) -> Result<()> {
        self.attese += 1;
        Ok(())
    }

    fn intervallo_attesa(&self) -> std::time::Duration {
        std::time::Duration::from_millis(100)
    }

    fn testo_progressivo(&self) -> bool {
        self.progressivo
    }

    fn frammento(&mut self, testo: &str) -> Result<()> {
        self.frammenti.push(testo.to_string());
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::comandi::Comando;

    #[test]
    fn menu_conferme_e_azioni() {
        let mut c = CanaleProva::nuovo(&["x", "2", "forse", "s", "", "/inv", "", "vai a nord"]);
        let voci = [Voce::nuova("uno"), Voce::nuova("due")];
        assert_eq!(c.scegli("MENU", &voci).unwrap(), Some(1));
        assert!(c.uscita.contains("Scelta non valida"));
        assert!(c.conferma("Sicuro?").unwrap());
        assert_eq!(
            c.chiedi_testo("Nome", Some("proposto")).unwrap().as_deref(),
            Some("proposto")
        );
        assert_eq!(
            c.leggi_azione().unwrap(),
            Input::Comando(Comando::Inventario)
        );
        assert_eq!(
            c.leggi_azione().unwrap(),
            Input::Azione("vai a nord".into())
        );
        assert_eq!(c.leggi_azione().unwrap(), Input::Fine);
        assert!(!c.conferma("Ancora?").unwrap());
        assert_eq!(c.scegli("MENU", &voci).unwrap(), None);
    }
}
