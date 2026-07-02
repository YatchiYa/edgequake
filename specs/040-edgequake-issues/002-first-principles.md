# SPEC-040 — First Principles

**Lens:** First principles  
**Question:** What must be true for EdgeQuake to behave correctly under production load?

---

## Principle 1 — Single source of truth per concern (DRY)

| Concern | SSOT today | Violation |
| ------- | ---------- | --------- |
| Model catalog | Documented: runtime file → embed fallback | **Inverted** — embed always wins (#251) |
| Document duplicate identity | Should be: visible metadata OR no duplicate | Hash KV can exist without metadata (#253) |
| Release version | Should be: one semver per deploy | UI `package.json` ≠ API `Cargo.toml` (#250) |
| Graph workspace filter | Should be: indexed predicate on child `"Node"` | Queries use `_ag_label_vertex` union; indexes on parent (#262) |
| Active conversation | Should be: valid FK before any message INSERT | Stale ID can reach streaming assistant save (#259) |

**Invariant:** Every user-visible state must be derivable from one authoritative store without “ghost” rows in auxiliary indexes.

---

## Principle 2 — Planner truth = operator truth (PostgreSQL)

For AGE + pgvector:

1. **Rows live in child label tables** (`"Node"`, `"EDGE"`), not inheritance parents.
2. **Expression indexes must sit on the rel that is scanned** — `(agtype_to_json(properties)->>'workspace_id')` on `"Node"`, not `_ag_label_vertex`.
3. **Cast predicates need matching indexes** — `start_id::text` joins require `(start_id::text)` indexes (M072), not raw `graphid` btree alone.
4. **Statistics must reflect expressions** — `ANALYZE` on child tables after index creation; consider `CREATE STATISTICS` on hot expressions at >10k nodes.

**Invariant:** `EXPLAIN (ANALYZE, BUFFERS)` for workspace-scoped graph reads must show **Hash Join** or **Index Scan**, never Nested Loop with >10⁶ comparisons.

---

## Principle 3 — Fail fast, fail visible (UX + ops)

| Failure | Current | Required |
| ------- | ------- | -------- |
| models.toml ignored | Silent | `INFO` log: `Loaded models catalog from {path}` |
| Duplicate replace stuck | Dialog loops | Toast + recycle hash or force path |
| Conversation FK | 500 after long wait | Pre-flight `get_conversation` or create new thread |
| Version skew | Two numbers | One release ID or explicit “UI/API” with CI gate |

**Invariant:** Misconfiguration and stale client state must never present as silent success.

---

## Principle 4 — Workspace isolation is end-to-end

Isolation must hold at:

- HTTP headers (`X-Workspace-ID`)
- KV keys (`workspace_hash`, metadata `workspace_id`)
- Graph properties (`workspace_id` on nodes/edges)
- PDF rows (`pdf_documents.workspace_id`)
- **Conversation scope** (`conversations.workspace_id` + UI reset on switch)
- Query filters (`NodeListFilter.workspace_id`)

**Invariant:** Switching workspace must reset all client-held IDs that are workspace-scoped before the next mutating request.

---

## Principle 5 — Complexity budgets are product features (O(N))

| Operation | Budget | Enforcement |
| --------- | ------ | ----------- |
| Workspace stats (uncached) | <4s | `STATS_FETCH_TIMEOUT` — `stats.rs:84` |
| Graph materialization query | <15s | `run_timed_graph_query` — `graph_materialization.rs` |
| Popular nodes (27k graph) | <100ms | `graph_sota_tests.rs` target |
| Document list | O(docs in workspace) | wsdoc index prefix — SPEC-027 |

**Invariant:** No user-facing poll path may be O(V×E) on full graph; filter-first SQL is mandatory.

---

## Principle 6 — SOLID at the fix boundary

| Letter | Application |
| ------ | ----------- |
| **S** | `OrphanDuplicateRecycler` trait — PDF and KV-hash strategies separate |
| **O** | Extend `graph_lifecycle` index bootstrap; don’t fork per-endpoint SQL |
| **L** | Memory and Postgres graph adapters share `GraphReadOps` contracts |
| **I** | `ModelsCatalogLoader` interface — file vs embed implementations |
| **D** | Upload handlers depend on `admit_document_for_processing`, not duplicate SQL |

---

## Non-goals (avoid over-engineering)

- Do **not** reintroduce parent-table indexes “for completeness” — they add write amplification (M070 rationale).
- Do **not** raise 15s timeout as the primary #262 fix — that masks planner failure.
- Do **not** disable duplicate detection — fix ghost key lifecycle instead.

---

## Acceptance predicates (first-principles tests)

```text
∀ workspace W, graph G:
  stats(W) completes in <4s OR returns stale=true with last-good cache

∀ upload U with content hash H:
  duplicate(U) ⟺ visible_document(W, H) ∨ in_flight_staging(W, H)

∀ chat stream S:
  assistant_message(S) ⟹ conversation_exists(S.conversation_id) at T_insert

∀ deploy D:
  version_ui(D) = version_api(D) OR labeled dual-artifact with CI gate failure
```
