#!/usr/bin/env bash
# ─────────────────────────────────────────────────────────────────────────────
# UMC — Build release packages (Linux x86_64)
#
# Produces, in dist/:
#   umc-v1.0.0-linux-x86_64.tar.gz   (umc + umc-desktop + install.sh + .desktop)
#
# Usage:
#   ./packaging/build-release.sh
# ─────────────────────────────────────────────────────────────────────────────

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
VERSION="$(grep '^version' "$ROOT/umc-desktop/Cargo.toml" | head -1 | awk '{print $3}' | tr -d '"')"
DIST="$ROOT/dist"
STAGE="$DIST/umc-$VERSION-linux-x86_64"

echo "🔨 Building umc v${VERSION} (release)…"
(cd "$ROOT" && cargo build --release -p umc-cli -p umc-desktop)

mkdir -p "$STAGE"
cp "$ROOT/target/release/umc"        "$STAGE/umc"
cp "$ROOT/target/release/umc-desktop" "$STAGE/umc-desktop"
cp "$ROOT/packaging/install.sh"      "$STAGE/install.sh"
cp "$ROOT/packaging/umc.desktop"     "$STAGE/umc.desktop"
chmod +x "$STAGE/umc" "$STAGE/umc-desktop" "$STAGE/install.sh"

tar -C "$DIST" -czf "$DIST/umc-$VERSION-linux-x86_64.tar.gz" "umc-$VERSION-linux-x86_64"
rm -rf "$STAGE"

echo "✅ Package : $DIST/umc-$VERSION-linux-x86_64.tar.gz"
du -h "$DIST/umc-$VERSION-linux-x86_64.tar.gz"