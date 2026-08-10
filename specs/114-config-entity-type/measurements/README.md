# SPEC-114 measurements / runbook

## CI (deterministic mock + unit)

```bash
# Unit enforce (G-114-03 / G-114-12)
cargo test -p edgequake-pipeline --lib entity_type_policy

# G-114-15 — dual allowlists + typed edges on mock ingest → graph
cargo test -p edgequake-api --test e2e_spec114_extraction_schema -- --test-threads=1

# G-114-16 — gleaning relation/edge enforce
cargo test -p edgequake-pipeline --test e2e_spec114_gleaning_relations

# Config persistence (existing)
cargo test -p edgequake-api --test e2e_spec114_relation_types
cargo test -p edgequake-api --test e2e_spec114_relation_edges
```

Memory worker harness sets `EDGEQUAKE_VECTOR_BACKEND=legacy_tables` and
`EDGEQUAKE_CHUNK_TEXT_AUTHORITY=kv` (no PgPool for typed indexes). Does **not**
require `MISTRAL_API_KEY` or Ollama.

## Live Mistral extract (opt-in) — G-114-17

Pinned models: chat/extract `mistral-small-latest`, embed `mistral-embed` @ 1024d.

Soft EC matrix: happy, free-form, strict closed-world, permissive, typed-edge, entity-OTHER.

```bash
export MISTRAL_API_KEY=...
# DATABASE_URL from make postgres / /tmp/edgequake-db-url
make spec114-e2e-mistral-extract
```

Underlying cargo:

```bash
cargo test -p edgequake-api --features postgres --test e2e_spec114_mistral_extract \
  -- --ignored --nocapture --test-threads=1
```

Does **not** silently prefer OpenAI when `OPENAI_API_KEY` is set — harness uses
`create_postgres_mistral_app_or_skip()` (same pattern as `spec013-mistral-backend-bg`).

## Live Ollama extract (opt-in) — G-114-19

Pinned models: chat/extract `qwen3.6:35b-a3b`, embed `embeddinggemma:latest`
(fallback `nomic-embed-text`). Extract reasoning effort forced to `none` so
think-heavy Qwen does not stall JSON extract.

```bash
ollama pull qwen3.6:35b-a3b
ollama pull embeddinggemma:latest   # or: ollama pull nomic-embed-text
make spec114-e2e-ollama-extract
```

Combined (skips missing provider with a clear message):

```bash
make spec114-e2e-live-extract
```

## Playwright (G-114-18)

```bash
# UI config cases (mock/live stack via E2E_LIVE_STACK)
cd edgequake_webui && pnpm exec playwright test e2e/spec114-kg-schema.spec.ts

# Optional live ingest smoke skips cleanly without MISTRAL_API_KEY
```
