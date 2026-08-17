# 08 — Remediation Plan (SPEC-104)

Ordered steps. Laws: [01](01-first-principles.md). Matrix: [02](02-cross-ref-matrix.md).

## R1 — Inspector naming SSOT (issues #1, #2)

1. Add `InspectorConfig::for_namespace(namespace: &str)` using `PostgresConfig { namespace, ..Default::default() }.table_prefix()`:
   - `kv_table = format!("eq_{prefix}_kv")`
   - `vector_table = format!("eq_{prefix}_vectors")`
   - `graph_name = format!("eq_{prefix}_graph")`
2. `Default` delegates to `for_namespace("default")`.
3. Wire boot (`state/postgres.rs`) and admin handlers to `for_namespace` (default workspace namespace).
4. INV-D2: `WHERE workspace_id::text = $1`; log errors; UUID-only table parse.

## R2 — INV-03 → `public.chunks` (issue #3)

1. Replace KV-only check with `chunks.document_id` NOT EXISTS query.
2. Remove silent early-return that skips the invariant when KV is absent.
3. Optional: if `chunks` relation missing, emit schema Critical (should not happen post-002).

## R3 — Tenant slug get-or-create (issue #4)

1. `INSERT ... ON CONFLICT (slug) DO NOTHING`.
2. If no row inserted, `SELECT` by slug and return existing.
3. HTTP: `201` if new `tenant_id` matches request; `200` if returned existing.

## R4 — Node-counts GIN visibility (issue #5)

1. Inspector schema check: `idx_node_source_ids_gin` exists on `{graph}."Node"`.
2. Docs/ops SQL in [07](07-issue-05-node-counts-timeout.md) + [11](11-v22-docker-repro.md).
3. Keep SPEC-089 batch/timeout; extend smoke via E2E-104-05.

## R5 — Tutorial + release gates

1. Fix `docs/tutorials/multi-tenant.md` PK to `workspace_id`.
2. Encode blockers in [12-release-lessons.md](12-release-lessons.md).

## R6 — E2E

Implement matrix in [10-e2e-test-matrix.md](10-e2e-test-matrix.md); `cargo test` green for new cases.

## Done when

- [x] Docs 01–12 + fixtures
- [x] Code R1–R4 landed
- [x] E2E-104-01..05 green (`contract_spec104_datalayer`)
- [x] V22 repro notes captured in measurements/
- [x] Post-fix assessment ([13](13-fix-assessment.md)) + edge-case status ([09](09-edge-cases.md))

## Residual follow-ups (not blocking this patch)

| ID | Item | Owner hint |
|----|------|------------|
| EC-05 | Multi-workspace `InspectorConfig::for_namespace` loop | next inspector SPEC |
| EC-11 | 409 when slug exists but body identity differs | API product |
| EC-16 | Document in release notes: deploy after chunk backfill | ops |
| EC-18 | Retarget INV-01 off KV | SPEC-091 monitor follow-up |
