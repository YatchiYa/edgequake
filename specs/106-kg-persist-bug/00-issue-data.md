# 00 — Issue data (#356)

| Field | Value |
|-------|-------|
| Issue | [#356](https://github.com/raphaelmansuy/edgequake/issues/356) |
| Reporter | ankursingh-devops |
| Reported version | **0.12.11** (Docker) |
| Confirmed still broken | **v0.24.0** (tag + `edgequake/Cargo.toml`) |
| Symptom SQLSTATE | `42883` undefined_function / missing operator |
| Exact error | `operator does not exist: ag_catalog.graphid = ag_catalog.graphid` |
| User-facing | `Knowledge graph persist failed (relational FK or document status)` |
| Technical wrap | `1 knowledge-graph merge error(s) during persist` → Batch query failed |

## Reproduce (reporter)

1. Upload a document that yields relationships.
2. Wait for KG extraction + persist.
3. Observe graph merge failure.

## Reproduce (code-level)

Call `GraphStorage::get_edges_for_nodes_batch` on AGE after upserting ≥1 edge whose both endpoints are in the id set — pre-fix SQL planned `src.vid = e.start_id` (raw graphid).
