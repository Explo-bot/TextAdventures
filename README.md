# Avventura: AI-narrated text adventures

**English** · [Italiano](README.it.md)

**Avventura** is a text adventure engine written in Rust. The game world (rooms, items, puzzles, secrets, win and lose conditions) is described in plain Markdown files. On every turn the player types an action in natural language and an artificial intelligence model acts as the **narrator**, following the adventure's rules. The engine keeps track of the game state (location, inventory, flags), validates the AI's replies and saves everything to disk.

The game is available in **English, Italian, French, Spanish and German**: on first launch it asks which language you want (suggesting your system's or Telegram app's language) and you can switch at any time with `/language`. Each adventure is written once, in English, and the AI narrator tells it in your language. The full manual and the generation scripts are in Italian.

You can play **in a terminal** or **on Telegram** (from your phone too), using either a **local model** on your own PC or a **cloud model**.

```
> open the trapdoor with the brass key
The key turns stiffly in the salt-crusted lock. With a creak the trapdoor
lifts: below, a wooden ladder leads down into the darkness of the cellar...
```

## Features

- **Adventure menu** at startup, then *Continue / Load / New game*.
- **Named saves** (`/save`, `/load`), undo of the last turn (`/undo`), inventory, help and other commands.
- **State on disk**: a game resumes exactly where you left it.
- **Validation of the AI's replies**: exits, items and flags are checked by the engine, so the narrator can't "cheat".
- **Five languages** (English, Italian, French, Spanish, German): menus, messages, `/help` and confirmations in every language, and every command has an alias in each language (`/save`, `/salva`, `/sauver`, `/guardar`, `/speichern`). Adventures are written in a single language and the narrator answers in the player's language; an optional `traduzioni/<language>.toml` file translates what the engine shows by itself (title, introduction, room and item names, exits). The included adventures are written in English, because AI models (small local ones especially) follow English instructions more reliably.
- **Telegram bot**: menus and confirmations become buttons; several players can use it at the same time, each with their own games, saves and language. It uses long polling, so you don't need to open any ports on your router.
- **New adventures** are easy to write: one `gioco.md` file plus one file per room (see the manual, Part IV).
- **Offline simulated AI** (`prova` profile) to try the program with no network and no API keys.

## Included adventures

| Adventure | Genre | Plot |
|---|---|---|
| **The Punta Nera Lighthouse** (*Il Faro di Punta Nera*) | seafaring tale | October 1893, a stormy night. You are Nicola, seventeen, the lighthouse keeper's assistant. The light has gone out and a brigantine is heading straight for the rocks: you have little time to relight it. |
| **Star Trucker and the Space Chicken of Doom** | B-movie sci-fi comedy | Year 2122, aboard the space freighter *Rusty Badger*. Some alien eggs have hatched and the "chicks" have grown. A lot. You are Molly, the ship's mechanic, maybe the only survivor (apart from Mr. Whiskers, the ship's cat). |

Both adventures are written in English (`avventure/faro-punta-nera/`, `avventure/space-chicken/`) and include Italian, French, Spanish and German translations of their title, introduction and names (`traduzioni/it.toml`, `fr.toml`, `es.toml`, `de.toml`), so they can be played in any of the five languages. Adding a folder is all it takes for a new adventure to appear in the menu.

## Supported AI models

The narrator can be a **cloud** model or a **local** model. You switch provider in the configuration file or with `--profilo`, without recompiling.

### Cloud

| Provider | Service | API key (environment variable) |
|---|---|---|
| `gemini` | **Google Gemini** (e.g. `gemini-3.8-flash`) | `GEMINI_API_KEY` |
| `anthropic` | Anthropic **Claude** (e.g. `claude-sonnet-5-5`) | `ANTHROPIC_API_KEY` |
| `openai` | **OpenAI** and services with the same API (e.g. **OpenRouter**) | `OPENAI_API_KEY` (or the one set in `api_key_env`) |

> ### ⭐ Gemini on the free tier
> The easiest way to start **without spending anything** is Google Gemini: all you need is a Google account to create a key on **[Google AI Studio](https://aistudio.google.com/apikey)**. The *Flash* models can be used on the **free tier**, which has per-minute and per-day request limits but is more than enough to play. A ready-made `gemini` profile is in [`config/config.esempio.toml`](config/config.esempio.toml).
>
> Good to know: on the free tier Google may use the text you send to improve its services (don't put personal data in your actions), and the limits and available models may change: check the current terms on AI Studio. If several people play through the Telegram bot, keep `max_turni_paralleli` low so you stay within the per-minute limits.

The other cloud services are pay-as-you-go: every turn sends the whole adventure plus the game state (a few thousand tokens), so setting a spending limit is a good idea.

### Local (free, private, no internet needed)

| Provider | How it works |
|---|---|
| `ollama` | An [Ollama](https://ollama.com) server on your PC. |
| `lmstudio` | The [LM Studio](https://lmstudio.ai) server (OpenAI-compatible API on `localhost:1234`). |
| `locale` | A **GGUF** file loaded directly by the program through llama.cpp (executable built with `--features locale`, optionally with **CUDA** GPU support). |

With a local model no text ever leaves your computer and there is no cost per turn; quality and speed depend on the model and the GPU. Models with a context of at least 16,000 tokens are recommended.

## Quick start (Windows)

1. Download the zip from `dist/`, unzip it, then run the installer:

   ```powershell
   .\installa.ps1
   ```

2. Set the key for your chosen provider (Gemini in this example), then reopen the terminal:

   ```powershell
   [Environment]::SetEnvironmentVariable("GEMINI_API_KEY", "your-key", "User")
   ```

3. Play:

   ```powershell
   Copy-Item config\config.esempio.toml config\config.toml   # once
   avventura gioca --profilo gemini
   .\gioca.ps1 -Offline -Lingua en   # try it in English with no AI and no network (also fr, es, de, it)
   ```

On Linux and macOS you build from source (`cargo build --release` in the `motore/` folder) and use `installa.sh` and `gioca.sh`.

### Telegram bot

Create a bot with **@BotFather**, put its token in the `TELEGRAM_BOT_TOKEN` environment variable, add your Telegram ID to `utenti_ammessi` (allowed users) and `amministratori` (administrators) in the `[telegram]` section of the configuration, then run:

```powershell
.\gioca-telegram.ps1
```

If you don't know your ID, just message the bot: it will reply with your numeric ID.

By default the bot generates one AI reply at a time (`max_turni_paralleli = 1`, right for local models), so other players see "the narrator is busy, you are in the queue…". With a cloud model raise it (for example to 4).

## Configuration files

| File | Use |
|---|---|
| [`config/config.esempio.toml`](config/config.esempio.toml) | Every option, with comments and sample profiles for each provider. |

API keys must **never be written in these files**: they are read from environment variables. Your personal configuration goes in `config/config.toml`, which git ignores.

## Documentation

- [`docs/MANUALE.md`](docs/MANUALE.md): the full manual, in Italian (installation, AI setup, game commands, languages, Telegram bot, writing new adventures, also in English).
- [`LEGGIMI.md`](LEGGIMI.md): how the project was built, in Italian. The engine is generated by a series of **Markdown scripts for Claude Code** (`01_SPECIFICHE.md` … `09_MANUALE.md`) which, run in order, produce the code, the adventures, the installers and the manual.

## Repository layout

```
avventure/         the adventures (gioco.md + stanze/*.md + traduzioni/*.toml)
config/            sample configuration files
motore/            Rust source code of the engine
docs/              manual and test report
dist/              ready-to-use Windows package
0X_*.md            generation scripts for Claude Code
```
