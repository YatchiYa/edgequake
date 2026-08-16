# SPEC-130 — Fleet mirror relationship identity (#380)

> **Mission:** Stop typed RelVectors fleet mirror from re-guessing `relationships.id` by name after the sink already wrote the edge — pass sink UUIDs into the mirror so dense-document KG persist stops fail-closing with `resolved 0/N` on every retry.
>
> **Trigger:** [GitHub #380](https://github.com/raphaelmansuy/edgequake/issues/380) (intake [`zz-raw.md`](zz-raw.md)).

## Short verdict

| Layer | Finding |
|-------|---------|
| Symptom | `SPEC-091: typed fleet mirror resolved 0/N` with relationship miss samples |
| Reporter claim | Unordered RelGraph ↔ RelVectors race; SELECT ~1s before INSERT |
| Code fact | Typed path already **RelGraph → RelVectors** (await); ~1s `created_at` gap expected |
| Retries | Identical misses + leftover SQL spine ⇒ **not** a pure visibility race |
| Root gap | Sink **discards** relationship UUID; mirror **re-resolves** via `EntityNameIndex` |
| Fix | Sink RETURNING / map → mirror by UUID; keep order; fix error hint |
| Non-fix | Sleep/retry as primary; soften fail-closed; widen DDL |

```ascii
  TODAY                         TARGET
  -----                         ------
  sink INSERT ──► discard id    sink INSERT … RETURNING id ──► map
  mirror SELECT by name ──X──►  mirror upsert by Uuid ────────► OK
```

## Document map

```ascii
 00-why
  → 01-first-principles (LAW-130-*)
  → 02-cross-ref-matrix
  → 03-code-as-is
  → 04-target-architecture
  → 05-lenses/ (PO, fullstack, DB, UX, front, AI)
  → 06-ux-ui-spec
  → 07-implementation-plan
  → 08-test-protocol
  → 09-acceptance
  → 10-edge-cases
  → 11-honest-assessment
  → zz-raw.md (intake, not the contract)
```

## Status board

| ID | Item | Status |
|----|------|--------|
| D0 | Intake `zz-raw.md` / #380 | Done |
| D1 | Doc pack (this folder) | Done |
| C1 | GitHub #380 honest RC comment | Done |
| I0 | Sink returns relationship id map | Done |
| I1 | RelVectors mirror prefers map | Done |
| I2 | Fail-closed hint rewrite | Done |
| T1–T6 | Order / identity-map / happy-path / hint / offline / arrow | Done |

## Related

- [#380](https://github.com/raphaelmansuy/edgequake/issues/380) — this bug
- [SPEC-091](../091-simplify-data-layer/) — typed fleet / graph-before-fleet
- [SPEC-098](../098-data-access-hardening/) — spine before fleet; LAW-098-1…4
- SPEC-120 — EntityNameIndex oldest-wins (divergence vs sink last-wins)
- [SPEC-129](../129-touchd_document_faill/) — sibling pack pattern

## Non-goals

- Treating bounded retry/sleep in `resolve_relationship_id_pool` as the fix
- Marking GraphMerge non-permanent to “absorb” identity bugs
- Changing AGE edge semantics or adding DDL for this fix
- Fixing placeholder-endpoint entity spine (residual sibling)
- UI redesign beyond honest Failed / miss-sample copy

## Cross-refs

- Why: [00-why.md](00-why.md)
- Laws: [01-first-principles.md](01-first-principles.md)
- Plan: [07-implementation-plan.md](07-implementation-plan.md)
