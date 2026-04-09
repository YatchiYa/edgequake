#!/usr/bin/env sh
# EdgeQuake — One-Command Quickstart
#
# Usage (no git clone required):
#   curl -fsSL https://raw.githubusercontent.com/raphaelmansuy/edgequake/edgequake-main/quickstart.sh | sh
#
# Or with a pinned version:
#   curl -fsSL https://raw.githubusercontent.com/raphaelmansuy/edgequake/edgequake-main/quickstart.sh | \
#     EDGEQUAKE_VERSION=0.9.4 sh
#
# Prerequisites: Docker (https://docs.docker.com/get-docker/)

set -e

# ── Configurable defaults ──────────────────────────────────────────────────────
EDGEQUAKE_VERSION="${EDGEQUAKE_VERSION:-latest}"
EDGEQUAKE_PORT="${EDGEQUAKE_PORT:-8080}"
FRONTEND_PORT="${FRONTEND_PORT:-3000}"
COMPOSE_FILE="${COMPOSE_FILE:-docker-compose.quickstart.yml}"
RAW_BASE="https://raw.githubusercontent.com/raphaelmansuy/edgequake/edgequake-main"

# ── Colour helpers (disabled when not a TTY) ──────────────────────────────────
if [ -t 1 ]; then
  BOLD="\033[1m"; RESET="\033[0m"; GREEN="\033[32m"; YELLOW="\033[33m"; RED="\033[31m"; BLUE="\033[34m"
else
  BOLD=""; RESET=""; GREEN=""; YELLOW=""; RED=""; BLUE=""
fi

header() { printf "\n${BOLD}${BLUE}%s${RESET}\n\n" "$1"; }
ok()     { printf "  ${GREEN}✓${RESET} %s\n" "$1"; }
info()   { printf "  ${YELLOW}→${RESET} %s\n" "$1"; }
fail()   { printf "  ${RED}✗${RESET} %s\n" "$1" >&2; }

# ── Pre-flight checks ──────────────────────────────────────────────────────────
header "EdgeQuake Quickstart"

# Docker
if ! command -v docker > /dev/null 2>&1; then
  fail "Docker is not installed. Install it from https://docs.docker.com/get-docker/ and re-run."
  exit 1
fi
ok "Docker found: $(docker --version | head -1)"

# docker compose (v2 plugin or standalone v1)
if docker compose version > /dev/null 2>&1; then
  COMPOSE_CMD="docker compose"
elif command -v docker-compose > /dev/null 2>&1; then
  COMPOSE_CMD="docker-compose"
else
  fail "docker compose (v2 plugin) or docker-compose (v1) is required."
  fail "Install: https://docs.docker.com/compose/install/"
  exit 1
fi
ok "Compose found: $($COMPOSE_CMD version --short 2>/dev/null || echo 'v1')"

# ── Download / refresh compose file ────────────────────────────────────────────
# WHY always refresh: a cached compose file from a previous run may be an older
# version that is missing new env vars or service definitions.  We download a
# fresh copy every time, backing up any local customisations first.
_download_compose() {
  if command -v curl > /dev/null 2>&1; then
    curl -fsSL "${RAW_BASE}/docker-compose.quickstart.yml" -o "$COMPOSE_FILE"
  elif command -v wget > /dev/null 2>&1; then
    wget -qO "$COMPOSE_FILE" "${RAW_BASE}/docker-compose.quickstart.yml"
  else
    fail "curl or wget is required to download the compose file."
    exit 1
  fi
}

if [ -f "$COMPOSE_FILE" ]; then
  info "Refreshing compose file (backing up existing → ${COMPOSE_FILE}.bak)…"
  cp "$COMPOSE_FILE" "${COMPOSE_FILE}.bak"
  _download_compose
  ok "Compose file updated: ./${COMPOSE_FILE}"
else
  info "Downloading compose file…"
  _download_compose
  ok "Compose file saved to ./${COMPOSE_FILE}"
fi

# ── LLM + Embedding provider auto-detection ───────────────────────────────────
if [ -n "$OPENAI_API_KEY" ]; then
  EDGEQUAKE_LLM_PROVIDER="${EDGEQUAKE_LLM_PROVIDER:-openai}"
  # WHY: When OpenAI is used for LLM inference, prefer OpenAI embeddings by
  # default so that the workspace is fully self-contained without requiring a
  # local Ollama instance.  The user can still override these via env vars.
  EDGEQUAKE_EMBEDDING_PROVIDER="${EDGEQUAKE_EMBEDDING_PROVIDER:-openai}"
  EDGEQUAKE_LLM_MODEL="${EDGEQUAKE_LLM_MODEL:-gpt-5-mini}"
  EDGEQUAKE_EMBEDDING_MODEL="${EDGEQUAKE_EMBEDDING_MODEL:-text-embedding-3-small}"
  ok "OpenAI API key detected — using OpenAI for LLM (${EDGEQUAKE_LLM_MODEL}) and embeddings (${EDGEQUAKE_EMBEDDING_MODEL})"
