# SPEC-050 — Shared Entity & Relationship Edge Cases

## First Principles

> **P1 — Source Tracking:** Every entity and relationship stores `source_ids` — the list of  
> chunk IDs (and by extension document IDs) that contributed to its existence.  
> An entity can only be removed when ALL its source documents are gone.

> **P2 — Bounded Cascade:** The cascade must touch ONLY graph objects whose `source_ids`  
> reference the deleted document. Never `get_all_nodes()` × workspace size.  
> O(document_entities + document_edges) — not O(workspace_nodes).

> **P3 — Non-Destructive Update:** Shared entities that survive deletion must have their  
> source_ids pruned and description rebuilt from the remaining sources — preserving  
> the entity while accurately reflecting its reduced evidence base.

---

## Taxonomy of Affected Entities

```
Entity (node in knowledge graph)
  │
  ├─ EXCLUSIVE: source_ids contains ONLY chunks from Document D
  │     → REMOVED on delete of D
  │
  ├─ SHARED: source_ids contains chunks from D AND other documents
  │     → UPDATED on delete of D  (source_ids pruned, description rebuilt)
  │     → SURVIVES in the graph with reduced evidence
  │
  └─ INJECTED: source_ids is empty (knowledge injected directly, not from any doc)
        → SKIPPED on delete of D  (never touched by document cascade)
        → SURVIVES unchanged
```

---

## Edge Cases

### EC-1: Exclusive Entity (No Shared Sources)

```
Before:  ENTITY_A { source_ids: ["doc-X-chunk-0", "doc-X-chunk-3"] }
Delete:  Document X
After:   ENTITY_A → DELETED
```

**Backend:** `remaining.is_empty()` → `graph.delete_node(&node.id)`  
**Impact count:** `entities_to_remove += 1`  
**UI:** Red badge "removed from graph" ✓

---

### EC-2: Shared Entity (Multiple Source Documents)

```
Before:  ENTITY_A { source_ids: ["doc-X-chunk-0", "doc-Y-chunk-5", "doc-Z-chunk-2"] }
Delete:  Document X
After:   ENTITY_A { source_ids: ["doc-Y-chunk-5", "doc-Z-chunk-2"] }
         description rebuilt from doc-Y and doc-Z chunks only
```

**Backend:** `remaining.len() < sources.len()` → `graph.upsert_node` with pruned props  
**Impact count:** `entities_to_update += 1`  
**UI:** Amber badge "updated, other sources remain" ✓  
**Key guarantee:** Entity SURVIVES. Its description and embeddings are refreshed from remaining sources.

---

### EC-3: Orphaned Relationship (Endpoint Was Exclusive Entity)

```
Before:  ENTITY_A (exclusive to doc-X) → ENTITY_B (exclusive to doc-X)
         edge { source_ids: ["doc-X-chunk-0"] }
Delete:  Document X
After:   ENTITY_A → DELETED (exclusive)
         ENTITY_B → DELETED (exclusive)
         edge    → DELETED (endpoint gone, and was exclusive anyway)
```

**Backend:** `get_edges_for_nodes_batch(deleted_node_ids)` collects all edges touching  
any deleted node. Then `!source_exists || !target_exists` → `delete_edge`.  
**Impact count:** `relationships_removed += 1`  
**Key guarantee:** No orphan edges — cascade cleans up all edges connected to removed nodes.

---

### EC-4: Cross-Document Relationship (Source Entity Shared, Target Entity Exclusive)

```
Before:  ENTITY_S (shared: doc-X + doc-Y) → ENTITY_T (exclusive: doc-X only)
         edge { source_ids: ["doc-X-chunk-0"] }
Delete:  Document X
After:   ENTITY_S → UPDATED (still in graph, source: doc-Y)
         ENTITY_T → DELETED (exclusive to doc-X)
         edge    → DELETED (ENTITY_T is gone, AND edge was exclusive to doc-X)
```

**Backend:**  
1. Node loop: ENTITY_S updated; ENTITY_T deleted; deleted_node_ids = {ENTITY_T}  
2. Edge loop: edge collected via `get_edges_for_nodes_batch({ENTITY_T})`; deleted  
**Impact count:** `entities_updated += 1`, `entities_removed += 1`, `relationships_removed += 1`

---

### EC-5: Cross-Document Relationship (Both Endpoints Shared, Edge Exclusive)

```
Before:  ENTITY_S (shared: doc-X + doc-Y) → ENTITY_T (shared: doc-X + doc-Z)
         edge { source_ids: ["doc-X-chunk-0"] }   ← only from doc-X
Delete:  Document X
After:   ENTITY_S → UPDATED (source: doc-Y)
         ENTITY_T → UPDATED (source: doc-Z)
         edge    → DELETED (exclusive to doc-X, source list is empty after pruning)
```

