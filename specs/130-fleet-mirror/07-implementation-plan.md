# 07 — Implementation plan

## Work packages

```ascii
  WP-0  Spec pack + GitHub #380 comment     (docs) ← this mission
  WP-1  Sink / trait return RelationshipSinkReport (id map)
  WP-2  Postgres INSERT … ON CONFLICT … RETURNING id (SSOT)
  WP-3  Merger RelVectors prefer map in fleet mirror API
  WP-4  Fail-closed error hint rewrite (rel identity)
  WP-5  e2e / contracts T1–T6
  WP-6  cargo test / clippy on touched crates
```

## WP-0 — Docs + comment

- Pack under `specs/130-fleet-mirror/` (LAW-130, lenses, edges).
- Post honest RC on [#380](https://github.com/raphaelmansuy/edgequake/issues/380).

## WP-1 — Trait return type

Files:

- `edgequake-pipeline/src/merger/mod.rs` — `RelationalEntitySink` / `upsert_relationships_batch` signature
- Memory / mock sinks implementing the trait
- Call sites in `relationship.rs`

Behavior:

- Return `RelationshipSinkReport { ids: HashMap<String, Uuid>, missing_fk: u64 }` (names may vary).
- Map keys = legacy id format from shared helper (same as vector ids).

## WP-2 — Postgres sink RETURNING

File: `edgequake-api/src/postgres_entity_sink.rs`

- Change batch INSERT to `RETURNING id, source_id, target_id, relation_type` (or join names) and build map.
- ON CONFLICT DO UPDATE must still return the row id.
- Align bare-name resolution policy with EntityNameIndex **or** rely solely on returned ids for mirror (preferred: returned ids).

## WP-3 — Mirror prefers map

Files:

- `edgequake-storage/.../traits/domain/fleet_embedding_index.rs`
- `fleet_embedding_index.rs` Postgres impl
- `merger/mod.rs` `upsert_vectors_chunked` / RelVectors call

```ascii
  for each relationship legacy row:
    if let Some(rid) = known_ids.get(legacy_id) { use rid }
    else { miss / optional coverage resolve }
```

Do not call `resolve_relationship_id_pool` for in-session mapped keys.

## WP-4 — Hint

File: `merger/mod.rs` (~621–630)

- Relationship chunk: identity / sink-map language (LAW-130-7).
- Entity chunk: keep entity-spine language.

## WP-5 — Tests

See [08-test-protocol.md](08-test-protocol.md).

## WP-6 — Gates

```bash
cargo fmt --check
cargo clippy -p edgequake-pipeline -p edgequake-storage -p edgequake-api --all-targets -- -D warnings
cargo test -p edgequake-pipeline --lib
cargo test -p edgequake-storage --features postgres --test contract_spec091_fleet_mirror_fk
# + new e2e_spec130_* tests
```

## DRY / SOLID checklist

- [x] One legacy-key formatter shared sink ↔ vectors (`format_relationship_legacy_key`)
- [x] One identity producer (sink report)
- [x] In-session mirror does not duplicate name resolve (map preferred)
- [x] Coverage/migration keep name resolve (LAW-130-5)
- [x] Rel type uppercase SSOT unchanged
- [x] No sleep/retry primary path
- [x] Typed RelGraph → RelVectors order unchanged
- [x] Hint copy corrected

## Cross-refs

- Architecture: [04-target-architecture.md](04-target-architecture.md)
- Tests: [08-test-protocol.md](08-test-protocol.md)
- Fullstack lens: [05-lenses/002-fullstack.md](05-lenses/002-fullstack.md)