else
  EDGEQUAKE_LLM_PROVIDER="${EDGEQUAKE_LLM_PROVIDER:-ollama}"
  EDGEQUAKE_EMBEDDING_PROVIDER="${EDGEQUAKE_EMBEDDING_PROVIDER:-ollama}"
  info "No API key found — using Ollama (must be running on port 11434)"
  info "  To use OpenAI: export OPENAI_API_KEY=sk-... && sh quickstart.sh"
fi

# ── Prior installation detection ───────────────────────────────────────────────
# WHY: Detect stopped or running containers from a previous install and inform
# the user clearly before proceeding.  This prevents silent data overwrites and
# helps users understand what is happening on re-runs.
_edgequake_containers_exist() {
  docker ps -a --filter "name=edgequake-api" --filter "name=edgequake-postgres" \
    --format "{{.Names}}" 2>/dev/null | grep -q "edgequake"
}

_edgequake_containers_running() {
  docker ps --filter "name=edgequake-api" --filter "status=running" \
    --format "{{.Names}}" 2>/dev/null | grep -q "edgequake-api"
}

_edgequake_volume_exists() {
  docker volume ls --filter "name=edgequake" --format "{{.Name}}" 2>/dev/null | grep -q "edgequake"
}

if _edgequake_containers_running; then
  printf "\n${BOLD}${YELLOW}⚠  Prior EdgeQuake installation detected (containers are RUNNING)${RESET}\n\n"
  printf "  Running services:\n"
  docker ps --filter "name=edgequake" --format "    • {{.Names}}  [{{.Status}}]" 2>/dev/null
  printf "\n\n"
  printf "  Options:\n"
  printf "    ${BOLD}Update${RESET}  (pull latest images + restart with current config) — ${BOLD}press Enter${RESET}\n"
  printf "    ${BOLD}Quit${RESET}    (leave existing stack unchanged)                    — type ${BOLD}q${RESET} + Enter\n"
  printf "\n"
  printf "  Choice [Enter/q]: "
  read -r _choice 2>/dev/null || _choice=""
  if [ "$_choice" = "q" ] || [ "$_choice" = "Q" ]; then
    ok "Leaving existing installation unchanged."
    printf "\n${BOLD}Management commands:${RESET}\n"
    printf "  Logs:   $COMPOSE_CMD -f $COMPOSE_FILE logs -f\n"
    printf "  Stop:   $COMPOSE_CMD -f $COMPOSE_FILE down\n"
    printf "  Update: sh quickstart.sh\n\n"
    exit 0
  fi
  ok "Proceeding with update…"
elif _edgequake_containers_exist; then
  printf "\n${BOLD}${YELLOW}⚠  Prior EdgeQuake installation detected (containers are STOPPED)${RESET}\n\n"
  docker ps -a --filter "name=edgequake" --format "    • {{.Names}}  [{{.Status}}]" 2>/dev/null
  printf "\n"
  if _edgequake_volume_exists; then
    ok "Existing data volume found — your data will be preserved on restart."
  fi
  printf "\n"
  ok "Restarting existing installation with latest images…"
fi

# ── Handle existing containers (idempotent re-runs) ───────────────────────────
# WHY --force-recreate: `docker compose up -d` reuses existing containers even
# when environment variables have changed.  Force-recreating ensures the latest
# provider config is always applied on every run.
# WHY --remove-orphans: removes containers for services that were removed in an
# updated compose file, keeping the stack clean on upgrades.
_compose_env() {
  # WHY: Only forward OPENAI_API_KEY / OPENAI_BASE_URL when non-empty.
  # Docker Compose evaluates ${VAR:-} to "" when the host variable is unset,
  # passing an empty string into the container. The OpenAI provider reads the
  # env var unconditionally and uses an empty string as the API base URL, which
  # causes reqwest to fail with "builder error" on every request.
  EDGEQUAKE_VERSION="$EDGEQUAKE_VERSION" \
  EDGEQUAKE_LLM_PROVIDER="$EDGEQUAKE_LLM_PROVIDER" \
  EDGEQUAKE_EMBEDDING_PROVIDER="$EDGEQUAKE_EMBEDDING_PROVIDER" \
  EDGEQUAKE_LLM_MODEL="${EDGEQUAKE_LLM_MODEL:-}" \
  EDGEQUAKE_EMBEDDING_MODEL="${EDGEQUAKE_EMBEDDING_MODEL:-}" \
  EDGEQUAKE_PORT="$EDGEQUAKE_PORT" \
  FRONTEND_PORT="$FRONTEND_PORT" \
  "$@"
}

