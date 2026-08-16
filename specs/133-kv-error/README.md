# SPEC-133 — Relationship legacy-key delimiter collision (fleet mirror near-miss)

> **Mission:** Make typed fleet mirror survive entity names that contain the legacy
> relationship delimiter `->`, so manuscript / diagram PDFs no longer fail at
> `resolved N-ε / N` with SPEC-098 miss samples that look like valid edges.
>
> **Trigger:** UI Failed on `0001_Note_manuscrite.pdf` — `typed fleet mirror resolved 995/1000`
> with five relationship keys whose **targets** contain `->` ([zz-raw.md](zz-raw.md)).

## Short verdict

| Layer | Finding |
|-------|---------|
| Gap | `format_relationship_legacy_key` / `parse_relationship_legacy_key` use raw `->` + `:` without escaping — classic **delimiter collision** |
| Prior fix | `rsplit_once("->")` fixed **source**-contains-arrow (v0.24.2 / SPEC-098); CHANGELOG already names the residual |
| Residual | When **target** contains `->`, last-arrow parse invents wrong `(src,tgt)` → FK miss |
| In-session | SPEC-130 UUID map bypasses parse **only when** the key is in the sink map; map miss / empty map falls back to broken parse |
| Fix | Index-guided split SSOT (both endpoints must resolve in `entities`) + keep SPEC-130 UUID fast path + unit/e2e for screenshot keys |
| Non-fix | Rewriting all historical vector ids; sanitizing LLM entity names to forbid `->`; widening fail-open |

```ascii
  format(src, tgt, rel) = "{src}->{tgt}:{REL}"     (lossy when src/tgt contain "->")

  parse rsplit (today):
    LEFT_MARGIN->LEFT_MARGIN_VALUE_1->_00_->_+:RELATED_TO
         └────────── wrong ──────────────────┘  └wrong┘
         intended: LEFT_MARGIN | LEFT_MARGIN_VALUE_1->_00_->_+

  parse index-guided (fix):
    try every "->" split; keep splits where entities resolve both sides
         → unique both-resolve → correct endpoints
```

## Document map

```ascii
 00-why
  → 01-first-principles (LAW-133-*)
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
  → zz-raw.md (intake)
  → evidence/
```

## Status board

| ID | Item | Status |
|----|------|--------|
| D0 | Intake + reproduction (5 screenshot keys) | Done |
| D1 | Doc pack (this folder) | Done |
| I0 | `parse_relationship_legacy_key_with_resolver` SSOT | Done |
| I0b | `EntityNameIndex::parse_relationship_legacy_key` DRY wrapper | Done |
| I1 | Wire fleet mirror + iw2/backfill/stamp/coverage | Done |
| I2 | Unit + `contract_spec133_fleet_mirror_target_arrow` | Done |
| I3 | `e2e_spec133_target_arrow_map_miss` (+ fail-closed) | Done |
| I4 | Ops residual closed + CHANGELOG | Done |
| C1 | Live reprocess of manuscript PDF | Operator after deploy |

## Related

- [SPEC-091](../091-simplify-data-layer/) — typed fleet mirror fail-closed
- [SPEC-098](../098-data-access-hardening/) — spine ensure; near-miss ops note
- [SPEC-111](../111-issues/) — coverage / name index
- [SPEC-120](../120-race-condition-legacy-vector/) — concurrent mirror absorb
- [SPEC-130](../130-fleet-mirror/) — sink RETURNING UUID map (in-session)
- [SPEC-106](../106-kg-persist-bug/) — different KG persist class (AGE graphid)
- Ops: [`docs/operations/spec098-entity-spine-ensure.md`](../../docs/operations/spec098-entity-spine-ensure.md)

## Non-goals

- Escaping / versioned key format migration for all stored `legacy_vector_id` rows (optional follow-up)
- Softening fail-closed mirror (`resolved < eligible` stays fatal under typed authority)
- Forbidding `->` in extracted entity names (AI lens may recommend later)
- Treating this as missing spine / re-running migrations 139–140 solely for this class
