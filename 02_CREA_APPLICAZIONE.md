# 02 — Creazione dell'applicazione Rust

**Modello consigliato:** Claude Opus 5.5 (`claude-opus-5-5`) — è lo script più impegnativo: architettura, molti moduli, test.
**Durata indicativa:** 30–60 minuti.
**Prerequisiti:** Rust installato (`cargo --version` funziona). Se manca, vedi la sezione "Prerequisiti" in `LEGGIMI.md`.

---

## Istruzioni per Claude Code

Sei un ingegnere Rust esperto. Devi creare da zero il progetto descritto in `01_SPECIFICHE.md`, nella cartella `motore/` accanto a questo file.

**Ambito di questo script:** motore completo in modalità terminale, con tutti i fornitori AI via HTTP (`openai`, `lmstudio`, `anthropic`, `gemini`, `ollama`) e `finto`. Il **bot Telegram** (§7.5) è nello script 03 e il fornitore **`locale`** (GGUF, §5.5) nello script 04: qui prepara l'architettura perché li accolga senza riscritture (trait `Canale`, `Narratore: Send + Sync`, `sessione.rs` indipendente dal canale, feature Cargo `locale`/`cuda` dichiarate ma vuote, stub di `ai/locale.rs` che restituisce l'errore esplicativo).

### Passo 0 — Leggi e pianifica
1. Leggi **per intero** `01_SPECIFICHE.md`. È vincolante.
2. Verifica gli strumenti: `cargo --version`, `rustc --version`. Se mancano, **fermati** e spiega all'utente come installarli (vedi `LEGGIMI.md`); non installare nulla a livello di sistema senza chiedere.
3. Se la cartella `motore/` esiste già, non cancellarla: analizzala e completa/correggi ciò che manca.
4. Crea una lista di attività (task list) con i passi seguenti e aggiornala man mano.