**Backend:** Both nodes updated; edge found via `find_document_edges`; `remaining.is_empty()` → deleted  
**Impact count:** `entities_updated += 2`, `relationships_removed += 1`

---

### EC-6: Cross-Document Relationship (Both Endpoints Shared, Edge Also Shared)

```
Before:  ENTITY_S (shared) → ENTITY_T (shared)
         edge { source_ids: ["doc-X-chunk-0", "doc-Y-chunk-1"] }
Delete:  Document X
After:   ENTITY_S → UPDATED
         ENTITY_T → UPDATED
         edge    → UPDATED (source: ["doc-Y-chunk-1"])
```

**Backend:** Both nodes updated; edge updated with pruned source_ids  
**Impact count:** `entities_updated += 2`, `relationships_updated += 1`

---

### EC-7: Injected Entity (No Source Document)

```
Before:  ENTITY_INJ { source_ids: [] }   ← injected via POST /api/v1/injection
Delete:  Document X
After:   ENTITY_INJ unchanged
```

**Backend:** `if sources.is_empty() { continue; }` — injected entities are NEVER touched  
**Impact count:** NOT counted in `entities_to_remove` or `entities_to_update`  
**UI note:** The impact card cannot show injected entity count (bounded scan principle)

---

### EC-8: Bulk Delete — Sequential Cascade for Shared Entities

```
Before:  ENTITY_S { source_ids: ["doc-A-chunk-0", "doc-B-chunk-0"] }
Bulk delete: { doc-A, doc-B }

Step 1 (delete doc-A):
  ENTITY_S remaining = ["doc-B-chunk-0"] → UPDATED

Step 2 (delete doc-B):
  ENTITY_S remaining = [] → REMOVED (exclusive now)

Impact visible to user:
  Doc A impact:  entities_to_update += 1   ← "will survive"
  Doc B impact:  entities_to_remove += 1   ← "will be removed"
```

**Key insight:** Each document deletion is independent. Impact previews show the  
state AT THE TIME of deletion, not the cumulative effect.  
**UI note:** `BulkDeleteConfirmDialog` shows aggregate warning but cannot model cross-doc  
cascading. Users deleting multiple documents that share entities must understand  
that entities shared ONLY between those documents will eventually be removed.

---

### EC-9: Reprocess — Partial Entity Rebuild

```
Before:  ENTITY_A { source_ids: ["doc-X-chunk-0"], description: "old desc" }
Reprocess doc-X (entities mode):
  1. cascade_remove_document_sources(doc-X) — removes doc-X's entity contributions
  2. Run pipeline again — re-extracts entities from doc-X
  3. Re-merge entities back into graph
After:   ENTITY_A potentially has updated description, same or different source_ids
```

**This is NOT a deletion** — it is a clear-then-re-ingest.  
**Shared entities** from other documents are unaffected during the clear phase  
(same cascade rules as above), then re-linked during re-ingestion.

---

### EC-10: KV/ID Mismatch (SPEC-045)

```
KV key:    "old-prefix-metadata"
JSON id:   "new-uuid-id"
```

When `actual_key_prefix != document_id`, the scope is built with BOTH prefixes:
```rust
DocumentSourceScope::with_key_prefix(document_id, actual_key_prefix)
// source_prefixes = [actual_key_prefix, document_id]
```

`source_belongs_to_document` checks ALL prefixes → both old and new chunk formats  
are removed correctly.

---

## What the UI Must Communicate

| Scenario                        | Count field               | Colour | User message                                                |
| ------------------------------- | ------------------------- | ------ | ----------------------------------------------------------- |
| Entities fully removed          | `entities_to_remove`      | Red    | "Will be permanently removed"                               |
| Entities survive (updated)      | `entities_to_update`      | Amber  | "Will survive with fewer supporting sources"                |
| Relationships fully removed     | `relationships_to_remove` | Red    | "Will be permanently removed"                               |
| Relationships survive (updated) | `relationships_to_update` | Amber  | "Will survive with fewer supporting sources"                |
| Injected entities               | (not shown)               | —      | "Injected knowledge is never affected by document deletion" |

---

## Implementation Proof: `resource_safety_delete_cascade_bounded_scope`

The automated proof test in `edgequake-api/tests/resource_safety_proof.rs` confirms:
- `entities_removed == 2` (exclusive-to-doc entities)
- `entities_updated == 1` (SHARED_ENTITY keeps its other-doc source)
- `PROOF_ENTITY_000000` (noise node) is untouched
- All edges connected to deleted nodes are removed

Run: `cargo test -p edgequake-api resource_safety_delete_cascade_bounded_scope -- --nocapture`
