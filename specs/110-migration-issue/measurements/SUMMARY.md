# SPEC-110 measurements — SUMMARY (brutal assessment)

> Date: 2026-08-06  
> Proof command: `make spec110-migrate-118-proof`  
> Tree version pin: still **0.24.1** (no tag cut this wave)

## Verdict

**The SQL defect is fixed and proven on Postgres.** Partner-class failure (multi-ws `wsdoc` → `21000`) reproduces with the old body and succeeds with patched 118/121. Checksum lock + repair modules are in tree. **This is not yet a released GHCR image** — PPD still needs **v0.24.2** (or a private rebuild from this commit).

## What passed (evidence)

| Gate | Result | Artifact |
|------|--------|----------|
| E2E-110-01 old 118 fails 21000 | **PASS** | `e2e110-patched-ok.txt` / `e2e110-repro-0241.txt` |
| E2E-110-02/03 patched 118 + COALESCE + idempotent | **PASS** | `e2e110-patched-ok.txt` |
| E2E-110-04 patched 121 multi-ws injection | **PASS** | `e2e110-patched-ok.txt` |
| E2E-110-05 source `DISTINCT ON` | **PASS** | always-on `#[test]` + contract |
| contract_spec110 (4 tests) | **PASS** | `e2e110-source-guard.txt` |
| spec083 `contract_checksum_drift_fails_loud` includes m118/m121 | **PASS** | needs `--features postgres` |
| `checksums.lock` / `check_migration_checksums.sh` | **PASS** | `e2e110-checksums-after.txt` |

Fixed digests:

```text
118 → a35e70d52e12215abe84283e4b0f853add44fb7ce9f2740f0673e840fdb385cb91eab4367bb0bf84c4c4894cdc370d9a
121 → 57088e874c47e6c558279388b7812946864dabcd3def5d97f417d105a656fba15bced1d3a5c4bd0b190c30a1978e0ef1
```

## What was NOT proven (do not oversell)

1. **No GHCR `0.24.2` image** — E2E-110-07 docker compare against published `0.24.2` was **not** run. Local proof = `raw_sql` of patched files + cargo tests, not `docker run …:0.24.2 migrate`.
2. **No live partner DB** — fixture is synthetic multi-ws wsdoc/injection keys on scratch `edgequake_test`, not the PPD dump.
3. **Checksum repair UPDATE path not DB-integration tested** — unit/source contracts assert DEV_MODE refuse + distinct SHA constants; we did **not** simulate `_sqlx_migrations` broken→fixed UPDATE in a dedicated e2e (Path B). Real-world Path B still depends on that code path.
4. **`cargo test -p edgequake-api --lib` is broken in this working tree** — SPEC-109 WIP leaves `CreateWorkspaceRequest` / API request structs missing `default_reasoning_effort` / `llm_roles` in three in-crate test initializers. SPEC-110 proof deliberately avoids `--lib` for that reason. **Honest: merge/CI may still be red until SPEC-109 test fixtures are finished.**
5. **Scratch DB VersionMismatch was observed mid-wave** — after editing 118 in place, an already-migrated `edgequake_test` printed `migration 118 was previously applied but has been modified` during provision (exactly the field risk LAW-M3 addresses). Dropping `edgequake_test` and re-provisioning cleared it for the clean proof run. Operators with applied-old-118 need the repair path or manual checksum UPDATE.
6. **LAW-M5 membership collapse** — multi-ws same `document_id` keeps **one** `workspace_id` (lexicographic min). Other memberships remain only in KV until drop 125. This can surprise operators who expected multi-workspace share of one UUID; product model never supported that relationally.

## Residual risks

| Risk | Severity | Notes |
|------|----------|-------|
| Partner stays on `0.24.1` image | **High** | Repo fix does nothing until rebuild/tag |
| Mixed fleet: some DBs applied old 118 | **Med** | Need DEV_MODE one-shot or manual checksum |
| Wrong workspace wins on collapse | **Low–Med** | Deterministic; document in ops reply |
| CI `--lib` red from SPEC-109 | **Med** | Unrelated to 110; blocks clean release train |

## Partner one-liner

Oui, le bug est corrigé dans le code et prouvé en e2e local. **Il faut encore une image ≥ correctif (cible 0.24.2)** pour PPD — `0.24.1` embarque toujours le SQL cassé.

## How to re-run

```bash
export DATABASE_URL="$(cat /tmp/edgequake-db-url)"
# If provision complains about modified 118: DROP DATABASE edgequake_test;
make spec110-migrate-118-proof
```
