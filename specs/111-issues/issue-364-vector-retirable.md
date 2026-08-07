# issue-364 — Vector drop readiness emptiness gate

**GH:** https://github.com/raphaelmansuy/edgequake/issues/364  
**Status:** Confirmed present on HEAD and last published **v0.24.1**  
**Severity:** P0 (advisor/UX drift vs SQL safety)

## WHY

Operators must know when irreversible DROP is safe. Emptiness of the table being dropped cannot be the readiness signal.

## Code law

| Piece | Behavior |
|-------|----------|
| `count_legacy_chunk_rows` | Live COUNT on legacy tables |
| `chunk_retirable` | Requires count == 0 |
| Migration 126 guard | Requires uncovered == 0 |
| Migration 126 body | DELETE chunks then DROP empty tables |

## Fix (summary)

Retirable := backend typed + uncovered==0 + verify policy. See [04-fix-plan](04-fix-plan.md) Phase C.

## E2E

E2E-111-06, E2E-111-07 in [05-e2e-test-matrix](05-e2e-test-matrix.md).

## Operator note (until fix ships)

If typed coverage is independently verified and you accept backup risk: migration **126 in-SQL guard** is the real safety check on `--confirm-drop`. Advisor RED for “legacy rows un-migrated” alone is **not** proof of uncovered data — but also fix #363/#verify before trusting job status.
