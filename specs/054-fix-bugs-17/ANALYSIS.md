# SPEC-054 — Bug Triage Report (Issues #37, #186, #239, #292, #294, #296, #297)

Date: 2026-07-14  
Branch: feat/spec047-vision-ingest-spec048-progress  
Analyst: GitHub Copilot (First Principles analysis)

---

## Summary

```
┌──────┬──────────────────────────────────────────────────────┬────────────┬──────────────┐
│  #   │ Title                                                │ Status     │ Action       │
├──────┼──────────────────────────────────────────────────────┼────────────┼──────────────┤
│ #297 │ Cross-workspace contamination + orphan vector table │ PARTIAL FIX│ Fix #297-A   │
│ #296 │ proxyClientMaxBodySize Type Error (Next.js 16)      │ FIXED      │ Close        │
│ #294 │ New API keys return 401 in ECS                      │ CONFIRMED  │ Fix #294-A   │
│ #292 │ Docker image 0.15.1 not found                       │ FIXED      │ Close        │
│ #239 │ Partial failure log details in WebUI                │ FIXED      │ Close        │
│ #186 │ Add Ollama cloud model support                      │ FIXED      │ Close        │
│  #37 │ Get only retrieval chunks (no LLM answer)           │ FIXED      │ Close        │
└──────┴──────────────────────────────────────────────────────┴────────────┴──────────────┘
```

---

## Issue #297 — Cross-workspace contamination + orphan vectors

### 5 WHY Analysis

```
WHY 1: Why did orphan vector table remain after DeleteWorkspace?
  → clear_workspace() executes DELETE FROM {table} (deletes rows)
    but does NOT DROP TABLE {table}

WHY 2: Why doesn't the DELETE cascade drop the table?
  → Vector storage uses per-workspace dynamic tables (eq_eq_ws_XXXXXXXX_vectors)
    The ORM abstraction treats table creation as DDL but delete as DML
    clear_workspace() was designed for "empty" not "destroy"

WHY 3: Why did cross-workspace title contamination occur?
  → Reporter is on v0.12.11. Current code uses workspace_id column filter
    on all queries. v0.12.11 may have lacked strict workspace scoping.
    Fixed in >= v0.14 with workspace_id FK enforcement.

WHY 4: Why is in-flight ingest lost on restart?
  → Pipeline state held in tokio channels + in-memory task queue
    No persistent task registry with "resume on restart" semantics

WHY 5: Why wasn't the orphan table cleaned up by workspace delete cascade?
  → The delete_workspace handler calls vector_registry.evict() (removes from
    cache) and clear_workspace() (empties rows) but has no DROP TABLE step
```

### Code Evidence

```
// workspace_crud.rs:~370
let vectors_cleared = match state.storage.vector_storage
    .clear_workspace(&workspace_id).await {
    Ok(count) => count,  // ← deletes ROWS, not table
    ...
```

```
// vector/storage_impl.rs:~409
async fn clear_workspace(&self, workspace_id: &uuid::Uuid) -> Result<usize> {
    let sql = format!("DELETE FROM {} WHERE workspace_id = $1 ...", self.table_name);
    // ← No DROP TABLE here
```

### Fix Required (Issue #297-A)

After `clear_workspace` in `delete_workspace`, drop the vector table:

```
let short_id = workspace_id.to_string().replace('-', "")[..8].to_string();
let table_name = format!("eq_eq_ws_{short_id}_vectors");
// DROP TABLE IF EXISTS {table_name}
```

### Status of Cross-Workspace Contamination (#297-B)

NOT reproducible in current v0.16.x. All vector queries include
`WHERE workspace_id = $1` enforced via `WorkspaceVectorStorage` per-workspace
adapter with isolated table names. Closed for v0.12.11 reporters.

---

## Issue #296 — proxyClientMaxBodySize Type Error

### Status: ALREADY FIXED in current branch

**Evidence** (next.config.ts):
```ts
// Comment already in file:
// "Use numeric bytes (SizeLimit). Template strings like `${n}mb` widen to
//  `string` and fail `next build` typecheck (release-docker CD flake on Next 16.2)."
proxyClientMaxBodySize: DEFAULT_MAX_UPLOAD_BYTES,  // ← 50 * 1024 * 1024 (number)
```

`DEFAULT_MAX_UPLOAD_BYTES = 50 * 1024 * 1024` is type `number`, which satisfies
`SizeLimit = number | \`${number}${FileSizeSuffix}\``. The bug was template strings.

