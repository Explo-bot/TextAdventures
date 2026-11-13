# 07 — Script di installazione

**Modello consigliato:** Claude Sonnet 5.5 (`claude-sonnet-5-5`).
**Durata indicativa:** 5 minuti.
**Prerequisiti:** script 05 e 06 completati (servono `dist/` e l'avventura di esempio).

> Le **istruzioni di installazione** per l'utente non stanno in un documento separato: le scrive lo script 09 dentro il manuale (`docs/MANUALE.md`, Parte I). Questo script crea gli installatori che il manuale descriverà.

---

## Istruzioni per Claude Code

Crea nella radice `installa.ps1` (Windows) e `installa.sh` (Linux/macOS), conformi a `01_SPECIFICHE.md` (in particolare §2: cartella `avventure/` con una sottocartella per avventura, cartella `partite/` con una sottocartella per avventura).

### Layout installato (da rispettare)

```
<Destinazione>/
├── avventura(.exe)
├── avventura-cuda.exe         (solo se presente in dist/ e scelto, vedi sotto)
├── modelli/                   (creata vuota con LEGGIMI.txt: qui i file GGUF)
├── config.toml                (creato da config.esempio.toml solo se non esiste)
├── config.esempio.toml
├── avventure/
│   ├── faro-punta-nera/ …     avventure di esempio (scritte in inglese, con traduzioni it/fr/es/de)
│   └── …                      (+ eventuali altre avventure presenti in dist/avventure/)
└── partite/                   (creata dal programma al primo avvio)
```

L'eseguibile trova `avventure/` e `config.toml` accanto a sé (§5, §7): verificalo nel codice; se non è così, correggi il motore (con test) e annotalo nel report.

### `installa.ps1`
- Parametri: `-Destinazione` (default `$env:LOCALAPPDATA\Programs\Avventura`), `-DaSorgente` (switch: compila con `cargo build --release` prima di copiare), `-ConModelloLocale` (switch: installa anche la variante `avventura-cuda.exe`; con `-DaSorgente` la compila con `--features cuda`, o `locale` se manca CUDA), `-NoPath` (switch: non toccare il PATH).
- Se c'è una GPU NVIDIA (`nvidia-smi` risponde) e `avventura-cuda.exe` è disponibile ma `-ConModelloLocale` non è stato indicato, lo propone con una domanda.
- **Non tocca mai** la cartella `modelli/` esistente (i file GGUF dell'utente).
- Se `-DaSorgente` e `cargo` non è disponibile: messaggio chiaro (installare Rust con `winget install Rustlang.Rustup` e i Microsoft C++ Build Tools con il carico di lavoro "Sviluppo di applicazioni desktop con C++") e uscita con codice 1.
- Copia l'eseguibile (da `dist/` o da `motore/target/release/`), `config.esempio.toml` e **tutte** le sottocartelle di `avventure/`.
  - Avventure già presenti nella destinazione: **sostituisce per intero** le cartelle delle avventure del pacchetto (così non restano file di una versione precedente, per esempio le stanze con gli id italiani), toglie le vecchie cartelle separate `faro-punta-nera-en` e `space-chicken-en` e **non cancella** avventure aggiunte dall'utente. La cartella `partite\` non viene mai toccata: le partite di una versione precedente con stanze che non esistono più il gioco le segnala come non continuabili.
  - **Non tocca mai** `partite/` (stati e salvataggi) né un `config.toml` esistente.
- Aggiunge la destinazione al `PATH` **utente** solo se non presente.
- Messaggi in italiano colorati (`Write-Host -ForegroundColor`); riepilogo finale con i prossimi passi: scegliere un profilo in `config.toml` (cloud con chiave, Ollama/LM Studio, modello GGUF in `modelli\`), lanciare `avventura` o il bot con `avventura telegram`, dove aggiungere nuove avventure (`<Destinazione>\avventure\<nome>\`), e che al primo avvio il gioco chiede la lingua (italiano, inglese, francese, spagnolo o tedesco; si cambia con `/language`).
- Idempotente; compatibile con Windows PowerShell 5.1 e PowerShell 7.
- Opzione `-Disinstalla`: rimuove eseguibile e voce dal PATH; chiede conferma separata prima di cancellare `partite/` (i salvataggi) e `avventure/`.

### `installa.sh`
Equivalente in bash: destinazione default `~/.local/share/avventura`, link simbolico in `~/.local/bin/avventura`, stesse regole su `config.toml`, `partite/` e avventure dell'utente; opzioni `--da-sorgente`, `--destinazione DIR`, `--disinstalla`.

**Pacchetto Debian 13.** `installa.sh` deve funzionare, senza `--da-sorgente` e senza root, dentro lo scompattato di `dist/avventura-<versione>-x86_64-debian13.tar.gz` (script 05), dove l'eseguibile `avventura`, `avventure/` e `config.esempio.toml` stanno accanto allo script (la ricerca dei percorsi cerca sia in `dist/` sia nella cartella dello script). Dopo l'installazione `~/.local/bin/avventura` è un link a `<Destinazione>/avventura`; se `~/.local/bin` non è nel PATH lo script suggerisce la riga da aggiungere a `~/.bashrc` (su Debian lo mette già `~/.profile` al login successivo, se la cartella esiste).

### Controlli
- Su Windows: esegui `installa.ps1 -Destinazione <cartella temporanea> -NoPath` **due volte** (idempotenza), aggiungi a mano una seconda avventura finta nella destinazione e un file in `partite/`, rilancia e verifica che non vengano toccati. Lancia l'eseguibile installato con `elenca` e verifica che veda le avventure. Poi chiedi conferma prima di cancellare la cartella temporanea.
- Altrove (Linux): stessi controlli con `installa.sh`, anche dal pacchetto `.tar.gz` per Debian 13 con `HOME` temporanea; per `installa.ps1` almeno la verifica di sintassi con `pwsh` se disponibile.
- Copia `installa.ps1` e `installa.sh` in `dist/` e rigenera lo zip.

### Report finale
File creati, esito dei controlli, eventuali correzioni al motore.
