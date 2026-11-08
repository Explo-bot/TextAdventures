//! Canale terminale (§7.4): riga di input con cronologia e modifica
//! (`rustyline`, prompt `> `), testo a capo alla larghezza configurata
//! (`textwrap`), colori (disattivati con `NO_COLOR` o `interfaccia.colori =
//! false`), indicatore di attesa con i secondi trascorsi, trascrizione Markdown
//! facoltativa, UTF-8/ANSI su Windows, uscita pulita con Ctrl+C.
//!
//! Testo progressivo (`interfaccia.testo_progressivo`): appena il narratore
//! comincia a scrivere, l'indicatore di attesa lascia il posto alla narrazione,
//! che compare man mano (già mandata a capo) seguita da un cursore animato
//! finché la risposta non è completa. Se la narrazione validata coincide con
//! quella mostrata non viene ristampata; altrimenti (risposta da correggere,
//! titolo di una nuova stanza, avvisi) il testo progressivo viene cancellato e
//! si stampa normalmente.
//!
//! Se lo standard input non è un terminale (comandi da file per collaudo) si
//! leggono righe semplici senza `rustyline`, ogni riga letta viene ristampata
//! dopo il prompt e l'indicatore di attesa è disattivato. A fine input: in gioco
//! [`Input::Fine`] (= `/quit` senza domande), nei menu e nelle conferme
//! «annulla»/«no».
//!
//! È l'unico modulo che scrive su stdout/stderr o legge da stdin: anche i
//! sottocomandi non interattivi stampano tramite [`scrivi`] e [`scrivi_errore`].

use std::fs::File;
use std::io::{self, BufRead, BufWriter, IsTerminal, Write};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};

use anyhow::{Context, Result};
use owo_colors::OwoColorize;

use super::{
    Canale, Input, IoRighe, Stile, Voce, chiedi_testo_a_righe, conferma_a_righe,
    leggi_azione_a_righe, scegli_a_righe,
};
use crate::config::ConfigInterfaccia;

/// Vero mentre l'indicatore di attesa occupa la riga corrente.
static ATTESA_IN_CORSO: AtomicBool = AtomicBool::new(false);

/// Stampa una riga su stdout (sottocomandi non interattivi).
pub fn scrivi(testo: &str) {
    let mut out = io::stdout().lock();
    let _ = writeln!(out, "{testo}");
    let _ = out.flush();
}

/// Stampa una riga su stderr (errori finali e avvisi dei sottocomandi).
pub fn scrivi_errore(testo: &str) {
    let mut err = io::stderr().lock();
    let _ = writeln!(err, "{testo}");
    let _ = err.flush();
}

/// Prepara la console di Windows: codepage UTF-8 e sequenze ANSI.
#[cfg(windows)]
pub fn prepara_console() {
    use windows_sys::Win32::System::Console::{
        ENABLE_VIRTUAL_TERMINAL_PROCESSING, GetConsoleMode, GetStdHandle, STD_ERROR_HANDLE,
        STD_OUTPUT_HANDLE, SetConsoleCP, SetConsoleMode, SetConsoleOutputCP,
    };
    const UTF8: u32 = 65001;
    // SAFETY: chiamate Win32 senza puntatori condivisi; gli handle provengono da
    // GetStdHandle e `modo` è una variabile locale valida per tutta la chiamata.
    unsafe {
        SetConsoleOutputCP(UTF8);
        SetConsoleCP(UTF8);
        for id in [STD_OUTPUT_HANDLE, STD_ERROR_HANDLE] {
            let handle = GetStdHandle(id);
            let mut modo = 0;
            if GetConsoleMode(handle, &mut modo) != 0 {
                SetConsoleMode(handle, modo | ENABLE_VIRTUAL_TERMINAL_PROCESSING);
            }
        }
    }
}

/// Su sistemi non Windows non serve nulla.
#[cfg(not(windows))]
pub fn prepara_console() {}

