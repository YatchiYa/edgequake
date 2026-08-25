# 09 — Ops runbook (0.25 → 0.26 including leftover 091)

Confirm-drop remains consent-gated. Backup before DROP OLD.

```text
1. Backup (pg_dump -Fc / volume snapshot)
2. Deploy v0.26.x images; do not require confirm for 149
3. edgequake migrate
   - applies SAFE SCHEMA (149 if pending)
   - leftover 125/126/131 stay pending (legal to serve)
4. If checksum drift on already-applied 125/131:
   EDGEQUAKE_ALLOW_CHECKSUM_REPAIR=125,131 edgequake migrate
   then UNSET the var
5. If drop-readiness RED:
   - w3-chunk-embedding-backfill
   - iw2-fleet-embedding-backfill
   - iw2-fleet-provenance-stamp
   EDGEQUAKE_MIGRATION_VERIFY_EQUALITY=0 if regenerated embeddings
6. edgequake migrate dry-run
   edgequake migrate guard
7. GREEN + backup:
   edgequake migrate --confirm-drop
   (alias: --drop-confirm)
8. edgequake migrate   # 142 emptiness assert
9. Verify /health version 0.26.x + PDF ingest smoke
```

## 149 does not need confirm

`ALTER TABLE tasks ADD COLUMN IF NOT EXISTS document_id` — expandable.

## AGE

Do not `DROP SCHEMA <graph> CASCADE`. Knowledge graph stays in AGE.

## Detail

- Cluster A / stamp: [`specs/111-issues/09-ops-runbook.md`](../111-issues/09-ops-runbook.md)
- From 0.22: [`docs/operations/spec091-upgrade-from-v0.22.0.md`](../../docs/operations/spec091-upgrade-from-v0.22.0.md)
- Product pin: [`docs/operations/upgrade-to-0.26.1.md`](../../docs/operations/upgrade-to-0.26.1.md)
  (leftover 091 ladder: [`upgrade-to-0.26.0.md`](../../docs/operations/upgrade-to-0.26.0.md))
