# 05 — Compilazione, test e pacchetto di distribuzione

**Modello consigliato:** Claude Opus 5.5 (`claude-opus-5-5`) se ci sono errori di compilazione da correggere; Claude Sonnet 5.5 (`claude-sonnet-5-5`) basta se gli script 02–04 sono terminati senza errori.
**Durata indicativa:** 5–20 minuti (la prima compilazione release scarica e compila le dipendenze).
**Prerequisiti:** script 02, 03 e 04 completati.

---

## Istruzioni per Claude Code

Devi verificare, compilare in modalità release e preparare la cartella `dist/` del motore descritto in `01_SPECIFICHE.md`. Lavora dalla radice (la cartella che contiene questo file).

### Passo 1 — Ambiente
1. Esegui `rustc --version`, `cargo --version`, `rustup show` e annota la toolchain.
2. Su Windows verifica che il target sia `x86_64-pc-windows-msvc` (o `aarch64-pc-windows-msvc`). Se il linker `link.exe` manca (errore tipico: *"linker `link.exe` not found"*), **fermati** e indica all'utente di installare "Visual Studio Build Tools" con il carico di lavoro "Sviluppo di applicazioni desktop con C++" (oppure `winget install Microsoft.VisualStudio.2022.BuildTools`), poi di riaprire il terminale. Non tentare altri workaround.
3. Se `motore/Cargo.toml` non esiste, fermati: vanno prima eseguiti gli script 02–04.

### Passo 2 — Qualità del codice
Da `motore/`:
```
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test
```
Se qualcosa fallisce, correggi il **codice** (non i lint, non cancellare test) e ripeti. Riporta quali correzioni hai fatto.

### Passo 3 — Compilazione release
Due varianti:
1. **Standard** (tutti i fornitori via rete + Telegram, nessun requisito nativo):
   ```
   cargo build --release
   ```
   → `motore/target/release/avventura(.exe)`.
2. **Con modello locale** (fornitore `locale`, §5.5), solo se i prerequisiti dello script 04 sono presenti: con GPU NVIDIA e CUDA Toolkit `cargo build --release --features cuda`, altrimenti `--features locale` (solo CPU). Usa una cartella di build separata per non sovrascrivere la variante standard (`--target-dir target-locale`). Il file risultante si chiamerà in `dist/` `avventura-cuda.exe` (o `avventura-locale.exe` se solo CPU).
Se la variante 2 non è compilabile, non è un errore bloccante: riportalo e prosegui.
Annota dimensioni dei file e tempi di compilazione.

