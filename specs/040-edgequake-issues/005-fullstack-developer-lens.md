# SPEC-040 — Full Stack Developer Lens

**Lens:** Full stack (Rust API + Next.js + worker + PostgreSQL)  
**Focus:** Contract gaps, DRY boundaries, SOLID fix surfaces

---

## Architecture map

```
┌─────────────────────────────────────────────────────────────────┐
│  edgequake_webui (Next.js)                                      │
│  ├─ use-file-upload.ts ──────► duplicate dialog / replace flow  │
│  ├─ use-query-streaming.ts ───► chatCompletionStream            │
│  ├─ use-query-conversation-lifecycle.ts ► workspace isolation   │
│  └─ app-version.ts ──────────► sidebar vs header semver           │
└───────────────────────────┬─────────────────────────────────────┘
                            │ HTTP
┌───────────────────────────▼─────────────────────────────────────┐
│  edgequake-api (Axum)                                             │
│  ├─ handlers/documents/upload/* ─► document_admission.rs (SSOT)   │
│  ├─ handlers/pdf_upload/upload.rs ► pdf_workspace_dedup.rs       │
│  ├─ handlers/chat/streaming.rs ──► conversation_service          │
│  ├─ handlers/workspaces/stats.rs ► graph_storage counts          │
│  ├─ handlers/graph/* ────────────► graph_materialization.rs       │
│  └─ state/bundled_models.rs ─────► /api/v1/models/*              │
└───────────────────────────┬─────────────────────────────────────┘
                            │
┌───────────────────────────▼─────────────────────────────────────┐
│  edgequake-storage + PostgreSQL (AGE + pgvector + conversations)│
└─────────────────────────────────────────────────────────────────┘
```

---

## Issue #262 — Full stack trace

| Layer | Component | Behavior |
| ----- | --------- | -------- |
| UI | `use-workspace-stats.ts`, dashboard cards | Polls `/stats` every 30s |
| API | `get_workspace_stats` | 4s timeout; falls back to stale cache |
| Storage | `try_kv_storage_stats` | Calls `node_count_by_workspace` + `edge_count_by_workspace` |
| SQL | `analytics_ops.rs` | COUNT on `_ag_label_vertex` with workspace predicate |
| Graph UI | `graph_stream.rs` | `get_popular_nodes_with_degree` under 15s tokio timeout |

**Full-stack fix:**

1. **DB:** Ensure child indexes + ANALYZE (see AGE lens doc).
2. **API:** Keep 4s/15s budgets — do not mask with longer timeouts.
3. **UI:** Already handles `stale: true` (SPEC-021 P-G13) — verify banner copy.

---

## Issue #259 — Full stack trace

