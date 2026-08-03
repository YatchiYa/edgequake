# 07 — Release Assessment v0.24.0

> **As-of:** 2026-08-03 · product cut **v0.24.0** (SPEC-104 + SPEC-105).

## Ship verdict

**YES** — ship v0.24.0.

## Gate evidence

| Gate                         | Result        | Notes                                                                                                                                               |
| ------------------------------| ---------------| -----------------------------------------------------------------------------------------------------------------------------------------------------|
| `make release-gates`         | PASS          | fmt/clippy/SPEC-006/018/WebUI/version+OpenAPI parity **0.24.0**                                                                                     |
| `contract_spec104_datalayer` | PASS 11/11    | PG soft-skips where expected                                                                                                                        |
| `contract_spec105_legacy`    | PASS 8/8      | mid-upgrade census note OK                                                                                                                          |
| `spec027_api_contract`       | PASS          | OpenAPI `info.version` = **0.24.0**                                                                                                                 |
| `make spec091-upgrade-soak`  | **PASS** 19/0 | Applied 125/126/131/**142**; health + multi-tenant gates GREEN                                                                                      |
| `make bench001-doctor`       | PASS          | EQ healthy; Acc keys/LightRAG present                                                                                                               |
| Acc pack attestation         | **ATTEST**    | `publish/latest` `valid: true` (2026-08-02, medical-mid n=200, profile `P0_mistral_small_mix_chunk1200_*`); SPEC-104/105 do not change Acc SUT pins |

## Spec assessments

- SPEC-104: [13-fix-assessment.md](../104-fix-datalayer/13-fix-assessment.md) — ship as **0.24.0**
- SPEC-105: [06-post-assessment.md](06-post-assessment.md) — A / A− grades; 142 deferral verified live

## Residuals (do not block)

1. SPEC-104 #5 node-count `57014` under load — capacity / SPEC-089
2. Full `make spec93-migration-assessment` (PG16/17/18) — optional deeper realism; soak smoke is the cut gate
3. Full `make bench` n=200 re-run — attested existing pack for this cut

## CD verify (Phase 5)

| Check | Result |
|-------|--------|
| Workflow `release-docker` run 30812944260 | **success** (all 16 jobs) |
| `gh release view v0.24.0` | **OK** — https://github.com/raphaelmansuy/edgequake/releases/tag/v0.24.0 |
| `ghcr.io/raphaelmansuy/edgequake:0.24.0` | **OK** multi-arch amd64+arm64 |
| `edgequake-frontend:0.24.0` | **OK** multi-arch amd64+arm64 |
| `edgequake-postgres:0.24.0` | **OK** multi-arch amd64+arm64 |
| `edgequake-postgres:0.24.0-pg16` | **OK** |
| `edgequake-postgres:0.24.0-pg17` | **OK** |
| `edgequake-postgres:0.24.0-pg18` | **OK** |
