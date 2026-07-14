# SPEC-050 — O(n) Expert Lens

## Complexity Analysis

### Single Document Delete

| Phase                             | Complexity   | Notes                           |
| --------------------------------- | ------------ | ------------------------------- |
| resolve_kv_key_prefix (fast path) | O(1)         | Direct key lookup               |
| resolve_kv_key_prefix (slow path) | O(D)         | D = total docs in workspace     |
| keys_with_prefix (chunk scan)     | O(C)         | C = chunks of this doc          |
| cascade_remove (graph)            | O(E + R)     | E = entities, R = relationships |
| delete vectors                    | O(C)         | Batch DELETE, one round trip    |
| delete KV keys                    | O(C)         | Batch DELETE, one round trip    |
| **Total**                         | O(E + R + C) | Bounded by document scope       |

**SLO:** Single delete should complete in < 2s for any document up to 500 chunks.

### Bulk Delete (N documents)

| Phase                | Complexity          | Notes                 |
| -------------------- | ------------------- | --------------------- |
| load_scoped_entries  | O(D)                | D = docs in workspace |
| per-document cascade | O(N × (E + R + C))  | Sequential today      |
| total                | O(N × avg_doc_cost) |                       |

For 100 docs × 50 chunks avg = O(5000 operations).  
At 10ms/op → ~50s for 100 docs — acceptable if progress is shown.

### Impact Analysis (GET /deletion-impact)

| Phase             | Complexity   | Notes                     |
| ----------------- | ------------ | ------------------------- |
| chunk count       | O(C)         | prefix scan               |
| entity/rel impact | O(E + R)     | graph scan bounded to doc |
| **Total**         | O(E + R + C) | Same as delete itself     |

**SLO:** Impact analysis < 500ms for any document.

### WS Broadcast Overhead

Each broadcast is a `tokio::sync::broadcast::send()` — O(subscribers).  
Typical: 1-5 concurrent WS connections → O(1) effectively.  
No concern.

### Memory (Frontend)

`deletingDocumentIds: Set<string>` — O(concurrent_deletes) ≈ O(1) in practice.

### Key Risks

1. **Slow path in `resolve_kv_key_prefix`** — O(D) full metadata scan.  
   Mitigation: Only triggered when fast path misses (id mismatch). Acceptable.

2. **Bulk delete sequential loop** — O(N) sequential round trips.  
   Mitigation: Show progress per document so it doesn't look hung.
   Future: Batch the vector deletes for all docs at once.

3. **`analyze_deletion_impact` called every time dialog opens** — O(E+R+C).  
   Mitigation: `staleTime: 30_000` in `useDeletionImpact` — cached for 30s.
   If user opens dialog twice for same doc, second call is free.
