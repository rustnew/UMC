#!/usr/bin/env bash
# ─────────────────────────────────────────────────────────────────────────────
# UMC Desktop — installation locale (Linux)
#
# Construit l'app en release et l'installe dans ~/.local :
#   - binaire  : ~/.local/bin/umc-desktop
#   - entrée   : ~/.local/share/applications/umc.desktop
#   - icône    : ~/.local/share/icons/hicolor/512x512/apps/umc.png
#
# Usage :
#   ./packaging/install.sh            # build + install
#   ./packaging/install.sh --no-build # installer seulement (binaire déjà construit)
# ─────────────────────────────────────────────────────────────────────────────

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PREFIX="${HOME}/.local"
BIN_DIR="${PREFIX}/bin"
APP_DIR="${PREFIX}/share/applications"
ICON_DIR="${PREFIX}/share/icons/hicolor/512x512/apps"

NO_BUILD=0
[[ "${1:-}" == "--no-build" ]] && NO_BUILD=1

if [[ "$NO_BUILD" -eq 0 ]]; then
  echo "🔨 Build release de umc-desktop…"
  (cd "$ROOT" && cargo build --release -p umc-desktop)
fi

BIN="$ROOT/target/release/umc-desktop"
if [[ ! -x "$BIN" ]]; then
  echo "❌ Binaire introuvable : $BIN (lancez ./packaging/install.sh sans --no-build)" >&2
  exit 1
fi

echo "📦 Installation dans ${PREFIX}…"
mkdir -p "$BIN_DIR" "$APP_DIR" "$ICON_DIR"

install -m 0755 "$BIN" "$BIN_DIR/umc-desktop"

# Icône : génère un PNG simple (carré bleu avec « UMC ») si ImageMagick est dispo,
# sinon on s'appuie sur l'icône générique du thème.
if command -v convert >/dev/null 2>&1; then
  convert -size 512x512 xc:"#4f9de9" \
    -gravity center -fill white -pointsize 220 -annotate 0 "UMC" \
    "$ICON_DIR/umc.png" 2>/dev/null || true
fi

sed "s|^Exec=.*|Exec=${BIN_DIR}/umc-desktop|" "$ROOT/packaging/umc.desktop" > "$APP_DIR/umc.desktop"
chmod 0644 "$APP_DIR/umc.desktop"

# Rafraîchit le menu d'applications si possible.
command -v update-desktop-database >/dev/null 2>&1 && update-desktop-database "$APP_DIR" || true

echo ""
echo "✅ UMC Desktop installé !"
echo "   Lancez-le depuis le menu d'applications ou : ${BIN_DIR}/umc-desktop"