### Passo 4 — Prove di fumo con l'eseguibile release
Dalla radice (sostituisci il percorso dell'eseguibile secondo il sistema):
1. `motore/target/release/avventura --help` e `--version`.
2. `… valida --avventura motore/tests/dati/avventure/mini` → codice 0; `… elenca --cartella-avventure motore/tests/dati/avventure` mostra le avventure.
3. Se esiste già `avventure/faro-punta-nera/`, anche `… valida --avventura avventure/faro-punta-nera` → codice 0 (eventuali avvisi vanno riportati).
4. Partita offline: usa `--profilo prova` (fornitore `finto`) e prova una breve sequenza non interattiva. Se il motore non ha un'opzione per leggere comandi da file, usa la redirezione dello standard input (`Get-Content comandi.txt | …` in PowerShell, `< comandi.txt` in bash) e assicurati che il programma termini correttamente a fine input (EOF = `/quit`, e EOF dentro un menu o una domanda di conferma = annulla/esci, senza blocchi). Se a fine input il programma resta bloccato, correggilo.
   La sequenza deve passare dal menu di avvio (scelta avventura → Nuova partita) e contenere anche `/save prova`, qualche azione, `/load prova` (risposta di conferma `s`), `/saves`, `/quit`.
   Ripeti una sequenza breve con `--lingua en`: menu, messaggi del motore, `/help` e frasi dell'AI simulata devono essere in inglese; con `--lingua it` su un'avventura scritta in inglese con `traduzioni/it.toml` (script 06), titolo, introduzione, nomi delle stanze, `/uscite` e `/inventario` devono essere in italiano. Ripeti con `--lingua fr`, `es` e `de` (comandi come `est`/`prends le bidon`/`/sorties`, `este`/`coge la lata`/`/salidas`, `osten`/`nimm den kanister`/`/ausgänge`, e `/help` nella lingua scelta): con `traduzioni/fr.toml`, `es.toml`, `de.toml` anche i testi dell'avventura sono tradotti. I test automatici fissano l'italiano, quindi non dipendono dalla lingua di Windows.
5. Profili e fornitori: `… profili` con il config di esempio; `… prova-ai --profilo prova` → ok; `… prova-ai --profilo claude` senza `ANTHROPIC_API_KEY` → errore chiaro (lo stesso per `gemini` e `openai`); `… prova-ai --profilo ollama` senza Ollama avviato → "server non risponde" rapido (timeout di connessione), non attesa di 900 s; con l'eseguibile standard `… prova-ai --profilo gguf` → messaggio che spiega di usare la variante con modello locale.
6. Telegram senza token: `… telegram --verifica` → errore chiaro con le istruzioni; con token finto (`TELEGRAM_BOT_TOKEN=123:abc`) → errore di autenticazione leggibile, il token non compare nell'output.
7. Verifica che `partite/<id>/stato.json` sia stato creato, sia JSON valido, che `storico/` contenga i turni e che `salvataggi/prova.json` esista con `salvataggio_corrente = "prova"` nello stato.

### Passo 5 — Cartella `dist/`
Crea (o aggiorna) nella radice:
```
dist/
├── avventura(.exe)              eseguibile standard
├── avventura-cuda.exe           (se compilato) variante con modello locale su GPU NVIDIA
├── config.esempio.toml          copia da config/
├── modelli/LEGGIMI.txt          dove mettere i file GGUF (3–5 righe)
├── avventure/                   copia delle avventure di esempio, con le cartelle traduzioni/ (se esistono già)
└── LEGGIMI.txt                  istruzioni minime: 15–20 righe in italiano (scegliere il profilo, impostare la chiave o il modello locale, lanciare in terminale o come bot Telegram, lingue disponibili: italiano, inglese, francese, spagnolo, tedesco)
```
Poi crea un archivio `dist/avventura-<versione>-<target>.zip` con questi file (un secondo zip `…-cuda.zip` se la variante CUDA esiste, perché è molto più grande) (su Windows usa `Compress-Archive`; altrove `zip`). La versione si legge da `motore/Cargo.toml`.

### Passo 6 — Prova con AI reali (facoltativa)
Per ciascun fornitore cloud (`openai`, `anthropic`, `gemini`) la cui chiave è già impostata (`OPENAI_API_KEY`, `ANTHROPIC_API_KEY`, `GEMINI_API_KEY`), **chiedi all'utente** se vuole usare crediti per una prova; se sì esegui `… prova-ai --profilo <nome> --avventura faro-punta-nera` (se l'avventura esiste, altrimenti senza) e riporta latenza, validità del JSON, token di prompt. Non stampare mai le chiavi. Fornitori senza chiave: salta e dillo.

### Passo 6b — Prova con un modello locale (facoltativa)
Se sulla macchina risponde Ollama (`http://localhost:11434/api/tags`) o LM Studio (`http://localhost:1234/v1/models`), chiedi all'utente se vuole provarlo. Se sì, con una configurazione temporanea senza chiave:
1. `… prova-ai --profilo ollama` (o `--profilo lmstudio`) con `--avventura faro-punta-nera` se esiste, e riporta: latenza, contesto consigliato, eventuale avviso di troncamento.
2. Un paio di turni reali con un modello già installato (non scaricare modelli senza chiedere: possono pesare diversi GB).
Se nessun server locale risponde, salta il passo e dillo.

### Criteri di accettazione
- [ ] fmt, clippy e test passano.
- [ ] Eseguibile release creato e funzionante.
- [ ] Partita offline con `--profilo prova` completata e stato salvato correttamente.
- [ ] Errori chiari per chiavi mancanti, server locale spento, eseguibile senza supporto `locale`, token Telegram mancante.
- [ ] `dist/` e lo zip pronti.

### Report finale
Tabella con: toolchain, esito di ogni comando, numero di test passati, varianti compilate e loro dimensione, percorso dello zip, problemi trovati e correzioni applicate.
