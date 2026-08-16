# 02 — Cross-ref matrix

| Ref | Role for SPEC-133 |
|-----|-------------------|
| UI Failed 995/1000 | Trigger — [zz-raw.md](zz-raw.md), [evidence/](evidence/) |
| SPEC-091 IW2 | Typed fleet mirror fail-closed when `resolved < eligible` |
| SPEC-098 | Spine ensure; ops near-miss note; `rsplit` source-arrow fix |
| SPEC-111 | `EntityNameIndex` + coverage resolve helpers |
| SPEC-120 | Concurrent mirror absorb (orthogonal) |
| SPEC-130 | Sink RETURNING `HashMap<legacy_id, uuid>` fast path |
| SPEC-106 | Unrelated KG persist (AGE join) |
| CHANGELOG v0.24.2 | Documents residual: target names with `->` stay ambiguous |
| `docs/operations/spec098-entity-spine-ensure.md` | Operator guidance for near-miss class |
| Wikipedia delimiter collision | First-principles framing for LAW-133-1 |

```ascii
  Extraction (AI)
       │ names may contain "->"
       ▼
  format_relationship_legacy_key   ──► RelVectors.id (Plane B)
       │
       ├── RelGraph + PostgresEntitySink RETURNING ──► Plane C (SPEC-130)
       │
       └── fleet mirror
              ├── known.get(id)           → UUID (preferred)
              └── parse + EntityNameIndex → UUID (SPEC-133 disambiguate)
                     │
                     X (bug) naive rsplit when tgt contains "->"
```

## Doc ↔ code anchors

| Concern | Path |
|---------|------|
| Format / parse SSOT | `edgequake-storage/src/embedding_family.rs` |
| Fleet mirror | `edgequake-storage/src/adapters/postgres/fleet_embedding_index.rs` |
| Name index | `edgequake-storage/src/migration_engine/coverage.rs` |
| Sink RETURNING map | `edgequake-api/src/postgres_entity_sink.rs` |
| RelVectors collect | `edgequake-pipeline/src/merger/relationship.rs` |
| Fail-closed message | `edgequake-pipeline/src/merger/mod.rs` (`upsert_vectors_chunked`) |
| Source-arrow contract | `contract_spec091_fleet_mirror_fk.rs` / unit in `embedding_family` |
| Classifier | `edgequake-tasks/src/ingestion_reliability.rs` (`GraphMerge`) |

## Cross-refs

- Code as-is: [03-code-as-is.md](03-code-as-is.md)
- Target: [04-target-architecture.md](04-target-architecture.md)
