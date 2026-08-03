# 02 — Cross-Reference Matrix (SPEC-106)

| Symptom | Law | Smoking gun | Fix | E2E | Status |
|---------|-----|-------------|-----|-----|--------|
| #356 persist 42883 | G1 | `src.vid = e.start_id` in `pg_get_edges_for_nodes_batch` | `vid_text = e.start_id::text` | E2E-106-01 | **Closed** |
| Incomplete #214 | G1, G2 | degrees fixed; edges-batch not | this spec | E2E-106-02 | **Closed** |
| Empty ids | G1 | early return | keep | E2E-106-03 | **Closed** |

## Call graph (persist)

```ascii
 DefaultIngestionPersister
   → KnowledgeGraphMerger::merge_relationships
        → get_edges_for_nodes_batch   ← ★ was raw graphid JOIN
        → upsert_edges_batch
```
