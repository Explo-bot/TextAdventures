# 03 — Modalità bot Telegram

**Modello consigliato:** Claude Opus 5.5 (`claude-opus-5-5`): concorrenza tra thread, sessioni per utente e adattamento dei menu richiedono attenzione.
**Durata indicativa:** 30–45 minuti.
**Prerequisiti:** script 02 completato (motore con trait `Canale` e `sessione.rs` indipendente dal canale).

---

## Istruzioni per Claude Code

Aggiungi al progetto in `motore/` la modalità **bot Telegram** descritta nel §7.5 di `01_SPECIFICHE.md`: tutta l'interazione con il giocatore avviene in una chat Telegram, con lo stesso motore, le stesse avventure, gli stessi menu e comandi della modalità terminale.

### Passo 0 — Verifiche
1. Rileggi §2, §4, §4.1, §5 (sezione `[telegram]`), §7.1–7.3, §7.5, §7.6, §8, §10, §11 di `01_SPECIFICHE.md`.
2. Controlla che lo script 02 abbia rispettato la separazione motore/canale: nessun `println!`/lettura da stdin fuori da `canale/terminale.rs`, menu scritti su `Canale`, `Narratore: Send + Sync`. Se non è così, **prima** sistema il codice esistente (con test), poi procedi.
3. Consulta la documentazione ufficiale dell'API Bot (https://core.telegram.org/bots/api) per i metodi usati: `getMe`, `deleteWebhook`, `getUpdates`, `sendMessage`, `editMessageText`, `editMessageReplyMarkup`, `deleteMessage`, `answerCallbackQuery`, `sendChatAction`, `setMyCommands`; tipi `Update`, `Message`, `CallbackQuery`, `InlineKeyboardMarkup`. Non inventare campi.

### Passo 1 — Client API (`telegram/api.rs`)
- Client `reqwest` blocking con `base_url` dell'API configurabile **solo per i test** (default `https://api.telegram.org`).
- Strutture `serde` minime per le chiamate elencate (l'utente comprende `language_code`, la lingua dell'app); gestione di `ok: false` con `description`; 429 → attesa di `parameters.retry_after`; errori di rete → errore tipizzato.
- `setMyCommands` accetta un `language_code` facoltativo.
- Il token non deve mai comparire in log o errori: maschera gli URL (`bot<TOKEN>` → `bot***`).
- Funzioni di utilità: escape HTML; divisione di un testo in parti ≤ 4.096 caratteri senza spezzare parole né tag HTML.

### Passo 2 — Sessioni (`telegram/sessione.rs`)
- Un thread per utente, creato al primo messaggio, che riceve gli aggiornamenti di quell'utente da un canale `mpsc` ed esegue `sessione.rs` (lo stesso ciclo di gioco del terminale) su un `CanaleTelegram`.
- `CanaleTelegram` implementa `Canale`:
  - `mostra` → `sendMessage` (HTML, stili del §7.5, divisione dei messaggi lunghi);
  - `scegli` → messaggio con tastiera inline (`callback_data` breve con identificativo della domanda + indice); attende la `callback_query` corrispondente; poi `answerCallbackQuery` ed `editMessageText` che mostra la scelta e rimuove i pulsanti; callback di domande vecchie → "Questa scelta non è più valida";
  - `chiedi_testo` → messaggio con il nome proposto come pulsante; accetta il pulsante o un messaggio di testo;
  - `conferma` → pulsanti "Sì" / "No" ("Yes" / "No", "Oui" / "Non", "Sí" / "No", "Ja" / "Nein"); come testo valgono le risposte sì/no di tutte le lingue (`lingua::e_si`, `e_no`);
  - `leggi_azione` → attende il prossimo messaggio di testo (comando o azione);
  - `attesa` → al primo giro un messaggio in corsivo «✍️ Il narratore sta scrivendo…» nella lingua del giocatore, poi `sendChatAction typing` ogni 4 s; `fine_attesa` → `deleteMessage` del messaggio (la risposta arriva come messaggio nuovo, con notifica);
  - `interattivo` → vero.
