# Lens 002 — Full Stack Developer

## Surfaces

| Layer | Touch |
|-------|-------|
| Storage | `embedding_family` parse + `fleet_embedding_index` fallback |
| Pipeline | Fail-closed message clarity (optional); keep RelGraph→RelVectors order |
| API | Sink RETURNING map unchanged (SPEC-130) |
| Tasks | Classifier already maps fleet miss → `GraphMerge` permanent |
| WebUI | Failed banner already shows message; optional miss-class chip later |

## Implementation notes

- Prefer small helper + wire call sites over a new crate.
- Keep `parse_relationship_legacy_key` for `classify_legacy_vector_id` (shape test only).
- Do not duplicate format/parse in pipeline or API.

## Test hooks

- Unit: zz-raw five keys with mock `exists`.
- Contract: postgres fleet mirror with seeded entities whose names contain `->`.
- e2e: map intentionally incomplete → index parse still mirrors (extend SPEC-130 pattern).

## DRY / SOLID checklist

- [x] Single resolver parse SSOT
- [x] Mirror / backfill / stamp share it
- [x] No second HashMap of ad-hoc splits in callers
- [x] SPEC-130 path still preferred when map hits

## Cross-refs

- Code as-is: [../03-code-as-is.md](../03-code-as-is.md)
- Plan: [../07-implementation-plan.md](../07-implementation-plan.md)
- Tests: [../08-test-protocol.md](../08-test-protocol.md)