### Passo 1 — Scheletro del progetto
1. `cargo new motore --bin --name avventura` (dalla radice).
2. Aggiungi le dipendenze del §8 con `cargo add` (per `reqwest`: `cargo add reqwest --no-default-features --features blocking,json,rustls-tls`; per `clap`: `--features derive`; per `serde`: `--features derive`; per `chrono`: `--features serde`). Non scrivere numeri di versione a mano.
3. In `Cargo.toml` imposta `[profile.release]` con `lto = "thin"`, `codegen-units = 1`, `strip = true`.
4. Crea i file dei moduli elencati nel §8 con commenti di intestazione in italiano che ne descrivono la responsabilità.
5. Implementa per primo `lingua.rs` (§7.6): enum `Lingua { It, En, Fr, Es, De }` (italiano, inglese, francese, spagnolo, tedesco), lingua dell'interfaccia **per thread** (`thread_local!`, valore iniziale italiano), macro esportate `tr!("italiano", "english", "français", "español", "deutsch")` e `tr_in!(lingua, …)` con le stesse cinque varianti che valutano solo il ramo scelto (vanno bene anche `format!`); `tr_in!` accetta anche la forma a due varianti `tr_in!(lingua, "italiano", "english")` per i testi destinati all'AI (prompt e motivi di validazione esistono solo in italiano e in inglese: le altre lingue usano l'inglese, `Lingua::lingua_prompt`). Inoltre: codici e nomi delle lingue (`fr-FR`, `de_DE.UTF-8`, `français`…), nome di una lingua scritto in un'altra (`nome_in`), risposte sì/no in tutte le lingue (`e_si`, `e_no`), tabella delle direzioni standard nelle cinque lingue con sinonimi (§3.4: `traduci_direzione`, `stessa_direzione`, `direzione_canonica`), rilevamento della lingua del sistema (`LC_ALL`/`LC_MESSAGES`/`LANG`, poi crate `sys-locale`), `AVVENTURA_LINGUA`, lettura e scrittura di `preferenze.json`. **Da qui in poi ogni testo mostrato al giocatore** (menu, messaggi del motore, `/help`, conferme, riquadro finale, avvisi) si scrive con `tr!` in tutte e cinque le lingue (traduzioni curate, non letterali: «Sauvegarde annulée.», «Partida guardada como «x»», „Spielstand gelöscht.“); i messaggi tecnici dei sottocomandi e degli errori di configurazione possono restare in italiano.

### Passo 2 — Modello dati e caricamento avventura
Implementa in quest'ordine, ciascuno con i propri test unitari prima di passare al successivo:
1. `avventura/frontmatter.rs`: separa front matter `+++ … +++` dal corpo; tollera BOM UTF-8 e CRLF; errore chiaro se manca.
2. `avventura/sezioni.rs`: mappa ordinata `titolo sezione → testo` dai `## `; confronto case-insensitive; i titoli standard inglesi sono equivalenti a quelli italiani (§3: `Description` = `Descrizione`…).
3. `avventura/mod.rs`: struct `Gioco` (con `lingua` e `storia`, §3.1), `Stanza`, `Uscita`, `Traduzione` (§3.4), `Avventura` (gioco + mappa stanze + catalogo oggetti + traduzioni per lingua), funzione `Avventura::carica(dir)` e `Avventura::lingua()` (lingua in cui è scritta l'avventura; il prompt è in italiano solo per `it`, altrimenti in inglese); i metodi per i testi mostrati al giocatore (`titolo()`, `introduzione()`, `nome_stanza()`, `nome_oggetto()`, `descrizione_uscita()`…) usano la traduzione nella lingua dell'interfaccia, se c'è, e le varianti `_originale` servono al prompt.
4. `validazione_statica.rs` e sottocomando `valida` (§3.3), compresi gli avvisi sulle traduzioni.
5. `uscite.rs`: disponibilità uscite con motivazione testuale.

### Passo 3 — Stato e configurazione
1. `stato.rs` (§4): `Stato::nuovo(&Avventura)`, `carica`, `salva_atomico`, storico per turno, `annulla`. Percorsi sempre per avventura: `partite/<id-avventura>/…`.
2. `salvataggi.rs` (§4.1): validazione dei nomi, `elenca`, `salva`, `carica`, `elimina`, `progressi_non_salvati`, campi `salvataggio_corrente` e `turno_ultimo_salvataggio`.
3. `catalogo_avventure.rs` (§2, §7.1): scansione delle sottocartelle di `avventure/`, riepilogo per il menu (titolo, `descrizione_breve`, partita in corso, numero salvataggi, lingua, storia), avventure non valide segnalate con il motivo.
4. `config.rs` + `ai/profili.rs` (§5): struct con default (incluse le sezioni `[ai.locale]` e `[telegram]`, anche se usate dagli script 03–04, e `interfaccia.lingua = "auto"` validata come `auto`/`it`/`en`/`fr`/`es`/`de`), ricerca file, profili `[profili.*]` e `--profilo`/`AVVENTURA_PROFILO`, precedenze, predefiniti per fornitore (`base_url`, `api_key_env`, `ragionamento`), sinonimi di compatibilità (`protocollo`, `base_url = "finto://"`), validazione dei valori (timeout 1–86.400), regole della chiave (§5.1), `Debug` che maschera ogni segreto.

### Passo 4 — AI
Prima di scrivere ogni client, **verifica sulla documentazione ufficiale corrente** gli endpoint e i nomi dei campi (in particolare per Anthropic: output strutturato e ragionamento esteso; per Gemini: `generateContent`, `responseMimeType`, `thinkingConfig`; per OpenAI: `reasoning_effort`, `max_completion_tokens`). Se trovi differenze con il §6.1, segui la documentazione e annota la differenza nel report finale.
1. `ai/mod.rs`: trait `Narratore` (`Send + Sync`), `RichiestaAi` (messaggi + `ContestoTurno`), `RispostaGrezza` (testo, token di prompt dichiarati, durata), fabbrica dei fornitori.
2. `ai/finto.rs` (§9) — implementalo **prima** dei client reali: permette di provare tutto offline. Riconosce le azioni (verbi e articoli per ciascuna delle cinque lingue, §9) e risponde nella lingua dell'interfaccia (`ContestoTurno.lingua`); `nord` trova anche un'uscita `north` e viceversa (anche `norden`, `norte`…).
3. `ai/http.rs` (§5.1, §5.3, §6.5 punto 5): parti comuni — timeout di connessione e lettura separati (default 10 s e 900 s), retry/backoff solo per 429/5xx/529/connessione, domanda all'utente (tramite `Canale`) dopo un timeout di lettura, retry una volta senza il parametro rifiutato (400), header extra, nessun segreto nei log.
4. `ai/schema.rs` e `ai/ragionamento.rs` (§5.4, §6.4): schema JSON della risposta; mappatura di `ragionamento`; rimozione dei blocchi di ragionamento dal testo.
5. Fornitori (§6.1), uno alla volta, ciascuno con i propri test contro un server HTTP di prova prima di passare al successivo: `ai/openai.rs` (anche `lmstudio`), `ai/ollama.rs`, `ai/anthropic.rs`, `ai/gemini.rs`. Ognuno implementa anche `turno_in_flusso` (streaming, §6.1) ricomponendo la stessa risposta completa di `turno`; `ai/flusso.rs` contiene le righe SSE e l'estrattore incrementale della narrazione.
6. `ai/locale.rs`: solo lo stub (la vera implementazione è nello script 04).
7. `ai/token.rs` (§5.2): stima dei token, avviso se la stima supera `contesto_token`, rilevamento del troncamento confrontando stima e token dichiarati dal server, calcolo del contesto consigliato per un'avventura (usato da `prova-ai --avventura` e da `valida`).
8. `prompt.rs` (§6.2–6.3): testo delle regole del narratore e dello schema in due costanti ben scritte, una in italiano e una in inglese, l'italiana per le avventure in italiano e l'inglese per tutte le altre (§7.6); intestazioni, righe delle uscite, marcatori e messaggio di correzione anch'essi nella lingua del prompt; i nomi dei campi JSON restano italiani; se la lingua della narrazione (quella dell'interfaccia, passata come parametro) è diversa da quella dell'avventura, aggiungi la sezione "lingua della narrazione" (nome della lingua scritto nella lingua del prompt: "German", "tedesco") con il glossario dei nomi tradotti in quella lingua (§6.2). Funzioni pure `costruisci_messaggi(&Avventura, &Stato, azione, lingua_narrazione) -> Vec<Messaggio>`.
9. `risposta.rs` (§6.4–6.5): estrazione tollerante, deserializzazione, validazione severa/permissiva con elenco errori leggibile, nella lingua del prompt (gli errori tornano all'AI).

### Passo 5 — Motore, canale, comandi
0. `canale/mod.rs`, `canale/prova.rs` (§8): trait `Canale` (con `interattivo()`) e implementazione di prova. Tutto ciò che segue usa **solo** `Canale` per l'input/output, mai `println!`/stdin direttamente. `con_attesa` passa la lingua dell'interfaccia al thread che attende l'AI; `con_attesa_e_frammenti` fa lo stesso e consegna al canale (`Canale::frammento`) i pezzi di narrazione che arrivano in streaming (§6.1, §7.4).
1. `sessione.rs` + `motore.rs`: scelta della lingua all'avvio (`prepara_lingua`, §7.6, con domanda al primo utilizzo solo se il canale è interattivo), subito dopo l'avviso open source con l'indirizzo GitHub e il link al racconto di Vinny Falco su https://gronk.it (`avviso_open_source`, §7.1), ciclo di turno completo (§6.6), incluso il turno di apertura `[INIZIO PARTITA]` e i messaggi di correzione nei tentativi.
2. `menu.rs` (§7.1, §7.3, §7.6): menu della lingua, menu di avvio (scelta avventura con una sola versione per storia e voce della lingua → Continua/Carica/Nuova/Indietro) e menu interattivi di `/save` (salvataggio corrente / sovrascrivi esistente / nuovo nome) e `/load` (lista, scelta per numero o nome), scritti con `Canale::scegli`, `chiedi_testo`, `conferma`.
3. `comandi.rs`: tutti i comandi `/…` del §7.2 con i loro alias in italiano, francese, spagnolo e tedesco (compreso `/language`), incluse le conferme sui progressi non salvati per `/load`, `/restart`, `/menu`, `/quit`; `/help` in tutte le lingue (nome inglese + alias della lingua corrente + descrizione, da un'unica tabella); elenco dei comandi del bot con le descrizioni nelle cinque lingue, iniziale maiuscola (usato dallo script 03).
4. `canale/terminale.rs` (§7.4): rustyline, wrapping, colori (rispettando `NO_COLOR` e `interfaccia.colori`), spinner durante la chiamata AI, testo progressivo (narrazione mostrata man mano che arriva, con cursore animato in fondo finché la risposta non è completa, non ristampata se coincide con quella validata), trascrizione Markdown opzionale, abilitazione ANSI/UTF-8 su Windows.
5. `cli.rs` + `main.rs`: `avventura` senza argomenti = `gioca`; sottocomandi `gioca`, `elenca`, `salvataggi`, `valida`, `nuova`, `profili`, `prova-ai` con le opzioni del §7 (`--avventura`, `--carica`, `--nuova`, `--cartella-avventure`, `--profilo`, `--timeout`, `--lingua`…). Il sottocomando `telegram` va dichiarato e per ora risponde "disponibile dopo lo script 03". Messaggi d'errore finali in italiano, senza backtrace salvo `RUST_BACKTRACE`.
6. Log con `tracing` su `partite/avventura.log` a rotazione (oltre 5 MB diventa `avventura.log.1`), livello `info` (stato e dimensione delle risposte, tempi di ogni tentativo HTTP e del flusso, interruzioni con i byte già ricevuti, per Gemini `finishReason` e token compresi quelli di ragionamento: vedi §11 delle specifiche) e `debug` con `--debug` (corpi completi di richieste e risposte, senza chiave, per diagnosi).

### Passo 6 — Test di integrazione
In `motore/tests/`:
1. Una cartella di avventure di prova `motore/tests/dati/avventure/` con **due** avventure: `mini/` (3 stanze, un oggetto, un'uscita con `richiede_oggetto`, una con `richiede_flag`) e `mini2/` (2 stanze), più `rotta/` (con errori voluti, deve comparire come non valida nel menu).
2. Test `valida` su mini-avventura (ok) e su una variante rotta (errori attesi).
3. Partita scriptata con `finto://`: entra, prende l'oggetto, attraversa l'uscita bloccata, verifica `stato.json` finale e la presenza dello storico.
4. Test di tutti i fornitori HTTP contro un server HTTP locale di prova (`tiny_http` o `mockito`) per i casi del §10, inclusi: header corretti per fornitore e loro assenza quando non serve la chiave, `options.num_ctx` e `think` con ollama, `output_config.format` con anthropic, `responseMimeType` con gemini, parametri di ragionamento, avviso di troncamento, timeout di lettura (1 s nel test) senza retry automatico, profili e precedenze.
5. Test di `/undo` e dei salvataggi con nome (§10): crea, sovrascrive il corrente, sovrascrive un esistente, nuovo nome, nome non valido, carica per numero e per nome, elimina, conferma sui progressi non salvati.
6. Test di isolamento: partita e salvataggi di `mini` non compaiono né interferiscono con `mini2`.
7. Test del menu di avvio con input simulato: elenco delle avventure (con `rotta` non selezionabile), Continua/Carica/Nuova.
8. Test delle lingue (§10, §7.6) in `tests/lingue.rs`. I test che controllano testi italiani fissano la lingua (`interfaccia.lingua = "it"` nella configurazione di prova, `AVVENTURA_LINGUA=it` per i test del binario), così non dipendono dalla lingua del computer.

### Passo 7 — Configurazione di esempio
Crea `config/config.esempio.toml` nella radice (non in `motore/`) con tutte le chiavi del §5 commentate in italiano (compresa `interfaccia.lingua = "auto"`). Sezione `[ai]` pronta per OpenAI e, come **profili** già pronti (`[profili.*]`), uno per ciascun fornitore: `openai`, `openrouter` (fornitore openai con base_url di OpenRouter), `claude` (anthropic), `gemini`, `ollama` (nativo, `contesto_token = 16384`, `ragionamento = "spento"`), `ollama-rete` (server su un altro PC, `chiave_richiesta = false`), `lmstudio`, `gguf` (fornitore `locale` con percorso d'esempio), `prova` (finto), più la sezione `[telegram]` commentata. Ogni profilo ha un commento di una riga che dice cosa serve per usarlo (chiave, programma da installare…). I nomi dei modelli negli esempi vanno verificati sulle pagine ufficiali dei fornitori e indicati come esempi modificabili.
Aggiorna il `.gitignore` della radice (esiste già) aggiungendo, se mancano: `modelli/`, `*.gguf`.

### Passo 8 — Controllo finale (obbligatorio)
Esegui da `motore/`:
```
cargo fmt --all
cargo clippy --all-targets -- -D warnings
cargo test
```
Correggi finché tutto passa. Non disattivare lint né test per farli passare: correggi il codice. Se un test è sbagliato, spiega perché lo modifichi.

### Criteri di accettazione
- [ ] `cargo build` e `cargo test` passano senza warning di clippy.
- [ ] `cargo run -- valida --avventura tests/dati/avventure/mini` termina con codice 0.
- [ ] `cargo run -- elenca --cartella-avventure tests/dati/avventure` mostra le due avventure valide e quella rotta con il motivo.
- [ ] Menu di avvio, `/save` e `/load` si comportano come negli esempi del §7.1 e §7.3.
- [ ] Con `fornitore = "finto"` (o `--profilo prova`) si può giocare una partita dall'inizio senza rete.
- [ ] `avventura profili` elenca i profili; `--profilo` cambia fornitore senza modificare il file.
- [ ] Nessun modulo fuori da `canale/terminale.rs` scrive direttamente su stdout o legge da stdin.
- [ ] Nessun `unwrap()`/`expect()` fuori dai test (verifica con una ricerca).
- [ ] Nessuna chiave API compare in log o output (verifica con un test).
- [ ] Con fornitori locali (`ollama`, `lmstudio`) e nessuna chiave il programma parte senza errori; con `openai`/`anthropic`/`gemini` senza chiave dà un errore chiaro con il nome della variabile.
- [ ] Il timeout predefinito è 900 s ed è modificabile da file, `AVVENTURA_TIMEOUT` e `--timeout`.
- [ ] Ogni modulo ha commenti di documentazione in italiano.
- [ ] Con `--lingua en` (o `fr`, `es`, `de`, oppure `interfaccia.lingua`) tutta l'interfaccia di gioco è in quella lingua; al primo avvio interattivo con `auto` viene chiesta la lingua, proponendo quella del sistema.

### Report finale
Termina con un riepilogo breve: moduli creati, numero di test, eventuali scostamenti dalla specifica (con motivazione) e cose rimaste aperte. Non implementare ancora Telegram (script 03) né il fornitore `locale` (script 04); non creare l'avventura di esempio né la documentazione (script 06–09).
