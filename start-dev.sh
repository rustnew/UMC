#!/usr/bin/env bash
# ─── UMC Development Startup Script ───────────────────────────────────────────
# Launches the Rust backend for local development.
#
# Usage: ./start-dev.sh
# ──────────────────────────────────────────────────────────────────────────────

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${YELLOW}🚀 Starting UMC development services...${NC}"

# ─── Check prerequisites ──────────────────────────────────────────────────────

command -v cargo >/dev/null 2>&1 || { echo -e "${RED}❌ Rust/Cargo not found. Install from https://rustup.rs${NC}"; exit 1; }

# ─── 1. Rust Backend (umc-api) ────────────────────────────────────────────────

echo -e "${YELLOW}🔧 Starting umc-api (Rust backend) on port 8080...${NC}"
cargo run -p umc-api &
API_PID=$!
echo -e "${GREEN}✓ umc-api started (PID: $API_PID)${NC}"

# ─── Summary ──────────────────────────────────────────────────────────────────

echo ""
echo -e "${GREEN}═══════════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}  UMC Development Services Started${NC}"
echo -e "${GREEN}═══════════════════════════════════════════════════════════${NC}"
echo -e "  API:      http://localhost:8080  (PID: $API_PID)"
echo -e "${GREEN}═══════════════════════════════════════════════════════════${NC}"
echo ""
echo "Press Ctrl+C to stop."

# Wait for all background processes
wait