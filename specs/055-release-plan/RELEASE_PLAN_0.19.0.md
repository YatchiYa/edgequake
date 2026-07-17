# SPEC-055 — Release Plan: v0.19.0

Date: 2026-07-17  
Target: **v0.19.0**  
Branch: edgequake-main

---

## First Principles

```
Release = Version bump + OpenAPI refresh + Quality gates + Git tag + CI/CD publishes artifacts

WHY local gates first: Tag quality-gates re-run fmt/clippy/SPEC proofs.
Catch failures locally before push/tag.

WHY OpenAPI refresh: Committed snapshot must track CARGO_PKG_VERSION / VERSION.
```

---

## Release Checklist

```
Phase 1 — CHANGELOG + inventory
[x] 1-A  CHANGELOG.md  [Unreleased] → [0.19.0] — 2026-07-17
[x] 1-B  This RELEASE_PLAN_0.19.0.md

Phase 2 — Version bump (0.18.0 → 0.19.0)
[x] 2-A  make version-bump VERSION=0.19.0
[x] 2-B  README + docker-quickstart + release-and-cd + compose pins → 0.19.0
[x] 2-C  make codegen-openapi-refresh + spec027_api_contract

Phase 3 — Local quality gates
[x] 3-A  migration checksum, fmt, clippy, nextest --lib, doc, release build
[x] 3-B  Test Quality Gates (invariants, ≥870, e2e lint/UI)
[x] 3-C  RELEASE_SKIP_*=1 make release-gates + ops17-smoke + spec046-acc
[x] 3-D  SPEC-057 postgres contracts + migration e2e + spec013-proof-pr

Phase 4 — Commit + push
[x] 4-A  git commit -m "release: bump to v0.19.0"
[x] 4-B  git push origin edgequake-main (includes 5 prior unreleased commits)
[x] 4-C  Wait CI green (CI + Quality Gates + Release Gates + Migration Guard + AGE)

Phase 5 — Tag + CI/CD
[x] 5-A  git tag v0.19.0 && git push origin v0.19.0
[x] 5-B  gh release view v0.19.0 — https://github.com/raphaelmansuy/edgequake/releases/tag/v0.19.0
[x] 5-C  docker buildx imagetools inspect GHCR tags (api/frontend/postgres + pg16/pg17/pg18)
```

---

## What ships in v0.19.0

### Features (since 0.18.0)
- **SPEC-057**: claim/lease delivery SSOT, PDF Cancelled, status mapper, convert→ingest split, multi-replica, compensate observability
- Migrations 087–089
- Multimodal error-handling hardening; cancel fairness park

---

## CI/CD Artifacts Produced

```
ghcr.io/raphaelmansuy/edgequake:0.19.0
ghcr.io/raphaelmansuy/edgequake:latest
ghcr.io/raphaelmansuy/edgequake-frontend:0.19.0
ghcr.io/raphaelmansuy/edgequake-postgres:0.19.0
ghcr.io/raphaelmansuy/edgequake-postgres:0.19.0-pg16|pg17|pg18
```
