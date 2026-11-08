# 08 — Lanciatori e collaudo completo

**Modello consigliato:** Claude Sonnet 5.5 (`claude-sonnet-5-5`); passa a Opus 5.5 se il collaudo trova bug nel motore da correggere.
**Durata indicativa:** 10–15 minuti.
**Prerequisiti:** script 02–07 completati.

---

## Istruzioni per Claude Code

Crea i lanciatori `gioca.ps1`, `gioca.bat`, `gioca.sh`, `gioca-telegram.ps1`, `gioca-telegram.sh` nella radice e collauda tutto il programma: **scelta tra più avventure**, **salvataggi con nome**, **fornitori AI**, **bot Telegram** (§4.1, §5, §7.1–7.5 di `01_SPECIFICHE.md`).

### `gioca.ps1`
- Parametri: `-Avventura` (id; se omesso il programma mostra il menu di scelta), `-Carica NOME`, `-Nuova`, `-Debug`, `-Profilo NOME`, `-Offline` (equivale a `-Profilo prova`), `-Config`, `-Lingua it|en|fr|es|de` (con `ValidateSet`, passato come `--lingua`, §7.6).
- Cerca l'eseguibile: `PATH` → `dist\` → `motore\target\release\`; se il profilo scelto usa `fornitore = "locale"` cerca prima `avventura-cuda.exe`/`avventura-locale.exe`. Se non lo trova propone di compilare e compila se l'utente accetta.
- Parametro `-Timeout SECONDI` passato come `--timeout`.
- Controlli preliminari delegati al motore: esegue `avventura prova-ai --profilo <profilo> --solo-config` (aggiungi questa opzione al motore se non esiste: verifica configurazione, chiave e raggiungibilità del server senza inviare prompt). Se il controllo fallisce mostra il messaggio e chiede se avviare in modalità offline (`-Profilo prova`).
- Console in UTF-8 (`[Console]::OutputEncoding = [Text.Encoding]::UTF8`, `chcp 65001 > $null`).
- Passa le opzioni al motore (`gioca --avventura … --carica … --nuova …`) e restituisce il suo codice di uscita.

### `gioca.bat`
- `powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0gioca.ps1" %*` — funziona col doppio clic.
- `pause` finale solo se lanciato con doppio clic.

### `gioca.sh`
Equivalente bash (`--avventura`, `--carica`, `--nuova`, `--debug`, `--profilo`, `--offline`, `--config`, `--timeout`, `--lingua`).
I messaggi dei lanciatori stessi restano in italiano; il gioco segue la lingua scelta (§7.6).

### `gioca-telegram.ps1` / `gioca-telegram.sh`
- Parametri: `-Profilo NOME`, `-Config`, `-Verifica`.
- Se `TELEGRAM_BOT_TOKEN` non è impostato: spiega in breve come ottenere il token da @BotFather e come impostarlo (`[Environment]::SetEnvironmentVariable("TELEGRAM_BOT_TOKEN", "…", "User")`), rimanda al manuale ed esce.
- Se `utenti_ammessi` è vuoto e non c'è `codice_invito`: avvisa che nessuno potrà giocare e spiega come trovare il proprio ID (scrivere al bot: risponde con l'ID).
- Esegue `avventura telegram --verifica`, poi `avventura telegram` in primo piano; console UTF-8; restituisce il codice di uscita.
- Opzione `-InstallaAvvioAutomatico` (solo `.ps1`): crea con `Register-ScheduledTask` un'attività "Avventura - bot Telegram" che avvia il bot all'accesso dell'utente, con cartella di lavoro corretta; `-RimuoviAvvioAutomatico` la elimina. Chiede conferma prima di crearla.

### Collaudo end-to-end (tutto offline: profilo `prova` e server di prova locali)

Prepara una cartella di collaudo temporanea con `avventure/` contenente il Faro **e** una seconda avventura minima (copia di `motore/tests/dati/avventure/mini2`), così da avere un vero menu di scelta. Usa input da file (redirezione standard input). Verifica ogni punto e annota l'esito:

