# 01 — Test Protocol

## 1. Preconditions

- Docker Engine available; can pull from GHCR.
- Rust toolchain for building HEAD `edgequake` (`cargo build -p edgequake --features postgres`).
- Host tools: `curl`, `jq`, `python3`, `sha256sum` (or `shasum -a 256`).
- Free disk for three ephemeral Postgres volumes + dumps.

### Isolation (no EdgeForce / other apps)

- Compose projects are only `spec93soak-pg{16,17,18}` with volumes scoped to that project.
- Host publishes **ephemeral** ports on `127.0.0.1` only (never `:8787`, `:55432`, `:8080`, …).
- HEAD API binds an ephemeral loopback port; cleanup kills **only** that PID (never `make kill-app` / host-wide `pkill`).
- Harness snapshots foreign listeners before/after each major and fails the matrix if any PID changes.

## 2. Images

| Role | Image |
| --- | --- |
| Seed API | `ghcr.io/raphaelmansuy/edgequake:0.22.0` |
| Postgres PG16 | `ghcr.io/raphaelmansuy/edgequake-postgres:0.22.0-pg16` |
| Postgres PG17 | `ghcr.io/raphaelmansuy/edgequake-postgres:0.22.0-pg17` |
| Postgres PG18 | `ghcr.io/raphaelmansuy/edgequake-postgres:0.22.0-pg18` |
| Migrate / serve | HEAD binary built from this workspace |

Compose file: [`docker-compose.spec091-soak.yml`](../../docker-compose.spec091-soak.yml).  
Project name per major: `spec93soak-pg16` | `spec93soak-pg17` | `spec93soak-pg18`.

## 3. Realism seed model (default)

| Dimension | Default | Env override |
| --- | --- | --- |
| Tenants | 5 | `SPEC93_TENANTS` |
| Workspaces / tenant | 3 | `SPEC93_WORKSPACES` |
| Docs / workspace | 40 | `SPEC93_DOCS_PER_WS` |
| Upload concurrency | 8 | `SPEC93_UPLOAD_CONCURRENCY` |
| Profile | `realism` | `SPEC93_PROFILE=smoke` → 3×2×1 |

Each document body includes a unique token `TOKEN_<tenant>_<workspace>_<n>` across ≥2 paragraphs so list/isolation checks are meaningful.

**Visibility gate before dump:** ≥90% of seeded documents appear in workspace lists; sample ≥3 workspaces for non-empty list stable across two polls.

## 4. Per-major sequence

```text
1. compose pull + up (0.22.0 API + 0.22.0-pgN)
2. assert max(_sqlx_migrations) < 125
3. seed corpus → write seed.env
4. pg_dump -Fc → artifacts dump + record SHA/size in verdict
5. stop API; keep Postgres volume
6. HEAD migrate dry-run → assert DRY-RUN + 125 + IRREVERSIBLE; ledger unchanged
7. HEAD migrate (no confirm) → refuse or expandable-first soft path
8. HEAD migrate --confirm-drop → assert applied 125 + KV dropped; ledger max ≥ 137
9. start HEAD API (relational flags, EDGEQUAKE_SERVING_FENCE=on)
10. assert AC gates (see 02-acceptance-criteria.md)
11. write verdict.json + verdict.md under reports/pgN/
```

## 5. Matrix aggregation

After all majors (or the selected subset), write `reports/matrix-summary.md` with:

- PASS/FAIL per major
- Wall-clock seconds
- Postgres `version()` string
- Pre/post migration max
- Seeded doc count
- Dump SHA256 + byte size (dump binary may live under `artifacts/spec93-migration-assessment/`)

## 6. Pass / fail rules

- Any hard `die` in the harness → major FAIL.
- Soft WARN lines allowed; FAIL counter must be 0.
- Matrix PASS only if every requested major is GREEN.

## 7. Smoke profile

`SPEC93_PROFILE=smoke` (used by `make spec091-upgrade-soak`) keeps the historical tiny corpus for fast local iteration. Smoke does **not** satisfy AC-M-02 / AC-M-08.
