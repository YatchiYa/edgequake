# 08 — Test protocol

## Static reproduction (done in investigation)

| Check | Command / method | Result (2026-08-16) |
|-------|------------------|---------------------|
| RelGraph before RelVectors (typed) | Read `merger/mod.rs` ~1045–1114 | Pass — await order |
| Sink returns `()` only | Read `postgres_entity_sink.rs` batch | Pass — identity discarded (`Result<()>` / no RETURNING) |
| Index oldest vs sink last | `EntityNameIndex::from_rows` uses `or_insert` (oldest); sink `by_name.insert` (last) | Pass — divergence class |
| Compensation omits SQL spine | `compensation.rs` rolls back AGE nodes/edges + vectors only — no `DELETE FROM public.relationships` | Pass — leftover SQL edges expected (LAW-130-9) |
| Legacy key parse (`->` in source) | `cargo test -p edgequake-storage --lib embedding_family` | **7/7 ok** (incl. `contract_iw2_parse_relationship_key_arrow_in_source`) |
| Merge progress phases emit | `cargo test -p edgequake-pipeline --lib test_merge_with_progress_emits_phases` | **ok** |
| FK miss → permanent GraphMerge | `cargo test -p edgequake-tasks --lib typed_fleet_mirror` | **ok** (`typed_fleet_mirror_fk_miss_is_graph_merge`) |

## Executable gates (WP-5)

### T1 — Order invariant

```bash
cargo test -p edgequake-pipeline --lib -- merge_with_progress
# Add contract: under typed env, when both rel graph + rel vectors fire,
# phase list index(RelationshipGraph) < index(RelationshipVectors)
```

### T2 — Name-resolve miss → UUID map fix

Postgres e2e (`e2e_spec130_rel_identity_map`):

1. Insert entities + relationship; capture `relationships.id`.
2. Rename entity endpoints so `EntityNameIndex` / `resolve_relationship_id_pool` miss.
3. `mirror_legacy_batch(..., None)` → `resolved == 0`.
4. `mirror_legacy_batch(..., Some(map))` → `resolved == eligible`.

(Note: `entities_unique_name` prevents same-workspace duplicate bare names; rename proves the same identity class.)

```bash
cargo test -p edgequake-storage --features postgres --test e2e_spec130_rel_identity_map
```

### T3 — Happy path sink map

```bash
cargo test -p edgequake-api --features postgres --test e2e_spec130_sink_returning_mirror
```

### T4 — Hint source contract

```bash
# Grep / contract test: fail-closed string mentions sink-returned relationships.id
cargo test -p edgequake-pipeline --lib -- spec130_hint
```

### T5 — Offline name resolve preserved

```bash
cargo test -p edgequake-storage --features postgres --test contract_spec098_fleet_mirror_report
cargo test -p edgequake-storage --lib -- resolve_relationship_id_pool
```

### T6 — Arrow in source name + map

```bash
cargo test -p edgequake-storage --features postgres --test contract_spec091_fleet_mirror_fk
# Extend or add e2e_spec130_arrow_with_map
```

## Live optional (post-impl)

```bash
export EDGEQUAKE_TASK_MAX_WORKERS=1
# reprocess one dense failed doc; expect Completed, not identical 0/N
```

## Cross-refs

- Edges: [10-edge-cases.md](10-edge-cases.md)
- Acceptance: [09-acceptance.md](09-acceptance.md)
- As-is evidence: [03-code-as-is.md](03-code-as-is.md)
