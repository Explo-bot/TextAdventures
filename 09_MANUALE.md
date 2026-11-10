# 09 — Manuale: installazione, utilizzo, creazione di avventure

**Modello consigliato:** Claude Sonnet 5.5 (`claude-sonnet-5-5`).
**Durata indicativa:** 10–15 minuti.
**Prerequisiti:** script 02–08 completati. Il manuale descrive il programma **reale**: è l'ultimo script proprio per questo.

---

## Istruzioni per Claude Code

Scrivi `docs/MANUALE.md` in italiano: **un unico manuale** che spiega come **installare** il programma (inclusa la configurazione di ogni fornitore AI: OpenAI, Anthropic, Gemini, Ollama, LM Studio, modello GGUF dal disco), come **usarlo** da terminale, come **attivare e usare il bot Telegram**, come **scegliere la lingua** (italiano, inglese, francese, spagnolo, tedesco, §7.6) e come **creare nuove avventure** (anche in inglese o in più lingue).

Verifica anche che `README.md` (inglese) e `README.it.md` (italiano) nella radice siano coerenti con il programma reale: avventure incluse (scritte in inglese, giocabili in italiano, inglese, francese, spagnolo e tedesco), fornitori, Gemini con il piano gratuito, lingue, file di configurazione pubblicati in `config/` (senza chiavi né dati personali). Correggili se serve, mantenendo le due versioni allineate.

### Fonti (in ordine di autorità)
1. Il codice in `motore/src/` e l'help reale: esegui `avventura --help` e `avventura <sottocomando> --help` per ogni sottocomando, e `/help` dentro il gioco. Documenta solo comandi e opzioni che esistono davvero.
2. Gli script reali `installa.ps1`, `installa.sh`, `gioca.ps1`, `gioca.bat`, `gioca.sh`, `gioca-telegram.ps1`, `gioca-telegram.sh`, `config/config.esempio.toml` e il contenuto di `dist/`.
3. `01_SPECIFICHE.md`.
4. `avventure/faro-punta-nera/` (con `traduzioni/it.toml`, `fr.toml`, `es.toml`, `de.toml`) come esempio; `docs/COLLAUDO.md` per sapere cosa è stato verificato.

Se codice e specifica divergono, documenta il codice e segnala la differenza nel report finale (non modificare il codice in questo script).

