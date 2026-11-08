#!/usr/bin/env bash
# Installa (o disinstalla) il motore di avventure testuali "Avventura".
# Uso: ./installa.sh [--da-sorgente] [--con-modello-locale] [--destinazione DIR] [--disinstalla]
# Non tocca mai: modelli/ esistente, partite/ (stati e salvataggi), un config.toml esistente
# e le avventure aggiunte dall'utente.
set -euo pipefail

RADICE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DEST="${HOME}/.local/share/avventura"
BIN_DIR="${HOME}/.local/bin"
DA_SORGENTE=0
DISINSTALLA=0
CON_LOCALE=0

if [ -t 1 ]; then C=$'\033[36m'; V=$'\033[32m'; G=$'\033[33m'; R=$'\033[31m'; N=$'\033[0m'; else C=; V=; G=; R=; N=; fi
info()   { printf '%s%s%s\n' "$C" "$*" "$N"; }
ok()     { printf '%s%s%s\n' "$V" "$*" "$N"; }
avviso() { printf '%s%s%s\n' "$G" "$*" "$N"; }
errore() { printf '%s%s%s\n' "$R" "$*" "$N" >&2; }

USO="Uso: $0 [--da-sorgente] [--con-modello-locale] [--destinazione DIR] [--disinstalla]"
while [ $# -gt 0 ]; do
  case "$1" in
    --da-sorgente) DA_SORGENTE=1 ;;
    --con-modello-locale) CON_LOCALE=1 ;;
    --disinstalla) DISINSTALLA=1 ;;
    --destinazione)
      [ $# -ge 2 ] || { errore "--destinazione richiede una cartella"; exit 1; }
      DEST="$2"; shift ;;
    -h|--help) echo "$USO"; exit 0 ;;
    *) errore "Opzione sconosciuta: $1"; echo "$USO" >&2; exit 1 ;;
  esac
  shift
done

mkdir -p "$DEST"
DEST="$(cd "$DEST" && pwd)"
LINK="${BIN_DIR}/avventura"

chiedi() { local r; read -r -p "$1 (s/n) " r || r=n; case "$r" in s|S|si|SI|sì|y|Y|yes) return 0 ;; *) return 1 ;; esac; }

# ------------------------------------------------------------ disinstallazione
if [ "$DISINSTALLA" = 1 ]; then
  info "Disinstallazione da: $DEST"
  rm -f "$DEST/avventura" "$DEST/avventura-locale" "$DEST/config.esempio.toml"
  if [ -L "$LINK" ] && [ "$(readlink "$LINK")" = "$DEST/avventura" ]; then rm -f "$LINK"; ok "Rimosso link $LINK"; fi
  if [ -d "$DEST/partite" ]; then
    if chiedi "Cancellare anche partite/ (stati e SALVATAGGI)? Irreversibile"; then rm -rf "$DEST/partite"; ok "partite/ rimossa."; else info "partite/ conservata."; fi
  fi
  if [ -d "$DEST/avventure" ]; then
    if chiedi "Cancellare anche avventure/ (comprese le tue)? Irreversibile"; then rm -rf "$DEST/avventure"; ok "avventure/ rimossa."; else info "avventure/ conservata."; fi
  fi
  if [ -f "$DEST/config.toml" ]; then
    if chiedi "Cancellare anche config.toml?"; then rm -f "$DEST/config.toml"; ok "config.toml rimosso."; else info "config.toml conservato."; fi
  fi
  if [ -d "$DEST/modelli" ]; then
    if [ -z "$(find "$DEST/modelli" -mindepth 1 ! -name LEGGIMI.txt -print -quit)" ]; then
      rm -rf "$DEST/modelli"
    elif chiedi "Cancellare anche modelli/ (i tuoi file GGUF)? Irreversibile"; then rm -rf "$DEST/modelli"; ok "modelli/ rimossa."; else info "modelli/ conservata."; fi
  fi
  rmdir "$DEST" 2>/dev/null || true
  ok "Disinstallazione completata."
  exit 0
fi

# ------------------------------------------------------------ sorgenti
EXE=""; EXE_LOCALE=""; CFG=""
if [ "$DA_SORGENTE" = 1 ]; then
  if ! command -v cargo >/dev/null 2>&1; then
    errore 'Impossibile compilare: "cargo" non trovato.'
    avviso 'Installa Rust da https://rustup.rs (curl --proto "=https" --tlsv1.2 -sSf https://sh.rustup.rs | sh)'
    exit 1
  fi
  PRODOTTO="$RADICE/motore/target/release/avventura"
  STAGING="$(mktemp -d)"
  trap 'rm -rf "$STAGING"' EXIT
  info "Compilazione in corso (cargo build --release)..."
  (cd "$RADICE/motore" && cargo build --release)
  cp "$PRODOTTO" "$STAGING/avventura"; EXE="$STAGING/avventura"
  if [ "$CON_LOCALE" = 1 ]; then
    FEATURE=locale
    if command -v nvcc >/dev/null 2>&1 || [ -n "${CUDA_PATH:-}" ]; then FEATURE=cuda; fi
    info "Compilazione della variante con modello locale (--features $FEATURE)..."
    (cd "$RADICE/motore" && cargo build --release --features "$FEATURE")
    cp "$PRODOTTO" "$STAGING/avventura-locale"; EXE_LOCALE="$STAGING/avventura-locale"
  fi
  DIR_AVV="$RADICE/avventure"; [ -d "$DIR_AVV" ] || DIR_AVV="$RADICE/dist/avventure"
  for c in "$RADICE/config/config.esempio.toml" "$RADICE/dist/config.esempio.toml"; do [ -f "$c" ] && { CFG="$c"; break; }; done
