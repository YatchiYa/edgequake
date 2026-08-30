# 05 — E2E test matrix

Proof target: `make spec139-migrate-engine-proof`

Unfakable means: **cannot** pass by mocking the insert, asserting `written > 0`
without a 21000 fixture, or using a single vector table for the max-vs-sum bug.

| ID | Gate | Where |
|----|------|--------|
| U-139-DEDUP | Last-write-wins on conflict key; unique keys keep order | `conflict_dedupe` unit |
| E2E-139-01 | Two `entity:` ids that normalize to one spine in one batch. Raw DO UPDATE SQL raises **21000**. Patched `run_batch` commits; one typed row; provenance set | `e2e_spec139_engine` |
| E2E-139-02 | Two relationship keys that normalize to the same `(src,tgt,rel)`. Raw DO UPDATE raises **21000**. Patched path writes one row | `e2e_spec139_engine` |
| E2E-139-03 | Two vector tables with 3+5 **uncovered** chunks plus ≥10 unrelated typed rows **and** a malformed `not-a-uuid-chunk-0`. After fix `expected=8` (UUID shape only), `actual=0` then `8` | `e2e_spec139_engine` |
| E2E-139-04 | Plant W3 job `failed` + `verify_failed`; reclaim → `claim_lease` succeeds | `e2e_spec139_engine` |
| E2E-139-05 | Lineage KV, no document; 119-shaped insert skips; 122-shaped shell; remainder inserts artifact | `e2e_spec139_engine` |
| E2E-139-06 | First job `run_batch` Err; second job still runs (`run_engine`) | `e2e_spec139_engine` |
| E2E-139-07 | Metadata KV, no document; `wc-shell-remainder` inserts `documents`; `doc_shells=0` | `e2e_spec139_engine` |
| E2E-139-08 | Orphan lineage (no parent); remainder `verify` **passes**; advisor residue lineage ≥ 1 | `e2e_spec139_engine` |
| Contract | `contract_spec091_advisor_matches_125_guard` / 126; 131 abort source + `e2e_spec111_17_abort_without_provenance` | existing tests |

## Skip policy

Lib-only (U-139-DEDUP) may run without Postgres.

**Proof is fail-closed:** `make spec139-migrate-engine-proof` requires
`DATABASE_URL` (or `/tmp/edgequake-db-url`), sets
`EDGEQUAKE_REQUIRE_POSTGRES_TESTS=1`, and fails if any e2e is skipped/ignored
or if `UNFAKABLE` facts are missing from `measurements/e2e139-engine.txt`.
