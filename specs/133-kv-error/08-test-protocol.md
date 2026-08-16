# 08 — Test protocol

## T1 — Unit (always)

`embedding_family` tests:

| Case | Input | exists set | Expect |
|------|-------|------------|--------|
| Arrow-free | `A->B:TYPE` | {A,B} | (A,B,TYPE) |
| Source-arrow | `27_->_25_STRENGTHENING->CLAIM_FRONTIER:STRENGTHENS` | both endpoints | source-arrow split |
| Target-arrow ×5 | zz-raw keys | intended endpoints | intended parse |
| Empty exists | any multi-arrow | ∅ | naive rsplit fallback |
| Classify | target-arrow key | — | `EmbeddingFamily::Relationship` |

## T2 — Contract postgres (`contract_spec133_fleet_mirror_target_arrow`)

1. Seed workspace entities including target names with `->`.
2. Seed `relationships` row for intended pair.
3. Call `mirror_legacy_batch` with vector id = `format(src,tgt,rel)` and **no** known map.
4. Expect `report.is_complete()` and `misses` empty.
5. Control: wrong exists-only world still misses (fail-closed).

Skip if `DATABASE_URL` unset (same pattern as `contract_spec091_fleet_mirror_fk`).

## T3 — e2e sink map miss (`e2e_spec133_…` or extend `e2e_spec130_…`)

1. Build sink map **without** the colliding key (simulate incomplete map).
2. Ensure entities + relationship spine exist.
3. Mirror must still resolve via index-guided parse.
4. With neither map nor matching entities → fail-closed miss.

## T4 — Regression

- `contract_iw2_parse_relationship_key_arrow_in_source`
- `contract_spec091_fleet_mirror_arrow_in_source_name`
- `e2e_spec130_sink_returning_mirror`

## T5 — Optional live

Reprocess `0001_Note_manuscrite.pdf` (or equivalent) after deploy; expect no `995/1000` arrow miss class.

## Commands

```bash
cargo test -p edgequake-storage --lib parse_relationship_legacy_key
cargo test -p edgequake-storage --lib contract_spec133_index_parse
cargo test -p edgequake-storage --features postgres --test contract_spec133_fleet_mirror_target_arrow
cargo test -p edgequake-storage --features postgres --test contract_spec091_fleet_mirror_fk
cargo test -p edgequake-api --features postgres --test e2e_spec133_target_arrow_map_miss
cargo test -p edgequake-api --features postgres --test e2e_spec130_sink_returning_mirror
```

## Cross-refs

- Acceptance: [09-acceptance.md](09-acceptance.md)
- Edges: [10-edge-cases.md](10-edge-cases.md)
