---
title: "Upgrade to EdgeQuake v0.36.0"
description: "Upgrade from v0.35.7 to v0.36.0. Schema moves from 207 to 214."
---

# Upgrade to EdgeQuake v0.36.0

> **From:** v0.35.7 (schema **207**) · **To:** v0.36.0 (schema **214**)

`edgequake migrate` applies migrations **208** through **214**.

| Migration | What it does |
|-----------|----------------|
| 208 | Row-security policies call `current_tenant_id()`, `current_workspace_id()`, and `current_user_id()` once per statement. Policy text only. |
| 209 | Ordered graph scope indexes for list pages. |
| 210 | Typo label search through an owner function, so the trigram index works under row security. |
| 211 | Adds `relationships.source_chunk_keys`. Expand only. |
| 212 | Backfills `source_chunk_keys` from graph edge lineage. Safe to re-run. On a large graph this is the slow step. |
| 213 | `tenant_invitations`. One open invite per organization and email. |
| 214 | `user_access_links`. One open magic or password-reset link per user and purpose. |

Password auth stays the demo default. This cut does not turn on Keycloak.

The published Postgres images are the same as v0.35.5: pgvector 0.8.5, `pg_trgm`, `btree_gin`, and `uuid-ossp`, and no Apache AGE. A data directory that already loaded AGE still cannot open on these images. Drop AGE with `edgequake migrate --drop-age` on the v0.35.4 image before moving that volume. See [upgrade-to-0.35.5.md](upgrade-to-0.35.5.md).

**SPEC-001 Acc:** scored on this cut, medical-mid n=200, archive [`medical-mid-20261019T141211Z`](../../specs/001-benchmark/e2e/artifacts/history/medical-mid-20261019T141211Z/) (`valid: true`). EdgeQuake **0.8240**, LightRAG **0.8016**, delta **+0.0223**, 95% CI **[-0.0132, +0.0780]** (n_paired 97). The interval includes 0, so this is a statistical tie. EdgeQuake does not claim a win.

## Operator steps

```bash
pg_dump -Fc "$DATABASE_URL" > edgequake-$(date +%F).dump
edgequake migrate
curl -sf localhost:8080/health
```

Expect `schema.latest_version` 214, `schema.pending_count` 0, `schema.migration_required` false, and `components.graph_available` true. Full command reference: [upgrading.md](upgrading.md).
