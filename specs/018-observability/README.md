# Spec 018 — Observability & OTEL

**Date:** 2026-06-06 (post-implementation)  
**Status:** Implemented — see [015-brutal-post-implementation.md](./015-brutal-post-implementation.md)

---

## Verdict (current)

| Pillar | Grade | Evidence |
|--------|-------|----------|
| **Correlation** | A | `X-Request-ID` + W3C `traceparent` (echo/synthesize/inject); WebUI client |
| **Structured logs** | A+ | `EDGEQUAKE_LOG_FORMAT=json`; levelled `ErrorEvent`; domain error/warn |
| **Traces** | A | `http_request` + pipeline + query spans; Jaeger overlay |
| **Error context** | A+ | API `details.diagnostics` with category, retryable, auth reason, phase |
| **Metrics** | A | HTTP, query, LLM, document, storage, pipeline, db pool gauges |
| **OTLP export** | A- | `docker-compose.observability.yml` + `--profile observability` |
| **Audit** | B+ | Auth, query, upload, PDF, delete, workspace CRUD |
| **Resource pressure signals** | A | 503 `ServiceUnavailable` with `retry_after_secs`; correlated via `request_id` / `traceparent` |

Run proofs: [`e2e/run_observability_proof.sh`](./e2e/run_observability_proof.sh)

**Related:** [SPEC-006 resource safety](../../specifications/006-ensure-perf/000-index.md) — graph 503s surface via `ApiError` diagnostics. [SPEC-020 E2E QC](../020-e2e-quality-control/README.md) — release gate for ingest/query/delete/isolation.

---

## P0 gaps — resolution status

| ID | Gap | Status |
|----|-----|--------|
| OBS-P0-001 | No OTEL exporter | **FIXED (opt-in)** — `edgequake-observability/otel` + env |
| OBS-P0-002 | request_id not on spans | **FIXED** — `with_http_span`, task-local |
| OBS-P0-003 | WebUI no correlation headers | **FIXED** — `client.ts` X-Request-ID + traceparent |
| OBS-P0-004 | `/metrics` placeholder | **FIXED** — Prometheus recorder + describe_* |
| OBS-P0-005 | Audit unwired | **FIXED (partial)** — query, auth, upload, workspace |
| OBS-P0-006 | Docs vs JSON logging | **FIXED** — `init_observability` + `EDGEQUAKE_LOG_FORMAT` |

---

## Key crates

| Crate | Role |
|-------|------|
| `edgequake-observability` | Subscriber, metrics, `ErrorEvent`, HTTP spans, correlation |
| `edgequake-api` | `observability_middleware`, `ApiError` diagnostics, audit helpers |

---

## Production quick start

```bash
# Build with OTLP
docker compose -f edgequake/docker/docker-compose.yml build \
  --build-arg ENABLE_OTEL=true

# Runtime
export EDGEQUAKE_LOG_FORMAT=json
export OTEL_EXPORTER_OTLP_ENDPOINT=http://collector:4317
export RUST_LOG=edgequake_api=info,edgequake_storage=warn
```

Operator guide: [`docs/OBSERVABILITY.md`](../../docs/OBSERVABILITY.md)  
Resource safety runbook: [`specifications/006-ensure-perf/009_operator_runbook.md`](../../specifications/006-ensure-perf/009_operator_runbook.md)

### CI gates (release pipeline — v0.12.11+)

```bash
make release-gates           # fmt + clippy (all workspace crates) + tests + proofs
make observability-proof     # SPEC-018 Rust + WebUI
make resource-proof          # SPEC-006 P0–P9 (mock; Postgres e2e in postgres-age-tests job)
make spec020-qc-proof-strict # SPEC-020 E2E (migration-038 + /ready)
make spec020-qc-proof-full   # SPEC-020 + Ollama required (local prod gate)
```

GitHub Actions: [`release-gates.yml`](../../.github/workflows/release-gates.yml) (manual + tag preflight), [`ci.yml`](../../.github/workflows/ci.yml) (`observability-proof` job), [`e2e-quality-gates.yml`](../../.github/workflows/e2e-quality-gates.yml) (`spec020-qc`).

### CD / Docker publication (v0.12.11+)

```bash
make release-gates
git tag v0.12.11 && git push origin v0.12.11
# → release-docker.yml: preflight gates → GHCR multi-arch images
# → ghcr.io/raphaelmansuy/edgequake:{version}
# → ghcr.io/raphaelmansuy/edgequake-frontend:{version}
# → ghcr.io/raphaelmansuy/edgequake-postgres:{version}

# Verify publication
gh release view v0.12.11
docker buildx imagetools inspect ghcr.io/raphaelmansuy/edgequake:0.12.11

make stack-pull   # pull prebuilt GHCR images
make stack        # start API + UI + DB
```

**v0.12.11 release notes (document scope):** workspace tenant/workspace matching is centralized in `workspace_scope.rs` (list, delete, PDF dedup). Observability correlation headers (`X-Request-ID`, `traceparent`) continue to propagate through upload/list handlers — audit events on PDF upload remain wired via `record_compliance_event`.

---

## Document index (audits — historical)

Subfolder audits describe **pre-implementation** state. Use this README + `015-brutal-post-implementation.md` as source of truth.

| Folder | Component |
|--------|-----------|
| [003-edgequake-api](./003-edgequake-api/001-audit.md) | HTTP middleware |
| [005-edgequake-pipeline](./005-edgequake-pipeline/001-audit.md) | Ingestion |
| [013-edgequake-webui](./013-edgequake-webui/001-audit.md) | Client (updated in code) |
