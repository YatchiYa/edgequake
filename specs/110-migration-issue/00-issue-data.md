# 00 — Issue Data (Partner PPD)

> **Source:** Partner thread (migration failure analysis + docker log).  
> **Image:** `ghcr.io/raphaelmansuy/edgequake:0.24.1`  
> **Command:** `migrate --confirm-drop`  
> **DB:** `postgresql://edgequake:***@10.79.3.189:35656/edgequake`  
> **Timestamp:** `2026-08-05T14:38:48Z` (log)

## Command

```bash
docker run --rm --env-file /etc/edgequake/.env \
  ghcr.io/raphaelmansuy/edgequake:0.24.1 migrate --confirm-drop
```

## Observed banner / preflight (abridged)

```text
EdgeQuake migrate v0.24.1
database: postgresql://edgequake:***@10.79.3.189:35656/edgequake

preflight: 24 pending migration(s)
  pending 118 — spec091 wsdoc backfill  [SAFE SCHEMA — expandable]
  pending 119 — spec091 artifact backfill  [SAFE SCHEMA — expandable]
  ...
  pending 125 — spec091 kv drop  [DROP OLD — irreversible KV tables]
  pending 126 — spec091 vector drop  [DROP OLD — irreversible chunk vectors]
  pending 131 — spec091 fleet vector drop  [DROP OLD — irreversible vector fleet]
  ...
  pending 142 — spec105 legacy cutover assert  [ASSERT — SPEC-105]

APPLY INTENT
  pending total:              24
  SAFE SCHEMA (will apply):   21
  DROP OLD (needs confirm):    3 → [125, 126, 131]
  consent: INCLUDED — --confirm-drop / fresh-install gate open
```

## Migration bootstrap log (failure)

```json
{"message":"Database migration bootstrap starting","step":"bootstrap_start","total_embedded":140,"migrate_cli":true,"mode":"All","max_version":"None"}
{"message":"Migration preflight complete","step":"preflight","applied":116,"pending":24,"latest_applied":117}
{"message":"Pending migration queued","step":"pending","progress":"1/24","version":118,"description":"spec091 wsdoc backfill"}
...
{"message":"Applying sqlx migrations (advisory lock held)","step":"sqlx_run","count":24}
```

## Error

```text
migrate failed: while executing migration 118: error returned from database:
  ON CONFLICT DO UPDATE command cannot affect row a second time
hint: re-run with RUST_LOG=edgequake.migration=info,edgequake=info; if stuck on tasks DDL,
  check pg_locks / other backends holding locks on public.tasks
Error: migrate failed

Caused by:
    0: while executing migration 118: error returned from database:
       ON CONFLICT DO UPDATE command cannot affect row a second time
    ...
    3: ON CONFLICT DO UPDATE command cannot affect row a second time
```

## Facts extracted (code is law)

| Fact | Value |
|------|-------|
| Product version in image | `0.24.1` |
| Latest successful migration | **117** |
| First failing migration | **118** (`spec091 wsdoc backfill`) |
| Postgres class | Cardinality violation (`21000`) |
| Consent / DROP OLD | Not reached — fail before 125/126/131 |
| sqlx record of 118 | **Not** success (transaction rolled back) |
| Partner hypothesis | New EdgeQuake version required — **correct** |

## Repo smoking gun (v0.24.1 / current tree pre-SPEC-110 fix)

File: `edgequake/migrations/118_spec091_wsdoc_backfill.sql`

```sql
INSERT INTO public.documents (id, workspace_id, content, status)
SELECT DISTINCT split_part(kv.key, ':', 3)::uuid,  -- document_id (conflict key)
                split_part(kv.key, ':', 2)::uuid,  -- workspace_id
                '', 'indexed'
FROM %I kv
WHERE kv.key LIKE 'wsdoc:%%'
...
ON CONFLICT (id) DO UPDATE SET
    workspace_id = COALESCE(public.documents.workspace_id, EXCLUDED.workspace_id)
```

Lockfile (broken content):

```text
331967467fdbeb58aeeb41ca92b6e3ec87ee84ace9286166275e14af9699a4cb862f1a92516043ee9c2489138a560629  118_spec091_wsdoc_backfill.sql
```

## Partner-proposed SQL direction (accepted baseline)

Use `DISTINCT ON (doc_id) … ORDER BY doc_id, ws_id` so each conflict key appears once. Same harden for migration **121**. SPEC-110 **adds** the missing checksum-repair + release requirements the partner analysis omitted — see [04-fix-plan.md](04-fix-plan.md).