# ── Pull images ───────────────────────────────────────────────────────────────
info "Pulling EdgeQuake images (version: ${EDGEQUAKE_VERSION})…"
_compose_env $COMPOSE_CMD -f "$COMPOSE_FILE" pull

# ── LLM provider reachability check ───────────────────────────────────────────
if [ "$EDGEQUAKE_LLM_PROVIDER" = "ollama" ]; then
  _ollama_host="${OLLAMA_HOST:-http://localhost:11434}"
  if curl -sf "${_ollama_host}/api/tags" > /dev/null 2>&1; then
    ok "Ollama is reachable at ${_ollama_host}"
  else
    printf "\n${BOLD}${YELLOW}⚠  Ollama is not reachable at ${_ollama_host}${RESET}\n"
    printf "\n"
    printf "  EdgeQuake will start, but document processing will fail until Ollama is running.\n"
    printf "\n"
    printf "  To fix before continuing:\n"
    printf "    ${BOLD}ollama serve &${RESET}            # start in background\n"
    printf "    ${BOLD}ollama pull gemma4:latest${RESET}  # pull a model (first time only)\n"
    printf "\n"
    printf "  Or switch to OpenAI:\n"
    printf "    ${BOLD}export OPENAI_API_KEY=sk-... && sh quickstart.sh${RESET}\n"
    printf "\n"
    printf "  Continue anyway? [y/N]: "
    read -r _ollama_choice 2>/dev/null || _ollama_choice="n"
    case "$_ollama_choice" in
      y|Y) info "Continuing — remember to start Ollama before uploading documents." ;;
      *) fail "Aborted. Start Ollama and re-run the quickstart."; exit 1 ;;
    esac
  fi
fi

# ── Start stack ───────────────────────────────────────────────────────────────
info "Starting all services (detached)…"
_compose_env $COMPOSE_CMD -f "$COMPOSE_FILE" up -d --force-recreate --remove-orphans

# ── Wait for API health ───────────────────────────────────────────────────────
info "Waiting for API to be healthy (up to 90s)..."
i=0
while [ $i -lt 45 ]; do
  if curl -sf "http://localhost:${EDGEQUAKE_PORT}/health" > /dev/null 2>&1; then
    ok "API is healthy!"
    break
  fi
  printf "."
  sleep 2
  i=$((i + 1))
done
printf "\n"

if ! curl -sf "http://localhost:${EDGEQUAKE_PORT}/health" > /dev/null 2>&1; then
  fail "API did not become healthy within 90s."
  info "Check logs: $COMPOSE_CMD -f $COMPOSE_FILE logs -f api"
  exit 1
fi

# ── Done ──────────────────────────────────────────────────────────────────────
printf "\n${BOLD}${GREEN}✅  EdgeQuake is running!${RESET}\n\n"
printf "  🌐  Web UI:  ${BOLD}http://localhost:${FRONTEND_PORT}${RESET}\n"
printf "  🔗  API:     ${BOLD}http://localhost:${EDGEQUAKE_PORT}${RESET}\n"
printf "  📚  Swagger: ${BOLD}http://localhost:${EDGEQUAKE_PORT}/swagger-ui${RESET}\n"
printf "  🏥  Health:  ${BOLD}http://localhost:${EDGEQUAKE_PORT}/health${RESET}\n"
printf "\n"
if [ "$EDGEQUAKE_LLM_PROVIDER" = "openai" ]; then
  printf "  🤖  LLM:     ${BOLD}OpenAI — ${EDGEQUAKE_LLM_MODEL}${RESET}\n\n"
else
  printf "  🤖  LLM:     ${BOLD}Ollama — ${OLLAMA_HOST:-http://localhost:11434}${RESET}\n"
  printf "       Ensure a model is pulled: ${BOLD}ollama pull gemma4:latest${RESET}\n\n"
fi
printf "${BOLD}Next steps:${RESET}\n"
printf "  1. Open ${BOLD}http://localhost:${FRONTEND_PORT}${RESET} in your browser\n"
printf "  2. Upload a PDF or paste text to build your knowledge graph\n"
printf "  3. Ask questions — EdgeQuake retrieves graph-aware answers\n"
printf "\n"
printf "${YELLOW}Management:${RESET}\n"
printf "  Logs:   $COMPOSE_CMD -f $COMPOSE_FILE logs -f\n"
printf "  Status: $COMPOSE_CMD -f $COMPOSE_FILE ps\n"
printf "  Stop:   $COMPOSE_CMD -f $COMPOSE_FILE down\n"
printf "  Update: sh quickstart.sh\n"
printf "\n"
