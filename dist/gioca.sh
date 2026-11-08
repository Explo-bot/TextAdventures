#!/usr/bin/env bash
# Avvia il gioco "Avventura".
# Uso: ./gioca.sh [--avventura ID] [--carica NOME] [--nuova] [--debug] [--profilo NOME] [--offline] [--config FILE] [--timeout SECONDI] [--lingua it|en|fr|es|de]
set -uo pipefail

RADICE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
AVVENTURA=""; CARICA=""; NUOVA=0; DEBUG=0; OFFLINE=0; CONFIG=""; TIMEOUT=""; PROFILO=""; LINGUA=""

if [ -t 1 ]; then G=$'\033[33m'; R=$'\033[31m'; N=$'\033[0m'; else G=; R=; N=; fi
avviso() { printf '%s%s%s\n' "$G" "$*" "$N"; }
errore() { printf '%s%s%s\n' "$R" "$*" "$N" >&2; }
chiedi() { local r; read -r -p "$1 (s/n) " r || r=n; case "$r" in s|S|si|SI|sì|y|Y|yes) return 0 ;; *) return 1 ;; esac; }

while [ $# -gt 0 ]; do
  case "$1" in
    --avventura|--carica|--config|--timeout|--profilo|--lingua)
      [ $# -ge 2 ] || { errore "$1 richiede un valore"; exit 2; }
      case "$1" in
        --avventura) AVVENTURA="$2" ;; --carica) CARICA="$2" ;;
        --config) CONFIG="$2" ;; --timeout) TIMEOUT="$2" ;; --profilo) PROFILO="$2" ;;
        --lingua) LINGUA="$2" ;;
      esac
      shift ;;
    --nuova) NUOVA=1 ;;
    --debug) DEBUG=1 ;;
    --offline) OFFLINE=1 ;;
    -h|--help) sed -n '2,3p' "${BASH_SOURCE[0]}" | sed 's/^# //'; exit 0 ;;
    *) errore "Opzione sconosciuta: $1"; exit 2 ;;
  esac
  shift
done

[ "$NUOVA" = 1 ] && [ -n "$CARICA" ] && { errore "--nuova e --carica non possono essere usate insieme."; exit 2; }
if [ -n "$TIMEOUT" ]; then
  case "$TIMEOUT" in ''|*[!0-9]*) errore "--timeout deve essere un numero di secondi (1–86400)."; exit 2 ;; esac
  { [ "$TIMEOUT" -ge 1 ] && [ "$TIMEOUT" -le 86400 ]; } || { errore "--timeout deve essere tra 1 e 86400 secondi."; exit 2; }
fi
if [ "$OFFLINE" = 1 ] && [ -n "$PROFILO" ] && [ "$PROFILO" != prova ]; then
  errore "--offline e --profilo non possono essere usati insieme."; exit 2
fi
if [ -n "$CONFIG" ]; then
  [ -f "$CONFIG" ] || { errore "File di configurazione non trovato: $CONFIG"; exit 1; }
  CONFIG="$(cd "$(dirname "$CONFIG")" && pwd)/$(basename "$CONFIG")"
fi

# --offline: AI simulata, senza dipendere da [profili.prova] nel config.
if [ "$OFFLINE" = 1 ] || [ "$PROFILO" = prova ]; then export AVVENTURA_FORNITORE=finto; PROFILO=""; fi

comuni() {  # argomenti condivisi da tutti i sottocomandi
  [ -n "$PROFILO" ] && printf '%s\n' --profilo "$PROFILO"
  [ -n "$CONFIG" ] && printf '%s\n' --config "$CONFIG"
}
ARGS_COMUNI=(); while IFS= read -r riga; do ARGS_COMUNI+=("$riga"); done < <(comuni)

# ------------------------------------------------------------ eseguibile
trova_exe() {  # trova_exe nome...
  local n p c
  for n in "$@"; do command -v "$n" 2>/dev/null && return 0; done
  for n in "$@"; do
    for c in "$RADICE" "$RADICE/dist" "$RADICE/motore/target/release"; do
      p="$c/$n"; [ -x "$p" ] && { echo "$p"; return 0; }
    done
  done
  return 1
}
EXE="$(trova_exe avventura || true)"
if [ -z "$EXE" ]; then
  avviso 'Eseguibile "avventura" non trovato (PATH, dist/, motore/target/release/).'
  [ -f "$RADICE/motore/Cargo.toml" ] || { errore "Sorgenti non presenti: esegui installa.sh o usa un pacchetto già compilato."; exit 1; }
  command -v cargo >/dev/null 2>&1 || { errore "Rust (cargo) non è installato: vedi https://rustup.rs"; exit 1; }
  chiedi 'Compilo ora con "cargo build --release"?' || exit 1
  (cd "$RADICE/motore" && cargo build --release) || { errore "Compilazione fallita."; exit 1; }
  EXE="$(trova_exe avventura || true)"
  [ -n "$EXE" ] || { errore "Compilazione riuscita ma eseguibile non trovato."; exit 1; }
fi

# Il profilo scelto usa il fornitore "locale" (modello GGUF)? Serve la variante con llama.cpp.
usa_locale() {
  if [ -n "${AVVENTURA_FORNITORE:-}" ]; then [ "$(printf '%s' "$AVVENTURA_FORNITORE" | tr 'A-Z' 'a-z')" = locale ]; return; fi
  (cd "$RADICE" && "$EXE" profili "${ARGS_COMUNI[@]}" 2>/dev/null) | grep -E '^[[:space:]]*\*[[:space:]].*fornitore locale( |$)' >/dev/null
}
if usa_locale; then
  VARIANTE="$(trova_exe avventura-cuda avventura-locale || true)"
  if [ -n "$VARIANTE" ]; then EXE="$VARIANTE"
  else avviso 'Il profilo usa il fornitore "locale" ma non trovo avventura-cuda / avventura-locale: l'"'"'eseguibile standard ti darà una spiegazione.'; fi
fi

# ------------------------------------------------------------ controlli preliminari (delegati al motore)
if ! TESTO="$(cd "$RADICE" && "$EXE" prova-ai --solo-config "${ARGS_COMUNI[@]}" 2>&1)"; then
  printf '%s\n' "$TESTO"
  [ "$OFFLINE" = 1 ] && exit 1
  if chiedi 'Avvio in modalità offline (AI simulata, profilo "prova")?'; then
    export AVVENTURA_FORNITORE=finto; PROFILO=""
    ARGS_COMUNI=(); [ -n "$CONFIG" ] && ARGS_COMUNI=(--config "$CONFIG")
  else errore "Sistema la configurazione e riprova (vedi docs/MANUALE.md)."; exit 1; fi
fi

# ------------------------------------------------------------ avvio
ARGS=(gioca)
[ -n "$AVVENTURA" ] && ARGS+=(--avventura "$AVVENTURA")
[ -n "$CARICA" ] && ARGS+=(--carica "$CARICA")
[ "$NUOVA" = 1 ] && ARGS+=(--nuova)
[ "$DEBUG" = 1 ] && ARGS+=(--debug)
[ -n "$TIMEOUT" ] && ARGS+=(--timeout "$TIMEOUT")
[ -n "$LINGUA" ] && ARGS+=(--lingua "$LINGUA")
ARGS+=("${ARGS_COMUNI[@]}")

cd "$RADICE" || exit 1
exec "$EXE" "${ARGS[@]}"
