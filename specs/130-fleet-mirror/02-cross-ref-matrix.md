# 02 — Cross-ref matrix

## Specs / issues

| ID | Role |
|----|------|
| [#380](https://github.com/raphaelmansuy/edgequake/issues/380) | Trigger — fleet mirror 0/N relationship misses |
| [SPEC-091](../091-simplify-data-layer/) | Typed fleet, graph-before-fleet, fail-closed mirror |
| [SPEC-098](../098-data-access-hardening/) | Spine before fleet, relation type SSOT, miss samples |
| SPEC-120 | EntityNameIndex oldest-wins; concurrent lid absorb |
| [SPEC-129](../129-touchd_document_faill/) | Sibling doc-pack pattern (unrelated CHECK bug) |

## Code anchors

| Concern | Path |
|---------|------|
| Typed phase order | `edgequake-pipeline/src/merger/mod.rs` (`merge_with_progress`) |
| Rel vector id build | `edgequake-pipeline/src/merger/relationship.rs` (`collect_relationship_vector_batch`) |
| Rel sink call | `relationship.rs` (`upsert_relationships_batch` after AGE) |
| Sink INSERT | `edgequake-api/src/postgres_entity_sink.rs` |
| Name resolve | `edgequake-storage/.../coverage.rs` (`resolve_relationship_id_pool`) |
| EntityNameIndex | `coverage.rs` (`from_rows` / `resolve` — oldest-wins) |
| Mirror | `edgequake-storage/.../fleet_embedding_index.rs` (`mirror_legacy_batch`) |
| Fail-closed hint | `merger/mod.rs` (`upsert_vectors_chunked`) |
| Permanent class | `edgequake-tasks/.../ingestion_reliability.rs` (`typed fleet mirror` → GraphMerge) |
| Parse legacy key | `edgequake-storage/src/embedding_family.rs` (`parse_relationship_legacy_key`) |

## Existing tests (pre-SPEC-130)

| Test | Proves |
|------|--------|
| `contract_spec091_fleet_mirror_fk` | Bare/scoped names; `->` in source |
| `contract_spec098_fleet_mirror_report` | Miss sample / invalid workspace / rel case |
| `e2e_spec098_saturated_spine_ensure` | Missing entity spine → miss; ensure → resolve |
| `e2e_spec098_relation_type_case` | Uppercase SSOT |
| `e2e_spec120_concurrent_mirror_same_entity` | Concurrent entity mirror |
| `typed_fleet_mirror_fk_miss_is_graph_merge` | Permanent classification |

## SPEC-130 proof IDs (planned)

| ID | Proof |
|----|-------|
| T1 | Typed RelGraph before RelVectors order (contract/unit) |
| T2 | Duplicate-name deterministic miss under name resolve; UUID map fixes |
| T3 | Sink RETURNING / map → `resolved == eligible` happy path |
| T4 | Error hint mentions relationship identity (source contract) |
| T5 | Offline/coverage still uses name resolve |
| T6 | Arrow-in-source-name + UUID path (parity with SPEC-091) |

## Cross-refs

- Laws: [01-first-principles.md](01-first-principles.md)
- Code as-is: [03-code-as-is.md](03-code-as-is.md)
- Tests: [08-test-protocol.md](08-test-protocol.md)
