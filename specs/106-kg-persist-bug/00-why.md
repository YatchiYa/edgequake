# 00 — Why SPEC-106

Knowledge-graph persist is a hard requirement for GraphRAG. Issue #356 shows documents that extract relationships fail at persist with:

`operator does not exist: ag_catalog.graphid = ag_catalog.graphid`

## Residuals after #214

Issue #214 (v0.12.1) fixed `get_nodes_with_degrees_batch` (graph viz / degrees) via `::text` casts. The **relationship-merge pre-read** `get_edges_for_nodes_batch` kept raw `graphid = graphid` JOINs and shipped through **v0.24.0**.

## User impact

- Entity-only docs may complete; docs with relationships fail KG persist.
- Cascade delete / any caller of `get_edges_for_nodes_batch` can hit the same 42883.
- User-facing: `Knowledge graph persist failed (relational FK or document status)`.

## Non-goals

- Replacing AGE; registering upstream `graphid` operators; rewriting all edge reads onto property path in this cut.