/// Ctrl+C fuori dalla riga di input (es. mentre l'AI risponde): uscita pulita.
/// Lo stato non si corrompe mai perché ogni scrittura è atomica.
pub fn installa_ctrl_c() {
    let _ = ctrlc::set_handler(|| {
        let mut out = io::stdout().lock();
        if ATTESA_IN_CORSO.load(Ordering::Relaxed) {
            let _ = write!(out, "\r\x1b[2K");
        }
        let _ = writeln!(
            out,
            "\nInterrotto. La partita in corso è salvata automaticamente."
        );
        let _ = out.flush();
        std::process::exit(130);
    });
}

/// Vero se i colori vanno usati (configurazione, `NO_COLOR`, terminale).
pub fn colori_attivi(config: &ConfigInterfaccia) -> bool {
    let no_color = std::env::var_os("NO_COLOR").is_some_and(|v| !v.is_empty());
    config.colori && !no_color && io::stdout().is_terminal()
}

/// Fotogrammi del cursore animato in fondo al testo progressivo.
const CURSORE: [&str; 6] = ["█", "▓", "▒", "░", "▒", "▓"];

/// Narrazione mostrata man mano che arriva (testo progressivo).
#[derive(Debug, Default)]
struct Progressivo {
    /// Testo ricevuto finora.
    testo: String,
    /// Righe (già mandate a capo) ora sullo schermo.
    righe: Vec<String>,
    /// Il cursore animato occupa una colonna dopo l'ultima riga.
    cursore: bool,
    /// Risposta terminata: il testo resta in attesa della narrazione definitiva.
    concluso: bool,
}

/// Righe fisiche occupate da una riga di `larghezza` colonne sul terminale.
fn righe_fisiche(larghezza: usize, colonne: usize) -> usize {
    larghezza.div_ceil(colonne.max(1)).max(1)
}

/// Canale sul terminale reale.
pub struct Terminale {
    colori: bool,
    larghezza: usize,
    editor: Option<rustyline::DefaultEditor>,
    interattivo: bool,
    trascrizione: Option<BufWriter<File>>,
    passo_attesa: usize,
    testo_progressivo: bool,
    progressivo: Option<Progressivo>,
}

impl Terminale {
    /// Crea il canale; `trascrizione` = file Markdown in cui copiare l'output.
    pub fn nuovo(config: &ConfigInterfaccia, trascrizione: Option<&Path>) -> Result<Terminale> {
        prepara_console();
        installa_ctrl_c();
        let interattivo = io::stdin().is_terminal() && io::stdout().is_terminal();
        let editor = if interattivo {
            rustyline::DefaultEditor::new().ok()
        } else {
            None
        };
        let larghezza = if config.larghezza_testo == 0 {
            textwrap::termwidth().max(20)
        } else {
            config.larghezza_testo
        };
        let trascrizione = match trascrizione {
            Some(p) => {
                let file = File::create(p).with_context(|| {
                    format!("impossibile creare la trascrizione «{}»", p.display())
                })?;
                let mut w = BufWriter::new(file);
                writeln!(w, "# Trascrizione della partita\n")?;
                Some(w)
            }
            None => None,
        };
        Ok(Terminale {
            colori: colori_attivi(config),
            larghezza,
            editor,
            interattivo,
            trascrizione,
            passo_attesa: 0,
            testo_progressivo: config.testo_progressivo && interattivo,
            progressivo: None,
        })
    }

    fn a_capo(&self, testo: &str) -> String {
        textwrap::fill(testo, self.larghezza)
    }

    /// Righe fisiche dalla riga `da` del testo progressivo alla fine (cursore compreso).
    fn altezza_progressivo(p: &Progressivo, da: usize) -> usize {
        let colonne = textwrap::termwidth();
        let ultima = p.righe.len().saturating_sub(1);
        p.righe
            .iter()
            .enumerate()
            .skip(da)
            .map(|(i, r)| {
                let extra = usize::from(p.cursore && i == ultima);
                righe_fisiche(textwrap::core::display_width(r) + extra, colonne)
            })
            .sum()
    }

