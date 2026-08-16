# 03 — Code as-is (investigation)

## Typed call order (HEAD)

```ascii
  DefaultIngestionPersister::persist
    → KnowledgeGraphMerger::merge_with_progress
         │
         │  typed_authority = vector_backend_reads_typed(...)
         │
         ├─ EntityGraph      merge_entities_batch → PostgresEntitySink entities
         ├─ EntityVectors    upsert_vectors_chunked(..., entities=true)
         │                     → mirror_legacy_batch(count_as_entities=true)
         ├─ RelGraph         merge_relationships_batch
         │                     → AGE upsert_edges_batch
         │                     → PostgresEntitySink::upsert_relationships_batch
         │                        (INSERT … ON CONFLICT; **no RETURNING / no map**)
         └─ RelVectors       upsert_vectors_chunked(..., entities=false)
                               → mirror_legacy_batch(count_as_entities=false)
                                  → parse_relationship_legacy_key
                                  → resolve_relationship_id_pool  ← name re-lookup
                                  → upsert relationship_embeddings
```

Await barriers between phases: RelGraph **completes** (including sink) before RelVectors starts. This is **not** unordered concurrent writers under typed authority.

Legacy (non-typed) order differs: RelVectors before RelGraph — out of product default path when `EDGEQUAKE_VECTOR_BACKEND` reads typed.

## Why the ~1s `created_at` gap is expected

```ascii
  t0  EntityGraph sink     entities.created_at = NOW()     e.g. …:56.261
      EntityVectors        embed + fleet entity mirror     (wall time)
      RelGraph AGE         edge upsert                     (wall time)
  t1  RelGraph sink        relationships.created_at = NOW() e.g. …:57.258
  t2  RelVectors mirror    SELECT relationships …          after t1 await
```

Reporter’s MELISSA_BOTHA / FLAT_4 timestamps match this shape. Gap alone does **not** prove SELECT-before-INSERT.

## Why identical retries falsify pure timing race

```ascii
  Fail at RelVectors
       │
       ├─ compensation: AGE / vector artifacts may roll back
       └─ public.entities / public.relationships  NOT compensated (LAW-130-9)

  Reprocess same doc
       │
       └─ if miss were only “edge not committed yet”, leftover spine
          would make second resolve succeed — reporter sees same misses
```

Permanent classification (`typed fleet mirror` → `GraphMerge`) also prevents soft auto-heal; manual reprocess still hits the same resolve logic.

## Identity discard (actual gap)

**Sink** (`postgres_entity_sink.rs` batch):

- Resolves endpoint entity UUIDs via `name = ANY($bare)` (HashMap last-wins on duplicate names).
- `INSERT INTO relationships … ON CONFLICT DO UPDATE` — **does not return ids** to caller.

**Mirror** (`fleet_embedding_index.rs`):

- Loads `EntityNameIndex` (SPEC-120: **oldest** `created_at` wins for name keys).
- `resolve_relationship_id_pool(src, tgt, rel_type, workspace)` → SELECT by those ids.

```ascii
  Duplicate / alias name in workspace
       Sink by_name.insert → LAST id
       Index from_rows or_insert → OLDEST id
       SELECT source_id=oldest AND target_id=…  → 0 rows
       JOIN-by-name still shows an edge on LAST ids
       ⇒ reporter evidence pattern + identical retries
```

Other deterministic miss classes (same symptom):

| Class | Mechanism |
|-------|-----------|
| Workspace meta skew | Vector metadata `workspace_id` ≠ sink workspace |
| Rel type drift | Unnormalized type on one side (mitigated by LAW-098-3; still guard) |
| Batch entity FK miss | Sink skips rows when bare name not in `entities` (scoped-only names) — then no edge; different from “edge exists” |
| Placeholder endpoints | AGE placeholders without entity spine (sibling residual) |

## Fail-closed error (misleading hint)

```rust
// merger/mod.rs upsert_vectors_chunked — paraphrase
"SPEC-091: typed fleet mirror resolved {}/{} rows \
 (relational entity/rel FK miss or name mismatch — \
 bare entities.name must match entity:NAME; ensure \
 PostgresEntitySink wrote the spine before fleet mirror; \
 SPEC-098 misses: {:?})"
```

For relationship chunks, misses look like `SRC->TGT:REL`. Hint still prioritizes **entity spine** → steered #380.

## Reproduction evidence (static + unit)

Recorded 2026-08-16 against workspace `edgequake/` (product pin **0.24.4** in crate versions).

| Check | Result |
|-------|--------|
| Typed RelGraph before RelVectors in `merge_with_progress` | Confirmed in source (~1045–1114) |
| Sink batch has no RETURNING / no id map | Confirmed (`upsert_relationships_batch` → `Result<()>`; `by_name.insert` last-wins) |
| EntityNameIndex oldest-wins vs sink HashMap last-wins | Confirmed (`or_insert` in `coverage.rs` vs `insert` in sink) |
| Compensation leaves SQL spine | Confirmed — `compensate_orphan_graph_writes` deletes AGE edges/nodes only |
| `embedding_family` parse contracts | **7/7 passed** (`cargo test -p edgequake-storage --lib embedding_family`) |
| Live dense-corpus reprocess | Optional post-impl; not required to establish LAW-130-4 |

See [08-test-protocol.md](08-test-protocol.md) for executable proofs T1–T6.

## Cross-refs

- Target: [04-target-architecture.md](04-target-architecture.md)
- Matrix: [02-cross-ref-matrix.md](02-cross-ref-matrix.md)
- Intake: [zz-raw.md](zz-raw.md)