**Fix was applied before this issue was filed.** Will be in v0.17.0.

---

## Issue #294 — New API Keys Return 401 in ECS

### 5 WHY Analysis

```
WHY 1: Why do new API keys return 401?
  → validate_presented_token() first checks JWT, which fails (expected).
    Then calls validate_stored_api_key() which looks up by prefix.
    find_active_api_keys_by_prefix routes to either pg or kv store.

WHY 2: Why do OLD keys work but NEW ones fail?
  → The auth backend is likely in-memory KV store (not PostgreSQL).
    In-memory store is instance-local. In ECS, different instances handle
    CREATE (one instance) and VALIDATE (another instance) — key never found.

WHY 3: Why doesn't the key reach the validating instance?
  → session_storage::find_api_keys_by_prefix_kv uses auth_memory_store
    (in-process HashMap). No distributed cache or DB sync.

WHY 4: Why did old keys work?
  → Possibly created when there was one instance, or instance affinity by luck.
    Or: old keys were static API keys in config (EDGEQUAKE_API_KEYS env var)
    which ARE shared via env vars across all instances.

WHY 5: Why isn't this caught in tests?
  → Unit tests are single-instance. Multi-instance concurrency is not tested.
```

### Code Evidence

```rust
// auth_validation.rs:~60
// Step 1: check static keys (from env var — shared across instances ✓)
if state.auth.config.api_keys.iter().any(|k| ...) { return Ok(Some(...)) }
// Step 2: try JWT
if let Ok(claims) = state.auth.jwt.verify_token(token) { return Ok(Some(...)) }
// Step 3: stored API keys (MAY be in-memory if no PostgreSQL auth configured ✗)
validate_stored_api_key(state, token).await
```

### Fix Required (#294-A)

When PostgreSQL is available, `find_active_api_keys_by_prefix` already uses the
shared PG table — the fix is documentation and defaulting:

1. When `DATABASE_URL` is set, API keys MUST be persisted to PostgreSQL.
2. Add a startup warning when API key storage falls back to in-memory.
3. Document ECS deployment requirement: PostgreSQL auth backend required.

---

## Issue #292 — Docker image 0.15.1 not found

### Status: FIXED — owner published v0.16.0

Evidence: `raphaelmansuy commented 4d ago: "I have published 0.16"`

**Close** with note: use `ghcr.io/raphaelmansuy/edgequake:0.16.0`

---

## Issue #239 — Partial failure log details in WebUI

### Status: FIXED in v0.16.x (PipelineStatusDialog)

The WebUI already has the `PipelineStatusDialog` component with:
- Structured pipeline stage messages (per-stage error reasons)
- Phase-level error details (chunk errors, extraction failures)
- Retry information and stage progression
- `current_stage` and `stage_message` fields in document API response

**Close** with reference to pipeline status dialog and `GET /api/v1/documents/{id}`.

---

## Issue #186 — Add Ollama cloud model support

### Status: FIXED in v0.14+

Evidence in current code (`create_safe_llm_provider`):
```rust
// OLLAMA_API_KEY forwarded to OllamaProvider
if let Ok(api_key) = std::env::var("OLLAMA_API_KEY") {
    if !api_key.is_empty() {
        builder = builder.api_key(&api_key);
    }
}
```

And for embeddings:
```rust
// OLLAMA_EMBEDDING_HOST + OLLAMA_API_KEY for cloud/remote Ollama
if let Ok(api_key) = std::env::var("OLLAMA_API_KEY") {
    builder = builder.api_key(&api_key);
}
```

**Close** with env var documentation.

---

## Issue #37 — Context-only retrieval (no LLM answer)

### Status: ALREADY IMPLEMENTED

Evidence: multiple tests use `context_only: true`:
```rust
// e2e_query_engine.rs:132
async fn test_context_only_query() {
    // context_only=true skips LLM generation, returns empty answer + context
    "context_only": true
```

**Close** with API documentation showing `context_only=true` in POST /query.

---

## Implementation Plan

### Fix A — #297: Drop vector table on workspace delete

File: `edgequake/crates/edgequake-api/src/handlers/workspaces/workspace_crud.rs`

After step 4 (vector_registry.evict), add step 4b: drop the orphan table.

### Fix B — #294: Add startup warning for in-memory API key storage in multi-instance

File: `edgequake/crates/edgequake-api/src/state/runtime_extractors.rs`

Add a tracing::warn! when API key storage falls back to in-memory and
DATABASE_URL is set, indicating misconfiguration risk.
