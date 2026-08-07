# 07 — Similar Issues Audit (migrations 117–124)

> Audit date: SPEC-110 authoring. Scope: SPEC-091 family backfills that upsert into typed tables.

## Summary

| Ver | File | Conflict target | Dedup / upsert | LAW-M1 risk | Action |
|-----|------|-----------------|----------------|-------------|--------|
| 117 | `117_spec091_dedup_backfill.sql` | `(id)` on documents; `(workspace_id, content_hash, pipeline_version)` on dedup | `SELECT DISTINCT` + **`ON CONFLICT DO NOTHING`** | **Low** — DO NOTHING allows duplicate proposed keys | None (v1) |
| **118** | `118_spec091_wsdoc_backfill.sql` | `(id)` | `SELECT DISTINCT` + **`DO UPDATE`** | **High** — membership index multi-ws | **Patch** |
| 119 | `119_spec091_artifact_backfill.sql` | `(document_id, kind)` | Key → `(uuid, kind)` bijective per key | **Low** — one key → one arbiter tuple | None |
| 120 | `120_spec091_auth_kv_purge.sql` | (purge / delete oriented) | N/A upsert pattern | **None** for U126 | None |
| **121** | `121_spec091_injection_backfill.sql` | `(id)` | No DISTINCT; workspace-prefixed injection keys + **`DO UPDATE`** | **Medium** — same id under two workspaces possible | **Harden** |
| 122 | `122_spec091_shell_backfill.sql` | `(id)` | Key is `staging:{uuid}-metadata` / `{uuid}-metadata` — id derived 1:1 from unique key | **Low** — bijective key→id | None |
| 123 | `123_spec091_job_control_states.sql` | DDL / state (no wsdoc-style multi-arbiter upsert) | — | **None** for this defect class | None |
| 124 | `124_spec091_llm_cache_drain.sql` | drain / move | — | **None** for this defect class | None |

## Detail — 117

Documents insert uses `SELECT DISTINCT (value::uuid, workspace_id, …)` then `ON CONFLICT (id) DO NOTHING`.

- Same document id under two workspaces could propose two rows with same `id`.
- **`DO NOTHING` does not raise U126** for duplicate proposed conflict keys (Postgres allows it; only one insert attempt “wins” as no-op on conflict).
- Residual: which workspace “would have” been preferred is moot because updates are not applied. Optional future harden: `DISTINCT ON (id)` for clarity — **not required for SPEC-110**.

## Detail — 119

```sql
SELECT left(kv.key, 36)::uuid, substring(kv.key FROM 38), kv.value
...
ON CONFLICT (document_id, kind) DO UPDATE …
```

Each KV key encodes a unique `(document_id, kind)` pair. Duplicate keys in one table would still be duplicate full rows; distinct keys cannot share the same conflict pair without identical kind suffix. **Safe.**

## Detail — 121

```sql
SELECT replace(split_part(kv.key, ':', 5), '-metadata', '')::uuid,  -- inj id
       split_part(kv.key, ':', 3)::uuid,  -- workspace
       …
ON CONFLICT (id) DO UPDATE SET …
```

No pre-dedup. If the same injection id were stored under two workspace prefixes, U126 fires. Structurally analogous to 118 → **harden with `DISTINCT ON (inj_id)`**.

## Detail — 122

Staging/final metadata keys embed a single UUID per key (`staging:{uuid}-metadata` or `{uuid}-metadata`). Conflict key equals that UUID. Two different keys ⇒ two different ids (or staging vs final same id but separate statements). Within one statement, duplicate keys would be identical full rows. **Safe** relative to multi-workspace membership.

## Broader rule for future migrations

Any new migration that combines:

1. a non-bijective source key family (extra columns not in the arbiter), and  
2. `ON CONFLICT (…) DO UPDATE`

must include **LAW-M2** dedup in code review / migration-guard checklist. Prefer a CI grep that flags `ON CONFLICT` + `DO UPDATE` without `DISTINCT ON` in the same file as a warning (optional follow-up; not blocking SPEC-110).

## Conclusion

Only **118** is the confirmed field failure. **121** is the required sibling harden. **117** is theoretically multi-row on conflict key but uses `DO NOTHING`. **119/122** are bijective enough. No additional in-place edits in 117–124 for v1.
