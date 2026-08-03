# SPEC-055 — Release Plan: v0.23.0

Date: 2026-08-02  
Target: **v0.23.0** (minor — SPEC-091 relational cutover + parse API + LLM cache + UX)  
Branch: `feat/version-023`

---

## First Principles

```
Release = Version bump + OpenAPI refresh + Quality gates + Git tag + CI/CD publishes artifacts

WHY minor: Ships migrations 106–141 (irreversible KV/vector drops), LD-15 boot
never migrates, SPEC-094 parse API, SPEC-103 LLM cache, wizard/UX — product-facing
schema and operator behavior, not a 0.22.x patch.

WHY OpenAPI refresh: Committed snapshot must track CARGO_PKG_VERSION / VERSION.

WHY no crates.io: Product CD is GHCR Docker only (workspace --no-publish).
```

---

## Release Checklist

```
Phase 1 — CHANGELOG + inventory
[x] 1-A  CHANGELOG.md  [Unreleased] → [0.23.0] — 2026-08-02
[x] 1-B  This RELEASE_PLAN_0.23.0.md

Phase 2 — Version bump (0.22.0 → 0.23.0)
[x] 2-A  make version-bump VERSION=0.23.0
[x] 2-B  README + release-and-cd + AGENTS.md + SPEC-091 pins → 0.23.0
[x] 2-C  make codegen-openapi-refresh + spec027_api_contract
[x] 2-D  bump edgequake-llm 0.10.2 → 0.10.3

Phase 3 — Local quality gates
[x] 3-A  make ops17-smoke + make spec046-acc + make spec103-llm-cache-proof
[x] 3-B  cargo fmt --check + cargo clippy -D warnings
[x] 3-C  RELEASE_SKIP_LIB_TESTS=1 RELEASE_SKIP_PER_CRATE_CLIPPY=1 make release-gates
[x] 3-D  make test-e2e-lint
[x] 3-E  Acc attest publish/latest (valid:true; medical-mid-20260802T135513Z)
[x] 3-F  make check-extension-pins

Phase 4 — Commit + push (gated — needs explicit approval)
[ ] 4-A  git commit -m "release: bump to v0.23.0"
[ ] 4-B  git push origin feat/version-023 (or merge to default cut branch)

Phase 5 — Tag + CI/CD (gated)
[ ] 5-A  git tag v0.23.0 && git push origin v0.23.0
[ ] 5-B  gh release view v0.23.0
[ ] 5-C  docker buildx imagetools inspect GHCR tags (api/frontend/postgres + pg16/pg17/pg18)
```

---

## What ships in v0.23.0

### Added / Changed (since 0.22.0)

- **SPEC-091** relational SSOT + migrations **106–141** (irreversible 125/126/131 behind `--confirm-drop`)
- **LD-15** boot never migrates (exit 78); `EDGEQUAKE_ALLOW_BOOT_MIGRATE` removed
- **SPEC-094** standalone PDF→Markdown parse API
- **SPEC-103** LightRAG-parity LLM cache (default on; Acc pins off)
- **SPEC-096/098/099/100/101/102** language extract, AGE harden, Documents CLS, wizard, entity colors
- **Cancel/fairness** holds (mig 138); product Mix default → Acc E2-occ profile
- **Deps:** `edgequake-llm` 0.10.3 · `edgequake-pdf2md` 0.9.10 · `edgeparse-core` 0.2.5

### Out of scope

- crates.io publish of workspace crates
- Acc Beat / medical-full promote (080 STOP)
- SPEC-120 `/operations` API
- IP3–IP5 / WP2–WP5 worker stage split

---

## Acc attestation (SPEC-001)

| Field | Value |
|-------|--------|
| Archive | `specs/001-benchmark/e2e/artifacts/history/medical-mid-20260802T135513Z/` |
| Publish pack | `specs/001-benchmark/e2e/artifacts/publish/latest/` |
| `valid` | `true` |
| Acc EQ / LR | **0.807** / **0.779** |
| Δ Acc 95% CI | **[-0.005, +0.059]** (statistical tie) |
| Claim | Peer / statistical tie only — **not** Acc Beat |
| Fair cold | `C1COLD_v1` EQ/LR p50 **1.02×** |

