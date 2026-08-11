#!/usr/bin/env bash
# ─────────────────────────────────────────────────────────────────────────────
# UMC — installation locale (Linux)
#
# Installe dans ~/.local :
#   - binaire  : ~/.local/bin/umc-desktop
#   - entrée   : ~/.local/share/applications/umc.desktop
#   - icône    : ~/.local/share/icons/hicolor/512x512/apps/umc.png
#
# Deux modes :
#   · depuis le repo    : ./packaging/install.sh            (build + install)
#   · depuis le package : ./install.sh                      (binaire fourni)
# ─────────────────────────────────────────────────────────────────────────────

set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PREFIX="${HOME}/.local"
BIN_DIR="${PREFIX}/bin"
APP_DIR="${PREFIX}/share/applications"
ICON_DIR="${PREFIX}/share/icons/hicolor/512x512/apps"

# Binaire fourni dans le package ? (install.sh et umc-desktop côte à côte)
BIN="$HERE/umc-desktop"
if [[ ! -x "$BIN" ]]; then
  ROOT="$(cd "$HERE/.." && pwd)"
  echo "🔨 Build release de umc-desktop…"
  (cd "$ROOT" && cargo build --release -p umc-desktop)
  BIN="$ROOT/target/release/umc-desktop"
fi

if [[ ! -x "$BIN" ]]; then
  echo "❌ Binaire introuvable. Compilez d'abord : cargo build --release -p umc-desktop" >&2
  exit 1
fi

# Fichier .desktop (package ou repo)
DESKTOP="$HERE/umc.desktop"
[[ -f "$DESKTOP" ]] || DESKTOP="$HERE/../packaging/umc.desktop"

echo "📦 Installation dans ${PREFIX}…"
mkdir -p "$BIN_DIR" "$APP_DIR" "$ICON_DIR"

install -m 0755 "$BIN" "$BIN_DIR/umc-desktop"

# Icône (carré bleu « UMC ») si ImageMagick est disponible.
if command -v convert >/dev/null 2>&1; then
  convert -size 512x512 xc:"#4f9de9" \
    -gravity center -fill white -pointsize 220 -annotate 0 "UMC" \
    "$ICON_DIR/umc.png" 2>/dev/null || true
fi

sed "s|^Exec=.*|Exec=${BIN_DIR}/umc-desktop|" "$DESKTOP" > "$APP_DIR/umc.desktop"
chmod 0644 "$APP_DIR/umc.desktop"

command -v update-desktop-database >/dev/null 2>&1 && update-desktop-database "$APP_DIR" || true

echo ""
echo "✅ UMC Desktop installé !"
echo "   Lancez-le depuis le menu d'applications ou : ${BIN_DIR}/umc-desktop"