1. **Menu avventure**: all'avvio compaiono entrambe con titolo, descrizione breve e stato `[nuova]`. Scelta per numero; input non valido gestito; `q` esce.
2. **Nuova partita** nel Faro: introduzione, qualche azione (`guarda`, `est`, `prendi tanica`, `ovest`).
3. **`/save` → Nuovo salvataggio** con nome `inizio`; poi altre 2 azioni; **`/save` → Salva nel corrente** (deve sovrascrivere `inizio` senza chiedere); `/save` → Nuovo con nome `secondo`; `/save` → **Sovrascrivi esistente** scegliendo `inizio` dalla lista con conferma `s`.
4. Nome non valido (`CON`, `a/b`, stringa vuota, 41 caratteri): rifiutato con il motivo.
5. **`/load`** senza argomenti: lista con numero, nome, turno, stanza, punti, data e `*` sul corrente; carica per **numero**; poi `/load secondo` per **nome**; conferma sui turni non salvati.
6. `/saves`, `/delete secondo` (con conferma), `/undo`.
7. **`/menu`** → torna all'elenco: il Faro ora mostra `[partita in corso: turno N · M salvataggi]`. Entra nella **seconda avventura**, fai una nuova partita e un `/save` chiamato `inizio` (stesso nome, avventura diversa: deve essere permesso e indipendente).
8. `/quit` con progressi non salvati: compare il promemoria; rispondi `n`.
9. Riavvio: scegli il Faro → **Continua** → riprende dal turno corretto; riavvio → Faro → **Carica un salvataggio** → lista corretta.
10. Opzioni dirette: `gioca --avventura faro-punta-nera --carica inizio` e `gioca --avventura faro-punta-nera --nuova` (con conferma se ci sono progressi non salvati).
11. Sottocomandi `elenca` e `salvataggi --avventura faro-punta-nera`.
12. Controlla su disco: `partite/faro-punta-nera/` e `partite/<seconda>/` separate, ciascuna con `stato.json`, `storico/`, `salvataggi/`; i JSON sono validi; `salvataggio_corrente` è coerente.
13. `gioca.ps1 -Offline` (o `gioca.sh --offline`) funziona e mostra il menu.
14. **Server locale simulato**: avvia un piccolo server HTTP di prova su `127.0.0.1` che imita `/api/chat` di Ollama (risposta JSON valida e `prompt_eval_count` basso) e verifica: nessuna chiave richiesta, nessun header `Authorization`, `options.num_ctx` presente nella richiesta, avviso di troncamento mostrato. Poi fallo rispondere dopo 3 s con `--timeout 1` e verifica la domanda "Riprovare? (s/n)" e che con `n` lo stato non cambi.

15. **Fornitori HTTP**: per `openai`, `lmstudio`, `anthropic`, `gemini`, `ollama` avvia un server di prova su `127.0.0.1` che imita l'API del fornitore (usa `base_url` nel profilo; per i cloud imposta una chiave finta) e gioca 3 turni per ciascuno: richieste nel formato giusto (header, JSON mode/schema, ragionamento), risposte lette correttamente, stato aggiornato.
16. **Profili**: `avventura profili`; `--profilo` e `AVVENTURA_PROFILO` cambiano fornitore; chiave mancante per un profilo cloud → errore chiaro prima del menu.
17. **Fornitore `locale`**: con l'eseguibile standard → messaggio esplicativo. Se esiste la variante con modello locale **e** l'utente ha un file GGUF (chiedi), 3 turni reali; altrimenti salta e annotalo.
18. **Bot Telegram simulato**: server di prova che imita l'API Bot su `127.0.0.1` (usa l'opzione per i test del base_url Telegram); due utenti ammessi e uno no. Verifica: utente non ammesso riceve solo il suo ID; `/start` → pulsanti delle avventure → Nuova partita → 3 azioni; `/save` (nuovo) e `/load` tramite pulsanti; secondo utente gioca in parallelo con cartella separata in `partite/telegram/`; messaggio durante un turno; Ctrl+C chiude in modo ordinato e salva l'offset.
19. **Bot Telegram reale (facoltativo)**: solo se l'utente ha un token e acconsente: `gioca-telegram.ps1 -Profilo prova`, chiedi all'utente di scrivere al bot dal telefono e di provare `/start`, una partita, `/save`, `/load`; riporta l'esito che ti comunica.
20. **Lingue** (§7.6): `gioca.ps1 -Offline -Lingua en` mostra il menu in inglese (*The Punta Nera Lighthouse*, *Star Trucker…*) e la voce "🌐 Language"; una partita breve con comandi inglesi (`east`, `take the can`, `/help`, `/exits`); `/language it` durante la partita (messaggio "Il narratore risponderà in italiano dal prossimo turno."); `gioca.ps1 -Offline -Lingua it` mostra le stesse avventure con titolo, introduzione, nomi delle stanze e `/uscite` in italiano (da `traduzioni/it.toml`) e accetta `nord`/`prendi` anche se le uscite sono scritte in inglese; in una cartella di collaudo nuova, un avvio interattivo con `interfaccia.lingua = "auto"` chiede la lingua e salva `partite/preferenze.json`, il secondo avvio non la chiede più. `gioca.ps1 -Offline -Lingua fr`, `-Lingua es`, `-Lingua de`: menu, `/help` (con gli alias `/aide`, `/ayuda`, `/hilfe`), titolo, introduzione, nomi e `/uscite` nella lingua scelta, comandi come `est`, `prends le bidon`, `este`, `coge la lata`, `osten`, `nimm den kanister`, conferme con `o`/`s`/`j`. Nel bot simulato: un utente con `language_code = "en"` riceve la domanda della lingua con "English" per primo, uno con `de` con "Deutsch" per primo.

Se un punto fallisce per un bug del motore: correggilo, aggiungi un test che lo riproduce, riesegui `cargo fmt`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, ricompila in release e ripeti il collaudo.

### Chiusura
- Aggiorna `dist/` con eseguibile (se ricompilato) e lanciatori; rigenera lo zip.
- Salva l'esito del collaudo (tabella punto → esito) in `docs/COLLAUDO.md`; servirà allo script 09.

### Report finale
Tabella dei 20 punti con esito, bug trovati e corretti.
