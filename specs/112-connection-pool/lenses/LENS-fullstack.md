# LENS — Full Stack Developer (SPEC-112)

## Path under study

```text
  HTTP / worker / migrate
        │
        ▼
  PoolRole → PgPoolBundle.for_role
        │
        ▼
  acquire ──► query / txn ──► release
        │                       │
        │                       ▼
        │              after_release: RESET ALL + search_path
        │
        ▼
  SIGTERM → HTTP drain (083) → bundle.close() (112) → exit
```

## Gaps on HEAD (code is law)

| Layer | Gap | Fix wave |
|-------|-----|----------|
| Connect | No `application_name` | A |
| Bundle options | No explicit idle/max lifetime | A |
| Shutdown | Drain without `close` | A |
| Boot | No fleet budget vs `SHOW max_connections` | B |
| Session | No idle-in-xact timeout at connect | C |
| Observe | Max not always paired with size/idle in UX | D |

## Implementation discipline

- **DRY:** one hygiene helper for single-pool and bundle.
- **SRP:** `PgPoolBundle::close` owns teardown; server owns when to call it.
- **Do not** open ad-hoc `PgPoolOptions::new()` in handlers — always go through storage adapters / bundle.
- Preserve SPEC-090 isolation e2e (T-112-14).

## Local verify loop

```bash
# After implementation
DATABASE_URL=... cargo test -p edgequake-storage --features postgres \
  --test e2e_spec090_multi_pool -- --nocapture
```

Check `pg_stat_activity.application_name` while the test process is alive.
