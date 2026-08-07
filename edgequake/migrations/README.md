# EdgeQuake SQL migrations

SSOT for schema evolution applied by sqlx at API bootstrap (`migration_bootstrap`).

## Tasks table PK (X-01)

| Migration | Role |
|-----------|------|
| `001_init_database.sql` | **SSOT** — creates `tasks` with `id UUID PRIMARY KEY` (+ tenant/workspace columns). |
| `002_add_tasks_table.sql` | **Dead / no-op** on any DB that already ran 001. Its `CREATE TABLE IF NOT EXISTS tasks` uses a legacy `track_id VARCHAR(50) PRIMARY KEY` shape that never applies after 001. |
| `026_fix_task_type_constraint.sql` | Documents that 002's CHECK is usually absent; repairs constraint names for odd deploy paths. |

Do **not** re-run or “fix” 002 expecting a PK change. Fresh installs inherit the 001 UUID PK. Operators inspecting `_sqlx_migrations` should treat 002 as a historical no-op marker.

## Audit logs (D-45)

| Source | Role |
|--------|------|
| `001_init_database.sql` | **SSOT** — partitioned `public.audit_logs` + `create_next_audit_log_partition()`. |
| `012_audit_logs_table.sql` | `CREATE TABLE IF NOT EXISTS` reconcile for upgrades that predate 001's audit block. |
| `005_add_audit_log_table.sql` | Different table: `edgequake.audit_log` (graph-edit audit), not security `audit_logs`. |
| `docker/init.sql` | Container bootstrap mirror of 001 (not a second migration SSOT). |

Bootstrap calls `SELECT create_next_audit_log_partition()` so month+1 inserts do not fail after the initial 12-month window.

## Checksum drift (X-02 / LAW-MIG)

**Default rule: never edit applied / shipped migration SQL.** Fix with a **new** migration.  
Full decision tree: [`specs/111-issues/10-migration-immutability.md`](../../specs/111-issues/10-migration-immutability.md).

Known broken→fixed checksum repairs (071/078/118/121/125/131) rewrite `_sqlx_migrations.checksum` only when authorized:

| Auth | Use |
|------|-----|
| `EDGEQUAKE_ALLOW_CHECKSUM_REPAIR=71,78,…` | **Preferred** — scoped; set by `make_dev` migrate |
| `EDGEQUAKE_DEV_MODE=true` | Broad (also affects auth); legacy / local |

Production leaves both unset → fail loud. Modules: `migration_bootstrap/reconcile/mNNN.rs` + shared `checksum_repair.rs`.
