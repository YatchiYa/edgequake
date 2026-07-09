#!/usr/bin/env bash
# Clear corrupted Next.js dev cache when route manifest is broken.
# Symptom: all pages 404, routes.d.ts has "type AppRoutes = never" or truncated merge.

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
ROUTES_FILE="$ROOT/.next/dev/types/routes.d.ts"

if [[ ! -f "$ROUTES_FILE" ]]; then
  exit 0
fi

if grep -q 'type AppRoutes = never' "$ROUTES_FILE" \
  || grep -q 'extends Routes> = ParamMap\[Route\]' "$ROUTES_FILE"; then
  echo "[frontend] Corrupted Next.js route cache detected — clearing .next"
  rm -rf "$ROOT/.next" "$ROOT/node_modules/.cache"
fi
