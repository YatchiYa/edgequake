# SPEC-055 — Release Plan: v0.20.1

Date: 2026-07-22  
Target: **v0.20.1** (patch — delete / restart recovery)  
Branch: edgequake-main  
PR: [#312](https://github.com/raphaelmansuy/edgequake/pull/312) (merged)

---

## First Principles

```
Release = Version bump + OpenAPI refresh + Quality gates + Git tag + CI/CD publishes artifacts

WHY patch: Ships durable wipe (#309), graph-first cascade (#305), structured Interrupted (#304)
without product-facing Acc / Mix / vision feature churn from 0.20.0.

WHY OpenAPI refresh: Committed snapshot must track CARGO_PKG_VERSION / VERSION.

WHY no crates.io: Product CD is GHCR Docker only (workspace --no-publish).
```

---

## Release Checklist

```
Phase 1 — CHANGELOG + inventory
[x] 1-A  CHANGELOG.md  [Unreleased] → [0.20.1] — 2026-07-22
[x] 1-B  This RELEASE_PLAN_0.20.1.md

Phase 2 — Version bump (0.20.0 → 0.20.1)
[x] 2-A  make version-bump VERSION=0.20.1
[x] 2-B  README + release-and-cd + AGENTS.md pins → 0.20.1
[x] 2-C  make codegen-openapi-refresh + spec027_api_contract

Phase 3 — Local quality gates
[x] 3-A  make release-gates (CI-parity SKIP_LIB + SKIP_PER_CRATE; env-isolated flake tests verified)
[x] 3-B  make test-e2e-lint

Phase 4 — Commit + push
[x] 4-A  git commit -m "release: bump to v0.20.1" (`9cac5bc0`)
[x] 4-B  git push origin edgequake-main

Phase 5 — Tag + CI/CD
[x] 5-A  git tag v0.20.1 && git push origin v0.20.1
[x] 5-B  gh release view v0.20.1 — https://github.com/raphaelmansuy/edgequake/releases/tag/v0.20.1
[x] 5-C  docker buildx imagetools inspect GHCR tags (api/frontend/postgres + pg16/pg17/pg18 multi-arch)
```

Issues closed: #305, #304; #309 already closed + verified comment.
CD run: https://github.com/raphaelmansuy/edgequake/actions/runs/29854959325 (success)



---

## What ships in v0.20.1

### Fixed / Added (since 0.20.0)
- **#309** Durable workspace wipe (`WorkspaceWipe` task, HTTP 202 + `wipe_track_id`, WebSocket progress)
- **#305** Graph-first document cascade + legacy-null-safe discovery; tenant isolation preserved (SPEC-006)
- **#304** Structured Interrupted recovery (`failure_code=server_restart_interrupted`) on task + document orphan paths; WebUI Interrupted → Full reprocess default
- Contract / SDK / WebUI updates for wipe + planned delete counts; migration `095` (`WorkspaceWipe` task type)

### Out of scope
- Acc / Mix / Drawing display_name product changes (already in 0.20.0)
- Baseline AGE Cypher neighbor / SPEC-013 AGE `LOAD` flakes (triaged; not introduced by #312)

---

## Proofs (local)

```bash
cargo test -p edgequake-api --test resource_safety_proof
cargo test -p edgequake-api --test e2e_document_deletion issue309_wipe_opcount
cargo test -p edgequake-api --lib interrupted_restart
make release-gates && make test-e2e-lint
```

---

## CI/CD Artifacts Produced

```
ghcr.io/raphaelmansuy/edgequake:0.20.1
ghcr.io/raphaelmansuy/edgequake:latest
ghcr.io/raphaelmansuy/edgequake-frontend:0.20.1
ghcr.io/raphaelmansuy/edgequake-postgres:0.20.1
ghcr.io/raphaelmansuy/edgequake-postgres:0.20.1-pg16|pg17|pg18
```

---

## Close issues after verify

- #309, #305, #304 — with PR #312 + this release + proof commands
