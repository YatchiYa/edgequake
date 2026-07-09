# SPEC-045 — Quick Fix Runbook (Operators)

**Audience:** Production operators after EdgeQuake version migration  
**Time budget:** 15 minutes to triage; 30–60 minutes to remediate

---

## Step 0 — Confirm the failure mode (2 min)

```bash
# API health
curl -s http://localhost:8080/health | python3 -m json.tool

# Readiness (503 = traffic blocked)
curl -s -o /dev/null -w "%{http_code}" http://localhost:8080/ready
```

| `/ready` | Meaning | Jump to |
| -------- | ------- | ------- |
| 200 | API accepting uploads | [Step 1](#step-1--inspect-failed-document) |
| 503 | Migration degraded | [Step 2](#step-2--fix-readiness-block) |

---

## Step 1 — Inspect failed document

```bash
# List failed documents (replace workspace/tenant as needed)
curl -s "http://localhost:8080/api/v1/documents?status=failed" \
  -H "X-API-Key: $EDGEQUAKE_MASTER_API_KEY" | python3 -m json.tool

# Document detail — check failure_class
curl -s "http://localhost:8080/api/v1/documents/{DOCUMENT_ID}" \
  -H "X-API-Key: $EDGEQUAKE_MASTER_API_KEY" | python3 -m json.tool \
  | jq '{status, failure_class: .metadata.failure_class, action: .metadata.recommended_action, error: .metadata.error_message}'
```

### Decision tree by `failure_class`

| `failure_class` | Immediate action |
| --------------- | ---------------- |
| `provider_unavailable` | [Fix provider](#fix-llm-provider) → reprocess |
| `embedding_limit` | Reprocess; if repeats, split doc or switch embedding provider |
| `timeout_phase_convert` | Reprocess with EdgeParse backend |
| `timeout_phase_extract` | Reprocess; verify Ollama model loaded |
| `circuit_breaker` | Wait 5 min → reprocess EdgeParse |
| `document_too_large` | Split PDF or raise size limit |
| `unknown` + merge error in message | [Fix graph indexes](#fix-graph-indexes) → reprocess Full |
| (missing) + `processing` > 10 min | [Recover stuck](#recover-stuck-documents) |

---

## Step 2 — Fix readiness block

### M038 — Missing source_ids indexes

```bash
# Check health
curl -s http://localhost:8080/health | jq '.schema.source_ids_indexes'

# Apply indexes (large graphs — use CONCURRENTLY)
./edgequake/scripts/migrations/apply_038.sh --apply --concurrent --yes

# Verify
curl -s -o /dev/null -w "%{http_code}" http://localhost:8080/ready  # expect 200
```

### M042 — pgvector < 0.8

```bash
# Check version
psql "$DATABASE_URL" -c "SELECT extversion FROM pg_extension WHERE extname = 'vector';"

# If < 0.8.0: use EdgeQuake postgres image with bundled pgvector
make postgres-image-build
make db-stop && make db-start
# Restart API
make backend-bg
```

---

## Fix LLM provider

```bash
# Ollama
curl http://localhost:11434/api/tags
ollama serve &
ollama pull gemma3:latest

# Verify API sees provider
curl -s http://localhost:8080/health | jq '.llm_provider_name, .components'
```

For OpenAI:

```bash
export OPENAI_API_KEY="sk-..."
export EDGEQUAKE_LLM_PROVIDER=openai
make stop && make dev-bg
```

---

## Fix graph indexes

```bash
# M038 (required for large upgraded graphs)
./edgequake/scripts/migrations/apply_038.sh --apply --concurrent --yes

# Optional: M046 perf indexes (warn-only, helps scoped merge)
# Runs automatically on bootstrap — check logs:
grep migration_046 /tmp/edgequake-backend.log
```

---

## Recover stuck documents

```bash
# Auto-recover docs processing > 10 minutes (default threshold)
curl -X POST "http://localhost:8080/api/v1/documents/recover-stuck" \
  -H "Content-Type: application/json" \
  -H "X-API-Key: $EDGEQUAKE_MASTER_API_KEY" \
  -d '{"threshold_minutes": 10}'
```

---

## Reprocess failed documents

```bash
# Standard reprocess (cleans graph + requeues)
curl -X POST "http://localhost:8080/api/v1/documents/{DOCUMENT_ID}/reprocess" \
  -H "Content-Type: application/json" \
  -H "X-API-Key: $EDGEQUAKE_MASTER_API_KEY" \
  -d '{"mode": "Full"}'

# Bulk: reprocess all failed in workspace (if endpoint available)
curl -X POST "http://localhost:8080/api/v1/documents/reprocess-failed" \
  -H "X-API-Key: $EDGEQUAKE_MASTER_API_KEY"
```

**PDF with empty markdown:** always use `"mode": "Full"`.

---

## Post-migration smoke (5 min)

```bash
export DATABASE_URL=postgres://edgequake:edgequake@localhost/edgequake
./specs/045-fix-ingestion-errors/e2e/run_ingestion_health_proof.sh
```

---

## Log grep cheatsheet

```bash
# Merge failures
grep -E "merge_entities_batch_global|merge_relationships_batch_global|merge error" \
  /tmp/edgequake-backend.log | tail -20

# Compensation / quarantine
grep -i quarantine /tmp/edgequake-backend.log | tail -10

# Migration degraded
grep -E "migration_038_degraded|migration_042_degraded|ready_for_traffic" \
  /tmp/edgequake-backend.log | tail -10

# Embedding limits
grep -E "Too many tokens|Too many inputs|Embedding error" \
  /tmp/edgequake-backend.log | tail -10

# Provider
grep -E "11434|provider.*unavailable|Network error" \
  /tmp/edgequake-backend.log | tail -10
```

---

## Escalation matrix

| Symptom persists after runbook | Escalate with |
| ------------------------------ | ------------- |
| Merge errors after M038 applied | `document_id`, merger WARN lines, graph node count |
| `/ready` 503 after M042 fix | pgvector extversion, migration_042 log excerpt |
| Embedding 400 on all docs | embedding provider config, sample chunk count |
| List shows 0 docs | wsdoc index SQL (see e2e health SQL) |
| Auth 401 on upload | bootstrap admin status, `EDGEQUAKE_AUTH_ENABLED` |

---

## What NOT to do

| Action | Why |
| ------ | --- |
| Reprocess without checking graph cleanup | Duplicate entities (OODA-08) |
| `sqlx migrate revert` on production | Data loss risk |
| Force `processing` → `completed` in KV | UI lie; query returns empty |
| Skip M038 on large graphs | Merge timeouts continue |
| Run `make dev` defaults in production | Auth disabled (`EDGEQUAKE_DEV_MODE`) |
