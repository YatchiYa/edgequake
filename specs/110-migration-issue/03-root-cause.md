# 03 — Root Cause (SPEC-110)

## Five whys

1. Partner `migrate --confirm-drop` fails → sqlx reports error while executing migration **118**.
2. Postgres returns `ON CONFLICT DO UPDATE command cannot affect row a second time` (`21000`).
3. The INSERT proposes two (or more) rows that share the same conflict target `documents.id` in one statement.
4. Source query uses `SELECT DISTINCT` on `(document_id, workspace_id, …)` from KV keys `wsdoc:{workspace_id}:{document_id}` — distinct *tuples* keep multiple workspaces for one document id.
5. Legacy **wsdoc** is a workspace→document **membership index**; the same document id can appear under multiple workspaces. Relational target has a single `workspace_id` column and PK on `id`. Migration 118 never collapsed on the conflict key before upsert.

## Non-causes

| Hypothesis | Why rejected |
|------------|--------------|
| Bad `--confirm-drop` / operator misuse | Fail is SAFE SCHEMA 118; DROP OLD never reached |
| DB connectivity / pool | Preflight connected; advisory lock acquired |
| Missing workspaces FK | Rows without FK are filtered by `EXISTS`; surviving duplicates still conflict |
| Concurrent writers on `documents` | Error is intra-statement proposed-set duplication, not concurrent update |
| Migration 117 incomplete | `latest_applied=117`; 117 uses `ON CONFLICT DO NOTHING` (safe with dups) |
| Need for new append-only migration only | New version never runs if 118 still fails |

## Code (pre-fix) — migration 118

```sql
INSERT INTO public.documents (id, workspace_id, content, status)
SELECT DISTINCT split_part(kv.key, ':', 3)::uuid,
                split_part(kv.key, ':', 2)::uuid, '', 'indexed'
FROM %I kv
WHERE kv.key LIKE 'wsdoc:%%'
  ...
ON CONFLICT (id) DO UPDATE SET
    workspace_id = COALESCE(public.documents.workspace_id, EXCLUDED.workspace_id)
```

## Minimal reproduce (logical)

```text
workspaces: WS1, WS2
kv keys:    wsdoc:WS1:DOC , wsdoc:WS2:DOC
→ DISTINCT emits 2 rows with id=DOC
→ ON CONFLICT DO UPDATE touches DOC twice → 21000
```

## Code (post-fix) — normative

```sql
INSERT INTO public.documents (id, workspace_id, content, status)
SELECT DISTINCT ON (doc_id) doc_id, ws_id, '', 'indexed'
FROM (
    SELECT split_part(kv.key, ':', 3)::uuid AS doc_id,
           split_part(kv.key, ':', 2)::uuid AS ws_id
    FROM %I kv
    WHERE kv.key LIKE 'wsdoc:%%'
      ...
) src
ORDER BY doc_id, ws_id
ON CONFLICT (id) DO UPDATE SET
    workspace_id = COALESCE(public.documents.workspace_id, EXCLUDED.workspace_id)
```

## Why partner “edit 118 in repo” is incomplete alone

1. **LAW-M4** — GHCR `0.24.1` still embeds the old bytes until a new image is built/tagged.
2. **LAW-M3** — Fleets that already applied old 118 need checksum repair when the file SHA changes; partner analysis omitted this.
3. **121** — Same structural risk; harden in the same cut to avoid a second field incident.
