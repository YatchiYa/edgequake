# 00 — Issue data (raw anchors)

Last published product pin: **v0.24.1** (`ghcr.io/raphaelmansuy/edgequake:0.24.1`).  
Investigation HEAD (workspace): see `git rev-parse HEAD` at doc time.

## #364 — vector drop readiness emptiness gate

- URL: https://github.com/raphaelmansuy/edgequake/issues/364  
- Claim: `retirable()` / `fleet_retirable()` require `legacy_chunk_rows == 0` / `legacy_fleet_rows == 0` on live pre-drop tables; backfill does not prune; DROP is what zeros counts.  
- Secondary: `verify_*` numeric equality rejects regenerated embeddings.  
- Env: EdgeQuake 0.23.0, production history.

## #363 — iw2 silent relationship miss

- URL: https://github.com/raphaelmansuy/edgequake/issues/363  
- Claim: `iw2-fleet-embedding-backfill` reports `processed_count ≈ estimated_total`, `failed_count: 0`, while `relationship_embeddings` ≈ 0.3% of legacy relationship keys.  
- Join: exact `entities.name` equality vs normalized legacy keys `SRC->TGT:TYPE`.  
- Env: 0.21→0.23 upgrade, AGE→relational history.

## #362 — KV residue `::text` cast

- URL: https://github.com/raphaelmansuy/edgequake/issues/362  
- Claim: `d.id::text = substring(...)` (and `document_id::text`) defeats UUID PK index → ~326s vs ~1s with reverse cast.  
- Symptom: `advisor kv_durable_residue` statement_timeout → “readiness guard unavailable”.  
- Env: 0.23.0, ~72k KV rows.

## #361 — bulk upload slow

- URL: https://github.com/raphaelmansuy/edgequake/issues/361  
- Claim: multi-document upload/process takes “excessively long”.  
- Env: **0.12.11** Docker — sparse repro steps, no timings.

## #360 — Clear All incomplete

- URL: https://github.com/raphaelmansuy/edgequake/issues/360  
- Claim: after Clear All + confirm, some documents remain visible.  
- Env (form): **0.12.11** Docker.  
- Env (partner clarification 2026-08-06): **actually 0.24.1** — see #366.

## #366 — Clear All incomplete (v0.24.1 pin)

- URL: https://github.com/raphaelmansuy/edgequake/issues/366  
- Claim: identical to #360; leftover documents after Clear All + refresh.  
- Env: **0.24.1** Docker + PostgreSQL.  
- Relationship: same reporter / same bug; correct version pin for the live defect.

## Reproduction method used here

| Issue | Method | Result |
|-------|--------|--------|
| 362–364 | Static code proof + existing e2e/contracts | Fixed on HEAD (Cluster A) |
| 360 / 366 | Code proof: wipe skips residual KV + list suffix-fallback on empty membership | **Confirmed on v0.24.1 / HEAD**; fixed via LAW-111-9 |
| 361 | Capacity architecture + SPEC-090 | **Not a single code bug**; needs measurement |

Partner production DB not available — Cluster A + #366 do not need it (predicates / list merge are in source).
