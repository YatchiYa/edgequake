# 04 — Fix plan

## Locked approach

| Step | Action | Why |
|------|--------|-----|
| 1 | `dedupe_last_write_wins` on iw2 entity/rel/report batches | LAW-139-1 |
| 2 | Keep COALESCE provenance DO UPDATE; catch 21000 as defense | LAW-139-2 |
| 3 | W3 verify: per-table coverage COUNT, SUM actual | LAW-139-3 |
| 4 | Default `passes()` = coverage; equality only if env `=1` | Field mismatches |
| 5 | `reclaim_verify_failed_jobs` + reset cursor | LAW-139-4 |
| 6 | `run_engine` continues after job Err | LAW-139-5 |
| 7 | `w5-artifact-remainder` + `wc-shell-remainder` + `w2-dedup-remainder` | LAW-139-6 |
| 8 | Guard 42P01 typed-SSOT message names `edgequake migrate` | Honesty |
| 9 | E2E-139-01..08 + `make spec139-migrate-engine-proof` | LAW-139-7 |
| 10 | CHANGELOG Unreleased + `upgrade-to-0.26.3.md` | Honesty |

## Rejected alternatives

| Idea | Reject reason |
|------|----------------|
| Change DO UPDATE → DO NOTHING | Loses provenance stamp-on-copy |
| Widen `entity_embeddings` PK | Schema train; out of scope vs stamp/stall |
| Edit applied 119/117 | LAW-111-10 |
| Skip 125 if residue is “only aliases” | Destroys uncovered keys |
| Leave W3 `failed` + ops SQL only | Field will not discover the UPDATE |

## SOLID mapping

- **S:** helper / verify / remainder / runner isolation / DROP SQL.
- **O:** new DO UPDATE families call the same helper.
- **D:** e2e drives real Postgres, not a second parser.

## Implementation notes

- Do not change iw2 `DESCRIPTOR_DEF` without bumping `schema_generation` (field
  iw2 is still `preflight` — same sha reclaim works).
- Remainder jobs are new `step_id`s (fresh ledger rows).
- Reclaim only `last_error ? 'verify_failed'`, not crash loops.

## Acceptance

- [x] Spec pack (this directory)
- [x] iw2 dedupe + 21000 fixture
- [x] W3 coverage-sum verify + reclaim
- [x] Remainder jobs (dedup / shell / artifact, copy-complete verify)
- [x] Engine isolation + guard message
- [x] Proof target
- [x] E2E-139-07/08 + 131 abort source + GHCR honesty (Unreleased until tag)
