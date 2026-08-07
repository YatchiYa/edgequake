# 09 — Ops Runbook (SPEC-110)

> Audience: operators upgrading Postgres fleets past SPEC-091 migration 118.  
> Related: [spec091-upgrade-from-v0.22.0](../../docs/operations/spec091-upgrade-from-v0.22.0.md) · [release-and-cd](../../docs/operations/release-and-cd.md)

## Prerequisites

1. Verified backup (`pg_dump -Fc` recommended).
2. Image pin **≥ 0.24.2** (SPEC-110) — do **not** expect `0.24.1` to pass multi-ws wsdoc backfill.
3. `DATABASE_URL` / env-file reachable from the migrate one-shot container.
4. Maintenance window sized for pending train (118→142 on long-lived KV fleets).

## Diagnose

```bash
docker run --rm --env-file /etc/edgequake/.env \
  ghcr.io/raphaelmansuy/edgequake:0.24.2 migrate status
# or: … migrate dry-run
```

| Observation | Interpretation |
|-------------|----------------|
| `latest_applied: 117`, pending includes 118 | Partner PPD class — need patched 118 body |
| Fail message contains `cannot affect row a second time` on 118 | Classic SPEC-110; upgrade image |
| `migration 118 was previously applied but has been modified` | Old 118 checksum vs new binary — use Path B |
| Pending starts after 118 | Unrelated; follow normal 091 runbook |

Optional SQL:

```sql
SELECT version, description, success, encode(checksum, 'hex')
FROM _sqlx_migrations
WHERE version IN (117, 118, 121)
ORDER BY version;
```

Detect multi-ws membership residue (read-only):

```sql
-- Adjust eq_*_kv table name(s) for your fleet
SELECT split_part(key, ':', 3) AS doc_id, count(DISTINCT split_part(key, ':', 2)) AS ws_n
FROM eq_YOUR_kv
WHERE key LIKE 'wsdoc:%'
GROUP BY 1
HAVING count(DISTINCT split_part(key, ':', 2)) > 1
LIMIT 50;
```

## Path A — Stuck at ≤117 (never applied 118)

**Partner PPD path.**

```bash
# 1. Pin patched image
export EDGEQUAKE_VERSION=0.24.2

# 2. Optional: SAFE SCHEMA only first (omit --confirm-drop) if you stage DROP OLD later
docker run --rm --env-file /etc/edgequake/.env \
  ghcr.io/raphaelmansuy/edgequake:${EDGEQUAKE_VERSION} migrate

# 3. When ready for irreversible drops (125/126/131):
docker run --rm --env-file /etc/edgequake/.env \
  ghcr.io/raphaelmansuy/edgequake:${EDGEQUAKE_VERSION} migrate --confirm-drop
```

No checksum repair required for 118/121 if those versions never succeeded.

## Path B — Already applied old 118/121 (checksum drift)

After pulling 0.24.2, migrate/bootstrap may refuse:

```text
migration 118 was previously applied but has been modified
```

**Preferred (controlled one-shot):**

```bash
docker run --rm --env-file /etc/edgequake/.env \
  -e EDGEQUAKE_DEV_MODE=true \
  ghcr.io/raphaelmansuy/edgequake:0.24.2 migrate status
# then migrate as needed; unset DEV_MODE afterward
```

Repair rewrites `_sqlx_migrations.checksum` for known-broken → fixed SHA only (does **not** re-execute 118 SQL).

**Manual alternative** (if DEV_MODE forbidden by policy):

```sql
-- Use FIXED digests from checksums.lock / m118 constants after implementation
UPDATE _sqlx_migrations
SET checksum = decode('<FIXED_118_SHA384_HEX>', 'hex')
WHERE version = 118 AND success = true;

UPDATE _sqlx_migrations
SET checksum = decode('<FIXED_121_SHA384_HEX>', 'hex')
WHERE version = 121 AND success = true;
```

Then re-run migrate **without** leaving DEV_MODE on in production env files.

## Path C — Fresh install on 0.24.2+

Normal `migrate` / compose one-shot. Fixed 118/121 apply once; lockfile SHA stored.

## Post-migrate checks

```bash
docker run --rm --env-file /etc/edgequake/.env \
  ghcr.io/raphaelmansuy/edgequake:0.24.2 migrate status
```

Expect: no pending SAFE SCHEMA through current product max (142 on 0.24.x train), or only intentional deferred DROP OLD if you skipped `--confirm-drop`.

```sql
-- Spot-check collapsed membership
SELECT id, workspace_id, status FROM public.documents
WHERE id = '<DOC_THAT_HAD_MULTI_WSDOC>' ;
```

Remember LAW-M5: one workspace wins; other wsdoc keys may still exist in KV until drop **125**.

## Rollback

- Failed 118 on 0.24.1: no schema change recorded — restore not required solely for the failed statement; fix is upgrade forward.
- After successful DROP OLD (125+): rollback = **restore from backup** only.

## Logging

```bash
docker run --rm --env-file /etc/edgequake/.env \
  -e RUST_LOG=edgequake.migration=info,edgequake=info \
  ghcr.io/raphaelmansuy/edgequake:0.24.2 migrate --confirm-drop
```

Capture logs under `specs/110-migration-issue/measurements/` for incident closeout.
