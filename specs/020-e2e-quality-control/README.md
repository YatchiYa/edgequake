# SPEC-020 — Full E2E Quality Control

**24 Playwright tests** — live PostgreSQL stack quality gate.

```bash
make spec020-qc-proof

# Production strict gate (migration-038 + /ready):
make spec020-qc-proof-strict

# Full local prod gate (strict + Ollama required):
make spec020-qc-proof-full

# Auth-enabled login proof:
make spec020-qc-proof-auth

# Auto-repair is ON by default in the proof runner (ensure_migration_038.sh).
# Disable: SPEC020_AUTO_MIGRATION=0 make spec020-qc-proof
# Large graphs: SPEC020_MIGRATION_CONCURRENT=1 make spec020-qc-proof
```

| Item | Path |
|------|------|
| Spec | `edgequake_webui/e2e/spec020-quality-control.spec.ts` |
| Runner | `e2e/run_quality_control_proof.sh` |
| Proof | `e2e/001-quality-control-proof.md` |
| Screenshots | `e2e/screenshots/` (25+ PNGs) |
| CI | `.github/workflows/e2e-quality-gates.yml` → `spec020-qc` |

## Modules (DRY / SOLID)

`qc-api-route` · `qc-health` · `qc-documents` · `qc-query` · `qc-isolation` · `qc-workspace` · `qc-ui-upload` · `qc-graph` · `qc-api-errors` · `qc-auth` · `llm-availability` · `spec020-artifacts`

## Assessment

**Grade A** — 24/24 passed; `make spec020-qc-proof-strict` prod gate green. See [e2e/001-quality-control-proof.md](./e2e/001-quality-control-proof.md).

**Fixed release blockers:** sync-upload graph scope (FIX-SPEC020); migration-038 GIN indexes (FIX-MIG038-GIN); document delete cascade (FIX-SPEC020-CASCADE); metrics UUID bind (FIX-METRICS); audit INET cast (FIX-AUDIT-INET); dev UI/API port drift via Next.js proxy (FIX-DEV-PROXY).

**Remaining gaps:** vision PDF (multimodal), auth proof (`make spec020-qc-proof-auth`), CI Ollama skips unless `SPEC020_REQUIRE_OLLAMA=1`.
