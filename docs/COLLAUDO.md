# Collaudo del programma (script 08)

Data: 2026-11-08 · Ambiente: Windows 11, eseguibile release `avventura 0.1.0` (variante standard, senza
llama.cpp), PowerShell 7 e Windows PowerShell 5.1, Git Bash. Tutto in cartelle temporanee (il contenuto
reale di `partite/` non è stato toccato). Server di prova locali su `127.0.0.1` per i fornitori HTTP
(OpenAI, LM Studio, Anthropic, Gemini, Ollama) e per l'API Bot di Telegram; chiavi e token finti.

| # | Punto | Esito |
|---|---|---|
| 1 | Menu avventure: Faro e `mini2` con titolo, descrizione e `[nuova]`; input non valido (`x`, `9`) rifiutato con messaggio, riga vuota ignorata, `0`/`q` escono; scelta per numero | OK |
| 2 | Nuova partita nel Faro: introduzione, `guarda`, `est`, `prendi tanica`, `ovest` | OK |
| 3 | `/save`: nuovo «inizio»; «Salva nel corrente» sovrascrive senza chiedere; nuovo «secondo»; «Sovrascrivi esistente» scegliendo «inizio» dalla lista, con conferma `s` | OK |
| 4 | Nomi non validi rifiutati con il motivo: `CON` (riservato Windows), `a/b` (carattere vietato), 41 caratteri (max 40); Invio = nome predefinito `stanza-turno-N` | OK |
| 5 | `/load` senza argomenti: lista con nome, turno, stanza, punti, data e `*` sul corrente; caricamento per numero e per nome; conferma sui turni non salvati (`s` carica, `n` annulla); nome inesistente → messaggio | OK |
| 6 | `/saves`; `/delete secondo` (con `n` non elimina, con `s` sì); `/undo` ripetuto | OK |
| 7 | `/menu` avvisa dei turni non salvati; il Faro mostra `[partita in corso: turno N · M salvataggi]`; `inizio` salvato anche nella seconda avventura, indipendente | OK |
| 8 | `/quit` con progressi non salvati: promemoria, risposta `n` esce senza salvare (vedi **Bug 2**) | **BUG corretto**, poi OK |
| 9 | Riavvio → Faro → Continua riprende dal turno corretto; Faro → Carica un salvataggio → lista corretta | OK |
| 10 | `--avventura faro-punta-nera --carica inizio`; `--nuova` con conferma (`n` annulla, `s` procede); `--carica` inesistente → errore | OK |
| 11 | Sottocomandi `elenca` e `salvataggi --avventura faro-punta-nera` | OK |
| 12 | `partite/faro-punta-nera/` e `partite/mini2/` separate, ciascuna con `stato.json`, `storico/`, `salvataggi/`; JSON validi; `salvataggio_corrente` coerente col file esistente | OK |
| 13 | `gioca.ps1 -Offline` (PS 7 e PS 5.1), `gioca.bat -Offline`, `gioca.sh --offline` mostrano il menu; `-Avventura/-Nuova/-Debug/-Timeout` passati al motore; `-Nuova` con `-Carica` rifiutato (rc 2); `-Profilo` inesistente → messaggio del motore e proposta di avvio offline | OK |
| 14 | Server Ollama simulato: nessuna chiave richiesta, nessun header `Authorization`, `options.num_ctx` presente, avviso di troncamento mostrato; risposta dopo 3 s con `--timeout 1` → «Nessuna risposta entro 1 s. Riprovare? (s/n)» e con `n` il turno resta invariato | OK |
| 15 | Fornitori HTTP simulati, 3 turni ciascuno: **openai** (Bearer, `response_format`, `reasoning_effort`), **lmstudio** (nessun `Authorization`, `json_schema`), **anthropic** (`x-api-key`, `anthropic-version`, `system`, `output_config`, `thinking: adaptive`, niente `temperature`), **gemini** (`x-goog-api-key`, `:generateContent`, `systemInstruction`, `responseMimeType`, `thinkingLevel`), **ollama** (`/api/chat`, `num_ctx`, `stream:false`, `format:json`); risposte lette, punteggio e turno aggiornati; chiavi finte assenti da log e output | OK (33 controlli) |
| 16 | `avventura profili` elenca tutti i profili; `--profilo` e `AVVENTURA_PROFILO` cambiano fornitore (la riga di comando ha la precedenza); chiave mancante per un profilo cloud → errore chiaro prima del menu; profilo inesistente → errore | OK |
| 17 | Fornitore `locale`: con l'eseguibile standard messaggio esplicativo. Con la variante `cuda` compilata per l'occasione e `Qwen2.5-14B-Instruct-Q4_K_M.gguf` (8,37 GB, RTX 4060 Ti): `prova-ai --solo-config` carica il modello (48/48 strati su GPU, contesto 8.192, grammatica JSON dallo schema, 88 s); partita reale nel Faro con 4 turni (`guarda`, `prendi la tanica`, `vai a est`): narrazione coerente, stanza aggiornata alla rimessa, stato su disco corretto, ≈75 s in tutto compreso il caricamento | OK |
| 18 | Bot Telegram simulato (utenti 100 e 200 ammessi, 555 no): `--verifica` (token mai stampato); 555 riceve solo il proprio ID (testo, foto, `/start`) e non ha cartelle; `/start` → pulsanti delle avventure → nuova partita → azioni con risposte AI e formattazione HTML; `/save` (nuovo, nome proposto come pulsante) e `/load` (pulsanti, conferma, `★`); secondo utente in parallelo con cartelle separate `partite/telegram/<id>/<avventura>/`; messaggio durante un turno → «Sto ancora scrivendo la risposta precedente…» + `sendChatAction`; messaggio da gruppo → risposta esplicativa; Ctrl+C → uscita ordinata (rc 0) con `offset.json` aggiornato; token assente da output e log | OK (19 controlli) |
| 19 | Bot Telegram reale: **saltato**, `TELEGRAM_BOT_TOKEN` non è impostato su questo PC (nessun token da usare) | saltato |