| Step | Layer | Problem |
| ---- | ----- | ------- |
| 1 | UI | User switches workspace — `selectWorkspace()` only invalidates stats |
| 2 | UI | `activeConversationId` from previous workspace still in zustand persist |
| 3 | UI | User submits query → `chatCompletionStream({ conversation_id })` |
| 4 | API | Stream start: conversation exists → user message INSERT OK |
| 5 | API | Long RAG (#262) — 30–120s |
| 6 | API/User | Conversation deleted OR never existed in PG replica lag |
| 7 | API | Assistant `create_message` → **FK violation** |

**Full-stack fix (SOLID):**

```typescript
// tenant-workspace-selector OR use-query-conversation-lifecycle
onWorkspaceChange: () => {
  store.setActiveConversation(null);
  queryClient.removeQueries({ queryKey: conversationKeys.all });
}
```

```rust
// streaming.rs — before assistant create_message
if state.conversation_service.get_conversation(conversation_id).await?.is_none() {
    // emit SSE error CONVERSATION_GONE; do not INSERT
}
```

**DRY:** Reuse `isConversationNotFoundError` patterns from `use-query-streaming.ts` on both client and server error codes.

---

## Issue #253 — Full stack trace

| Step | Layer | Detail |
| ---- | ----- | ------ |
| 1 | API | `admit_document_for_processing` finds hash key → `resolve_workspace_duplicate_for_reingestion` |
| 2 | API | Metadata gone → `delete_document_for_reingestion` returns `Ok(false)` → `StillProcessing` |
| 3 | API | Response `duplicate_of: Some(id)` — `text_upload.rs:91-103` |
| 4 | UI | Dialog shown; user Replace |
| 5 | UI | `deleteDocument` 404; `handleFilesUpload` → duplicate again |

**Full-stack fix:**

1. **API (new):** `recycle_orphan_content_hash(state, hash_key, workspace_id)` — if metadata missing, DELETE hash key (mirror `recycle_orphan_workspace_pdf`).
2. **API:** Call from `resolve_workspace_duplicate_for_reingestion` before `StillProcessing`.
3. **UI:** Add `force_reindex` or dedicated `POST /documents/{id}/reprocess` for markdown (optional — backend auto-clear preferred).
4. **UI:** Surface errors from `doReplaceAll` via toast (today `console.warn` only).

---

## Issue #251 — Full stack trace

| Layer | File | Fix |
| ----- | ---- | --- |
| Rust | `bundled_models.rs` | Invert load order |
| Rust | `provider_catalog.rs` | No change — consumes config |
| UI | Model picker hooks | Automatically picks up API catalog |
| Docker | `docker-compose.quickstart.yml` | Document volume mount (already in issue) |
| Ops | Startup logs | `tracing::info!(source = path)` |

**Test:** Integration test with temp dir + `EDGEQUAKE_MODELS_CONFIG`.

---

## Issue #250 — Full stack trace

| Artifact | Version source |
| -------- | -------------- |
| API binary | `edgequake/Cargo.toml` |
| Web UI | `edgequake_webui/package.json` |
| Docker | Separate image tags on GHCR |

**Fix options (pick one):**

| Option | Effort | Description |
| ------ | ------ | ----------- |
| A (recommended) | Low | CI writes `NEXT_PUBLIC_APP_VERSION=$CARGO_PKG_VERSION` during `webui` build in release pipeline |
| B | Medium | UI fetches `/health` once at boot; footer shows API version |
| C | Low | Keep dual labels; fail CI if semver mismatch |

---

## DRY consolidation opportunities

| Smell | Locations | Unify into |
| ----- | --------- | ---------- |
| Orphan duplicate handling | `pdf_workspace_dedup.rs`, (missing) hash recycler | `services/orphan_duplicate.rs` |
| Conversation reset | selector, lifecycle hook, streaming catch | `lib/workspace/reset-query-context.ts` |
| Graph timed queries | `popular.rs`, `graph_stream.rs`, `traversal.rs` | Already uses `run_timed_graph_query` ✅ |
| Version display | sidebar, header, health | `useReleaseInfo()` hook |

---

## SOLID checklist for implementation

- [ ] **SRP:** `OrphanHashRecycler` only handles KV hash keys without metadata
- [ ] **OCP:** New duplicate sources (e.g. multimodal) extend recycler, not upload handlers
- [ ] **DIP:** Upload handlers depend on `DuplicateResolver` trait, not raw KV deletes
- [ ] **ISP:** Don't add graph methods to `AppState` — use existing `GraphQueryRuntime`

---

## Edge cases (full stack)

| Case | Expected |
| ---- | -------- |
| Replace during active worker processing | `StillProcessing` + UI message “wait or cancel task” |
| Workspace switch mid-stream | Abort stream; clear conversation; no FK error |
| Custom models.toml parse error | Log error; fall back to bundled with `WARN` |
| API upgraded, UI cached (PWA) | Service worker bump or version mismatch banner |
| Concurrent duplicate uploads | `staging_hash_key` prevents double enqueue (P-11) ✅ |
