# 04 — Residual Ops (SPEC-107)

Code for E1/E2/E4 and INV-03 **monitor** ships in **≥ v0.24.0**. This page is the remaining work after upgrade.

## Upgrade checklist

1. Confirm current image: expect partner still on `0.22.0` until upgrade.
2. Follow schema path: [migrate-to-0.23.md](../../docs/operations/migrate-to-0.23.md) (API never auto-migrates).
3. Deploy **≥ 0.24.0** (prefer pin **0.24.1** — includes SPEC-106 KG persist fix unrelated to this email).
4. **Never** run a **0.22** API against a DB that already applied mig **125** (`--confirm-drop`): old INV-03 is KV-only → false CRITICAL storm. No mixed 0.22/0.24 fleets.
5. Verify health: `curl http://<host>:8080/health` → `storage_mode: postgresql`.
6. Admin inspect: `GET /api/v1/admin/storage/inspect` — expect no `42703`/`42P01` in Postgres logs; INV-03 only if orphans remain.
7. Grep Postgres logs for 24h: `42703`, `edgequake."Node"`, `tenants_slug_key` should go to zero for app-originated traffic.

Open residuals (multi-ws EC-05, 57014 capacity): [07-residual-risks.md](07-residual-risks.md).

## INV-03 orphan triage

**Do not auto-mutate status in SAFE repair.** Ops chooses requeue vs delete.

### List orphans (typed SSOT post–SPEC-091)

```sql
SELECT d.id, d.status, d.updated_at, d.filename
FROM documents d
WHERE d.status IN ('indexed', 'completed')
  AND NOT EXISTS (
    SELECT 1 FROM public.chunks c WHERE c.document_id = d.id
  )
  AND (
    NOT EXISTS (
      SELECT 1 FROM information_schema.tables
      WHERE table_schema = 'public' AND table_name = 'eq_eq_default_kv'
    )
    OR NOT EXISTS (
      SELECT 1 FROM eq_eq_default_kv k
      WHERE k.key LIKE d.id::text || '-chunk-%'
    )
  )
ORDER BY d.updated_at DESC
LIMIT 100;
```

### Decision matrix

| Finding | Action |
|---------|--------|
| Doc should exist; PDF/source available | Re-upload / requeue pipeline for that document id |
| Doc abandoned / test garbage | Delete document via API (cascades chunks/vectors/graph per current delete path) |
| Chunks exist under another workspace table | Multi-ws scope — inspect wrong namespace (SPEC-104 EC-05); fix scope then re-check |

### Sample IDs from incident dump (SPEC-104)

`19edb004-…`, `6a5d1bf3-…`, `aba5b2c5-…` — confirm still orphan after upgrade before bulk action.

## What upgrade alone clears

| Class | Cleared by binary? | Cleared by data ops? |
|-------|--------------------|----------------------|
| E1 42703 | Yes | N/A |
| E2 42P01 | Yes | N/A |
| E3 INV-03 log CRITICAL | Only if orphan count below threshold | Yes (remove or rehydrate chunks) |
| E4 23505 on app create | Yes | N/A |

## Repair recommendation (code)

`build_repair_recommendations` emits `RepairAction::LogOnly` for INV-03 (SPEC-107): points ops at sample IDs + this runbook. No SAFE delete/status rewrite.
