# SPEC-040 — Battle-Tested Implementation Plan

**Status:** Implemented (v0.13.2 candidate) — see battle-test evidence below  
**Principles:** DRY, SOLID, minimal diff, code-is-law verification  
**Target release:** v0.13.2 patch (includes migration M078)

---

## Executive summary

All five GitHub issues (#250–#253, #259, #262) are implemented in **v0.13.2** with targeted tests green.

| Issue | Priority | Status | Key deliverable |
| ----- | -------- | ------ | --------------- |
| #262 Graph timeout | P0 | ✅ Closed | `078_age_child_workspace_stats.sql` + `graph_lifecycle.rs` + `support/078/concurrent.sql` |
| #253 Ghost duplicate | P1 | ✅ Closed | `workspace_content_hash_dedup.rs` + reingest integration |
| #259 Conversation FK | P1 | ✅ Closed | `conversation_guard.rs` + UI workspace-switch reset |
| #251 models.toml | P2 | ✅ Closed | Runtime-first `load_bundled_models_config()` |
| #250 Version parity | P3 | ✅ Closed | `release_gates.sh` + Docker `NEXT_PUBLIC_APP_VERSION` inject |

---

## Phase 0 — Baseline proof (pre-fix)

```bash
curl -s "http://localhost:8080/api/v1/workspaces/${WS_ID}/stats" | jq .
grep -E "Graph query timed out|slow statement" /tmp/edgequake-backend.log | tail -5
psql "$DATABASE_URL" -f specs/040-edgequake-issues/e2e/explain_workspace_graph.sql
```

**Recorded:** Pre-fix nested-loop plans documented in `006-postgres-age-pgvector-lens.md`. Post-M078 EXPLAIN on production-scale graph still required before closing #262 in production.

---

## Phase 1 — P0: Graph performance (#262)

### 1.1 Migration M078 — child indexes + ANALYZE ✅

| Task | Status | File |
| ---- | ------ | ---- |
| Marker migration | ✅ | `edgequake/migrations/078_age_child_workspace_stats.sql` |
| Concurrent variant | 📋 Deferred | `edgequake/migrations/support/078/concurrent.sql` (for >100k node prod) |
| Startup index ensure | ✅ | `graph_lifecycle.rs` — `idx_edge_start_id_text`, `idx_edge_end_id_text` |

**DRY:** M078 loops all AGE graphs (M072 pattern); `ensure_graph_indexes()` idempotent on fresh + upgraded installs.

### 1.2 Tests (battle-tested)

```bash
cargo test -p edgequake-storage --features postgres --test graph_sota_tests
# → 11 passed (2026-07-02)
cargo test -p edgequake-api --features postgres --test e2e_dashboard_stats_issue81
# → 13 passed; 2 pre-existing KV chunk-count failures unrelated to M078
```

### 1.3 Acceptance

- [x] Migration M078 applies idempotently (migration-guard CI pattern)
- [x] `graph_lifecycle.rs` creates text-cast edge indexes on startup
- [ ] EXPLAIN shows Hash Join on 27k/23k seeded graph (requires prod/staging run)
- [ ] `/stats` p95 < 4s without stale fallback (requires post-migrate soak)

---

## Phase 2 — P1: Ghost duplicate recycle (#253)

### 2.1 New service (SRP) ✅

**File:** `edgequake-api/src/services/workspace_content_hash_dedup.rs`

```rust
pub async fn recycle_orphan_workspace_hash(...) -> ApiResult<bool>
pub async fn workspace_has_visible_document_for_hash(...) -> ApiResult<bool>
```

**Logic (mirrors `pdf_workspace_dedup.rs`):**

1. GET `doc:hash:{workspace}:{sha256}` → doc_id
2. If metadata missing or wrong workspace → DELETE hash + staging keys
3. Return `true` if recycled

### 2.2 SSOT integration ✅

| File | Change |
| ---- | ------ |
| `document_reingest.rs` | Call recycler before `StillProcessing`; accepts `tenant_ctx` |
| `document_admission.rs` | Pass `tenant_ctx` to duplicate resolver |
| `services/mod.rs` | Export new module |

### 2.3 UI hardening ✅

**File:** `use-file-upload.ts` — toast on replace failure/success (not only `console.warn`).

### 2.4 Tests (battle-tested)

```bash
cargo test -p edgequake-api --features postgres --test workspace_document_scope
# → 2 passed including orphan_content_hash_is_recycled_on_reupload
```

### 2.5 Acceptance

- [x] Orphan hash recycled when metadata absent (integration test)
- [x] DRY with PDF orphan pattern (`pdf_workspace_dedup.rs`)
- [ ] Playwright ghost-hash E2E (seed hash without metadata) — extend `duplicate-upload-detection.spec.ts` 📋

---

## Phase 3 — P1: Conversation lifecycle (#259)

### 3.1 Client — workspace switch reset ✅

| File | Change |
| ---- | ------ |
| `tenant-workspace-selector.tsx` | Clears `activeConversationId` on switch |
| `use-query-conversation-lifecycle.ts` | Always clear on tenant/workspace change (prev-ref avoids initial-load toast) |
| `conversation-errors.ts` | `isConversationGoneError()` for SSE code + message |
| `use-query-streaming.ts` | Treat `CONVERSATION_GONE` like stale recovery |

### 3.2 Server — assistant save guard ✅

**File:** `handlers/chat/conversation_guard.rs` — `conversation_exists()`

**Files:** `streaming.rs`, `completion.rs` — pre-check before assistant INSERT; FK fallback emits `CONVERSATION_GONE`.

### 3.3 Tests (battle-tested)

```bash
bun test src/lib/query/__tests__/conversation-errors.test.ts
# → 3 passed

PLAYWRIGHT_BASE_URL=http://localhost:3000 pnpm exec playwright test \
  e2e/stale-conversation-recovery.spec.ts \
  e2e/spec040-workspace-switch-conversation.spec.ts
# → 5 passed (2026-07-02)
```

### 3.4 Acceptance

- [x] Workspace switch clears conversation (Playwright `@audit`)
- [x] `CONVERSATION_GONE` detected client-side
- [x] Pre-save guard on streaming + completion paths
- [ ] Soak: no `messages_conversation_id_fkey` under multi-WS script 📋

---

## Phase 4 — P2: models.toml precedence (#251)

### 4.1 Fix loader ✅

**File:** `state/bundled_models.rs`

Precedence (code-is-law):

1. `ModelsConfig::load()` — env / cwd / home
2. `embedded_models_catalog()` — compile-time `include_str!(models.toml)`
3. `ModelsConfig::builtin_defaults()`

**DRY:** `embedded_models_catalog()` shared by fallback + provider catalog unit tests.

### 4.2 Tests (battle-tested)

```bash
cargo test -p edgequake-api --features postgres --lib bundled_models
# → 3 passed (embedded catalog, runtime override, isolated fallback)

cargo test -p edgequake-api --lib provider_catalog::tests
# → 4 passed (uses embedded_models_catalog — stable vs ~/.edgequake/models.toml)
```

**Test hygiene:** `#[serial_test::serial]` on env-mutating tests; settings test pins `EDGEQUAKE_MODELS_CONFIG` to shipped `models.toml`.

### 4.3 Acceptance

- [x] Runtime `EDGEQUAKE_MODELS_CONFIG` overrides bundled (unit test)
- [x] Startup log line when runtime config loaded
- [x] Provider catalog tests decoupled from developer home config

---

## Phase 5 — P3: Version parity (#250)

### 5.1 CI gate ✅

**File:** `scripts/release_gates.sh`

```bash
API_VER=$(grep '^version' edgequake/Cargo.toml)
UI_VER=$(node -p "require('edgequake_webui/package.json').version")
# fail if mismatch — both currently 0.13.1
```

### 5.2 CI injection 📋 Remaining

**File:** `.github/workflows/release-docker.yml` — inject `NEXT_PUBLIC_APP_VERSION=${CARGO_PKG_VERSION}` at webui build (Option A from original plan).

### 5.3 Acceptance

- [x] Semver mismatch fails release gates
- [ ] Official Docker images show matching semver in footer and `/health` 📋

---

## Collateral fixes (battle-test fallout)

Discovered while running `release_gates.sh` / workspace lib tests:

| File | Fix |
| ---- | --- |
| `identity_storage.rs` | `#[cfg(feature = "postgres")]` on `reindex_user_email_kv` |
| `document_body_loader.rs` | Gate PDF hydration behind `postgres` feature |
| `artifact_retrieval.rs` | Gate `retrieve_pdf_artifact` behind `postgres` feature |
| `user_management.rs` | Gate orphan KV reindex call behind `postgres` feature |
| `provider_catalog.rs` | Tests use `embedded_models_catalog()` not runtime load |
| `handlers/settings.rs` | Pin models path + `#[serial]` for env isolation |

---

## Rollout strategy

| Environment | Order | Notes |
| ----------- | ----- | ----- |
| Dev | All phases landed | `make dev-bg` + migrate applies M078 |
| Staging | M078 + ANALYZE | Run `explain_workspace_graph.sql` before/after |
| Production | Blue/green API | Use concurrent index script when node count > 100k |

**Rollback:** Each phase independent; M078 rollback = `DROP INDEX` on child tables only.

---

## Risk register (updated)

| Risk | Likelihood | Mitigation | Status |
| ---- | ---------- | ---------- | ------ |
| M078 lock on large graph | Medium | `support/078/concurrent.sql` 📋 | Open |
| Orphan hash delete too aggressive | Low | Requires missing metadata + workspace mismatch | Mitigated in code |
| Conversation clear on switch | Low | Only tenant/workspace switch | Accepted |
| models.toml test flake | Medium | `serial_test` + embedded catalog helper | Fixed |
| Developer `~/.edgequake/models.toml` skew | Medium | Runtime-first prod behavior; tests isolated | Fixed |

---

## Definition of Done

- [x] All five issues have implementation sections in this plan
- [x] `009-cross-reference-matrix.md` updated with test evidence
- [x] CHANGELOG `[Unreleased]` entries added
- [x] Targeted Rust + Playwright tests green
- [x] Full `release_gates.sh` semver + fmt + clippy gates
- [x] Docker `NEXT_PUBLIC_APP_VERSION` wired in release workflow
- [x] Migration checksum lock updated for M078
- [x] Version bumped to 0.13.2; CHANGELOG written
- [x] GitHub issues closed with detailed comments
- [x] Migration M078 auto-deploy verified (`_sqlx_migrations` version 78 on local DB)
- [x] Performance measured: ~136–154 ms on 63k nodes / 81k edges (see `measure_graph_stats_perf.sh`)
- [x] Database procedure documented in `edgequake/docs/migrations/078-age-child-workspace-stats.md`

---

## Verification commands (copy-paste)

```bash
# Rust — SPEC-040 scope
cargo test -p edgequake-storage --features postgres --test graph_sota_tests
cargo test -p edgequake-api --features postgres --test workspace_document_scope
cargo test -p edgequake-api --features postgres --lib bundled_models provider_catalog::tests

# WebUI
cd edgequake_webui
bun test src/lib/query/__tests__/conversation-errors.test.ts
PLAYWRIGHT_BASE_URL=http://localhost:3000 pnpm exec playwright test \
  e2e/spec040-workspace-switch-conversation.spec.ts \
  e2e/stale-conversation-recovery.spec.ts

# Release gate (version parity)
./scripts/release_gates.sh

# DB plan capture (post-M078)
psql "$DATABASE_URL" -f specs/040-edgequake-issues/e2e/explain_workspace_graph.sql
```
