# 02 — Cross-ref Matrix

## Claim → Authority

| Claim | Authority |
|-------|-----------|
| Concurrent ingest fails on legacy unique | [#374](https://github.com/raphaelmansuy/edgequake/issues/374); local SQL repro (`10-reproduction.md`) |
| Unique is `(workspace_id, legacy_vector_id)` | Migration 144; `idx_*_embeddings_legacy_vector_id` |
| Mirror INSERT only ON CONFLICT PK | `fleet_embedding_index.rs` `upsert_batch` |
| One ON CONFLICT target per statement | [PostgreSQL INSERT](https://www.postgresql.org/docs/current/sql-insert.html) |
| Exact-name entity create race-safe | `postgres_entity_sink.rs` + `entities_unique_name` |
| EntityNameIndex load unordered | `coverage.rs` `load_entity_name_index_pool` |
| Fail-closed merge on mirror error | `merger/mod.rs` + `ingestion_persister.rs` |
| Stamp job fails closed on 23505 | `fleet_provenance_stamp.rs` (migration path ≠ live absorb) |
| Fix laws | SPEC-120 LAW-120-1..7 |

## Code SSOT (as-is → target)

| Concern | Path |
|---------|------|
| Fleet upsert / mirror | `edgequake-storage/.../postgres/fleet_embedding_index.rs` |
| Absorb conflict policy (DRY) | `edgequake-storage/.../postgres/fleet_legacy_absorb.rs` |
| Family table/FK metadata | `edgequake-storage/.../embedding_family.rs` |
| Merger concurrent e2e | `edgequake-api/tests/e2e_spec120_concurrent_merger_mirror.rs` |
| Mirror report type | `edgequake-storage/.../traits/domain/fleet_embedding_index.rs` |
| Name resolve index | `edgequake-storage/.../migration_engine/coverage.rs` |
| Merge call site | `edgequake-pipeline/.../merger/mod.rs` |
| Entity/rel sink UNIQUE | `edgequake-api/.../postgres_entity_sink.rs` |
| GraphMerge + compensate | `edgequake-pipeline/.../ingestion_persister.rs` + `compensation.rs` |
| Index DDL | `migrations/143_*.sql`, `144_*.sql` |

## Related specs / issues

| Spec / Issue | Relationship |
|--------------|--------------|
| GH #374 | This mission |
| SPEC-111 (143/144) | Introduced / rescoped legacy unique |
| SPEC-091 | Fleet mirror / typed embeddings |
| SPEC-098 | Typed authority; fail-closed when mirror incomplete |
| SPEC-083 graph-identity | Alias / normalize completeness (follow-up) |
| GH #362–#364 | Same subsystem Cluster A; fixed separately |

## DRY rule

One absorb helper serves **entity / relationship / report** families. Do not fork three incompatible ON CONFLICT strategies. Conflict absorption for live ingest lives in `upsert_batch`; do not silently change `fleet_provenance_stamp` fail-closed dual-legacy semantics without a separate mission.
