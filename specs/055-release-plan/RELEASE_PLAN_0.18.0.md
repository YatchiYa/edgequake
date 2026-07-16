# SPEC-055 — Release Plan: v0.18.0

Date: 2026-07-16  
Target: **v0.18.0**  
Branch: edgequake-main

---

## First Principles

```
Release = Version bump + OpenAPI refresh + Quality gates + Git tag + CI/CD publishes artifacts

WHY local gates first: Tag quality-gates re-run fmt/clippy/SPEC proofs (up to 90m).
Catch failures locally (fmt, clippy, nextest, SPEC-027, release-gates) before push.

WHY OpenAPI refresh: Committed snapshot can lag CARGO_PKG_VERSION; live Swagger
uses enrichment. Regenerating snapshot + schema.d.ts keeps Explorer/codegen honest.

WHY crates.io pdf2md: edgequake-pdf2md 0.9.7 is already on crates.io — no path
dep checkout in Docker CD.
```

---

## Release Checklist

```
Phase 1 — CHANGELOG + inventory
[x] 1-A  CHANGELOG.md  [Unreleased] → [0.18.0] — 2026-07-16
[x] 1-B  This RELEASE_PLAN_0.18.0.md

Phase 2 — Version bump (0.17.0 → 0.18.0)
[x] 2-A  make version-bump VERSION=0.18.0
[x] 2-B  edgequake-audit/tasks/rate-limiter → version.workspace = true
[x] 2-C  README + docker-quickstart + release-and-cd pins → 0.18.0
[x] 2-D  openapi.rs drop stale hardcoded info.version

Phase 3 — OpenAPI / Swagger
[x] 3-A  make codegen-openapi-refresh
[x] 3-B  cargo test -p edgequake-api --test spec027_api_contract
[x] 3-C  Live: /api-docs/openapi.json info.version == 0.18.0

Phase 4 — CI/CD lean-ups
[x] 4-A  release_gates.sh: crate + OpenAPI version parity
[x] 4-B  release-docker.yml: setup-rust + drop pdf2md checkout
[x] 4-C  Dockerfile: drop dead pdf2md COPY if present
[x] 4-D  docs/operations/release-and-cd.md OpenAPI checklist

Phase 5 — Local quality gates
[x] 5-A  migration checksum, fmt, clippy, nextest --lib
[x] 5-B  RELEASE_SKIP_*=1 make release-gates
[x] 5-C  test-quality (invariants, count floor, e2e lint/UI)
[x] 5-D  ops17-smoke, spec046-acc, check-extension-pins, spec013-proof-pr
[x] 5-E  release build + docs + docker API smoke

Phase 6 — Commit
[x] 6-A  git commit -m "release: bump to v0.18.0"

Phase 7 — Tag + CI/CD (after approval)
[ ] 7-A  git tag v0.18.0 && git push origin v0.18.0
[ ] 7-B  gh release view v0.18.0
[ ] 7-C  docker buildx imagetools inspect GHCR tags
```

---

## What ships in v0.18.0

### Features / Performance (since 0.17.0)
- **SPEC-054**: documents-list + Mix-scale + AGE/pgvector perf gates; batch lineage SQL; bootstrap index reconcile
- **Progress UX**: reprocess/delete/pending-doc reconcile; PDF progress identity

### CI/CD
- OpenAPI snapshot + release_gates parity
- Docker CD: shared Rust cache; no pdf2md sibling checkout

---

## CI/CD Artifacts Produced

```
ghcr.io/raphaelmansuy/edgequake:0.18.0
ghcr.io/raphaelmansuy/edgequake:latest
ghcr.io/raphaelmansuy/edgequake-frontend:0.18.0
ghcr.io/raphaelmansuy/edgequake-postgres:0.18.0
ghcr.io/raphaelmansuy/edgequake-postgres:0.18.0-pg16|pg17|pg18
```
