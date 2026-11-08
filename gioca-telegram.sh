#!/usr/bin/env bash
# Avvia il bot Telegram di "Avventura" in primo piano.
# Uso: ./gioca-telegram.sh [--profilo NOME] [--config FILE] [--verifica]
set -uo pipefail

RADICE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROFILO=""; CONFIG=""; VERIFICA=0

if [ -t 1 ]; then G=$'\033[33m'; R=$'\033[31m'; N=$'\033[0m'; else G=; R=; N=; fi
avviso() { printf '%s%s%s\n' "$G" "$*" "$N"; }
errore() { printf '%s%s%s\n' "$R" "$*" "$N" >&2; }

while [ $# -gt 0 ]; do
  case "$1" in
    --profilo|--config)
      [ $# -ge 2 ] || { errore "$1 richiede un valore"; exit 2; }
      if [ "$1" = --profilo ]; then PROFILO="$2"; else CONFIG="$2"; fi
      shift ;;
    --verifica) VERIFICA=1 ;;
    -h|--help) sed -n '2,3p' "${BASH_SOURCE[0]}" | sed 's/^# //'; exit 0 ;;
    *) errore "Opzione sconosciuta: $1"; exit 2 ;;
  esac
  shift
done

if [ -n "$CONFIG" ]; then
  [ -f "$CONFIG" ] || { errore "File di configurazione non trovato: $CONFIG"; exit 1; }
  CONFIG="$(cd "$(dirname "$CONFIG")" && pwd)/$(basename "$CONFIG")"
fi
if [ "$PROFILO" = prova ]; then export AVVENTURA_FORNITORE=finto; PROFILO=""; fi
ARGS_COMUNI=()
[ -n "$PROFILO" ] && ARGS_COMUNI+=(--profilo "$PROFILO")
[ -n "$CONFIG" ] && ARGS_COMUNI+=(--config "$CONFIG")

# ------------------------------------------------------------ eseguibile
trova_exe() {
  local p c
  command -v avventura 2>/dev/null && return 0
  for c in "$RADICE" "$RADICE/dist" "$RADICE/motore/target/release"; do
    p="$c/avventura"; [ -x "$p" ] && { echo "$p"; return 0; }
  done
  return 1
}
EXE="$(trova_exe || true)"
[ -n "$EXE" ] || { errore 'Eseguibile "avventura" non trovato: compila con "cargo build --release" in motore/ o esegui installa.sh.'; exit 1; }

# ------------------------------------------------------------ configurazione [telegram]
CFG_FILE="$CONFIG"
if [ -z "$CFG_FILE" ]; then
  for p in "$RADICE/config.toml" "$RADICE/config/config.toml"; do [ -f "$p" ] && { CFG_FILE="$p"; break; }; done
fi
AMMESSI=""; INVITO=""; NOME_VAR=""
if [ -n "$CFG_FILE" ]; then
  LETTURA="$(awk '
    /^[[:space:]]*\[/ { s=$0; gsub(/[[:space:]\[\]]/,"",s); sez=s; next }
    sez=="telegram" && /^[[:space:]]*utenti_ammessi[[:space:]]*=/ { v=$0; sub(/^[^=]*=/,"",v); sub(/#.*/,"",v); gsub(/[][[:space:]]/,"",v); print "A=" v }
    sez=="telegram" && /^[[:space:]]*token_env[[:space:]]*=/ { v=$0; sub(/^[^=]*=/,"",v); sub(/#.*/,"",v); gsub(/[[:space:]"]/,"",v); print "T=" v }
    sez=="telegram" && /^[[:space:]]*codice_invito[[:space:]]*=/ { v=$0; sub(/^[^=]*=/,"",v); gsub(/[[:space:]"]/,"",v); sub(/#.*/,"",v); print "I=" v }
  ' "$CFG_FILE" | tr -d '\r')"
  AMMESSI="$(printf '%s\n' "$LETTURA" | sed -n 's/^A=//p' | head -1)"
  NOME_VAR="$(printf '%s\n' "$LETTURA" | sed -n 's/^T=//p' | head -1)"
  INVITO="$(printf '%s\n' "$LETTURA" | sed -n 's/^I=//p' | head -1)"
fi
# ------------------------------------------------------------ token
NOME_VAR="${NOME_VAR:-TELEGRAM_BOT_TOKEN}"
if [ -z "${!NOME_VAR:-}" ]; then
  avviso "La variabile $NOME_VAR (token del bot) non è impostata."
  cat <<'EOT'
Come ottenere il token:
  1. In Telegram scrivi a @BotFather e invia /newbot.
  2. Scegli un nome e un username (deve finire con "bot"): BotFather ti risponde con il token.
  3. Imposta la variabile d'ambiente, per esempio nel tuo ~/.bashrc:
       export TELEGRAM_BOT_TOKEN="123456:ABC..."
     poi apri un nuovo terminale. Non scrivere il token in config.toml né in repository.
Per i dettagli vedi docs/MANUALE.md (capitolo sul bot Telegram).
EOT
  exit 1
fi

# ------------------------------------------------------------ utenti ammessi
UTENTI_FILE="$RADICE/partite/telegram/utenti.json"
if [ -z "$AMMESSI" ] && [ -z "$INVITO" ] && ! grep -q '[0-9]' "$UTENTI_FILE" 2>/dev/null; then
  avviso "Attenzione: utenti_ammessi è vuoto e non c'è un codice_invito: nessuno potrà giocare."
  echo "Per scoprire il tuo ID Telegram scrivi un messaggio al bot: risponde con il tuo ID."
  echo "Poi inseriscilo in config.toml, sezione [telegram], per esempio: utenti_ammessi = [123456789]"
fi

# ------------------------------------------------------------ avvio
cd "$RADICE" || exit 1
"$EXE" telegram --verifica "${ARGS_COMUNI[@]}" || { errore "Verifica non superata: sistema i problemi indicati sopra."; exit 1; }
[ "$VERIFICA" = 1 ] && exit 0
exec "$EXE" telegram "${ARGS_COMUNI[@]}"