SSOT: [docs/comparisons/eq-vs-lightrag-acc-bench.md](../../docs/comparisons/eq-vs-lightrag-acc-bench.md).

---

## External published libraries

| Crate | Pin | crates.io | Action |
|-------|-----|-----------|--------|
| edgequake-llm | 0.10.3 | 0.10.3 | Bumped from 0.10.2 |
| edgequake-pdf2md | 0.9.10 | 0.9.10 | Current |
| edgeparse-core | 0.2.5 | 0.2.5 | Current |

Workspace members: GHCR only (`cargo package` dry-run optional).

---

## Quality scorecard

| Dimension | Bar | Status |
|-----------|-----|--------|
| Schema/ops honesty | Upgrade runbook + LD-15 + `--confirm-drop` in README/CHANGELOG | **PASS** |
| Contract parity | OpenAPI `info.version` = 0.23.0; spec027 green | **PASS** |
| Lint/CI local | fmt + clippy `-D warnings` + release-gates | **PASS** |
| Acc claims | Peer/tie; README = SSOT; 080 STOP | **PASS** (attested) |
| Deps | crates.io pins current; no path patches | **PASS** |
| Residuals listed | Acc Beat STOP; SPEC-120; IP3–IP5/WP2–WP5; CI recheck after push | **DOCUMENTED** |

### Pre-tag fixes landed during this prep

- `StorageRuntime::for_memory_tests` — restore SPEC-027 auth_memory_store caller allowlist
- clippy: `cloned_ref_to_slice_refs` (GH-350 e2e); `await_holding_lock` → tokio Mutex in SPEC-103 contract; `unnecessary_get_then_check` in knowledge_rebuild
- e2e lint: drop hardcoded `:8080` / `networkidle` in chromium gate specs
- WebUI release tsc: Document.status includes `deleting`/`delete_failed`; `Boolean(entry.isPdf)`; exclude colocated `*.test.ts` from `tsconfig.release.json`

### Residuals (do not hide)

- Acc Beat / Acc Equal mid **STOP** ([080](../../specs/001-benchmark/001-edgquake-improvements/080-phase-g-promote-checklist.md))
- SPEC-120 `/operations` descoped
- IP3–IP5, WP2–WP5 open
- Prior CI flakes on feat/version-023 (Postgres Integration / Test Quality Gates) must be re-checked after push of the release commit
- Tag / GHCR publish **blocked** until explicit approval (Phase 4–5)

---

## Release quality verdict

**READY TO COMMIT locally.** Strong minor: relational data-layer cutover is the headline; Acc remains a statistical tie (not a win claim); upgrade path is the highest operational risk and is documented fail-closed (LD-15 + `--confirm-drop`).

**Do not tag** until: (1) release commit approved, (2) push CI green (CI + Release Gates + SPEC-091 + Quality Gates + Postgres Integration), (3) explicit `git tag v0.23.0 && git push origin v0.23.0`.

---

## Proofs (local) — executed 2026-08-02

```bash
make ops17-smoke                                    # PASS
make spec046-acc                                    # PASS
make spec103-llm-cache-proof                        # PASS
cd edgequake && cargo fmt --all -- --check          # PASS
cd edgequake && cargo clippy --workspace --all-targets -- -D warnings  # PASS
RELEASE_SKIP_LIB_TESTS=1 RELEASE_SKIP_PER_CRATE_CLIPPY=1 make release-gates  # PASS
make test-e2e-lint                                  # PASS
make check-extension-pins                           # PASS
```

---

## CI/CD Artifacts (after tag)

```
ghcr.io/raphaelmansuy/edgequake:0.23.0
ghcr.io/raphaelmansuy/edgequake:latest
ghcr.io/raphaelmansuy/edgequake-frontend:0.23.0
ghcr.io/raphaelmansuy/edgequake-postgres:0.23.0
ghcr.io/raphaelmansuy/edgequake-postgres:0.23.0-pg16|pg17|pg18
```
