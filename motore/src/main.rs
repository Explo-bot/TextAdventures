//! Punto di ingresso del binario `avventura`: legge la riga di comando, esegue
//! il sottocomando e mostra gli errori finali in italiano (senza backtrace,
//! salvo con `RUST_BACKTRACE` impostata).

use std::process::ExitCode;

use clap::Parser;

use avventura::canale::terminale::scrivi_errore;
use avventura::cli::Cli;
use avventura::sottocomandi;

fn main() -> ExitCode {
    let cli = Cli::parse();
    match sottocomandi::esegui(cli) {
        Ok(codice) => codice,
        Err(errore) => {
            scrivi_errore(&format!("Errore: {errore}"));
            for causa in errore.chain().skip(1) {
                scrivi_errore(&format!("  causa: {causa}"));
            }
            if std::env::var_os("RUST_BACKTRACE").is_some_and(|v| v != "0") {
                scrivi_errore(&format!("\n{errore:?}"));
            }
            ExitCode::FAILURE
        }
    }
}
