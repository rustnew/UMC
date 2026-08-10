#!/usr/bin/env bash
# ─── UMC Production Startup Script ────────────────────────────────────────────
# Launches all services via Docker Compose for production deployment.
#
# Usage: ./start-prod.sh
# ──────────────────────────────────────────────────────────────────────────────

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${YELLOW}🚀 Starting UMC production services via Docker Compose...${NC}"

# Check prerequisites
command -v docker >/dev/null 2>&1 || { echo -e "${RED}❌ Docker not found.${NC}"; exit 1; }
docker compose version >/dev/null 2>&1 || { echo -e "${RED}❌ Docker Compose not found.${NC}"; exit 1; }

# Check for required environment files
if [ ! -f "umc-api/.env" ]; then
    echo -e "${YELLOW}⚠️  umc-api/.env not found. Creating from example...${NC}"
    cp umc-api/.env.example umc-api/.env
    echo -e "${YELLOW}Please edit umc-api/.env with your real credentials before continuing.${NC}"
fi

if [ ! -f "umc-frontend/.env" ]; then
    echo -e "${YELLOW}⚠️  umc-frontend/.env not found. Creating from example...${NC}"
    cp umc-frontend/.env.example umc-frontend/.env
fi

# Build and start
echo -e "${YELLOW}🔨 Building and starting services...${NC}"
docker compose up -d --build

# Wait for services to be healthy
echo -e "${YELLOW}⏳ Waiting for services to start...${NC}"
sleep 5

# Run health check
if [ -f "$SCRIPT_DIR/healthcheck.sh" ]; then
    "$SCRIPT_DIR/healthcheck.sh"
fi

echo ""
echo -e "${GREEN}═══════════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}  UMC Production Services Started${NC}"
echo -e "${GREEN}═══════════════════════════════════════════════════════════${NC}"
echo -e "  API:      http://localhost:8080"
echo -e "  Frontend: http://localhost:8081"
echo -e "  Database: localhost:5432"
echo -e "${GREEN}═══════════════════════════════════════════════════════════${NC}"
echo ""
echo "To stop: docker compose down"