# 06 — Edge Cases

| Case | Expected |
|------|----------|
| Empty `node_ids` | Early `Ok([])` — no SQL, no operator |
| Entity-only merge (no relationships) | `get_edges_for_nodes_batch` not called |
| Both endpoints in set | Edge returned |
| Only one endpoint in set | Edge excluded (both-in-set semantics preserved) |
| Empty EDGE table | Query plans with `::text`; returns `[]` |
| Cascade delete using same API | Also fixed by LAW-G1 |
| Multigraph same endpoints | All matching both-in-set edges returned |