## Lanciatori

- `gioca.ps1` / `gioca.sh`: nuovi `-Profilo`/`--profilo`; i controlli preliminari sono delegati al motore
  (`avventura prova-ai --solo-config`); se falliscono propongono l'avvio offline; se il profilo usa il
  fornitore `locale` cercano prima `avventura-cuda` / `avventura-locale`. `-Offline` imposta
  `AVVENTURA_FORNITORE=finto` (così funziona anche senza `[profili.prova]` nel config).
- `gioca-telegram.ps1` / `.sh` / `.bat` (nuovi): controllano token e utenti ammessi con le istruzioni per
  BotFather e per scoprire il proprio ID, eseguono `telegram --verifica` e poi il bot in primo piano.
  `-InstallaAvvioAutomatico` / `-RimuoviAvvioAutomatico` (solo `.ps1`) gestiscono l'attività pianificata
  «Avventura - bot Telegram»; provata solo la parte di anteprima e il rifiuto (`n`): l'attività **non** è
  stata creata sul PC.
- `gioca.bat`: la pausa finale compare solo con il doppio clic (rilevato dal percorso completo nella
  riga di comando di `cmd`).
- Per Windows PowerShell 5.1: file `.ps1` con BOM UTF-8 (accenti) e controllo preliminare eseguito con
  `System.Diagnostics.Process`, che evita il rumore degli errori nativi.

## Compilazione della variante con modello locale (Windows)

- Riuscita con CMake 4.4, LLVM (LIBCLANG_PATH), CUDA 13.5 e Visual Studio 18 Build Tools in 19 min. Con CUDA 12.4 la configurazione fallisce (toolchain VS troppo recente).
- Servono: cartella di build **corta** (--target-dir C:\tc: i percorsi lunghi rompono MSBuild), CUDA_PATH/CudaToolkitDir che puntano a CUDA 13.5 e CMAKE_CUDA_ARCHITECTURES=89 (GPU Ada).
- A runtime l'eseguibile cerca le DLL CUDA (cudart64_13.dll, cublas64_13.dll...): senza di esse termina con codice 0xC0000135. Vanno messe nel PATH (...\CUDA\v13.5\bin\x64) o accanto all'eseguibile. L'eseguibile (62 MB) non è stato aggiunto a dist/ né allo zip.

## Bug trovati e corretti

1. **Un'opzione per il collaudo mancava** (non è un bug del programma): per provare il bot con l'eseguibile
   reale serviva poter puntare l'API Telegram a un server di prova. Aggiunta la variabile d'ambiente
   non documentata `AVVENTURA_TELEGRAM_API` (`motore/src/sottocomandi.rs`).
2. **`/quit` non accettava `s`/`n`**: il promemoria sui progressi non salvati è un menu numerato
   («1) Sì, salva — 2) No, esci senza salvare») e una risposta `n` veniva rifiutata («Scelta non valida»),
   mentre la specifica (§7.2) prevede «s/n/annulla». Ora `s` e `n` valgono come `1` e `2`, senza cambiare
   il testo del prompt (`domanda_tre_vie` in `motore/src/menu.rs`, prompt in `canale/mod.rs`). Test di
   regressione: `quit_accetta_s_e_n_oltre_ai_numeri` (`tests/salvataggi_e_menu.rs`).

## Note

- Nei menu numerati `0` equivale a `q` (annulla): comportamento voluto, non un errore.
- Le verifiche automatiche (menu, salvataggi, fornitori, bot) sono script Python temporanei che pilotano
  l'eseguibile reale tramite standard input e server HTTP finti; non fanno parte del repository.
- Qualità: `cargo fmt`, `cargo clippy --all-targets -- -D warnings` e `cargo test` (199 test) puliti;
  `dist/` riallineato (eseguibile ricompilato, lanciatori inclusi `gioca-telegram.*`) e
  `avventura-0.1.0-x86_64-pc-windows-msvc.zip` rigenerato (31 voci).

## Pacchetto Debian 13 (2026-11-13)

- Compilazione `cargo build --release` su Ubuntu 24.04 (WSL, glibc 2.39) in 4 min 31 s; sorgenti copiati fuori da `/mnt/d` senza `motore/target`. Eseguibile 8,4 MB; `ldd`: solo `libc.so.6` e `libgcc_s.so.1`; simboli glibc fino a `GLIBC_2.38` (Debian 13 ha la 2.41).
- `cargo test --release`: 241 test superati. Attenzione: `config::test::file_di_esempio_valido` legge `../config/config.esempio.toml`, quindi la copia dei sorgenti per la build deve includere anche `config/` (senza, il test fallisce).
- Installazione dal `.tar.gz` con `HOME` temporanea: `installa.sh` (due volte), file in `partite/` e avventura aggiunta dall'utente conservati, `avventura --version`, `avventura elenca`, partita offline in italiano e menu in inglese con `gioca.sh --offline`, `installa.sh --disinstalla`: tutto corretto.
- Non provato su un'installazione reale di Debian 13.
