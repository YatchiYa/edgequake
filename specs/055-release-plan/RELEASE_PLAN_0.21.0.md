# SPEC-055 — Release Plan: v0.21.0

Date: 2026-07-23  
Target: **v0.21.0** (minor — LightRAG query parity + D-30 eq_rel_type + SPEC-083)  
Branch: edgequake-main

---

## First Principles

```
Release = Version bump + OpenAPI refresh + Quality gates + Git tag + CI/CD publishes artifacts

WHY minor: Ships LightRAG query-API parity (074–085), D-30 multigraph arbiter
(eq_rel_type), and SPEC-083 defect closure as product-facing query/KG behavior
(same framing as 0.20.0), not a 0.20.x patch.

WHY OpenAPI refresh: Committed snapshot must track CARGO_PKG_VERSION / VERSION.

WHY no crates.io: Product CD is GHCR Docker only (workspace --no-publish).
```

---

## Release Checklist

```
Phase 1 — CHANGELOG + inventory
[x] 1-A  CHANGELOG.md  [Unreleased] → [0.21.0] — 2026-07-23
[x] 1-B  This RELEASE_PLAN_0.21.0.md

Phase 2 — Version bump (0.20.2 → 0.21.0)
[x] 2-A  make version-bump VERSION=0.21.0
[x] 2-B  README + release-and-cd + AGENTS.md pins → 0.21.0
[x] 2-C  make codegen-openapi-refresh + spec027_api_contract

Phase 3 — Local quality gates
[x] 3-A  make ops17-smoke + make spec046-acc
[x] 3-B  make release-gates (CI-parity SKIP_LIB + SKIP_PER_CRATE; workspace lib proved separately with --skip page_count::)
[x] 3-C  make test-e2e-lint

Phase 4 — Commit + push
[ ] 4-A  git commit -m "release: bump to v0.21.0"
[ ] 4-B  git push origin edgequake-main

Phase 5 — Tag + CI/CD
[ ] 5-A  git tag v0.21.0 && git push origin v0.21.0
[ ] 5-B  gh release view v0.21.0
[ ] 5-C  docker buildx imagetools inspect GHCR tags (api/frontend/postgres + pg16/pg17/pg18)
```

### Local gate notes

- `edgequake-pdf` `page_count::*` pdfium lib tests hang at 0% CPU on this host — workspace lib suite proved with `--skip page_count::`.
- Pre-bump fixes: X-30 typed vision timeout messages + `from_processing_error` timeout factory; startup_security remote-auth-off strict test; clippy `field_reassign_with_default` in orchestrator config precedence test.

---

## What ships in v0.21.0

### Added / Fixed (since 0.20.2)
- **LightRAG query-API parity (074–085)** — Mix/local/global grounding, BM25/L2, Acc honesty freeze
- **D-30 multigraph arbiter** — Native EDGE upserts keyed by `(eq_source_id, eq_target_id, eq_rel_type)`; M092 support SQL readiness
- **SPEC-083 defect closure** — KG persist split-brain, `/ready` AGE stub false positives, schema/RLS/pipeline/query honesty waves
- **Acc publish refresh** — medical-mid n=200 statistical tie; fair cold latency 1.02× (`C1COLD_v1`)

### Out of scope
- crates.io publish
- Full medical-full Acc run (optional follow-up)
- Rewriting AGE node ids for legacy UUID entities

---

## Proofs (local)

```bash
make ops17-smoke
make spec046-acc
RELEASE_SKIP_LIB_TESTS=1 RELEASE_SKIP_PER_CRATE_CLIPPY=1 make release-gates
make test-e2e-lint
```

---

## CI/CD Artifacts Produced

```
ghcr.io/raphaelmansuy/edgequake:0.21.0
ghcr.io/raphaelmansuy/edgequake:latest
ghcr.io/raphaelmansuy/edgequake-frontend:0.21.0
ghcr.io/raphaelmansuy/edgequake-postgres:0.21.0
ghcr.io/raphaelmansuy/edgequake-postgres:0.21.0-pg16|pg17|pg18
```
