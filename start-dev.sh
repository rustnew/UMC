#!/usr/bin/env bash
# ─── UMC Development Startup Script ───────────────────────────────────────────
# Launches the Rust backend and the React frontend in the background for
# local development.
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
command -v bun >/dev/null 2>&1 || command -v npm >/dev/null 2>&1 || { echo -e "${RED}❌ Bun or npm not found. Install Node.js ≥ 20${NC}"; exit 1; }

# ─── 1. Rust Backend (umc-api) ────────────────────────────────────────────────

echo -e "${YELLOW}🔧 Starting umc-api (Rust backend) on port 8080...${NC}"
cargo run -p umc-api &
API_PID=$!
echo -e "${GREEN}✓ umc-api started (PID: $API_PID)${NC}"

# ─── 2. Frontend (umc-frontend) ───────────────────────────────────────────────

echo -e "${YELLOW}🌐 Starting umc-frontend (Vite) on port 5173...${NC}"
cd "$SCRIPT_DIR/umc-frontend"

# Install deps if node_modules doesn't exist
if [ ! -d "node_modules" ]; then
    echo -e "${YELLOW}📦 Installing frontend dependencies...${NC}"
    if command -v bun >/dev/null 2>&1; then
        bun install
    else
        npm install
    fi
fi

if command -v bun >/dev/null 2>&1; then
    bun run dev &
else
    npm run dev &
fi
UI_PID=$!
echo -e "${GREEN}✓ umc-frontend started (PID: $UI_PID)${NC}"

# ─── Summary ──────────────────────────────────────────────────────────────────

echo ""
echo -e "${GREEN}═══════════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}  UMC Development Services Started${NC}"
echo -e "${GREEN}═══════════════════════════════════════════════════════════${NC}"
echo -e "  API:      http://localhost:8080  (PID: $API_PID)"
echo -e "  Frontend: http://localhost:5173  (PID: $UI_PID)"
echo -e "${GREEN}═══════════════════════════════════════════════════════════${NC}"
echo ""
echo "Press Ctrl+C to stop all services."

# Wait for all background processes
wait