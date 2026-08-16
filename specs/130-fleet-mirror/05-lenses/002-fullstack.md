# Lens 002 — Full Stack Developer

## Problem

Two writers share a merge session but **not an identity channel**. Sink and mirror both “know” the relationship, yet only by reconstructing it — with divergent name→id policies.

## Hotspots

| File | Change |
|------|--------|
| `edgequake-api/src/postgres_entity_sink.rs` | RETURNING / SELECT ids; report map |
| `edgequake-pipeline/src/merger/mod.rs` | Thread map RelGraph → RelVectors; hint text |
| `edgequake-pipeline/src/merger/relationship.rs` | Build legacy keys aligned with map keys |
| `edgequake-storage/.../fleet_embedding_index.rs` | Prefer known UUID map |
| `edgequake-pipeline` relational sink trait | Return type change |
| Memory / test sinks | Return synthetic maps |

## Implementation rules

1. One legacy-key formatter SSOT (`SRC->TGT:TYPE` + uppercase type) — DRY with `collect_relationship_vector_batch`.
2. In-session mirror: if key in map → use UUID; else miss (fail-closed) or optional offline resolve only when explicitly in coverage mode.
3. Do not add `tokio::sleep` / retry loops as the fix (LAW-130-8).
4. Keep typed RelGraph → RelVectors order (LAW-130-3).
5. Preserve `normalize_relation_type_str` everywhere (LAW-098-3 / LAW-130-6).

## Failure modes to handle

```ascii
  Map miss for a vector row     → count as unresolved (fail-closed)
  Sink missing entity FK        → no id in map; vector should not be eligible
                                 OR both skipped consistently
  ON CONFLICT update            → RETURNING still yields existing id
  Empty rel batch               → empty map; RelVectors no-op OK
```

## Tests owned

T1–T6 in [../08-test-protocol.md](../08-test-protocol.md).

## Cross-refs

- Architecture: [../04-target-architecture.md](../04-target-architecture.md)
- Plan: [../07-implementation-plan.md](../07-implementation-plan.md)
- As-is: [../03-code-as-is.md](../03-code-as-is.md)
