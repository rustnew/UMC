#!/usr/bin/env bash
# ─────────────────────────────────────────────────────────────────────────────
# UMC — local installation (Linux)
#
# Installs into ~/.local:
#   - binary  : ~/.local/bin/umc-desktop
#   - entry   : ~/.local/share/applications/umc.desktop
#   - icon    : ~/.local/share/icons/hicolor/512x512/apps/umc.png
#
# Two modes:
#   · from the repo   : ./packaging/install.sh            (build + install)
#   · from the package: ./install.sh                      (binary provided)
# ─────────────────────────────────────────────────────────────────────────────

set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PREFIX="${HOME}/.local"
BIN_DIR="${PREFIX}/bin"
APP_DIR="${PREFIX}/share/applications"
ICON_DIR="${PREFIX}/share/icons/hicolor/512x512/apps"

# Binary provided in the package? (install.sh and umc-desktop side by side)
BIN="$HERE/umc-desktop"
if [[ ! -x "$BIN" ]]; then
  ROOT="$(cd "$HERE/.." && pwd)"
  echo "🔨 Building release of umc-desktop…"
  (cd "$ROOT" && cargo build --release -p umc-desktop)
  BIN="$ROOT/target/release/umc-desktop"
fi

if [[ ! -x "$BIN" ]]; then
  echo "❌ Binary not found. Build it first: cargo build --release -p umc-desktop" >&2
  exit 1
fi

# .desktop file (package or repo)
DESKTOP="$HERE/umc.desktop"
[[ -f "$DESKTOP" ]] || DESKTOP="$HERE/../packaging/umc.desktop"

echo "📦 Installing into ${PREFIX}…"
mkdir -p "$BIN_DIR" "$APP_DIR" "$ICON_DIR"

install -m 0755 "$BIN" "$BIN_DIR/umc-desktop"

# Icon (blue "UMC" square) if ImageMagick is available.
if command -v convert >/dev/null 2>&1; then
  convert -size 512x512 xc:"#4f9de9" \
    -gravity center -fill white -pointsize 220 -annotate 0 "UMC" \
    "$ICON_DIR/umc.png" 2>/dev/null || true
fi

sed "s|^Exec=.*|Exec=${BIN_DIR}/umc-desktop|" "$DESKTOP" > "$APP_DIR/umc.desktop"
chmod 0644 "$APP_DIR/umc.desktop"

command -v update-desktop-database >/dev/null 2>&1 && update-desktop-database "$APP_DIR" || true

echo ""
echo "✅ UMC Desktop installed!"
echo "   Launch it from the applications menu or: ${BIN_DIR}/umc-desktop"