else
  for c in "$RADICE/dist/avventura" "$RADICE/avventura"; do [ -f "$c" ] && { EXE="$c"; break; }; done
  for c in "$RADICE/dist/avventura-locale" "$RADICE/avventura-locale" "$RADICE/dist/avventura-cuda" "$RADICE/avventura-cuda"; do [ -f "$c" ] && { EXE_LOCALE="$c"; break; }; done
  DIR_AVV="$RADICE/dist/avventure"; [ -d "$DIR_AVV" ] || DIR_AVV="$RADICE/avventure"
  for c in "$RADICE/dist/config.esempio.toml" "$RADICE/config.esempio.toml" "$RADICE/config/config.esempio.toml"; do [ -f "$c" ] && { CFG="$c"; break; }; done
  if [ "$CON_LOCALE" = 1 ] && [ -z "$EXE_LOCALE" ]; then
    avviso "Variante con modello locale non disponibile nel pacchetto: installo solo la versione standard (usa --da-sorgente)."
  fi
fi

[ -n "$EXE" ] && [ -f "$EXE" ] || { errore "Eseguibile non trovato. Compila o usa --da-sorgente."; exit 1; }
[ -d "$DIR_AVV" ] || { errore "Cartella delle avventure non trovata: $DIR_AVV"; exit 1; }

# ------------------------------------------------------------ copia
info "Installazione in: $DEST"
install -m 755 "$EXE" "$DEST/avventura"
ok "Copiato avventura."
VARIANTE=""
if [ "$CON_LOCALE" = 1 ] && [ -n "$EXE_LOCALE" ]; then
  install -m 755 "$EXE_LOCALE" "$DEST/avventura-locale"
  VARIANTE="avventura-locale"
  ok "Copiato avventura-locale."
fi

# modelli/ : creata se manca, mai toccata se esiste
if [ ! -d "$DEST/modelli" ]; then
  mkdir -p "$DEST/modelli"
  if [ -f "$RADICE/dist/modelli/LEGGIMI.txt" ]; then
    cp "$RADICE/dist/modelli/LEGGIMI.txt" "$DEST/modelli/LEGGIMI.txt"
  else
    cat > "$DEST/modelli/LEGGIMI.txt" <<'EOF'
Modelli locali (file GGUF)
=========================
Metti qui i file .gguf (ad esempio gemma-4-12b-it-Q4_K_M.gguf) e indica il percorso nel
profilo "gguf" di config.toml (campo percorso_modello).
Il fornitore "locale" funziona solo con la variante compilata con il supporto ai modelli
locali (avventura-locale), non con la versione standard.
Alternativa senza file GGUF: Ollama o LM Studio (profili ollama / lmstudio).
Questa cartella non viene mai toccata dall'installatore.
EOF
  fi
  ok "Creata la cartella modelli/ (qui vanno i file GGUF)."
else
  info "modelli/ esistente: non modificata."
fi

if [ -n "$CFG" ] && [ -f "$CFG" ]; then
  cp -f "$CFG" "$DEST/config.esempio.toml"
  if [ ! -f "$DEST/config.toml" ]; then cp "$CFG" "$DEST/config.toml"; ok "Creato config.toml da config.esempio.toml."; else info "config.toml esistente: non modificato."; fi
else
  avviso "config.esempio.toml non trovato: saltato."
fi

mkdir -p "$DEST/avventure"
# Versioni inglesi separate delle avventure di esempio, sostituite da un'unica
# versione con traduzioni/it.toml.
for vecchia in faro-punta-nera-en space-chicken-en; do
  if [ -d "$DEST/avventure/$vecchia" ]; then
    rm -rf "$DEST/avventure/$vecchia"
    info "Rimossa la vecchia versione separata: $vecchia"
  fi
done
n=0
for a in "$DIR_AVV"/*/; do
  [ -d "$a" ] || continue
  nome="$(basename "$a")"
  # Sostituisce l'avventura del pacchetto: non restano file di versioni precedenti.
  rm -rf "$DEST/avventure/$nome"
  mkdir -p "$DEST/avventure/$nome"
  cp -R "$a". "$DEST/avventure/$nome/"
  ok "Avventura aggiornata: $nome"
  n=$((n+1))
done
[ "$n" -gt 0 ] || avviso "Nessuna avventura trovata nel pacchetto."
# partite/ non viene mai toccata.

# ------------------------------------------------------------ link nel PATH
mkdir -p "$BIN_DIR"
ln -sf "$DEST/avventura" "$LINK"
ok "Link creato: $LINK"
case ":$PATH:" in
  *":$BIN_DIR:"*) ;;
  *) avviso "$BIN_DIR non è nel PATH: aggiungi  export PATH=\"\$HOME/.local/bin:\$PATH\"  al tuo ~/.bashrc o ~/.zshrc." ;;
esac

echo
ok "=== Installazione completata ==="
info "Prossimi passi:"
echo "  1. Scegli un profilo in  $DEST/config.toml"
echo "       - cloud (OpenAI, Anthropic, Gemini): export OPENAI_API_KEY=\"la-tua-chiave\"  (aggiungila al tuo ~/.bashrc)"
echo "       - Ollama o LM Studio: nessuna chiave, basta che il server sia avviato"
echo "       - modello GGUF locale: copia il file in  $DEST/modelli/  e usa il profilo gguf"
[ -z "$VARIANTE" ] || echo "         (e avvia  $DEST/$VARIANTE  invece di avventura)"
echo "  2. Prova il collegamento:   avventura prova-ai"
echo "  3. Avvia il gioco:          avventura"
echo "     Oppure il bot Telegram:  avventura telegram"
echo "  4. Nuove avventure: copiale in  $DEST/avventure/<nome>/  (con gioco.md e stanze/)"
echo "Salvataggi e partite: $DEST/partite/ (creata dal programma al primo avvio)"