### Esempi a schermo reali
Per le schermate (menu delle avventure, menu della partita, menu di `/save`, lista di `/load`, esempio di partita) esegui davvero il programma offline (`--profilo prova`) con input da file e incolla l'output reale, togliendo i codici colore. Per l'esempio di partita puoi rifinire il testo della narrazione perché somigli a quello di un'AI reale, dichiarandolo come illustrativo. Nessuno spoiler del Faro oltre la rimessa nelle Parti I–III. Per le schermate Telegram usa il server di prova dell'API Bot dei test (o, se l'utente acconsente, il suo bot reale) e riportale come testo/tabella dei pulsanti.

### Struttura richiesta di `docs/MANUALE.md`

**Indice** con link alle sezioni (ancore Markdown funzionanti).

**Introduzione** — cos'è il motore; i quattro elementi (avventure in sottocartelle, file di gioco, file di stanza, stato e salvataggi, AI); schema ASCII del turno: input → motore → prompt → AI → validazione → stato → output.

**Parte I – Installazione**
1. Requisiti (Windows 10/11; Linux/macOS).
2. Installazione da pacchetto pronto: scompattare lo zip, eseguire `installa.ps1` (con i parametri reali, incluso il caso dei criteri di esecuzione: `powershell -ExecutionPolicy Bypass -File .\installa.ps1`) oppure usare il programma direttamente dalla cartella scompattata; equivalente con `installa.sh`.
3. Installazione da sorgente: Rust (`winget install Rustlang.Rustup` o https://rustup.rs), Microsoft C++ Build Tools (`winget install Microsoft.VisualStudio.2022.BuildTools` + carico di lavoro "Sviluppo di applicazioni desktop con C++"), riaprire il terminale, `cargo build --release`, `installa.ps1 -DaSorgente`. Linux/macOS: rustup + strumenti di compilazione di sistema; nessuna dipendenza OpenSSL.
4. **Configurare l'AI: i concetti** — cos'è un *fornitore* (tabella dei sette fornitori del §5 con "cosa serve" e "costo"), cos'è un *profilo*, come si sceglie (`--profilo`, `AVVENTURA_PROFILO`, profilo predefinito `[ai]`), `avventura profili` per vedere quelli configurati. Tabella completa delle chiavi di `config.toml` (nome, tipo, default, descrizione) ricavata dal codice, incluse `[ai.locale]` e `[telegram]`; tutte le variabili d'ambiente. Come impostare una variabile d'ambiente utente in modo permanente (`[Environment]::SetEnvironmentVariable("NOME", "valore", "User")` e riaprire il terminale; bash: `export` in `~/.bashrc`) e perché è meglio che scrivere la chiave nel file.
5. **AI in cloud** — una sottosezione per ciascuno, con: dove creare l'account e la chiave (link alla console ufficiale), nome della variabile d'ambiente, profilo d'esempio, nota sui costi:
   - **OpenAI** (e servizi compatibili come OpenRouter: stesso fornitore `openai` con `base_url` diverso);
   - **Anthropic (Claude)**: chiave dalla Console Anthropic, `ANTHROPIC_API_KEY`, output JSON garantito dallo schema;
   - **Google Gemini**: chiave da Google AI Studio, `GEMINI_API_KEY`; evidenzia il **piano gratuito** (limiti di richieste al minuto e al giorno, testi che possono essere usati da Google per migliorare i servizi: verificare le condizioni attuali), il profilo `gemini` di `config.esempio.toml` (`gemini-3.8-flash`, `ragionamento = "basso"`) e il fatto che alcuni modelli Flash rifiutano il ragionamento minimo (`ragionamento = "basso"`);
   - il parametro `ragionamento` (§5.4): quando conviene `spento` (turni più rapidi ed economici) e quando un livello più alto.
6. **Modelli in locale con un server (Ollama, LM Studio)** — sezione dedicata e completa:
   - perché farlo (nessun costo per turno, privacy, funziona senza internet) e cosa serve (memoria RAM/VRAM indicativa in base alla dimensione del modello; GPU consigliata);
   - **nessuna chiave API** necessaria; `chiave_richiesta = false` per un server su un altro PC della rete;
   - **Ollama**: installazione (`winget install Ollama.Ollama` o dal sito ollama.com), `ollama pull <modello>`, `ollama list`, profilo con `fornitore = "ollama"` (il motore imposta da solo il contesto con `num_ctx` e disattiva il ragionamento con `ragionamento = "spento"`); verificare con `ollama ps` che il modello sia tutto in GPU;
   - **LM Studio**: scaricare un modello, avviare il server locale (scheda sviluppatore o `lms server start`), impostare la lunghezza del contesto al caricamento del modello (interfaccia o `lms load <modello> --context-length 16384`) e lo stesso valore in `contesto_token`; profilo con `fornitore = "lmstudio"`;
   - **il contesto, spiegato semplice**: perché conta, cosa succede se è troppo piccolo (troncamento silenzioso → l'AI "dimentica" regole e segreti), come leggere gli avvisi del motore, come ottenere il valore consigliato con `avventura prova-ai --avventura <id>` o `avventura valida`, e che un contesto più grande richiede più memoria;
   - **timeout**: default 900 s, come cambiarlo (`timeout_secondi`, `AVVENTURA_TIMEOUT`, `--timeout`), cosa succede alla scadenza (domanda "Riprovare?", stato invariato), differenza con il timeout di connessione;
   - scelta del modello locale: serve un modello che segua bene le istruzioni e produca JSON affidabile; i modelli molto piccoli tendono a violare lo schema o a ignorare le meccaniche; come valutarli (prima `prova-ai`, poi qualche turno con `--debug`); indicazioni per fasce di memoria della GPU (8, 12, 16, 24 GB) espresse come dimensione e quantizzazione del modello, citando eventuali nomi di modelli solo come esempi da verificare;
   - tabella dei problemi tipici in locale: server non avviato, modello non scaricato, risposte lentissime, avviso di troncamento, JSON non valido ripetuto, memoria esaurita.
7. **Modello GGUF caricato direttamente dal disco** (fornitore `locale`):
   - quando conviene rispetto a Ollama/LM Studio (nessun programma in più da avviare; un solo eseguibile) e quando no (va usata la variante dell'eseguibile con supporto, file più grande);
   - quale eseguibile usare: `avventura-cuda.exe` per GPU NVIDIA (o la variante solo CPU), oppure come compilarlo (`cargo build --release --features cuda`, prerequisiti CMake, LLVM/Clang, CUDA Toolkit);
   - dove trovare i file GGUF (es. Hugging Face), come scegliere la quantizzazione (Q4_K_M come compromesso; file ≈ memoria necessaria + spazio per il contesto), dove metterli (`modelli\`);
   - profilo d'esempio (`fornitore = "locale"`, `[ai.locale] percorso_modello`, `gpu_layers = -1`, `contesto_token`, `ragionamento = "spento"`), `AVVENTURA_MODELLO_GGUF`;
   - `prova-ai --profilo gguf`: come leggere tempo di caricamento e token/s;
   - problemi: memoria GPU esaurita (ridurre `gpu_layers` o usare una quantizzazione più piccola), eseguibile senza supporto, file corrotto, template di chat non riconosciuto (`template_chat`).
8. Scelta del modello AI in generale: temperatura 0.6–0.8; affidabilità del JSON più importante della creatività; tabella riassuntiva "quale fornitore scegliere" (qualità, costo, velocità, privacy, difficoltà di installazione).
9. Costi e privacy: ogni turno invia gioco + stanza + stato; stima dei token (~4 caratteri per token); tutto il testo viene inviato al fornitore configurato.
10. Verifica: `avventura --version`, `avventura elenca`, `avventura valida`, `avventura profili`, `avventura prova-ai` (anche `--profilo`, `--avventura <id>`, `--solo-config`), partita offline con `--profilo prova`.
11. Aggiornare e disinstallare (`installa.ps1 -Disinstalla`); i salvataggi in `partite/` non vengono toccati dagli aggiornamenti.
12. Problemi di installazione: `link.exe` non trovato, `cargo`/`avventura` non riconosciuto (PATH, terminale non riaperto), SmartScreen/antivirus su un eseguibile non firmato, criteri di esecuzione PowerShell, proxy aziendali (`HTTPS_PROXY`).

**Parte II – Utilizzo**
1. Avvio: `avventura` (o doppio clic su `gioca.bat`, o `.\gioca.ps1`); l'avviso iniziale che il programma è open source, con l'indirizzo GitHub, e che il racconto di Vinny Falco si legge gratis su https://gronk.it (anche a ogni `/start` del bot); opzioni utili (`--avventura`, `--carica`, `--nuova`, `--profilo`, `--debug`, `--trascrizione`, `--lingua`, `-Offline`, `-Lingua`).
1b. **Lingua (italiano, inglese, francese, spagnolo, tedesco)**: domanda al primo avvio (con la lingua di Windows come prima voce), `preferenze.json`, `/language`, voce "🌐 Lingua" nel menu, `--lingua`, `AVVENTURA_LINGUA`, `interfaccia.lingua`; il narratore AI scrive nella lingua scelta anche se l'avventura è scritta in un'altra (le avventure di esempio sono in inglese), con titolo, introduzione, nomi di stanze e oggetti e uscite tradotti da `traduzioni/<lingua>.toml`; perché le avventure sono scritte in inglese (i modelli seguono meglio il prompt, soprattutto quelli piccoli) e i limiti della traduzione al volo (tono, giochi di parole; l'AI simulata riporta le descrizioni in inglese); tabella degli alias dei comandi e delle risposte sì/no per lingua; i messaggi tecnici (sottocomandi, errori di configurazione, lanciatori) restano in italiano.
2. **Scelta dell'avventura**: schermata reale del menu, significato di `[nuova]` / `[partita in corso…]` / "non valida", titolo e descrizione nella lingua dell'interfaccia (se l'avventura ha la traduzione), ultima voce della lingua.
3. **Scelta della partita**: Continua / Carica un salvataggio / Nuova partita / Indietro, con schermata reale.
4. Come scrivere le azioni: esempi buoni e meno buoni; un'azione per volta; il narratore capisce frasi naturali. **Mentre il narratore risponde**: l'indicatore "Il narratore riflette…", poi il testo che compare man mano con il cursore animato in fondo (testo progressivo); perché a volte il testo viene cancellato e riscritto (risposta corretta dal motore, stanza nuova); come disattivarlo (`interfaccia.testo_progressivo = false`).
5. Tabella di tutti i comandi `/…` con gli alias in italiano, francese, spagnolo e tedesco ed esempi.
6. **Salvare e caricare** (sezione dedicata, con schermate reali):
   - differenza tra salvataggio automatico (`stato.json`, partita in corso) e salvataggi con nome;
   - `/save`: menu con salvataggio corrente / sovrascrivi esistente / nuovo; `/save NOME`; cos'è il "salvataggio corrente" (`*`);
   - `/load`: lista dei salvataggi e scelta per numero o nome; `/load NOME`; avviso sui turni non salvati;
   - `/saves`, `/delete`, `/undo`;
   - regole sui nomi;
   - ogni avventura ha i propri salvataggi (stesso nome in avventure diverse = salvataggi diversi);
   - dove stanno i file e come farne un backup o copiarli su un altro PC.
7. Cambiare avventura durante il gioco (`/menu`) e uscire (`/quit` e promemoria sui progressi non salvati).
8. Fine partita (vittoria/sconfitta) e cosa si può fare dopo.
9. Se il narratore "sbaglia": riformulare, `/undo`, `/load`, `--debug` con `/state` e `/raw`, log in `partite/avventura.log` (a rotazione, 5 MB; testi completi solo con `--debug`).
10. Esempio di sessione (prime 4–5 mosse del Faro, con un `/save`).
11. Uso non interattivo per autori e collaudo (comandi da file via standard input).
12. Problemi frequenti in gioco: 401 chiave errata, 404 modello/URL, 429 limiti, timeout (con rimando alla sezione sui modelli locali), avvisi di contesto troncato, `json_mode` non supportato, risposte non valide ripetute, accenti nel terminale Windows (Windows Terminal, `chcp 65001`), colori (`NO_COLOR`, `interfaccia.colori = false`).

**Parte III – Giocare tramite Telegram**
Sezione completa e pratica, passo per passo, pensata anche per chi non ha mai creato un bot:
1. Come funziona: il programma gira sul tuo PC (che deve restare acceso e connesso), i giocatori usano l'app Telegram anche da telefono; non serve aprire porte del router né avere un indirizzo pubblico.
2. **Creare il bot con @BotFather**: aprire la chat con @BotFather (verificare la spunta ufficiale), `/newbot`, scegliere nome visualizzato e username (deve finire con `bot`), copiare il **token**; consigli facoltativi (`/setdescription`, `/setuserpic`). Avvertenza: il token è una password, chi lo possiede controlla il bot; se trapela, `/revoke` in BotFather.
3. **Impostare il token** come variabile d'ambiente utente `TELEGRAM_BOT_TOKEN` (comando PowerShell e bash) e riaprire il terminale.
4. **Scegliere chi può giocare**: avviare una volta il bot, scrivergli dal proprio account: risponde con il tuo ID numerico; inserirlo in `utenti_ammessi` (e in `amministratori`); come aggiungere amici (chiedere loro di scrivere al bot e comunicarti l'ID) oppure usare `codice_invito` (`/start <codice>`; non riammette chi è stato revocato; blocco di 15 minuti dopo 5 codici sbagliati); comandi `/utenti`, `/ammetti`, `/revoca`.
5. **Scegliere l'AI del bot**: `telegram.profilo` o `--profilo`; con modelli locali lasciare `max_turni_paralleli = 1`, con un servizio in cloud alzarlo (es. 4) tenendo conto dei limiti al minuto del piano; spiegare che il messaggio "Il narratore è occupato con un altro giocatore, sei in coda…" dipende da questo limite; tempi di risposta, messaggio «✍️ Il narratore sta scrivendo…» (che sparisce quando arriva la risposta) e indicatore "sta scrivendo…" in alto nella chat.
5b. **Lingua nel bot**: al primo messaggio il bot chiede la lingua proponendo quella dell'app Telegram; ogni utente ha la sua; `/language`; comandi del menu "/" nella lingua dell'app per chi la usa in italiano, francese, spagnolo o tedesco, in inglese per gli altri; alias inglesi dei comandi amministratore (`/users`, `/allow`, `/revoke`).
6. **Avviare il bot**: `.\gioca-telegram.ps1` (o `avventura telegram`), `--verifica`, cosa compare sul terminale, come fermarlo (Ctrl+C).
7. **Tenerlo sempre attivo**: `gioca-telegram.ps1 -InstallaAvvioAutomatico` (attività pianificata all'accesso) e rimozione; in alternativa Utilità di pianificazione a mano; Linux con `systemd --user` (file d'esempio). Impostazioni di risparmio energetico di Windows che metterebbero il PC in sospensione.
8. **Giocare dalla chat**: schermate (descritte o da esecuzione simulata) di `/start`, pulsanti delle avventure e della partita, un'azione, `/save` e `/load` con i pulsanti; elenco dei comandi nel menu "/" di Telegram; cosa succede scrivendo mentre il narratore sta rispondendo; domande che scadono dopo `inattivita_minuti`.
9. **Dati e privacy**: dove sono le partite di ogni utente (`partite/telegram/<id>/…`), cosa vede Telegram (i messaggi della chat), cosa vede il fornitore AI.
10. **Problemi**: il bot non risponde (programma non avviato, token errato, PC in sospensione, un webhook impostato altrove → il bot lo rimuove all'avvio), "Questo bot è privato" (ID non ammesso), risposte lente, errore 409 "Conflict" (due copie del bot avviate contemporaneamente).

**Parte IV – Creare nuove avventure**
1. Dove metterle: una **sottocartella** in `avventure/` (accanto all'eseguibile o nella cartella configurata); il nome della cartella è l'id (minuscole, cifre, trattini); compare automaticamente nel menu; ha automaticamente stato e salvataggi propri.
2. Procedura passo-passo per una nuova avventura, con un **mini esempio completo** di 2 stanze scritto apposta (non il Faro): creare la cartella, scrivere `gioco.md`, scrivere le stanze, `avventura valida --avventura <id>`, provarla offline, provarla con l'AI. Il mini esempio deve essere valido: crealo davvero in una cartella temporanea e validalo prima di incollarlo nel manuale.
3. Riferimento `gioco.md`: ogni campo del front matter TOML (inclusi `titolo`, `descrizione_breve`, `lingua`, `storia`, `stanza_iniziale`, `max_turni`, `inventario_iniziale`, `spostamenti_liberi`, `[flag_iniziali]`, `[oggetti]`) in tabella; ogni sezione e cosa è pubblico (solo Introduzione) o riservato; titoli delle sezioni in italiano o in inglese (tabella delle equivalenze).
3b. **Avventure in inglese e in più lingue**: `lingua = "en"` (prompt dell'AI in inglese, consigliato), chiavi del front matter sempre italiane, id/direzioni liberi nella lingua dell'avventura; il narratore scrive comunque nella lingua del giocatore; **file `traduzioni/<lingua>.toml`** (§3.4; `it`, `en`, `fr`, `es`, `de`) con esempio completo, cosa traduce (titolo, descrizione breve, introduzione, `[stanze]`, `[oggetti]`, `[uscite]` con chiavi `"id_stanza.direzione"`), cosa no (descrizioni e sezioni riservate: le racconta l'AI; direzioni standard: le traduce il motore, tabella delle direzioni nelle cinque lingue), avvisi di `valida`; in alternativa, versioni complete distinte della stessa storia con lo stesso campo `storia`.
4. Riferimento file stanza: front matter (`id`, `nome`, `oggetti`, `oggetti_nascosti`, `[uscite.<direzione>]` con `verso`, `descrizione`, `richiede_flag`, `richiede_oggetto`, `nascosta`) e sezioni Descrizione / Dettagli / Segreti / Meccaniche, con cosa vede il giocatore e cosa no.
5. Flag e oggetti: nomi in snake_case senza accenti; regola di "verità" dei flag; catalogo `[oggetti]`; perché dichiarare ogni oggetto.
6. **Scrivere buone Meccaniche per un'AI**: regole "Se il giocatore … allora imposta …"; nomi esatti di flag/oggetti; prevedere i tentativi sbagliati più ovvi; usare i vincoli deterministici (`richiede_*`) per ciò che non deve essere aggirabile; condizioni di vittoria espresse con flag; eventi a turni nelle Regole del mondo; non affidare all'AI conteggi complessi.
7. Esempio commentato: `scala.md` del Faro annotato riga per riga (con avviso "Attenzione: spoiler").
8. Collaudo di un'avventura: `valida`, partita offline, partita con l'AI usando `--debug` e `--trascrizione`, rigiocare una sequenza da file dopo ogni modifica, usare `/save` prima dei punti critici per riprovarli.
9. Distribuire un'avventura: copiare la sottocartella; compatibilità con i salvataggi esistenti se si modificano id di stanze/oggetti (cosa si rompe e come evitarlo).
10. Checklist finale per l'autore.

**Parte V – Riferimento tecnico**
1. Il file di stato: ogni campo con un esempio reale (generato da una partita offline), inclusi `salvataggio_corrente` e `turno_ultimo_salvataggio`; perché è l'unica memoria dell'AI (`memoria`, `ultimi_turni`); modificarlo a mano per test.
2. Il file di salvataggio con nome (formato).
3. Struttura delle cartelle `avventure/` e `partite/`.
4. Tutti i sottocomandi CLI con opzioni (dall'help reale, inclusi `telegram`, `profili`, `prova-ai --solo-config`), codici di uscita, posizione dei log, varianti dell'eseguibile (standard, cuda).
5. Il protocollo con l'AI in breve (cosa viene inviato, schema della risposta, come ogni fornitore lo traduce, streaming per il testo progressivo, come viene stimato il numero di token), per chi vuole capire o fare debug.

**Glossario** breve.

### Requisiti di forma
- Esempi per **PowerShell** (principale) e bash.
- Tono pratico e chiaro; tabelle per i riferimenti.
- Lunghezza indicativa 1300–2200 righe.

### Chiusura
- Copia `docs/MANUALE.md` in `dist/` e rigenera lo zip.
- Aggiungi in fondo a `LEGGIMI.md` (senza modificare il resto) una sezione "Stato del progetto" con data, versione, esito dei test e del collaudo, percorso dello zip.

### Controlli
- Ogni comando/opzione citato esiste nell'help reale (dichiaralo nel report).
- Le schermate dei menu, di `/save` e `/load` provengono da esecuzioni reali.
- Il mini esempio della Parte IV supera `avventura valida`.
- Le istruzioni per BotFather e per i siti dei fornitori (console delle chiavi) sono verificate sulle pagine ufficiali attuali; link inclusi.
- Tutti i link dell'indice funzionano.

### Report finale
Sezioni scritte, numero di righe, differenze codice/specifica trovate.
