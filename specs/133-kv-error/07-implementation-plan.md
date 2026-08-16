# 07 — Implementation plan

## Work packages

```ascii
  WP-0  Spec pack + evidence (this folder)           Done
  WP-1  parse_relationship_legacy_key_with_resolver  Done
  WP-2  Wire fleet mirror + EntityNameIndex wrapper  Done
  WP-3  Wire iw2 backfill / stamp / coverage          Done
  WP-4  Unit + contract_spec133_target_arrow           Done
  WP-5  e2e_spec133_target_arrow_map_miss            Done
  WP-6  Ops doc residual + CHANGELOG                 Done
  WP-7  cargo test / clippy on touched crates        Done
```

## WP-1 — SSOT helper

File: `edgequake/crates/edgequake-storage/src/embedding_family.rs`

- Add `parse_relationship_legacy_key_with_resolver` per [01-first-principles.md](01-first-principles.md) LAW-133-3.
- Unit tests:
  - zz-raw five keys with `exists` = intended name set → intended parse
  - source-arrow `27_->_25_STRENGTHENING->CLAIM_FRONTIER:STRENGTHENS` still OK
  - no-index / empty exists → falls back to rsplit
  - format↔naive parse still holds for arrow-free names

## WP-2 — Fleet mirror

File: `adapters/postgres/fleet_embedding_index.rs`

On known-map miss:

```rust
let Some((src, tgt, rel_type)) =
    parse_relationship_legacy_key_with_resolver(id, |n| index.resolve(n).is_some())
else { /* ineligible */ };
```

Keep known-map hit path unchanged (LAW-133-4).

## WP-3 — Backfill / stamp

Replace naive parse-before-resolve in:

- `migration_engine/fleet_embedding_backfill.rs`
- `migration_engine/fleet_provenance_stamp.rs`
- any coverage scan that parses legacy rel ids against an index

## WP-4 / WP-5 — Tests

See [08-test-protocol.md](08-test-protocol.md).

## WP-6 — Ops

Update `docs/operations/spec098-entity-spine-ensure.md` residual sentence:
target-arrow class closed by SPEC-133 index-guided parse; reprocess after ship.

## DRY / SOLID checklist

- [x] One resolver parse SSOT
- [x] No duplicated split loops in API/pipeline
- [x] SPEC-130 map remains primary
- [x] Fail-closed preserved
- [x] Edge matrix covered (EC-1…EC-N) — see [10-edge-cases.md](10-edge-cases.md)

## Order / risk

1. WP-1 + unit (safe, pure)
2. WP-2 (hot path)
3. WP-3 (migration parity)
4. WP-4/5 tests
5. WP-6 docs

## Cross-refs

- Architecture: [04-target-architecture.md](04-target-architecture.md)
- Tests: [08-test-protocol.md](08-test-protocol.md)
- Edges: [10-edge-cases.md](10-edge-cases.md)
