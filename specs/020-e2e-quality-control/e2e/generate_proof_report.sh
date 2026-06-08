#!/usr/bin/env bash
# Generate SPEC-020 proof markdown from test artifacts.
set -euo pipefail

PROOF_DIR="$(cd "$(dirname "$0")" && pwd)"
LOG="$PROOF_DIR/001-test-run.log"
OUT="$PROOF_DIR/001-quality-control-proof.md"
SCREENSHOTS="$PROOF_DIR/screenshots"

summary_field() {
  local field="$1"
  local line
  line="$(grep -E "^[[:space:]]+[0-9]+ ${field}" "$LOG" 2>/dev/null | tail -1 || true)"
  if [[ -z "$line" ]]; then
    echo "0"
    return
  fi
  echo "$line" | sed -E "s/^[[:space:]]+([0-9]+) ${field}.*/\\1/"
}

passed_count() { summary_field "passed"; }
skipped_count() { summary_field "skipped"; }
failed_count() { summary_field "failed"; }

PASSED="$(passed_count)"
SKIPPED="$(skipped_count)"
FAILED="$(failed_count)"
TOTAL=$((PASSED + SKIPPED + FAILED))
DATE="$(date -u +%Y-%m-%d)"
SHOT_COUNT="$(ls -1 "$SCREENSHOTS"/*.png 2>/dev/null | wc -l | tr -d ' ')"

MIG_READY="unknown"
if [[ -f "$PROOF_DIR/005-migration038-status.json" ]]; then
  MIG_READY="$(python3 -c "import json; d=json.load(open('$PROOF_DIR/005-migration038-status.json')); print('ready' if d.get('ready') else f\"degraded ({d.get('missingCount',0)} missing)\")" 2>/dev/null || echo unknown)"
fi

LIVE_GROUNDED="not run"
if [[ -f "$PROOF_DIR/010-live-llm-result.json" ]]; then
  LIVE_GROUNDED="$(python3 -c "import json; d=json.load(open('$PROOF_DIR/010-live-llm-result.json')); print('grounded' if d.get('grounded') else 'not grounded')" 2>/dev/null || echo unknown)"
fi

GRAPH_STATS="not run"
if [[ -f "$PROOF_DIR/019-graph-entities.json" ]]; then
  GRAPH_STATS="$(python3 -c "
import json
d=json.load(open('$PROOF_DIR/019-graph-entities.json'))
ingest=d.get('ingestEntityCount', d.get('uploaded',{}).get('entityCount',0))
delta=d.get('entityDelta',0)
synced=d.get('statsSynced', delta>0)
print(f'ingest={ingest} entities, stats_delta={delta}, synced={synced}')
" 2>/dev/null || echo unknown)"
fi

ISOLATION_STATS="not run"
if [[ -f "$PROOF_DIR/022-graph-isolation.json" ]]; then
  ISOLATION_STATS="$(python3 -c "
