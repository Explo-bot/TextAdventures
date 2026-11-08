# 04 — Fornitore "locale": modello GGUF caricato dal disco

**Modello consigliato:** Claude Opus 5.5 (`claude-opus-5-5`): integrazione con una libreria nativa (llama.cpp) e problemi di compilazione su Windows.
**Durata indicativa:** 30–60 minuti (la prima compilazione di llama.cpp, soprattutto con CUDA, è lunga).
**Prerequisiti:** script 02 completato (lo script 03 può essere fatto prima o dopo).

---

## Istruzioni per Claude Code

Implementa il fornitore `locale` descritto nel §5.5 di `01_SPECIFICHE.md`: un modello in formato **GGUF** letto direttamente dal disco ed eseguito dentro il programma tramite **llama.cpp**, senza Ollama, LM Studio o altri server.

### Passo 0 — Prerequisiti di compilazione (verifica, non installare senza chiedere)
Controlla e riporta in una tabella:
- `cmake --version` (necessario);
- Clang/LLVM (`clang --version`, o variabile `LIBCLANG_PATH`): necessario per generare i binding;
- Visual Studio Build Tools con C++ (già richiesti per Rust su Windows);
- GPU NVIDIA: `nvidia-smi` (driver e memoria) e `nvcc --version` (CUDA Toolkit) per la feature `cuda`;
- in alternativa a CUDA, presenza dell'SDK Vulkan (`VULKAN_SDK`) per un'eventuale feature `vulkan`.

Se manca qualcosa di necessario, **fermati** e indica all'utente i comandi per installarlo, chiedendo conferma prima di eseguirli. Comandi indicativi su Windows (verifica che gli ID winget esistano con `winget search` prima di proporli): `winget install Kitware.CMake`, `winget install LLVM.LLVM`, `winget install Nvidia.CUDA`. Dopo l'installazione serve un nuovo terminale.

### Passo 1 — Dipendenze e feature
1. Verifica su docs.rs/crates.io l'ultima versione di `llama-cpp-2` e la sua API attuale (caricamento del backend e del modello, contesto, tokenizzazione, template di chat, campionatori incluso quello a grammatica, generazione token per token). Non basarti sulla memoria: l'API cambia tra le versioni.
2. In `Cargo.toml`:
   ```toml
   [features]
   default = []
   locale = ["dep:llama-cpp-2"]
   cuda = ["locale", "llama-cpp-2/cuda"]
   # vulkan = ["locale", "llama-cpp-2/vulkan"]   # solo se la feature esiste nella versione del crate
   ```
   con `llama-cpp-2` come dipendenza `optional = true` (aggiunta con `cargo add llama-cpp-2 --optional`).
3. La build predefinita (`cargo build`) deve continuare a funzionare **senza** CMake/Clang/CUDA.

### Passo 2 — Implementazione (`ai/locale.rs` per le parti senza dipendenze, sempre compilate e testate; `ai/locale/llama.rs` per llama.cpp, compilato con `#[cfg(feature = "locale")]`)
- Caricamento **una sola volta** di backend e modello all'avvio (o al primo uso), con `gpu_layers` e `thread` dalla configurazione; messaggio d'attesa sul canale; errori chiari: file inesistente, file non GGUF, memoria insufficiente (suggerire meno `gpu_layers` o una quantizzazione più piccola).
- Per ogni richiesta: nuovo contesto con `contesto_token` (default 8.192 con avviso se 0), prompt costruito con il **template di chat del file GGUF** (o `template_chat`), conteggio **esatto** dei token con il tokenizzatore del modello; se il prompt non entra nel contesto applica la riduzione del §5.2 punto 3.
- Generazione vincolata da una **grammatica GBNF** che produce solo JSON valido (§5.5); se il crate offre la conversione da JSON Schema a grammatica, usa lo schema di `ai/schema.rs`; altrimenti incorpora una grammatica JSON generica.
- `ragionamento = "spento"`: disattiva l'eventuale ragionamento del template se previsto; in ogni caso scarta i blocchi di ragionamento (§5.4).
- Rispetto di `max_token` e di `timeout_secondi` (interrompe la generazione), con lo stesso comportamento dopo un timeout degli altri fornitori (§5.3).
- Testo progressivo (§6.1 "Streaming"): implementa anche `turno_in_flusso`, passando i pezzi di testo man mano che i token vengono generati; un token può contenere solo una parte di un carattere UTF-8, quindi si inviano solo i byte che formano caratteri completi (quelli non validi si saltano).
- `RispostaGrezza` con token di prompt esatti, token generati e durata; `prova-ai` per `locale` mostra i dati del §5.5 (file, dimensione, strati su GPU, contesto, tempo di caricamento, token/s).
- Thread-safety: il modello è condiviso (`Arc`); le generazioni sono serializzate da un `Mutex` (coerente con `max_turni_paralleli = 1` del bot Telegram).
- Senza la feature: lo stub dello script 02 resta e spiega come ottenere un eseguibile con il supporto (`cargo build --release --features locale` o `--features cuda`, oppure `avventura-cuda.exe` dal pacchetto).

### Passo 3 — Test
- Senza feature: test dello stub e della configurazione (percorso relativo al file di config, variabile `AVVENTURA_MODELLO_GGUF`, `gpu_layers`, errori).
- Con feature: test unitari della pulizia del ragionamento e della riduzione del prompt (funzioni pure, senza modello); un test di integrazione **`#[ignore]`** che usa un piccolo GGUF indicato da `AVVENTURA_TEST_GGUF` e verifica che la risposta sia JSON valido conforme allo schema.

### Passo 4 — Compilazione e prova
1. `cargo build --release` (senza feature) → deve passare.
2. `cargo build --release --features locale` → deve passare se i prerequisiti ci sono.
3. Se c'è CUDA: `cargo build --release --features cuda` (può richiedere molti minuti).
4. Se l'utente ha già un file GGUF (chiedi dove), prova `prova-ai --profilo gguf` e qualche turno del Faro (se lo script 06 è già stato eseguito) o della mini-avventura di test; riporta tempo di caricamento, token/s e validità del JSON. **Non scaricare modelli** senza il consenso esplicito dell'utente (pesano diversi GB); se acconsente, proponi un modello adatto alla sua GPU e una quantizzazione Q4_K_M da Hugging Face, verificando il link.
5. `cargo fmt --all`, `cargo clippy --all-targets -- -D warnings` (anche con `--features locale` se compilabile), `cargo test`.

### Criteri di accettazione
- [ ] La build predefinita non richiede CMake/Clang/CUDA.
- [ ] Con la feature, `fornitore = "locale"` gioca una partita producendo sempre JSON valido.
- [ ] Il modello si carica una volta sola e viene riusato.
- [ ] Messaggi d'errore chiari per file mancante, memoria insufficiente, eseguibile senza supporto.

### Report finale
Tabella dei prerequisiti trovati, feature compilate con successo (default / locale / cuda), risultati della prova con un modello reale se fatta, limiti noti.
