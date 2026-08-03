# 03 — Issue #1: `workspaces.id` does not exist (42703)

**Crit:** Critical · **Volume:** 2 304 · **Law:** LAW-I1, LAW-I2 · **E2E:** E2E-104-01

## Symptom (prod)

```sql
SELECT EXISTS (SELECT 1 FROM workspaces WHERE id::text = $1)
-- ERROR: 42703 undefined_column "id"
```

Hot path: StorageInspector INV-D2, startup + hourly, once per matching `eq_*_kv` / `eq_*_vectors` table.

## Why V22 has it

```ascii
 INV-D2 author assumed generic PK "id"
        │  (mirrors documents.id / wrong tutorial sketch)
        ▼
 SQL: WHERE id::text = $1
        │
        ▼
 Actual DDL (since mig 001):
   workspaces.workspace_id UUID PRIMARY KEY
        │
        ▼
 42703 every probe
        │
        ▼
 .unwrap_or(true) ──▶ assume workspace exists
        │
        ▼
 Postgres log spam + INV-D2 never flags orphans
```

**Code (pre-fix):** `edgequake-api/src/storage_inspector.rs` — `check_inv_d2_orphan_workspace_tables`.

**Schema SSOT:** `edgequake/migrations/001_init_database.sql` (`workspace_id UUID PRIMARY KEY`).

## V23 residual

**Unfixed by SPEC-091.** Same SQL still on HEAD. Mig 106–141 never renamed the PK.

## Remediation

1. `WHERE workspace_id::text = $1`.
2. On SQL error: `warn!` + treat as check failure (do **not** `.unwrap_or(true)`).
3. Parse only UUID-shaped `eq_<uuid>_kv|vectors` segments; skip non-UUID namespaces.
4. Align tutorial sketch in `docs/tutorials/multi-tenant.md`.

## Fix status (2026-08-03)

**Closed.** Grade A — see [13-fix-assessment.md](13-fix-assessment.md). Migration impact: **none** (code-only). No DDL.

## Ops note

After deploy: `grep -c 'workspaces WHERE id' /var/log/...` → 0; optional one-shot inspect via admin API.