import json
d=json.load(open('$PROOF_DIR/022-graph-isolation.json'))
s=d.get('stats',{})
o=s.get('ownerStats',{})
t=s.get('otherStats',{})
lag=s.get('statsEntityLag', False)
ingest=s.get('ingestEntityCount',0)
print(f\"owner docs={o.get('documentCount',0)}, ingest_entities={ingest}, other docs={t.get('documentCount',0)}, stats_entity_lag={lag}\")
" 2>/dev/null || echo unknown)"
fi

cat > "$OUT" <<EOF
# SPEC-020 — Full E2E Quality Control Proof

**Status:** $([ "$FAILED" = "0" ] && [ "$PASSED" -ge 20 ] && echo "✅ Proven" || echo "❌ Failed/incomplete") ($PASSED passed, $SKIPPED skipped, $FAILED failed)
**Date:** $DATE
**Spec:** \`edgequake_webui/e2e/spec020-quality-control.spec.ts\`

## Results (24 tests)

| # | Test | Scope |
|---|------|-------|
| 01 | Backend health + migration readiness | Operational health, /ready probe, migration-038 |
| 02 | 10 critical routes smoke | Dashboard through settings |
| 03 | Sync markdown ingestion + UI | Chunks + document list |
| 04 | Hybrid query + citations | Mock or live answer |
| 05 | Graph workspace context | Graph page load |
| 06 | PDF text-parser upload | API PDF → completed |
| 07 | Multi-tenant isolation | Cross-tenant leak guard |
| 08 | Unscoped API safety | Safe empty response (dev default tenant) |
| 09 | Source citations panel | Citations UI opens |
| 10 | Live Ollama grounded query | Sarah Chen RAG (conditional) |
| 11 | UI markdown upload (dropzone) | File input + table row |
| 12 | Document detail page | Chunks visible after ingest |
| 13 | Empty query edge case | No application crash |
| 14 | Streaming completion | Textarea re-enabled |
| 15 | Unknown document 404 | API error handling |
| 16 | UI PDF upload (API proxy) | Dropzone PDF + progress panel |
| 17 | Duplicate re-upload | Re-ingestion edge |
| 18 | Empty workspace query | Query without ingest |
| 19 | Ollama entity extraction | Ingest entity_count + workspace stats delta |
| 20 | Malformed upload rejection + empty graph search | API error paths |
| 21 | Auth login probe | Build auth detection (+ full login when SPEC020_AUTH_PROOF=1) |
| 22 | Workspace stats isolation | Owner populated, other empty |
| 23 | Vision PDF flag | Text-parser fallback with enable_vision |
| 24 | Document delete cascade | DELETE → 404 + absent from list |

**Playwright:** \`$PASSED passed\`, \`$SKIPPED skipped\`, \`$FAILED failed\` ($TOTAL total)

## Artifacts

- Screenshots: **$SHOT_COUNT** files in \`screenshots/\`
- \`002-health-response.json\` — health + migration038 ($MIG_READY)
- \`010-live-llm-result.json\` — live LLM ($LIVE_GROUNDED)
- \`019-graph-entities.json\` — entity extraction ($GRAPH_STATS)
- \`022-graph-isolation.json\` — workspace stats isolation ($ISOLATION_STATS)
- \`001-test-run.log\` — Playwright stdout

## Run

\`\`\`bash
make spec020-qc-proof

# Strict migration-038 gate (prod):
SPEC020_STRICT_MIGRATION=1 make spec020-qc-proof

# Full prod gate (strict + require Ollama — no skips on 10/19/22):
make spec020-qc-proof-full

# Auth-enabled login proof:
make spec020-qc-proof-auth
\`\`\`

---

## Brutal honest assessment

### Grade: **$(if [ "$FAILED" != "0" ] || [ "$PASSED" -lt 20 ]; then echo "C (stack or tests incomplete)"; elif [ "$PASSED" -ge 24 ] && [ "$SKIPPED" = "0" ] && [ "$LIVE_GROUNDED" = "grounded" ]; then echo "A+"; elif [ "$PASSED" -ge 24 ] && [ "$LIVE_GROUNDED" = "grounded" ]; then echo "A"; elif [ "$PASSED" -ge 22 ]; then echo "A-"; else echo "B+"; fi)**

**Validated when stack is healthy:** UI shell, routes, ingest, query, PDF API+UI, isolation, citations, streaming, 404, delete cascade, duplicate re-ingestion, malformed input, live Ollama (conditional), /ready probe (strict).

**Product fixes verified in this spec:**

| Fix | Verification |
|-----|--------------|
| FIX-SPEC020 sync upload graph \`workspace_id\` scope | Tests 19/22 stats delta + graph search |
| FIX-MIG038-GIN (\`::jsonb\` + \`jsonb_ops\`) | Test 01 strict + \`ensure_migration_038.sh\` auto-repair |
| FIX-SPEC020-CASCADE (\`agtype_to_json\` → \`::jsonb\` for source-prefix queries) | Test 24 document DELETE cascade |
| FIX-METRICS (UUID column bound as text) | Post-upload metrics snapshots |
| FIX-AUDIT-INET (\`\$13::inet\` SQL cast) | Audit log persistence |
| FIX-DEV-PROXY (Next.js dev rewrites) | UI :3001 + backend :8081 port drift |

**Conditional / recorded only:**

| Signal | Value |
|--------|-------|
| Live Ollama grounded | $LIVE_GROUNDED |
| Entity extraction (19) | $GRAPH_STATS |
| Workspace isolation (22) | $ISOLATION_STATS |
| Migration-038 | $MIG_READY |

**Still not validated (honest gaps):**

| Gap | Severity |
|-----|----------|
| Vision PDF (multimodal LLM) | High — test 23 only sets flag; text parser fallback |
| Auth login E2E | Medium — default proof uses auth off; run \`make spec020-qc-proof-auth\` |
| CI without Ollama | Medium — tests 10/19/22 skip; local full gate: \`make spec020-qc-proof-full\` |

**DRY/SOLID modules:** \`qc-api-route\`, \`qc-graph\`, \`qc-health\`, \`qc-workspace\`, \`qc-ui-upload\`, \`qc-query\`, \`qc-isolation\`, \`qc-documents\`, \`qc-api-errors\`, \`qc-auth\`, \`spec020-artifacts\`, \`llm-availability\`, \`ensure_migration_038.sh\`.
EOF

echo "Wrote $OUT"