    /// Aggiorna il testo progressivo sullo schermo riscrivendo solo dalla prima
    /// riga cambiata (di solito l'ultima) e disegna il cursore animato.
    fn ridisegna_progressivo(&mut self, cursore: bool) {
        let Some(p) = self.progressivo.as_ref() else {
            return;
        };
        let nuove: Vec<String> = textwrap::wrap(p.testo.trim_start(), self.larghezza)
            .into_iter()
            .map(|r| r.into_owned())
            .collect();
        let uguali = p
            .righe
            .iter()
            .zip(&nuove)
            .take_while(|(a, b)| a == b)
            .count();
        // L'ultima riga si riscrive sempre: lì c'è il cursore.
        let da = uguali.min(p.righe.len().saturating_sub(1));
        let mut uscita = String::new();
        if !p.righe.is_empty() {
            let su = Self::altezza_progressivo(p, da).saturating_sub(1);
            if su > 0 {
                uscita.push_str(&format!("\x1b[{su}A"));
            }
            uscita.push_str("\r\x1b[J");
        }
        uscita.push_str(&nuove.get(da..).unwrap_or_default().join("\n"));
        if cursore {
            let fotogramma = CURSORE[self.passo_attesa % CURSORE.len()];
            uscita.push_str(&if self.colori {
                fotogramma.bright_black().to_string()
            } else {
                fotogramma.to_string()
            });
        }
        let mut out = io::stdout().lock();
        let _ = write!(out, "{uscita}");
        let _ = out.flush();
        if let Some(p) = self.progressivo.as_mut() {
            p.righe = nuove;
            p.cursore = cursore;
        }
    }

    /// Toglie dallo schermo il testo progressivo (es. prima di un nuovo
    /// tentativo, o se la narrazione definitiva è diversa).
    fn cancella_progressivo(&mut self) {
        let Some(p) = self.progressivo.take() else {
            return;
        };
        if p.righe.is_empty() {
            return;
        }
        let su = Self::altezza_progressivo(&p, 0).saturating_sub(1);
        let mut out = io::stdout().lock();
        if su > 0 {
            let _ = write!(out, "\x1b[{su}A");
        }
        let _ = write!(out, "\r\x1b[J");
        let _ = out.flush();
    }

    /// Toglie l'indicatore «Il narratore riflette…», se presente.
    fn togli_indicatore() {
        if ATTESA_IN_CORSO.swap(false, Ordering::Relaxed) {
            let mut out = io::stdout().lock();
            let _ = write!(out, "\r\x1b[2K");
            let _ = out.flush();
        }
    }

    fn trascrivi(&mut self, testo: &str) {
        if let Some(t) = self.trascrizione.as_mut() {
            let _ = writeln!(t, "{testo}\n");
            let _ = t.flush();
        }
    }
}

impl IoRighe for Terminale {
    fn leggi(&mut self, prompt: &str) -> Option<String> {
        self.cancella_progressivo();
        let riga = if let Some(editor) = self.editor.as_mut() {
            match editor.readline(prompt) {
                Ok(r) => {
                    if !r.trim().is_empty() {
                        let _ = editor.add_history_entry(r.as_str());
                    }
                    Some(r)
                }
                // Ctrl+C, Ctrl+D o errore del terminale: fine dell'input.
                Err(_) => None,
            }
        } else {
            let mut out = io::stdout().lock();
            let _ = write!(out, "{prompt}");
            let _ = out.flush();
            drop(out);
            let mut riga = String::new();
            match io::stdin().lock().read_line(&mut riga) {
                Ok(0) | Err(_) => {
                    scrivi("");
                    None
                }
                Ok(_) => {
                    let riga = riga.trim_end_matches(['\r', '\n']).to_string();
                    // Eco della riga letta, così l'output resta leggibile.
                    scrivi(&riga);
                    Some(riga)
                }
            }
        };
        if let Some(r) = &riga {
            self.trascrivi(&format!("**{} {r}**", prompt.trim_end()));
        }
        riga
    }

