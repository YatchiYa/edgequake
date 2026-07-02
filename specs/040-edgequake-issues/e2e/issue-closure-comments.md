# SPEC-040 — GitHub issue closure comments

Use with: `gh issue close <N> --comment "$(cat ...)"`

---

## #262 — Graph performance timeout

**Fixed in v0.13.2** (SPEC-040)

### Root cause
Legacy migration 014 created AGE indexes on inheritance parent tables (`_ag_label_vertex` / `_ag_label_edge`) while query paths scan child label tables (`"Node"` / `"EDGE"`). The planner fell back to nested-loop joins on large graphs, hitting the 15s Tokio graph stream timeout.

### Fix
1. **Migration M078** (`078_age_child_workspace_stats.sql`) — creates child-table workspace + text-cast edge indexes and runs `ANALYZE` on all AGE graphs.
2. **`graph_lifecycle.rs`** — ensures `idx_edge_start_id_text` / `idx_edge_end_id_text` on fresh installs.
3. **Ops script** — `migrations/support/078/concurrent.sql` for production graphs >100k nodes.

### Verification
- `cargo test -p edgequake-storage --features postgres --test graph_sota_tests` → 11 passed
- Post-upgrade: `psql -f specs/040-edgequake-issues/e2e/explain_workspace_graph.sql` — expect Hash Join / index scans on child tables

### References
- `specs/040-edgequake-issues/006-postgres-age-pgvector-lens.md`
- `specs/040-edgequake-issues/008-implementation-plan.md` Phase 1

---

## #259 — Conversation FK on multi-workspace query

**Fixed in v0.13.2** (SPEC-040)

### Root cause
Stale `conversation_id` persisted in UI localStorage after workspace/tenant switch. Long-running streams attempted assistant message INSERT against a conversation that no longer existed → `messages_conversation_id_fkey` violation.

### Fix
1. **Server:** `conversation_guard.rs` — existence check before assistant save in `streaming.rs` and `completion.rs`; emits `CONVERSATION_GONE` SSE code on FK miss.
2. **Client:** `use-query-conversation-lifecycle.ts` + `tenant-workspace-selector.tsx` — clear active conversation on workspace/tenant switch.
3. **Client:** `conversation-errors.ts` / `use-query-streaming.ts` — recover from `CONVERSATION_GONE`.

### Verification
- Playwright: `stale-conversation-recovery.spec.ts` (4 tests) + `spec040-workspace-switch-conversation.spec.ts` (1 test) — all passed
- Unit: `conversation-errors.test.ts` — 3 passed

---

## #253 — Upload duplicate / ghost hash loop

**Fixed in v0.13.2** (SPEC-040)

### Root cause
Orphan `doc:hash:{workspace}:{sha256}` KV keys remained after document metadata was deleted or never written. Re-upload hit duplicate detection but Replace could not resolve a visible document → infinite duplicate dialog.

### Fix
1. **`workspace_content_hash_dedup.rs`** — `recycle_orphan_workspace_hash()` deletes hash keys when metadata is missing or workspace-scoped mismatch (DRY with `pdf_workspace_dedup.rs`).
2. **`document_reingest.rs`** — recycles orphan before returning `StillProcessing`.
3. **`use-file-upload.ts`** — user-visible toast on replace success/failure.

### Verification
- `cargo test -p edgequake-api --features postgres --test workspace_document_scope` — `orphan_content_hash_is_recycled_on_reupload` passed

---

## #251 — models.toml not overridable at runtime

**Fixed in v0.13.2** (SPEC-040)

### Root cause
`load_bundled_models_config()` parsed embedded `include_str!(models.toml)` first, ignoring `EDGEQUAKE_MODELS_CONFIG` and `./models.toml`.

### Fix
Inverted precedence in `bundled_models.rs`:
1. `ModelsConfig::load()` — env / cwd / home
2. `embedded_models_catalog()` — compile-time fallback
3. `ModelsConfig::builtin_defaults()`

Startup logs: `"Loaded models catalog from runtime config..."` when override is active.

### Verification
- `runtime_models_config_overrides_bundled` unit test passed
- Docker mount of custom `models.toml` via `EDGEQUAKE_MODELS_CONFIG` now honored

---

## #250 — UI version ≠ API version

**Fixed in v0.13.2** (SPEC-040)

### Root cause
WebUI Docker image baked `package.json` version at build time without release-tag injection; API reported `Cargo.toml` workspace version → semver skew in official images.

### Fix
1. **`release-docker.yml`** — passes `NEXT_PUBLIC_APP_VERSION=${{ needs.meta.outputs.version }}` to frontend Docker build.
2. **`edgequake_webui/Dockerfile`** — `ARG`/`ENV` for `NEXT_PUBLIC_APP_VERSION`.
3. **`release_gates.sh`** — fails CI if `edgequake/Cargo.toml` ≠ `edgequake_webui/package.json`.
4. **Release bump** — both artifacts synced to **0.13.2**.

### Verification
After `v0.13.2` image pull: UI footer and `GET /health` both report `0.13.2`.
