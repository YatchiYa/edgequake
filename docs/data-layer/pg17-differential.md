---
title: "PG17 differential measurement (SPEC-091 IW4)"
---

# PG17 differential measurement (SPEC-091 IW4)

**Status:** closed (LAW-I2) — measured against the IW1 typed-CRUD scorecard; **no PG17-specific SQL adopted**.

## Scope

GAP-091-31 asked whether PostgreSQL 17 planner improvements (skip scan, improved join order, etc.) justify diverging SQL or index shapes from the unified PG16/PG17/PG18 path.

## Method

- Same corpus and harness as IW1 (`perf_harness` / typed relational CRUD on `documents`, `chunks`, `chunk_embeddings`).
- Compared PG17 (`edgequake-postgres:pg17`) vs PG16 baseline on identical migration state and extension pins (pgvector **0.8.5**, AGE **1.7.0**).
- Looked for ≥10% p95 improvement on list/delete/search paths that would justify version-branched SQL.

## Result

No operation met the adoption bar. PG17 showed planner variance within noise on our indexed paths; the unified SQL (including the UNION workspace delete in `document_read_model.rs` and listing indexes from migration 128) remains the product default on all supported majors.

## Decision

- **Keep unified SQL** gated only by capability probes (`capabilities.rs`), not `server_version_num` string checks.
- **No PG17-only migrations** in this release train.
- Revisit only when a measured regression or ≥10% win appears on a named ref ID in [`version-matrix.md`](version-matrix.md).

## Related

- Capability matrix: `/health` → `schema.postgres_capabilities`
- Nightly full matrix: `.github/workflows/postgres-matrix-nightly.yml`
- PR smoke: `.github/workflows/spec091-data-layer.yml` (`spec091-pg-matrix-smoke`)
