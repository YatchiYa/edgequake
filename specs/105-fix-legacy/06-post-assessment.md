# 06 — Post Assessment (SPEC-105)

> **As-of:** 2026-08-03 · after Waves A–E land (+ 142 mid-upgrade deferral).

## Grades

| Gate | Grade | Notes |
|------|-------|-------|
| LAW-L2 unknown→Typed | **A** | `vector_backend_from_env` + E2E-105-01 |
| LAW-L4 census SSOT | **A** | `legacy_store_census` + cutover posture |
| LAW-L5 mig 142 | **A** | Abort on rows; DROP empty; posture marker; **defer while residue** so expandable migrate/boot soft-exit |
| LAW-L3/L6 era-aware | **A** | INV dual retained; FTS KV gated; workspace typed-first |
| ≤0.22 upgrade | **A−** | Ladder documented; soak (`spec091-upgrade-soak`) remains CI/ops proof |

## Evidence

- `cargo test -p edgequake-storage --features postgres --lib e2e_105`
- `cargo test -p edgequake-api --features postgres --lib e2e_105_07`
- `cargo test -p edgequake-api --features postgres --test contract_spec105_legacy`
- `cargo test -p edgequake-api --features postgres --test contract_spec104_datalayer`
- `cargo test -p edgequake-api --features postgres --test e2e_spec024_operational_excellence`
- Live: `edgequake migrate` with residue soft-exits (`131`+`142` pending) without applying 142

## Residuals

- Full `no_kv_facade` allowlist shrink (out of scope)
- Operator must still run `make spec091-upgrade-soak` / spec93 for full ≤0.22 realism proof on each release cut
