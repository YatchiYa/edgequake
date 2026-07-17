#!/usr/bin/env bash
# scripts/install_migration_hooks.sh
#
# PURPOSE: Install a pre-commit git hook that:
#   1. Blocks modification of existing migration files (issue #195 class).
#   2. Blocks adding new NNN_*.sql without staging checksums.lock (atomic lock rule).
#
# Usage:
#   ./scripts/install_migration_hooks.sh
#
# The hook is installed at .git/hooks/pre-commit.
# If a pre-commit hook already exists, a backup is created.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
HOOK_DIR="$REPO_ROOT/.git/hooks"
HOOK_FILE="$HOOK_DIR/pre-commit"

HOOK_CONTENT='#!/usr/bin/env bash
# Migration immutability pre-commit hook.
# Installed by: scripts/install_migration_hooks.sh
#
# Blocks modification of existing top-level sqlx migrations (NNN_*.sql).
# Auxiliary DDL under migrations/support/ remains editable (not in _sqlx_migrations).

STAGED_MODIFIED=$(git diff --cached --name-only --diff-filter=M -- '"'"'edgequake/migrations/'"'"' \
  | grep -E '"'"'^edgequake/migrations/[0-9]+_.*\.sql$'"'"' || true)

if [[ -n "$STAGED_MODIFIED" ]]; then
  echo ""
  echo "⛔  BLOCKED: You are modifying existing migration file(s):"
  echo ""
  for f in $STAGED_MODIFIED; do
    echo "    $f"
  done
  echo ""
  echo "  Migration files are IMMUTABLE once deployed to any database."
  echo "  SQLx stores SHA-384 checksums in _sqlx_migrations at first apply."
  echo "  Changing a deployed migration file will break all existing deployments."
  echo ""
  echo "  To make schema changes: create a NEW migration file."
  echo "  To bypass this check (e.g., reverting a broken migration):"
  echo "    git commit --no-verify"
  echo ""
  exit 1
fi

# New migrations must stage checksums.lock in the same commit (atomic lock rule).
LOCKFILE_STAGED=$(git diff --cached --name-only -- '"'"'edgequake/migrations/checksums.lock'"'"')
SQL_STAGED=$(git diff --cached --name-only --diff-filter=A -- '"'"'edgequake/migrations/*.sql'"'"' \
  | grep -E '"'"'^edgequake/migrations/[0-9]+_.*\.sql$'"'"' || true)
if [[ -n "$SQL_STAGED" && -z "$LOCKFILE_STAGED" ]]; then
  echo ""
  echo "⛔  BLOCKED: New migration file(s) staged without checksums.lock:"
  echo ""
  for f in $SQL_STAGED; do
    echo "    $f"
  done
  echo ""
  echo "  New migrations and checksums.lock must ship in the same commit."
  echo "  Run:"
  echo "    ./scripts/update_migration_checksums.sh"
  echo "  then stage edgequake/migrations/checksums.lock and retry."
  echo ""
  echo "  To bypass this check (anti-pattern; CI will still fail):"
  echo "    git commit --no-verify"
  echo ""
  exit 1
fi

exit 0
'

mkdir -p "$HOOK_DIR"

if [[ -f "$HOOK_FILE" ]]; then
  BACKUP="$HOOK_FILE.bak.$(date +%s)"
  cp "$HOOK_FILE" "$BACKUP"
  echo "Existing hook backed up to: $BACKUP"
fi

echo "$HOOK_CONTENT" > "$HOOK_FILE"
chmod +x "$HOOK_FILE"
echo "Installed pre-commit hook at: $HOOK_FILE"
echo ""
echo "The hook will now:"
echo "  - block modification of existing migration files"
echo "  - block new NNN_*.sql without a staged checksums.lock"
