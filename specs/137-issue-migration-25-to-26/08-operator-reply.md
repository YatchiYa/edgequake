# 08 — Operator reply (anonymized)

Upgrade **0.25 → 0.26** applied SAFE SCHEMA (migration **149**). Remaining
items are **optional DROP OLD** from the relational cutover (125 KV, 126 chunk
vectors, 131 vector fleet), not a 149 defect. Serving can stay up with those
pending.

Use the canonical flag (alias accepted after this fix):

```bash
edgequake migrate dry-run
edgequake migrate guard
# when lights are GREEN and backup exists:
edgequake migrate --confirm-drop
# or: edgequake migrate --drop-confirm
edgequake migrate   # deferred 142 emptiness assert
```

Do **not** set `EDGEQUAKE_MIGRATION_CONFIRM_DROP=1` in a shared env file.

If confirm-drop **errors**:

- `Wave D ABORT` — KV rows not in typed tables; finish family backfills.
- `W4 ABORT` — run `w3-chunk-embedding-backfill`.
- `IW2 ABORT` — run `iw2-fleet-embedding-backfill` / `iw2-fleet-provenance-stamp`.
- checksum drift — one-shot `EDGEQUAKE_ALLOW_CHECKSUM_REPAIR=125,131` then unset.
- Rollback after a successful drop = **restore from backup** (no undo SQL).

`migrate guard` never applies schema. Full sequence: [09-ops-runbook.md](09-ops-runbook.md).