    fn scrivi(&mut self, testo: &str, stile: Stile) {
        // Narrazione già sullo schermo grazie al testo progressivo: basta
        // chiudere la riga. Qualsiasi altro testo la sostituisce.
        if let Some(p) = &self.progressivo
            && p.concluso
            && stile == Stile::Narrazione
            && p.testo.trim() == testo.trim()
        {
            self.progressivo = None;
            scrivi("");
            self.trascrivi(testo);
            return;
        }
        self.cancella_progressivo();
        let c = self.colori;
        let (schermo, trascritto) = match stile {
            Stile::Narrazione => (self.a_capo(testo), testo.to_string()),
            Stile::Titolo => {
                let t = if c {
                    testo.bold().to_string()
                } else {
                    testo.to_string()
                };
                (format!("\n{t}"), format!("## {testo}"))
            }
            Stile::Motore => {
                let t = self.a_capo(testo);
                (
                    if c { t.cyan().to_string() } else { t },
                    format!("_{testo}_"),
                )
            }
            Stile::Avviso => {
                let t = self.a_capo(testo);
                (
                    if c { t.yellow().to_string() } else { t },
                    format!("> ⚠ {testo}"),
                )
            }
            Stile::Errore => {
                let t = self.a_capo(testo);
                (
                    if c { t.red().to_string() } else { t },
                    format!("**Errore:** {testo}"),
                )
            }
            Stile::Finale => (
                if c {
                    testo.bold().to_string()
                } else {
                    testo.to_string()
                },
                format!("```\n{testo}\n```"),
            ),
            Stile::Elenco => (testo.to_string(), format!("```\n{testo}\n```")),
        };
        scrivi(&schermo);
        self.trascrivi(&trascritto);
    }
}

impl Canale for Terminale {
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

    fn testo_progressivo(&self) -> bool {
        self.testo_progressivo
    }

    fn frammento(&mut self, testo: &str) -> Result<()> {
        if !self.testo_progressivo {
            return Ok(());
        }
        if self.progressivo.as_ref().is_some_and(|p| p.concluso) {
            // Resto di un tentativo precedente (risposta da correggere).
            self.cancella_progressivo();
        }
        Self::togli_indicatore();
        self.progressivo
            .get_or_insert_with(Progressivo::default)
            .testo
            .push_str(testo);
        self.ridisegna_progressivo(true);
        Ok(())
    }

    fn intervallo_attesa(&self) -> std::time::Duration {
        std::time::Duration::from_millis(125)
    }

    fn attesa(&mut self, secondi: u64) -> Result<()> {
        if !self.interattivo {
            return Ok(());
        }
        match &self.progressivo {
            // Il testo sta arrivando: si anima il cursore in fondo.
            Some(p) if !p.concluso => {
                self.passo_attesa += 1;
                self.ridisegna_progressivo(true);
                return Ok(());
            }
            Some(_) => self.cancella_progressivo(),
            None => {}
        }
        let puntini = ["   ", ".  ", ".. ", "..."][(self.passo_attesa / 2) % 4];
        self.passo_attesa += 1;
        let mut riga = crate::tr!(
            format!("Il narratore riflette{puntini} {secondi} s"),
            format!("The narrator is thinking{puntini} {secondi} s"),
            format!("Le narrateur réfléchit{puntini} {secondi} s"),
            format!("El narrador está pensando{puntini} {secondi} s"),
            format!("Der Erzähler denkt nach{puntini} {secondi} s")
        );
        if secondi >= 60 {
            riga.push_str(crate::tr!(
                " (i modelli locali possono essere lenti)",
                " (local models can be slow)",
                " (les modèles locaux peuvent être lents)",
                " (los modelos locales pueden ser lentos)",
                " (lokale Modelle können langsam sein)"
            ));
        }
        let riga = if self.colori {
            riga.bright_black().to_string()
        } else {
            riga
        };
        let mut out = io::stdout().lock();
        let _ = write!(out, "\r{riga}\x1b[K");
        let _ = out.flush();
        ATTESA_IN_CORSO.store(true, Ordering::Relaxed);
        Ok(())
    }

    fn fine_attesa(&mut self) -> Result<()> {
        Self::togli_indicatore();
        // Il testo progressivo resta (senza cursore) finché non arriva la
        // narrazione definitiva: se coincide non viene ristampato.
        if self.progressivo.as_ref().is_some_and(|p| !p.concluso) {
            self.ridisegna_progressivo(false);
            if let Some(p) = self.progressivo.as_mut() {
                p.concluso = true;
            }
        }
        self.passo_attesa = 0;
        Ok(())
    }
}
