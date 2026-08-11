# 13 — Close Decision (First Principles)

> Decision record after applying [`12-first-principles-gap-closability.md`](12-first-principles-gap-closability.md).

## Rule applied

Close a gap **only if** it is required for Product A (live ingest / #374) **and** does not falsify Product B/C.

## Decisions

| Gap | Necessary for Product A? | Action taken |
|-----|--------------------------|--------------|
| G1 Alias spine merge | No | **Deferred** → SPEC-083 (no code) |
| G2 Loser FK without embedding | No (residue of G1) | **Accepted** |
| G3 Unify stamp + live absorb | No — would break cutover | **Never** |
| G4 Historical / mig 131 | No | **Deferred** → SPEC-111 ops |
| G5 HTTP upload worker e2e | No (merger bound enough) | **Deferred** soak |
| Rel concurrent merger proof | **Yes** — #374 reports relationship lids too | **Closed** — `e2e_spec120_concurrent_merger_same_relationship_no_graph_merge` |
| Family FK metadata contract | DRY/SOLID guard for absorb OCP | **Closed** — `contract_spec120_family_typed_fk_metadata` |

## Product A close-box (complete)

```ascii
  LAW-120-1 Bookkeeping ≠ content     ✓ absorb Ok
  LAW-120-2 One lid owner             ✓ unique kept
  LAW-120-3 Arbiter completeness      ✓ fleet_legacy_absorb
  LAW-120-4 Exact-name create safe    ✓ sink (pre-existing)
  LAW-120-5 Alias debt fenced         ✓ deferred SPEC-083
  LAW-120-6 No compensate on absorb   ✓ Ok path
  LAW-120-7 Concurrency proof         ✓ contract + mirror + merger entity/rel
```

## Explicit non-actions (honesty)

- Did **not** normalize `ensure_entity_spine` display names (migration path → SPEC-083/111).
- Did **not** soft-Ok `fleet_provenance_stamp` 23505.
- Did **not** add HTTP dual-upload e2e.
- Did **not** auto-merge historical alias rows.
