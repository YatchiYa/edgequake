# 03 — Code as-is (pre-fix)

## Hot path (typed authority)

```ascii
  KnowledgeGraphMerger::merge_all
    → RelGraph: merge_relationships_batch
         → PostgresEntitySink::upsert_relationships_batch
              RETURNING → RelationshipSinkReport.ids
              keyed by format_relationship_legacy_key(src,tgt,rel)
    → RelVectors: collect_relationship_vector_batch
         → same format_relationship_legacy_key
    → upsert_vectors_chunked(..., known_relationship_ids)
         → fleet_index.mirror_legacy_batch(slice, false, known)
              for each id:
                classify → Relationship
                parse_relationship_legacy_key(id)   ← rsplit (lossy)
                rid ← known.get(id) OR resolve_relationship_id_pool(src,tgt,…)
                if None → report.push_miss(id)
              if typed ∧ !report.is_complete() → StorageError SPEC-091
```

## Failing parser

`embedding_family.rs`:

```rust
pub fn parse_relationship_legacy_key(id: &str) -> Option<(String, String, String)> {
    let (pair, rel_type) = id.rsplit_once(':')?;
    let (source, target) = pair.rsplit_once("->")?;  // last arrow only
    Some((source.into(), target.into(), rel_type.into()))
}
```

Documented residual in the same file and CHANGELOG:

> Residual ambiguity remains if the **target** name also contains `->`.

## Lab proof (zz-raw keys)

| Legacy id (abbrev) | rsplit `(src,tgt)` | Intended |
|--------------------|--------------------|----------|
| `LEFT_MARGIN->…->_+:RELATED_TO` | `(LEFT_MARGIN->…->_00_ , _+)` | `(LEFT_MARGIN , …->_+)` |
| `FLOW_DIRECTION->ARROW_1_(SHADED_BOX_->CIRCULAR_TARGET):…` | `(…SHADED_BOX_ , CIRCULAR_TARGET))` | `(FLOW_DIRECTION , ARROW_1_(…))` |

Index-guided split recovers intended when both entity names exist (reproduced in intake).

## Why SPEC-130 did not fully hide this

```ascii
  known_ids = Some(map) only if !rel_sink_report.ids.is_empty()

  If map empty / key absent:
    → parse rsplit → wrong endpoints → miss

  If map complete for that id:
    → known.get(id) hits regardless of arrows  (parse still runs but unused for FK)

  Near-miss 995/1000 ⇒ five ids not resolved by map+parse.
  Typical: map miss or empty map + arrow-in-target parse.
```

Secondary hazard (LAW-133-1): two distinct `(src,tgt)` pairs can **format to the same string** when arrows nest differently — HashMap keeps one UUID. Index-guided parse + future escape encoding address this; out of critical path for unique both-resolve cases.

## Working pieces (keep)

| Piece | Role |
|-------|------|
| `format_relationship_legacy_key` | Shared formatter (uppercase rel via normalize) |
| `EntityNameIndex::resolve` | Bare / normalized / `ws::` suffix lookup |
| SPEC-130 RETURNING map | In-session UUID identity |
| Fail-closed `resolved < eligible` | Honest Failed status |

## Cross-refs

- Target: [04-target-architecture.md](04-target-architecture.md)
- Matrix: [02-cross-ref-matrix.md](02-cross-ref-matrix.md)
