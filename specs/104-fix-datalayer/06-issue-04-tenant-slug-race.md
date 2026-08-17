# 06 — Issue #4: Duplicate key on `tenants_slug_key` (23505)

**Crit:** Medium · **Volume:** 6 (burst) · **Law:** LAW-I3 · **E2E:** E2E-104-04

## Symptom (prod)

```sql
INSERT INTO tenants (tenant_id, name, slug, ...) VALUES (...)
-- ERROR: 23505 unique_violation on tenants_slug_key
```

Slugs: `novagen-orga`, `novagen-orga-cff5cf8b`.

## Why V22 has it

```ascii
 Client / WebUI
   name="Novagen Orga"  (often no slug)
        │
        ▼
 generate_slug → "novagen-orga"   (deterministic)
 Tenant::new → fresh UUID every call
        │
        ▼
 plain INSERT (no ON CONFLICT)
        │
        ├─ first wins
        └─ concurrent / retry ──▶ 23505
                 │
                 ▼
 External client retries with suffix -cff5cf8b
                 │
                 └─ double-fire again ──▶ 23505 on suffixed slug
```

**Code:** `tenant_ops.rs` `pg_create_tenant`; handler maps duplicate → `ApiError::BadRequest` (not idempotent get).

Boot `ensure_defaults` is idempotent on `tenant_id` only — does not help user-created tenants.

## V23 residual

**Unfixed.** Same INSERT shape.

## Remediation (chosen)

**Get-or-create by slug** (matches Quantalogic retry pattern):

```sql
INSERT INTO tenants (...)
VALUES (...)
ON CONFLICT (slug) DO NOTHING
```

If `rows_affected == 0`, `SELECT ... WHERE slug = $1` and return existing tenant.

HTTP:

- New row → `201 Created`
- Existing slug + **same name** → `200 OK` (idempotent retry)
- Existing slug + **different name** → `409 Conflict` with existing `tenant_id` (EC-11)

Atomic SQL: `ON CONFLICT (slug) DO UPDATE SET name = tenants.name RETURNING …`.

## Fix status (2026-08-03 A+)

**Closed.** Grade **A+** ([13](13-fix-assessment.md) v3).  
`Error::Conflict` enforced in `WorkspaceService` (PG + in-memory); API maps to HTTP 409. Migration impact: none.
