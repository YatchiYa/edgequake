# SPEC-104 measurements

## V22 local repro (2026-08-03)

Stack: `fixtures/docker-compose.v22-repro.yml` · image `0.22.0` · ports API 18080 / PG 15432.

| Issue | Reproduced? | Evidence |
|-------|-------------|----------|
| #1 `workspaces.id` 42703 | **Yes** | `v22-sql-error-classes.txt`: `ERROR: column "id" does not exist` |
| #2 `edgequake."Node"` 42P01 | **Yes** | same file: `relation "edgequake.Node" does not exist`; graph SSOT = `eq_eq_default_graph` |
| #3 INV-03 | **Yes** | `v22-inspect.json`: INV-03 Warning, sample `bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb` |
| #4 tenants slug | **Yes** | first POST 201, second 400 `Tenant with slug 'novagen-orga' already exists` |
| #5 GIN / timeout | GIN **present** on fresh install (`idx_node_source_ids_gin`); 57014 not forced (needs scale) |

Artifacts: `v22-health.json`, `v22-seed.out`, `v22-sql-error-classes.txt`, `v22-inspect.json`, `v22-tenant-create-status.txt`.

## Post-fix (v23) staging inspect (2026-08-03)

Stack: local `make` / `BACKEND_URL=http://localhost:8090` · binary `0.23.0` build `2026-08-03T10:11:39Z` · git `9dc3a3317` · PostgreSQL 18.

| Gate | Result | Evidence |
|------|--------|----------|
| #1 42703 `workspaces.id` | **PASS** — zero since MARKER | `v23-errcode-grep-pg-since-marker.txt`; PK = `workspace_id` in `v23-sql-spotchecks.txt` |
| #2 42P01 `edgequake."Node"` | **PASS** — zero since MARKER | same; inspect uses `eq_eq_default_graph` |
| Inspect health | **PASS** — `has_critical=false`, 0 schema issues, 0 violations | `v23-inspect.json` |
| #3 INV-03 | **PASS** — no violations on this DB | `v23-inspect.json` (true orphans would still fire) |
| #5 GIN present | **PASS** (observability) | `eq_eq_default_graph.idx_node_source_ids_gin` |
| Contracts | **PASS** 11/11 (E2E-104-10 soft-skip: AGE `create_graph`/`graphid_ops`) | `v23-contract-spec104.txt` |
| `e2e_issue331` | **No matching test binary name in filter** | `v23-e2e-issue331.txt` (filter empty); GIN proven via SQL spotcheck |

**Note:** Full postgres/backend logs still contain **historical** pre-fix 42703/42P01 from older binaries (`v23-errcode-grep-pg.txt`). Ship gate is **zero new hits after MARKER** + clean inspect on the SPEC-104+ binary.

Assessment: [../13-fix-assessment.md](../13-fix-assessment.md).
