# SPEC-040 — Code Is Law

**Lens:** Code is law — every claim maps to a repository path  
**Baseline:** `main` @ v0.13.1 (2026-07-02)

---

## Issue #262 — Graph timeout

| Claim | Law (file:line) | Status |
| ----- | --------------- | ------ |
| 15s materialization timeout | `edgequake-api/src/services/graph_materialization.rs:33-74` | ✅ By design |
| Filter-first popular nodes SQL | `edgequake-storage/.../query_ops.rs:504-531` | ✅ Fixed pattern |
| Still queries `_ag_label_vertex` | `query_ops.rs:513`, `analytics_ops.rs:138` | ⚠️ Union parent — child indexes help via inheritance |
| Child `"Node"` workspace index | `graph_lifecycle.rs:170-177` | ✅ Created at `pg_initialize` |
| Parent indexes dropped | `migrations/070_consolidate_age_indexes.sql:88-100` | ✅ M070 |
| Edge text-cast indexes | `migrations/072_edge_text_cast_indexes.sql` | ✅ M072 |
| Workspace stats calls graph counts | `edgequake-api/.../workspaces/stats.rs:239-251` | ✅ Hot path |
| Stats 4s timeout + stale cache | `stats.rs:84-119` | ✅ P-G13 |

**Gap:** No migration **guarantees** child-table `idx_node_workspace_id` on graphs created before `graph_lifecycle` fix; upgrade path must re-run `ensure_graph_indexes()` or add M078.

---

## Issue #259 — Conversation FK

| Claim | Law | Status |
| ----- | --- | ------ |
| FK definition | `migrations/001_init_database.sql:354` | ✅ |
| User msg before stream | `handlers/chat/streaming.rs:157-169` | ✅ |
| Assistant msg after stream | `streaming.rs:535-547` | ✅ |
| Error string “Failed to save response” | `streaming.rs:644` | ✅ Matches issue |
| Stale ID recovery on 404 | `use-query-streaming.ts:218-228` | ✅ Partial |
| Workspace switch lifecycle | `use-query-conversation-lifecycle.ts:110-123` | ⚠️ Conditional clear |
| Workspace switch in selector | `tenant-workspace-selector.tsx:276-291` | ❌ No conversation reset |
| Chat verifies conversation exists | `streaming.rs:127-137` | ✅ At stream start only |

**Gap:** No re-validation before assistant `create_message`; no workspace-scoped conversation filter on submit.

---

## Issue #253 — Duplicate upload

| Claim | Law | Status |
| ----- | --- | ------ |
| Duplicate dialog UI | `duplicate-upload-dialog.tsx` | ✅ |
| Collect `duplicate_of` | `use-file-upload.ts:354-361` | ✅ |
| Replace: PDF force_reindex | `use-file-upload.ts:552-562` | ✅ |
| Replace: markdown delete + re-upload | `use-file-upload.ts:584-594` | ⚠️ Race / ghost hash |
| Admission SSOT | `document_admission.rs:102-134` | ✅ |
| Reingest delete | `document_reingest.rs:52-149` | ⚠️ Fails if metadata missing |
| PDF orphan recycle | `pdf_workspace_dedup.rs:62-76`, `upload.rs:410-421` | ✅ PDF only |
| FIX-WORKSPACE-DUP changelog | `CHANGELOG.md:82` | ✅ v0.12.11 |

**Gap:** No `recycle_orphan_workspace_hash()`; `deleteDocument` 404 leaves hash key → infinite duplicate loop.

---

## Issue #251 — models.toml

| Claim | Law | Status |
| ----- | --- | ------ |
| Embedded catalog | `bundled_models.rs:17-20` | ❌ Always used first |
| Documented priority | `edgequake/models.toml:6-10` | ❌ Contradicts code |
| API catalog builder | `provider_catalog.rs` | Uses `load_bundled_models_config()` |
| Query runtime models | `state/query_runtime.rs:53` | Test default only |

**Required one-line fix:**

```rust
// bundled_models.rs — runtime FIRST, embed LAST
match ModelsConfig::load() {
    Ok(cfg) => { tracing::info!("Loaded models catalog from runtime config"); cfg }
    Err(_) => ModelsConfig::from_toml(BUNDLED_MODELS).expect("bundled models.toml must parse"),
}
```

---

## Issue #250 — Version mismatch

| Claim | Law | Status |
| ----- | --- | ------ |
| UI version source | `edgequake_webui/package.json` → `app-version.ts` | ✅ 0.13.1 |
| Sidebar label | `sidebar.tsx:225`, `en.json:34` | ✅ “UI v{{version}}” |
| API version | `health.rs:150` `CARGO_PKG_VERSION` | ✅ |
| Header API label | `header.tsx:114`, `en.json:103` | ✅ “API v{{version}}” |
| Docker skew documented | `specs/019-0-12-7-control/e2e/001-option1-install-proof.md:74` | ⚠️ Known |

**Gap:** No build pipeline injects API semver into `NEXT_PUBLIC_APP_VERSION` for coupled releases.

---

## Shared modules (DRY touchpoints)

| Module | Issues | Role |
| ------ | ------ | ---- |
| `document_admission.rs` | #253 | Upload SSOT |
| `document_reingest.rs` | #253 | Hash resolution |
| `pdf_workspace_dedup.rs` | #253 | Orphan pattern to generalize |
| `graph_lifecycle.rs` | #262 | Index bootstrap SSOT |
| `graph_materialization.rs` | #262, #259 | Timeout + admission |
| `bundled_models.rs` | #251 | Catalog loader |
| `use-query-conversation-lifecycle.ts` | #259 | Client isolation |

---

## Test coverage map

| Issue | Existing test | Gap |
| ----- | ------------- | --- |
| #262 | `graph_sota_tests.rs`, `e2e_graph_performance.rs` | No CI gate on EXPLAIN plan |
| #259 | `stale-conversation-recovery.spec.ts` | No workspace-switch + submit race |
| #253 | `duplicate-upload-detection.spec.ts`, `workspace-duplicate-scope` | No ghost-hash scenario |
| #251 | `bundled_models_config_parses...` | No runtime override test |
| #250 | `app-version.test.ts` | No API/UI parity E2E |