- Lingua (§7.6): il thread della sessione parte con la lingua dell'app dell'utente; prima del benvenuto chiama `sessione::prepara_lingua` (domanda al primo utilizzo con la lingua dell'app come prima voce, poi preferenza in `partite/telegram/<id_utente>/preferenze.json`). L'avviso open source con l'indirizzo GitHub e il link gratuito al racconto di Vinny Falco su https://gronk.it (§7.1) arriva da `sessione::gioca` a ogni avvio della sessione; `/start` scritto durante una partita lo mostra (`sessione::avviso_open_source`) prima di tornare al menu. Tutti i testi del canale e del bot ("Annulla", "Questa scelta non è più valida", "sei in coda…", errori…) nelle cinque lingue con `tr!`. Il messaggio "sei in coda" parte dal thread che attende l'AI: la lingua gli arriva da `con_attesa`.
- Scadenza delle domande dopo `inattivita_minuti` (= annulla).
- Un secondo messaggio mentre l'AI sta generando → risposta immediata "Sto ancora scrivendo la risposta precedente…" (gestita dal thread di smistamento, non accodata come azione).
- Percorsi delle partite per utente: `partite/telegram/<id_utente>/<id_avventura>/` (§2).
- Panic in una sessione: catturato (`std::panic::catch_unwind` o thread separato con `join`), registrato nel log, l'utente riceve un messaggio di errore e la sessione viene ricreata al messaggio successivo.

### Passo 3 — Bot (`telegram/bot.rs`) e sottocomando
- `avventura telegram [--config] [--profilo] [--timeout] [--verifica] [--debug]` (§7).
- Avvio: lettura token (errore chiaro se manca, con le istruzioni per BotFather in breve e il rimando al manuale), `getMe`, `deleteWebhook`, `setMyCommands` cinque volte (comandi inglesi del §7.2 con descrizione in inglese per tutti, poi con `language_code` = `it`, `fr`, `es`, `de` e la descrizione in quella lingua), creazione del `Narratore` **una sola volta** e condivisione tra le sessioni (`Arc`), limitatore `max_turni_paralleli` (semaforo con `Mutex`+`Condvar` o canale), caricamento offset e `utenti.json`.
- Ciclo `getUpdates` (timeout 50 s, timeout HTTP 65 s), smistamento a sessioni, salvataggio dell'offset dopo ogni lotto, backoff sugli errori.
- Accesso (§7.5): utenti non ammessi ricevono solo il messaggio con il loro ID; `codice_invito` (che non riammette i revocati; blocco di 15 minuti dopo 5 codici sbagliati); comandi amministratore `/utenti`, `/ammetti`, `/revoca` (alias inglesi `/users`, `/allow`, `/revoke`), `/state`, `/raw`; chat non private ignorate con spiegazione. Prima di rispondere, il thread di smistamento imposta la lingua dell'utente (preferenza salvata → configurazione → `language_code` → inglese) e la passa alla sessione quando la crea.
- Messaggi non testuali → "Scrivi la tua azione come testo."
- Ctrl+C (`ctrlc`): smette di ricevere, attende la fine dei turni in corso (max 60 s), salva l'offset, esce.
- `--verifica`: `getMe`, configurazione AI (come `prova-ai` minimale), cartelle; stampa un riepilogo ed esce con codice 0/1.
- Sul terminale del server: una riga di log per evento significativo (utente, avventura, turno, durata AI), mai il contenuto dei token.

### Passo 4 — Test (§10, voce Telegram)
Con un server HTTP di prova che imita l'API Bot (registra le richieste e restituisce aggiornamenti scriptati) e il fornitore `finto`:
1. utente non ammesso; 2. `/start` → tastiera delle avventure → scelta → menu partita → Nuova partita; 3. qualche azione; 4. `/save` via pulsanti (nuovo nome digitato e nome proposto), `/load` via pulsanti; 5. due utenti in parallelo con cartelle separate e limitatore a 1 (con un narratore rallentato: messaggio «sta scrivendo…» inviato e poi cancellato con `deleteMessage`, `sendChatAction typing`); 6. messaggio lungo diviso; 7. escape HTML; 8. 429 con `retry_after`; 9. persistenza offset (riavvio del bot nel test); 10. messaggio da gruppo; 11. messaggio durante un turno; 12. scadenza domanda (con `inattivita_minuti` molto basso nel test, es. configurabile in secondi solo nei test); 13. callback di una domanda vecchia; 14. `codice_invito` e comandi amministratore (un revocato non rientra con il codice; blocco dopo 5 codici sbagliati, anche con quello giusto); 15. il token non compare in nessun log o errore; 16. lingua dall'app (`language_code = "en"` con `interfaccia.lingua = "auto"`): domanda della lingua con "English" per primo, benvenuto e menu in inglese, preferenza salvata, messaggio "privato" in inglese, `setMyCommands` chiamato per le cinque lingue (inglese senza `language_code`, poi `it`, `fr`, `es`, `de`). La configurazione di prova degli altri test fissa l'italiano.

### Passo 5 — Controllo finale
```
cargo fmt --all
cargo clippy --all-targets -- -D warnings
cargo test
```
Tutto deve passare, inclusi i test dello script 02 (nessuna regressione della modalità terminale).

### Criteri di accettazione
- [ ] `avventura telegram --verifica` senza token dà un errore chiaro e utile.
- [ ] Menu, `/save`, `/load`, conferme funzionano come pulsanti inline; comportamento identico al terminale nella sostanza.
- [ ] Ogni utente ha partite e salvataggi propri; avventure condivise.
- [ ] Ogni utente ha la propria lingua (italiano, inglese, francese, spagnolo o tedesco); senza preferenza il bot propone quella dell'app Telegram (inglese se non supportata).
- [ ] Un errore in una sessione non ferma il bot.
- [ ] Il token non compare mai in log, errori o output.

### Report finale
Moduli aggiunti, test aggiunti, eventuali modifiche al motore dello script 02, limiti noti. Facoltativo: se l'utente ha già creato un bot e impostato `TELEGRAM_BOT_TOKEN` e acconsente, avvia il bot con `--profilo prova` e chiedigli di provarlo dal telefono; altrimenti salta e dillo.
