# Lens 002 — Full Stack Developer

## Touch list

| Layer | File | Change |
|-------|------|--------|
| Storage | `fleet_embedding_index.rs` | DRY absorb upsert for 3 families |
| Trait | `fleet_embedding_index.rs` (domain) | `absorbed_legacy_collisions` on report |
| Resolve | `coverage.rs` | `ORDER BY created_at, id` |
| Tests | `contract_spec120_*`, `e2e_spec120_*` | Race + stamp-once + multi-WS |
| Pipeline | none required if absorb returns Ok | Verify fail path not hit |

## SOLID / DRY

- **S:** Absorb helper owns conflict policy; mirror owns resolve/build.
- **O:** New families reuse helper; no copy-paste SQL.
- **D:** Callers depend on `FleetEmbeddingIndex`; absorb is adapter detail.
- **DRY:** One SQL strategy for entity/rel/report.

## Pitfalls

- Targetless `ON CONFLICT DO NOTHING` skips COALESCE — hence **UPDATE first**.
- Do not mark mirror incomplete when absorb skips a lid (resolved count already counted FK hits before upsert).
- Keep multi-WS same-lid green (144).
