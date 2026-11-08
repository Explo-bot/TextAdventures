//! Motore "Avventura": avventure testuali in Markdown narrate da un modello AI.
//!
//! Il crate è diviso in una libreria (questo file e i moduli) e nel binario
//! `avventura` (`main.rs`), così che i test di integrazione possano usare
//! direttamente menu, client HTTP e ciclo di gioco.
//!
//! Il motore non conosce il canale di input/output: tutto passa per il trait
//! [`canale::Canale`] (terminale, prova e, con lo script 03, Telegram).

pub mod ai;
pub mod avventura;
pub mod canale;
pub mod catalogo_avventure;
pub mod cli;
pub mod comandi;
pub mod config;
pub mod lingua;
pub mod menu;
pub mod motore;
pub mod prompt;
pub mod risposta;
pub mod salvataggi;
pub mod sessione;
pub mod sottocomandi;
pub mod stato;
pub mod telegram;
pub mod uscite;
pub mod validazione_statica;
