# Avventura: avventure testuali narrate da un'AI

[English](README.md) · **Italiano**

**Avventura** è un motore per avventure testuali scritto in Rust. Il mondo di gioco (stanze, oggetti, enigmi, segreti, condizioni di vittoria e sconfitta) è descritto in semplici file Markdown; a ogni turno il giocatore scrive un'azione in linguaggio naturale e un modello di intelligenza artificiale fa da **narratore**, rispettando le regole dell'avventura. Il motore tiene lo stato della partita (posizione, inventario, flag), valida le risposte dell'AI e salva tutto su disco.

Si gioca in **italiano, inglese, francese, spagnolo o tedesco** (al primo avvio il gioco chiede la lingua, proponendo quella del sistema o dell'app Telegram; si cambia con `/language`), **da terminale** oppure **da Telegram** (anche dal telefono), con un **modello locale** sul tuo PC oppure con un **modello in cloud**.

```
> apri la botola con la chiave d'ottone
La chiave gira a fatica nella serratura incrostata di salsedine. Con un cigolio
la botola si solleva: sotto, una scala di legno scende nel buio della cantina...
```

## Caratteristiche

- **Menu delle avventure** all'avvio, poi *Continua / Carica / Nuova partita*.
- **Salvataggi con nome** (`/save`, `/load`), annullamento dell'ultimo turno (`/undo`), inventario, aiuto e altri comandi.
- **Stato su disco**: la partita riprende esattamente da dove l'hai lasciata.
- **Validazione delle risposte dell'AI**: uscite, oggetti e flag sono controllati dal motore, così il narratore non può "barare".
- **Cinque lingue** (italiano, inglese, francese, spagnolo, tedesco): menu, messaggi, `/help` e conferme in ogni lingua, e ogni comando ha un alias in ciascuna lingua (`/save`, `/salva`, `/sauver`, `/guardar`, `/speichern`). Ogni avventura è scritta in una lingua sola e il narratore risponde nella lingua del giocatore; un file facoltativo `traduzioni/<lingua>.toml` traduce ciò che il motore mostra da solo (titolo, introduzione, nomi di stanze e oggetti, uscite). Le avventure incluse sono scritte in inglese, perché i modelli AI (soprattutto quelli locali piccoli) seguono le istruzioni in inglese in modo più affidabile.
- **Bot Telegram**: menu e conferme diventano pulsanti; più giocatori possono usarlo nello stesso momento, ognuno con le proprie partite, i propri salvataggi e la propria lingua. Funziona in long polling: non serve aprire porte sul router.
- **Nuove avventure** facili da scrivere: un file `gioco.md` più un file per stanza (vedi il manuale, Parte IV).
- **AI simulata offline** (profilo `prova`) per provare il programma senza rete e senza chiavi.

## Avventure incluse

| Avventura | Genere | Trama |
|---|---|---|
| **Il Faro di Punta Nera** | racconto marinaresco | Ottobre 1893, notte di tempesta. Sei Nicola, diciassette anni, aiutante del guardiano del faro. Il faro è spento e un brigantino punta dritto verso gli scogli: hai poco tempo per riaccenderlo. |
| **Star Trucker and the Space Chicken of Doom** | commedia sci-fi da B-movie | Anno 2122, cargo spaziale *Rusty Badger*. Delle uova aliene si sono schiuse e i "pulcini" sono cresciuti. Molto. Sei Molly, la meccanica di bordo, forse l'unica sopravvissuta (a parte il gatto Mr. Whiskers). |

Le due avventure sono scritte in inglese (`avventure/faro-punta-nera/`, `avventure/space-chicken/`) e hanno la traduzione italiana, francese, spagnola e tedesca di titolo, introduzione e nomi (`traduzioni/it.toml`, `fr.toml`, `es.toml`, `de.toml`): si giocano in tutte e cinque le lingue. Per aggiungere un'avventura basta una nuova cartella in `avventure/`.

## Modelli AI supportati

Il narratore può essere un modello in **cloud** oppure un modello **locale**. Si cambia fornitore dal file di configurazione o con `--profilo`, senza ricompilare.

### In cloud

| Fornitore | Servizio | Chiave (variabile d'ambiente) |
|---|---|---|
| `gemini` | **Google Gemini** (es. `gemini-3.8-flash`) | `GEMINI_API_KEY` |
| `anthropic` | Anthropic **Claude** (es. `claude-sonnet-5-5`) | `ANTHROPIC_API_KEY` |
| `openai` | **OpenAI** e servizi compatibili con la stessa API (es. **OpenRouter**) | `OPENAI_API_KEY` (o quella indicata in `api_key_env`) |

> ### ⭐ Gemini con il piano gratuito
> Il modo più semplice per iniziare **senza spendere nulla** è Google Gemini: basta un account Google per creare una chiave su **[Google AI Studio](https://aistudio.google.com/apikey)**, e i modelli *Flash* sono utilizzabili con il **piano gratuito**, che ha limiti di richieste al minuto e al giorno ma è più che sufficiente per giocare. È pronto il profilo `gemini` in [`config/config.esempio.toml`](config/config.esempio.toml).
>
> Da sapere: con il piano gratuito Google può usare i testi inviati per migliorare i propri servizi (non scrivere dati personali nelle azioni) e i limiti e i modelli disponibili possono cambiare: controlla le condizioni aggiornate su AI Studio. Se usi il bot Telegram con più giocatori, tieni basso `max_turni_paralleli` per non superare i limiti al minuto.

Gli altri servizi in cloud si pagano a consumo: ogni turno invia l'intera avventura e lo stato (alcune migliaia di token), quindi conviene impostare un limite di spesa.

### In locale (gratis, privato, senza internet)

| Fornitore | Come funziona |
|---|---|
| `ollama` | Server [Ollama](https://ollama.com) sul tuo PC. |
| `lmstudio` | Server di [LM Studio](https://lmstudio.ai) (API compatibile OpenAI su `localhost:1234`). |
| `locale` | Un file **GGUF** caricato direttamente dal programma con llama.cpp (eseguibile compilato con `--features locale`, anche con GPU **CUDA**). |

Con un modello locale nessun testo esce dal computer e non c'è alcun costo per turno; la qualità e la velocità dipendono dal modello e dalla GPU. Consigliati modelli con almeno 16.000 token di contesto.

## Avvio rapido (Windows)

1. Scarica lo zip da `dist/` e scompattalo, poi esegui l'installatore:

   ```powershell
   .\installa.ps1
   ```

2. Imposta la chiave del fornitore scelto (esempio Gemini) e riapri il terminale:

   ```powershell
   [Environment]::SetEnvironmentVariable("GEMINI_API_KEY", "la-tua-chiave", "User")
   ```

3. Gioca:

   ```powershell
   Copy-Item config\config.esempio.toml config\config.toml   # una volta sola
   avventura gioca --profilo gemini
   .\gioca.ps1 -Offline        # prova senza AI e senza rete (-Lingua en, fr, es, de per le altre lingue)
   ```

Su Linux e macOS si compila dai sorgenti (`cargo build --release` nella cartella `motore/`) e si usano `installa.sh` e `gioca.sh`.

### Bot Telegram

Crea un bot con **@BotFather**, metti il token nella variabile `TELEGRAM_BOT_TOKEN`, aggiungi il tuo ID Telegram a `utenti_ammessi` e `amministratori` nella sezione `[telegram]` della configurazione, poi:

```powershell
.\gioca-telegram.ps1
```

Se non conosci il tuo ID, scrivi al bot: ti risponderà con il tuo ID numerico.

Per impostazione predefinita il bot genera una risposta dell'AI alla volta (`max_turni_paralleli = 1`, giusto per i modelli locali): gli altri giocatori vedono «Il narratore è occupato con un altro giocatore, sei in coda…». Con un modello in cloud alzalo (per esempio a 4).

## File di configurazione

| File | Uso |
|---|---|
| [`config/config.esempio.toml`](config/config.esempio.toml) | Tutte le opzioni commentate, con profili d'esempio per ogni fornitore. |

Le chiavi API **non vanno mai scritte nei file**: si leggono dalle variabili d'ambiente. La tua configurazione personale va in `config/config.toml`, che è escluso da git.

## Documentazione

- [`docs/MANUALE.md`](docs/MANUALE.md): manuale completo (installazione, configurazione dell'AI, comandi di gioco, lingue, bot Telegram, creazione di nuove avventure, anche in inglese).
- [`LEGGIMI.md`](LEGGIMI.md): come è stato costruito il progetto. Il motore è generato da una serie di **script Markdown per Claude Code** (`01_SPECIFICHE.md` … `09_MANUALE.md`) che, eseguiti in ordine, producono il codice, le avventure, gli installatori e il manuale.

## Struttura del repository

```
avventure/         le avventure (gioco.md + stanze/*.md + traduzioni/*.toml)
config/            file di configurazione d'esempio
motore/            sorgenti Rust del motore
docs/              manuale e collaudo
dist/              pacchetto pronto per Windows
0X_*.md            script di generazione per Claude Code
```
