# SPEC-055 — Release Plan: v0.20.0

Date: 2026-07-21  
Target: **v0.20.0**  
Branch: edgequake-main

---

## First Principles

```
Release = Version bump + OpenAPI refresh + Quality gates + Git tag + CI/CD publishes artifacts

WHY local gates first: Tag quality-gates re-run fmt/clippy/SPEC proofs.
Catch failures locally before push/tag.

WHY OpenAPI refresh: Committed snapshot must track CARGO_PKG_VERSION / VERSION.

WHY no crates.io: Product CD is GHCR Docker only (workspace --no-publish).
```

---

## Release Checklist

```
Phase 1 — CHANGELOG + inventory
[x] 1-A  CHANGELOG.md  [Unreleased] → [0.20.0] — 2026-07-21 (+ Performance testing)
[x] 1-B  This RELEASE_PLAN_0.20.0.md

Phase 2 — Version bump (0.19.0 → 0.20.0)
[x] 2-A  make version-bump VERSION=0.20.0
[x] 2-B  README + release-and-cd + AGENTS.md pins → 0.20.0
[x] 2-C  make codegen-openapi-refresh + spec027_api_contract

Phase 3 — Local quality gates
[x] 3-A  fmt, clippy -D warnings (--lib SSOT), release-gates
[x] 3-B  package dry-runs + vitest label-utils + targeted lib tests

Phase 4 — Commit + push
[x] 4-A  git commit -m "release: bump to v0.20.0"
[x] 4-B  git push origin edgequake-main

Phase 5 — Tag + CI/CD
[x] 5-A  git tag v0.20.0 && git push origin v0.20.0
[x] 5-B  gh release view v0.20.0
[x] 5-C  docker buildx imagetools inspect GHCR tags
```

---

## What ships in v0.20.0

### Features (since 0.19.0)
- **065** Smart/Mix = LightRAG three arms; `MIX_ARM_GATE` default false; chunk `vector_type=chunk`
- **066** Drawing `display_name` + Graph `graph_node_label` + WebUI identity
- Vision ingestion reliability (page count / budget / watchdog / checkpoints)

### Performance (binding language)
- Statistical Acc **tie** (EQ 0.731 vs LR 0.760 cold publish)
- Fair cold query p50 **1.013×**
- Acc Fact peer EQ Acc **0.801**
- Do **not** claim Acc Beat win

---

## CI/CD Artifacts Produced

```
ghcr.io/raphaelmansuy/edgequake:0.20.0
ghcr.io/raphaelmansuy/edgequake:latest
ghcr.io/raphaelmansuy/edgequake-frontend:0.20.0
ghcr.io/raphaelmansuy/edgequake-postgres:0.20.0
ghcr.io/raphaelmansuy/edgequake-postgres:0.20.0-pg16|pg17|pg18
```
