# SPEC-055 — Release Plan: v0.20.2

Date: 2026-07-22  
Target: **v0.20.2** (patch — opaque soft-labels + delete/ingest ops)  
Branch: edgequake-main

---

## First Principles

```
Release = Version bump + OpenAPI refresh + Quality gates + Git tag + CI/CD publishes artifacts

WHY patch: Ships SPEC-067–073 presentation/ops fixes and dual fairness lanes without
product-facing Acc / Mix / vision feature churn.

WHY OpenAPI refresh: Committed snapshot must track CARGO_PKG_VERSION / VERSION.

WHY no crates.io: Product CD is GHCR Docker only (workspace --no-publish).
```

---

## Release Checklist

```
Phase 1 — CHANGELOG + inventory
[x] 1-A  CHANGELOG.md  [Unreleased] → [0.20.2] — 2026-07-22
[x] 1-B  This RELEASE_PLAN_0.20.2.md

Phase 2 — Version bump (0.20.1 → 0.20.2)
[x] 2-A  make version-bump VERSION=0.20.2
[x] 2-B  README + release-and-cd + AGENTS.md pins → 0.20.2
[x] 2-C  make codegen-openapi-refresh + spec027_api_contract

Phase 3 — Local quality gates
[x] 3-A  make ops17-smoke + make spec046-acc
[x] 3-B  make release-gates (CI-parity SKIP_LIB + SKIP_PER_CRATE; workspace lib proved separately with --skip page_count::)
[x] 3-C  make test-e2e-lint + contract_067/072/073 + vitest label-utils/source-mapper

Phase 4 — Commit + push
[x] 4-A  git commit -m "release: bump to v0.20.2" (`48262c65`)
[x] 4-B  git push origin edgequake-main

Phase 5 — Tag + CI/CD
[x] 5-A  git tag v0.20.2 && git push origin v0.20.2
[x] 5-B  gh release view v0.20.2 — https://github.com/raphaelmansuy/edgequake/releases/tag/v0.20.2
[x] 5-C  docker buildx imagetools inspect GHCR tags (api/frontend/postgres + pg16/pg17/pg18)
```

CD run: https://github.com/raphaelmansuy/edgequake/actions/runs/29902599057 (success)

### Local gate notes

- `edgequake-pdf` `page_count::*` pdfium lib tests hang at 0% CPU on this host — workspace lib suite proved with `--skip page_count::`.
- `intent_rerank` env tests serialized with a mutex (parallel flake).
- SPEC-006 runbook sync anchors `WORKER_THREADS` / `MAX_TASKS_PER_TENANT` to pipeline `config.rs` (fairness-lanes SSOT).

---

## What ships in v0.20.2

### Fixed / Added (since 0.20.1)
- **067** Reject opaque UUID/GUID entity names at write; soft-label presentation SSOT
- **068** Text ingest progress parity with PDF upload UX
- **069** Reliable delete — DDL off hotpath
- **070** DB ops excellence (m086/m092 reconcile, vector DDL session, ops audit)
- **071** Lineage edge discovery via source-prefix GIN
- **072** Lineage label SSOT for document-scoped KG (id ≠ label)
- **073** Relationship endpoint `source_label`/`target_label` for Query Connections + remaining bypasses
- **Fairness lanes** Dual lanes so deletes do not starve PDF ingest

### Out of scope
- Full Wikidata opaque-id identity rewrite / SAME_AS ER
- Rewriting AGE node ids for legacy UUID entities (re-ingest remains the cleanup path)
- Acc / Mix / vision feature churn

---

## Proofs (local)

```bash
make ops17-smoke
make spec046-acc
make release-gates
make test-e2e-lint
cargo test -p edgequake-api --test contract_067_opaque_entity_names
cargo test -p edgequake-api --test contract_072_lineage_label_ssot
cargo test -p edgequake-api --test contract_073_relationship_endpoint_labels
```

---

## CI/CD Artifacts Produced

```
ghcr.io/raphaelmansuy/edgequake:0.20.2
ghcr.io/raphaelmansuy/edgequake:latest
ghcr.io/raphaelmansuy/edgequake-frontend:0.20.2
ghcr.io/raphaelmansuy/edgequake-postgres:0.20.2
ghcr.io/raphaelmansuy/edgequake-postgres:0.20.2-pg16|pg17|pg18
```
