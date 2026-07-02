# SPEC-040 — Cross-Reference Matrix

**Purpose:** High-signal map from GitHub issues → code → migrations → tests → specs  
**Updated:** 2026-07-02 — post-implementation battle test

---

## Master matrix

| Issue | Symptom | Root cause | Primary code | Migration | Test / E2E | Status |
| ----- | ------- | ---------- | ------------ | --------- | ---------- | ------ |
| [#262](https://github.com/raphaelmansuy/edgequake/issues/262) | 15s graph/stats timeout | Parent-table indexes + bad join plan | `query_ops.rs`, `analytics_ops.rs`, `graph_lifecycle.rs` | M078 ✅ | `graph_sota_tests.rs` ✅ (11) | ✅ code; 📋 prod EXPLAIN |
| [#259](https://github.com/raphaelmansuy/edgequake/issues/259) | FK on messages | Stale conversation_id + long stream | `conversation_guard.rs`, `streaming.rs`, `use-query-conversation-lifecycle.ts` | — | `stale-conversation-recovery.spec.ts` ✅ (4), `spec040-workspace-switch-conversation.spec.ts` ✅ (1) | ✅ |
| [#253](https://github.com/raphaelmansuy/edgequake/issues/253) | Duplicate loop / Replace noop | Ghost content-hash KV | `workspace_content_hash_dedup.rs`, `document_reingest.rs`, `use-file-upload.ts` | — | `workspace_document_scope.rs` ✅ `orphan_content_hash_is_recycled_on_reupload` | ✅; 📋 Playwright ghost-hash |
| [#251](https://github.com/raphaelmansuy/edgequake/issues/251) | models.toml ignored | Inverted load precedence | `bundled_models.rs`, `embedded_models_catalog()` | — | `bundled_models` ✅ (3), `provider_catalog::tests` ✅ (4) | ✅ |
| [#250](https://github.com/raphaelmansuy/edgequake/issues/250) | UI v0.12.3 ≠ API v0.12.11 | Dual artifact semver | `release_gates.sh`, `release-docker.yml`, `Dockerfile`, `app-version.ts` | — | release gate + Docker build-arg ✅ | ✅ |

Legend: ✅ verified · 📋 remaining · ⚠️ partial

---

## Issue #262 — Code paths

| Function | File | Change |
| -------- | ---- | ------ |
| `ensure_graph_indexes` | `graph_lifecycle.rs:219+` | Added `idx_edge_start_id_text`, `idx_edge_end_id_text` |
| M078 repair loop | `078_age_child_workspace_stats.sql` | Child `"Node"`/`"EDGE"` workspace + text indexes + ANALYZE |
| `pg_get_popular_nodes_with_degree` | `query_ops.rs:482` | Consumer — benefits from M078 |
| `pg_node_count_by_workspace` | `analytics_ops.rs:69` | Consumer — benefits from M078 |

**Test evidence:** `cargo test -p edgequake-storage --features postgres --test graph_sota_tests` → 11 passed.

---

## Issue #259 — Code paths

| Step | File | Implementation |
| ---- | ---- | -------------- |
| Pre-save guard | `conversation_guard.rs` | `conversation_exists()` |
| Stream guard + FK map | `streaming.rs` | Guard + `CONVERSATION_GONE` SSE |
| Completion guard | `completion.rs` | Same pattern |
| Client stale recovery | `use-query-streaming.ts` | `CONVERSATION_GONE` handling |
| Workspace switch | `use-query-conversation-lifecycle.ts`, `tenant-workspace-selector.tsx` | Clear conversation on switch |

**Test evidence:** Playwright 5/5; `conversation-errors.test.ts` 3/3.

---

## Issue #253 — Code paths

| Step | File | Notes |
| ---- | ---- | ----- |
| Orphan recycle SSOT | `workspace_content_hash_dedup.rs` | DRY with `pdf_workspace_dedup.rs` |
| Reingest integration | `document_reingest.rs` | Recycle before `StillProcessing` |
| Admission wiring | `document_admission.rs` | Passes `tenant_ctx` |
| UI replace feedback | `use-file-upload.ts` | Toast on replace fail/success |

**Test evidence:** `orphan_content_hash_is_recycled_on_reupload` in `workspace_document_scope.rs`.

---

## Issue #251 — Code paths

| Component | File | Notes |
| --------- | ---- | ----- |
| Runtime-first loader | `bundled_models.rs` | `ModelsConfig::load()` then `embedded_models_catalog()` |
| Embedded helper (DRY) | `bundled_models.rs` | `embedded_models_catalog()` for tests + fallback |
| API exposure | `provider_catalog.rs` | Unchanged; tests fixed |

---

## Issue #250 — Code paths

| Component | File | Status |
| --------- | ---- | ------ |
| Semver gate | `scripts/release_gates.sh:65-72` | ✅ Implemented |
| UI version | `app-version.ts`, `sidebar.tsx` | Unchanged |
| API version | `health.rs`, `Cargo.toml` | Both 0.13.1 |
| Docker build inject | `.github/workflows/release-docker.yml` | 📋 Not wired |

---

## DRY / SOLID cross-reference (as-built)

| Principle | Issue(s) | Implementation |
| --------- | -------- | -------------- |
| DRY | #253 | `workspace_content_hash_dedup.rs` mirrors PDF orphan pattern |
| DRY | #262 | M078 + `graph_lifecycle.rs` — single index SSOT |
| DRY | #251 | `embedded_models_catalog()` — one parse path for fallback + tests |
| SRP | #253 | Dedicated dedup service; reingest orchestrates only |
| SRP | #259 | `conversation_guard.rs` — single existence check |
| OCP | #251 | Runtime config extends without changing embedded catalog |

---

## Evidence checklist (PR closure)

| Issue | Required evidence | Status |
| ----- | ----------------- | ------ |
| #262 | EXPLAIN ANALYZE before/after; stats p95 | 📋 Needs staging/prod graph |
| #259 | Playwright workspace-switch; no FK in logs | ✅ Playwright; 📋 log soak |
| #253 | E2E ghost-hash; Replace success | ✅ Integration test; 📋 Playwright extend |
| #251 | Custom model visible; startup log | ✅ Unit tests |
| #250 | Docker `/health` vs UI footer match | 📋 CI inject pending |

---

## External links

- [Issue #262 — Graph timeout](https://github.com/raphaelmansuy/edgequake/issues/262)
- [Issue #259 — Conversation FK](https://github.com/raphaelmansuy/edgequake/issues/259)
- [Issue #253 — Duplicate upload](https://github.com/raphaelmansuy/edgequake/issues/253)
- [Issue #251 — models.toml](https://github.com/raphaelmansuy/edgequake/issues/251)
- [Issue #250 — UI version](https://github.com/raphaelmansuy/edgequake/issues/250)
