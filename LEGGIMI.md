# Motore per avventure testuali in Rust — script per Claude Code

Questa cartella contiene una serie di **script in Markdown** da far eseguire a **Claude Code**. Eseguiti in ordine, producono:

- un motore per avventure testuali in Rust (`avventura.exe`), giocabile **da terminale** o **tramite un bot Telegram**, con l'AI a scelta tra **OpenAI**, **Anthropic**, **Gemini**, **Ollama**, **LM Studio** o un **modello GGUF caricato direttamente dal disco**;
- interfaccia di gioco in **italiano, inglese, francese, spagnolo e tedesco** (lingua chiesta al primo avvio, proponendo quella di Windows o dell'app Telegram; si cambia con `/language`);
- due avventure di esempio, scritte **in inglese** e giocabili **in tutte e cinque le lingue** (il narratore AI risponde nella lingua del giocatore, `traduzioni/it.toml`, `fr.toml`, `es.toml` e `de.toml` traducono titolo, introduzione e nomi): *The Punta Nera Lighthouse* / *Il Faro di Punta Nera* (racconto marinaresco) e *Star Trucker and the Space Chicken of Doom* (commedia sci-fi da B-movie). Scritte in inglese, i modelli AI, soprattutto quelli locali piccoli, ne seguono meglio le regole;
- il manuale unico (`docs/MANUALE.md`): installazione e configurazione dell'AI, utilizzo, attivazione del bot Telegram, lingue, creazione di nuove avventure;
- la presentazione del repository su GitHub: `README.md` (inglese) e `README.it.md` (italiano); in `config/` è pubblicato solo `config.esempio.toml` (con un profilo per ogni fornitore): la configurazione personale `config/config.toml` e le altre restano fuori da git;
- gli installatori (`installa.ps1` / `installa.sh`), i lanciatori (`gioca.bat` / `gioca.ps1` / `gioca.sh` / `gioca-telegram.ps1` / `gioca-telegram.sh`) e un pacchetto pronto da distribuire (`dist/…zip`).

## Come funziona il motore (in breve)

| Elemento | File | Ruolo |
|---|---|---|
| Avventure | `avventure/<nome>/` | Una sottocartella per avventura. All'avvio il programma mostra il **menu delle avventure**, poi: continua la partita in corso / carica un salvataggio / nuova partita. Ogni avventura è scritta in una lingua sola (quelle di esempio in inglese) e si gioca in qualunque delle cinque lingue dell'interfaccia; `traduzioni/<lingua>.toml` traduce titolo, introduzione, nomi di stanze e oggetti e uscite. |
| Gioco | `avventure/<nome>/gioco.md` | Ambientazione, obiettivi, vittoria/sconfitta, tono, regole globali. Solo l'**Introduzione** è visibile al giocatore. |
| Stanze | `avventure/<nome>/stanze/<id>.md` | Una per stanza: **Descrizione** (pubblica), **Dettagli** (si scoprono esaminando), **Segreti** e **Meccaniche** (note solo al motore e all'AI). Uscite, oggetti e vincoli nel front matter. |
| Stato | `partite/<nome>/stato.json` | Posizione, inventario, flag, oggetti nelle stanze, memoria narrativa, ultimi turni. Riscritto a ogni turno. È **l'unica memoria dell'AI**. Ogni avventura ha il suo. |
| Salvataggi | `partite/<nome>/salvataggi/*.json` | Salvataggi con nome. `/save` salva nel salvataggio corrente, sovrascrive uno esistente o ne crea uno nuovo; `/load` mostra la lista e carica quello scelto. |
| AI | `config.toml` | Fornitori: `openai` (e compatibili, es. OpenRouter), `anthropic`, `gemini`, `ollama`, `lmstudio`, `locale` (file GGUF eseguito dentro il programma con llama.cpp), `finto` (offline per i test). **Profili** per passare dall'uno all'altro con `--profilo`. In locale non serve la chiave; il motore controlla il contesto e avvisa se viene troncato; ragionamento disattivabile; timeout 900 s configurabile. |
| Telegram | `avventura telegram` | Bot privato: menu e conferme diventano pulsanti, ogni utente ammesso ha le proprie partite, i propri salvataggi e la propria lingua. Più utenti giocano insieme, ognuno nella sua partita; `max_turni_paralleli` decide quante risposte dell'AI si generano contemporaneamente (1 per i modelli locali, di più con un servizio cloud). Long polling: niente porte da aprire sul router. |
| Lingue | `lingua.rs`, `preferenze.json`, `traduzioni/*.toml` | Interfaccia in italiano, inglese, francese, spagnolo e tedesco (§7.6 della specifica), con gli alias dei comandi in ogni lingua. Il prompt per l'AI è in italiano per le avventure in italiano e in inglese per tutte le altre; il narratore scrive nella lingua del giocatore. |

Ciclo di un turno: il giocatore scrive un'azione → il motore rilegge lo stato e costruisce il prompt (gioco + stanza corrente con i segreti + uscite disponibili + stato) → l'AI risponde in JSON (narrazione + aggiornamenti) → il motore valida, applica, salva → mostra la narrazione.

---

## Gli script

| # | File | Cosa fa | Modello consigliato |
|---|---|---|---|
| — | `01_SPECIFICHE.md` | Specifica tecnica vincolante. **Non si esegue**: la leggono tutti gli altri. | — |
| 1 | `02_CREA_APPLICAZIONE.md` | Crea il motore in `motore/`: modalità terminale, tutti i fornitori AI via rete, profili, test. | **Opus 5.5** |
| 2 | `03_BOT_TELEGRAM.md` | Aggiunge la modalità bot Telegram. | **Opus 5.5** |
| 3 | `04_MODELLO_LOCALE_GGUF.md` | Aggiunge il fornitore `locale` (GGUF dal disco con llama.cpp, opzionale con GPU CUDA). | **Opus 5.5** |
| 4 | `05_COMPILA_E_TESTA.md` | Lint, test, build release (standard e con modello locale), prove, `dist/` e zip. | Sonnet 5.5 (Opus 5.5 se ci sono errori) |
| 5 | `06_AVVENTURA_ESEMPIO.md` | Crea e collauda le avventure di esempio, scritte in inglese con le traduzioni in italiano, francese, spagnolo e tedesco: *The Punta Nera Lighthouse* / *Il Faro di Punta Nera* e *Star Trucker and the Space Chicken of Doom*. | Sonnet 5.5 |
| 6 | `07_INSTALLATORI.md` | Crea `installa.ps1` e `installa.sh` e li prova. | Sonnet 5.5 |
| 7 | `08_LANCIATORI_E_COLLAUDO.md` | Crea i lanciatori (terminale e Telegram) e collauda tutto: menu, salvataggi, fornitori, bot. | Sonnet 5.5 (Opus 5.5 se emergono bug) |
| 8 | `09_MANUALE.md` | Scrive `docs/MANUALE.md`. Ultimo, così descrive il programma reale. | Sonnet 5.5 |

**Perché questi modelli.** Gli script 02–04 scrivono codice complesso (molti moduli coerenti, concorrenza per il bot, integrazione con una libreria nativa): lì conviene **Claude Opus 5.5** (alias `opus`). Gli script 05–09 sono compilazione, contenuti, installatori, collaudo e documentazione guidati da istruzioni precise: **Claude Sonnet 5.5** (alias `sonnet`) è più veloce ed economico e basta. Sconsigliato Haiku per 02–04. Se il tuo account ha accesso a Fable (alias `fable` o `best`), puoi usarlo per 02–04, ma non è necessario.

Lo script 04 è **facoltativo**: saltalo se non ti interessa caricare file GGUF direttamente (Ollama e LM Studio funzionano comunque).

---

## Prerequisiti (una volta sola)

1. **Claude Code** (serve un piano Pro, Max, Team, Enterprise o un account Console). In PowerShell:
   ```powershell
   irm https://claude.ai/install.ps1 | iex
   ```
   oppure `winget install Anthropic.ClaudeCode`. Apri un **nuovo** terminale e verifica con `claude --version`.
2. **Git for Windows** (consigliato): https://git-scm.com/downloads/win — permette a Claude Code di usare la shell Bash; senza, usa PowerShell (va bene lo stesso).
3. **Rust** (necessario dallo script 02):
   ```powershell
   winget install Rustlang.Rustup
   ```
4. **Microsoft C++ Build Tools** (necessari per compilare Rust su Windows):
   ```powershell
   winget install Microsoft.VisualStudio.2022.BuildTools
   ```
   poi apri *Visual Studio Installer* → Modifica → seleziona **"Sviluppo di applicazioni desktop con C++"** → Installa.
5. Solo per lo script 04 (modello GGUF dal disco): **CMake** (`winget install Kitware.CMake`), **LLVM/Clang** (`winget install LLVM.LLVM`) e, per usare la GPU NVIDIA, il **CUDA Toolkit** (dal sito NVIDIA o `winget install Nvidia.CUDA`). Lo script 04 verifica cosa manca e te lo dice.
6. Chiudi e riapri il terminale; verifica: `rustc --version` e `cargo --version`.

Chiavi API e token Telegram **non servono** per creare e testare il motore: gli script usano l'AI simulata e server di prova locali. Servono solo per giocare davvero.

---

## Come lanciare gli script in Claude Code

### Modo consigliato: sessione interattiva

1. Apri PowerShell (meglio Windows Terminal) nella cartella:
   ```powershell
   cd "D:\AI (Artificial Intelligence)\Progetti Claude\TextAdventuresScript"
   ```
2. Avvia Claude Code con Opus per gli script 02–04:
   ```powershell
   claude --model opus
   ```
3. Al prompt di Claude Code scrivi:
   ```
   Esegui tutte le istruzioni del file @02_CREA_APPLICAZIONE.md, rispettando @01_SPECIFICHE.md.
   ```
   (il simbolo `@` allega il file; puoi anche scrivere semplicemente "Leggi 02_CREA_APPLICAZIONE.md ed eseguilo").
4. Claude Code ti chiederà il permesso per modificare file ed eseguire comandi (`cargo …`). Puoi approvarli uno a uno oppure scegliere "sì, e non chiedere più" per i comandi `cargo`. Premi `Shift+Tab` per passare alla modalità che accetta automaticamente le modifiche ai file.
5. Al termine, prima dello script successivo, svuota il contesto con:
   ```
   /clear
   ```
   e lancia allo stesso modo `@03_BOT_TELEGRAM.md` e poi `@04_MODELLO_LOCALE_GGUF.md`.
6. Dal 05 in poi passa a Sonnet:
   ```
   /model sonnet
   ```
   e continua con 05, 06, 07, 08, 09 (sempre `/clear` tra uno e l'altro).

Ordine: **02 → 03 → 04 → 05 → 06 → 07 → 08 → 09** (il 04 si può saltare). Ogni script si ferma e lo spiega se manca un prerequisito.

### Modo alternativo: non interattivo (un comando per script)

Utile per lasciarlo lavorare da solo. Va autorizzato in anticipo a modificare file ed eseguire i comandi necessari:

```powershell
claude -p "Leggi 02_CREA_APPLICAZIONE.md ed esegui tutte le istruzioni, rispettando 01_SPECIFICHE.md." `
  --model opus --permission-mode acceptEdits `
  --allowedTools "Bash(cargo:*)" "Bash(rustc:*)" "Bash(rustup:*)" "Bash(cmake:*)" "PowerShell"
```

Ripeti cambiando file e modello (`--model sonnet` dal 05 in poi). La modalità interattiva resta preferibile per la prima esecuzione: vedi cosa succede e puoi rispondere alle domande.

### Riprendere dopo un'interruzione

Se una sessione si interrompe, rilancia lo stesso script: le istruzioni dicono a Claude Code di analizzare quanto già esiste e completarlo, non di ricominciare da zero. Puoi anche riprendere la conversazione precedente con `claude --continue`.

---

## Dopo l'ultimo script: giocare

```powershell
# configura l'AI (una volta): copia l'esempio e scegli un profilo
Copy-Item config\config.esempio.toml config\config.toml
# per un fornitore cloud imposta la sua chiave, es.:
[Environment]::SetEnvironmentVariable("ANTHROPIC_API_KEY", "la-tua-chiave", "User")   # poi riapri il terminale

# gratis con Google Gemini (piano gratuito): chiave da https://aistudio.google.com/apikey
[Environment]::SetEnvironmentVariable("GEMINI_API_KEY", "la-tua-chiave", "User")      # poi riapri il terminale
.\gioca.ps1 -Profilo gemini

# gioca da terminale
.\gioca.ps1 -Profilo claude        # oppure: -Profilo ollama, -Profilo gguf, -Profilo gemini…
.\gioca.ps1 -Offline -Lingua en    # in inglese (al primo avvio il gioco chiede comunque la lingua)

# oppure avvia il bot Telegram (istruzioni BotFather nel manuale, Parte III)
.\gioca-telegram.ps1 -Profilo ollama
```

Senza chiave puoi provare subito in modalità offline: `.\gioca.ps1 -Offline`.
All'avvio scegli l'avventura dal menu. In gioco: `/save` per salvare con un nome, `/load` per caricare, `/language` per cambiare lingua, `/help` per tutti i comandi.
Tutti i dettagli sono in `docs/MANUALE.md`.

---

## Personalizzare

- Vuoi cambiare il formato dei file, il protocollo o aggiungere funzioni? Modifica `01_SPECIFICHE.md` **prima** di eseguire il 02 (o rilancia il 02 dopo averla modificata: Claude Code allineerà il codice).
- Vuoi scrivere una nuova avventura? Crea una sottocartella in `avventure/` seguendo la Parte IV del manuale, poi `avventura valida --avventura <nome>`: comparirà da sola nel menu, con stato e salvataggi propri.

---

## Stato del progetto

- **Data:** 2026-11-07
- **Versione:** avventura 0.1.0
- **Test:** `cargo test` superato (tutti i test unitari e di integrazione, nessun fallimento); fmt e clippy puliti all'ultimo collaudo.
- **Collaudo:** 14 punti su 14 superati (vedi `docs/COLLAUDO.md`); un bug (`--nuova` senza conferma) trovato e corretto.
- **Manuale:** `docs/MANUALE.md` (copia in `dist/MANUALE.md`).
- **Pacchetto:** `dist/avventura-0.1.0-x86_64-pc-windows-msvc.zip`

> Nota: questo stato si riferisce alla prima versione (script con la numerazione precedente). Dopo l'aggiornamento a fornitori AI multipli e bot Telegram vanno rilanciati gli script dal 02 in poi.

---

## Stato del progetto (aggiornamento dopo lo script 09)

- **Data:** 2026-11-08
- **Versione:** avventura 0.1.0
- **Test:** `cargo test` superato: 199 test, 0 falliti (collaudo dello script 08: 19 punti, 18 superati e 1 saltato — bot Telegram reale, nessun token sul PC — con un bug trovato e corretto; vedi `docs/COLLAUDO.md`).
- **Manuale:** `docs/MANUALE.md` riscritto sul programma reale (installazione con tutti i fornitori AI, utilizzo, bot Telegram, creazione di avventure, riferimento tecnico); copia in `dist/MANUALE.md`.
- **Pacchetto:** `dist/avventura-0.1.0-x86_64-pc-windows-msvc.zip` (31 voci, include il manuale aggiornato).

---

## Modifiche successive alla prima esecuzione degli script

Fatte direttamente sul progetto e già riportate negli script 01–09 (rieseguendoli si ottiene lo stesso risultato):

1. **Gemini**: riconoscimento dei parametri rifiutati anche quando Google li cita a parole ("Thinking level MINIMAL is not supported" → nuovo tentativo senza `thinkingLevel`); profilo `gemini` di `config.esempio.toml` (`gemini-3.8-flash`, `ragionamento = "basso"`, perché il livello minimo non è supportato da quel modello).
2. **Bot Telegram con più giocatori**: il messaggio "Il narratore è occupato con un altro giocatore, sei in coda…" dipende da `telegram.max_turni_paralleli` (default 1, adatto ai modelli locali); con Gemini in cloud le configurazioni pubblicate usano 4.
3. **Pubblicazione su GitHub**: `README.md` in inglese e `README.it.md` in italiano; configurazioni `config/*.toml` pubblicate senza chiavi API, ID Telegram né codice d'invito (i dati personali restano in `config/config.toml`, escluso da git); `docs/ApiKeyVari.txt` escluso da git.
4. **Space Chicken**: la catena di fast-food del complotto si chiama ora **BFB - Betelgeuse Fried Blobchicken**.
5. **Lingue** (§7.6): interfaccia italiano/inglese con rilevamento della lingua (Windows/`LANG` o app Telegram, inglese se non determinabile), domanda al primo utilizzo, `/language`, `--lingua`/`-Lingua`, `interfaccia.lingua`, `AVVENTURA_LINGUA`; prompt per l'AI nella lingua dell'avventura; titoli delle sezioni anche in inglese; campo `storia` per eventuali versioni complete in più lingue; AI simulata bilingue; comandi del bot registrati nelle due lingue.
6. **Una sola versione per avventura** (§3.4, §7.6): il narratore AI scrive nella lingua dell'interfaccia del giocatore anche se l'avventura è scritta in un'altra (sezione "lingua della narrazione" del prompt, con il glossario dei nomi tradotti); file facoltativi `traduzioni/<lingua>.toml` per titolo, descrizione breve, introduzione, nomi di stanze e oggetti e descrizioni delle uscite, controllati da `valida`; direzioni standard tradotte dal motore (`nord` = `north`). Le avventure di esempio restano **solo in inglese** (cartelle `faro-punta-nera` e `space-chicken`, con `traduzioni/it.toml`); le versioni italiane sono state tolte. Le vecchie partite italiane sul PC di sviluppo sono state spostate in `partite/_archivio-versione-italiana/` (gli id delle stanze sono cambiati); in generale una partita o un salvataggio con una stanza che non esiste più viene segnalato come "di una versione precedente" invece di essere caricato. Gli installatori sostituiscono per intero le avventure del pacchetto e tolgono le vecchie cartelle `-en`.
7. **Francese, spagnolo e tedesco** (§7.2, §7.6, §9): interfaccia completa anche in francese, spagnolo e tedesco (menu, messaggi, `/help`, conferme `o`/`s`/`j`, bot Telegram con i comandi registrati in cinque lingue); alias dei comandi in ogni lingua (`/aide`, `/ayuda`, `/hilfe`, `/sauver`, `/guardar`, `/speichern`…); direzioni standard nelle cinque lingue con sinonimi (`norden` = `north` = `nord`); AI simulata che capisce e risponde nelle cinque lingue; `traduzioni/fr.toml`, `es.toml`, `de.toml` per le due avventure di esempio. Il prompt per l'AI resta in italiano o in inglese (per le avventure scritte in francese, spagnolo o tedesco è in inglese).
8. **Oggetti che ricomparivano** (§6.2): il narratore descriveva come presenti oggetti già raccolti, perché riceveva gli `oggetti` iniziali nel front matter e la Descrizione originale che li cita. Ora il front matter della stanza corrente va al prompt senza `oggetti` e `oggetti_nascosti`, e una sezione «Stato attuale della stanza» (oggetti presenti ora, nascosti ancora da scoprire, inventario, note) prevale esplicitamente sulla Descrizione; la regola 7 del narratore lo ribadisce.
9. **Space Chicken**: Molly raccoglie la mozzarella acida solo dopo aver scoperto in laboratorio a cosa serve (`weakness_found`); prima si rifiuta con una scusa legata alla puzza, senza svelarne l'utilità. I bancali nella stiva non finiscono mai.
10. **Log per i timeout** (§11): `avventura.log` registra a livello `info` i tempi di ogni tentativo HTTP e del flusso (intestazioni, primo dato, pausa massima, totale), le interruzioni con i byte già ricevuti e, per Gemini, `finishReason`, token (anche di ragionamento) e `modelVersion`.
11. **Avviso open source** (§7.1): a ogni avvio (terminale e bot, dopo la scelta della lingua) e a ogni `/start` durante una partita su Telegram, il programma dice nelle cinque lingue che è open source e che il codice è su GitHub (`https://github.com/Explo-bot/TextAdventures`).
12. **Terza avventura: *Vinny Falco: The Devil's Slide Affair*** (`avventure/vinny-falco/`, script 06): noir hard-boiled nella San Francisco del 1928, ispirato ai personaggi e alla trama del romanzo *Vinny Falco Detective* di Peter A. Ray (testi originali). 11 stanze, 100 turni, scritta in inglese con traduzioni it/fr/es/de; Madame Sybilla fa da sistema di suggerimenti; due finali vincenti (Julius salvato o finale amaro come nel romanzo).

## Stato del progetto (cinque lingue)

- **Data:** 2026-11-08
- **Versione:** avventura 0.1.0
- **Test:** `cargo test` superato: 222 test, 0 falliti; `cargo fmt` e `cargo clippy --all-targets -- -D warnings` puliti.
- **Prove manuali:** partite offline sul Faro (scritto in inglese) in francese, spagnolo e tedesco con comandi nella lingua scelta (`est`/`prends le bidon`/`/sorties`, `este`/`coge la lata`/`/salidas`, `osten`/`nimm den kanister`/`/ausgänge`), titolo, introduzione, nomi, `/uscite`, `/inventario` e `/help` tradotti; in precedenza partita in italiano e menu in italiano e in inglese; `valida` senza errori né avvisi per le due avventure; `dist\installa.ps1` provato sopra una vecchia installazione simulata (cartelle `-en` tolte, stanze italiane sostituite, avventure dell'utente conservate).
- **Pacchetto:** `dist/avventura-0.1.0-x86_64-pc-windows-msvc.zip` rigenerato con le due avventure (traduzioni in quattro lingue) e il manuale aggiornato.
