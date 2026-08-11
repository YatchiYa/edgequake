# 11 — Honest Assessment (final)

> Closability study: [`12-first-principles-gap-closability.md`](12-first-principles-gap-closability.md)  
> Close decisions: [`13-close-decision.md`](13-close-decision.md)

## What is proven (Product A / #374)

| Claim | Evidence |
|-------|----------|
| Dual-FK same lid no longer 23505 | `contract_spec120_legacy_vector_id_race` (entity/rel/report) |
| Stamp-once + multi-WS 144 invariant | Same contract |
| Concurrent **merger** entity path `errors == 0` | `e2e_spec120_concurrent_merger_same_entity_no_graph_merge` |
| Concurrent **merger** relationship path `errors == 0` | `e2e_spec120_concurrent_merger_same_relationship_no_graph_merge` |
| Storage mirror race Ok | `e2e_spec120_concurrent_mirror_same_entity` |
| One absorb policy (DRY) | `fleet_legacy_absorb.rs` + family FK metadata contract |

## What First Principles says we must **not** claim closed

| Gap | Status | Why |
|-----|--------|-----|
| Alias entity auto-merge | Deferred SPEC-083 | Different product (identity) |
| Loser FK may lack embedding | Accepted residue | Fixed only via alias merge |
| Stamp job soft-Ok on 23505 | **Never** | Would false-GREEN cutover |
| Historical unstamped / 131 | Deferred SPEC-111 | Ops cutover |
| HTTP upload+worker dual-doc | Deferred soak | Merger bound suffices for LAW-120-7 |

## Scorecard

| Dimension | Score | Note |
|-----------|-------|------|
| Diagnosis | 9/10 | Nuanced vs issue text |
| Symptom fix (#374) | **9/10** | Entity + relationship merger e2e |
| Durable identity | 3/10 | Correctly out of SPEC-120 |
| Test honesty | **9/10** | Contract + mirror + merger entity/rel; HTTP skipped on purpose |
| DRY/SOLID | **8/10** | Absorb module + family metadata |
| Ship state | 4/10 | Still unmerged locally |

## Bottom line

**Product A is First-Principles-complete for #374.** Remaining “gaps” are Product B/C or optional soak — closing them under SPEC-120 would either be scope creep or actively harmful (stamp soft-Ok). Ship = commit/PR + close the issue after merge.
