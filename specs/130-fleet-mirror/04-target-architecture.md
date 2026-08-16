# 04 — Target architecture

## SSOT

```ascii
  ┌─────────────────────────────────────────────────────────────┐
  │  PostgresEntitySink::upsert_relationships_batch             │
  │  INSERT … ON CONFLICT … RETURNING id (or equivalent SELECT) │
  │  Output: HashMap<legacy_key, Uuid>                          │
  │          legacy_key = "SRC->TGT:TYPE" (bare + uppercase)    │
  └────────────────────────────┬────────────────────────────────┘
                               │ LAW-130-1
                               ▼
  ┌─────────────────────────────────────────────────────────────┐
  │  KnowledgeGraphMerger RelVectors path                       │
  │  Pass map into fleetFleetEmbeddingIndex mirror API        │
  │  Prefer UUID; do not call resolve_relationship_id_pool      │
  │  for keys present in the map                                │
  └────────────────────────────┬────────────────────────────────┘
                               │ LAW-130-2
                               ▼
  ┌─────────────────────────────────────────────────────────────┐
  │  relationship_embeddings upsert (FK = relationships.id)     │
  └─────────────────────────────────────────────────────────────┘

  Offline / migration / coverage
       └── resolve_relationship_id_pool still name-based (LAW-130-5)
```

## Writers table

| Writer | Responsibility | Must not |
|--------|----------------|----------|
| Rel sink | Create/upsert spine; **return** ids keyed by legacy id | Silently drop ids |
| RelVectors mirror (in-session) | Embed by UUID from map | Re-guess identity by name for mapped keys |
| Name resolve (coverage) | Backfill / migration only | Be the hot-path SSOT for merge |
| Error formatter | State relationship identity miss clearly | Blame entity spine only |

## Trait / API shape (concrete)

Prefer extending the relational sink + fleet traits without a second parallel codepath:

```rust
// Conceptual — exact names in WP-1/WP-2
async fn upsert_relationships_batch(
    &self,
    rows: &[RelationshipSinkRow],
) -> Result<RelationshipSinkReport>;
// RelationshipSinkReport { ids: HashMap<String /* legacy */, Uuid>, missing_fk: u64 }

async fn mirror_legacy_batch_with_ids(
    &self,
    rows: &[(String, Vec<f32>, Value)],
    count_as_entities: bool,
    known_relationship_ids: Option<&HashMap<String, Uuid>>,
) -> Result<MirrorLegacyReport>;
```

DRY: single legacy-key formatter shared with `collect_relationship_vector_batch` (`SRC->TGT:TYPE` + `normalize_relation_type_str`).

SOLID:

- **S** — sink owns SQL identity; mirror owns embeddings.
- **O** — extend mirror with optional map; keep name resolve for tools.
- **L** — memory/test sinks return synthetic maps.
- **I** — do not force coverage tools through merge-only APIs.
- **D** — merger depends on sink/fleet traits, not sqlx details.

## Sequencing (unchanged invariant)

```ascii
  EntityGraph → EntityVectors → RelGraph(+sink map) → RelVectors(+map)
```

No new cross-worker lock. Same-document await order remains sufficient once identity is retained.

## Error hint (target copy)

```text
SPEC-091: typed fleet mirror resolved R/E rows
  (relationship UUID missing from sink map or unresolved legacy key —
   in-session RelVectors must use sink-returned relationships.id;
   SPEC-098 misses: [...])
```

Entity-chunk failures keep a separate entity-spine phrasing.

## Cross-refs

- Laws: [01-first-principles.md](01-first-principles.md)
- Plan: [07-implementation-plan.md](07-implementation-plan.md)
- As-is: [03-code-as-is.md](03-code-as-is.md)
