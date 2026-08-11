# Lens 007 — System Engineer

## Runtime topology

```ascii
  ingest workers (N concurrent)
       │
       ▼
  pipeline merge ──► Postgres
       │                 │
       │                 ├─ entities UNIQUE (exact name)
       │                 └─ entity_embeddings
       │                      PK + partial UNIQUE(lid)
       ▼
  typed authority fail-closed on StorageError
```

## Failure modes mitigated

| Mode | Before | After |
|------|--------|-------|
| Dual FK same lid | 23505 → GraphMerge | Absorb → Ok |
| Cross-WS same lid | Allowed (144) | Unchanged |
| Stamp overwrite | COALESCE protects | UPDATE WHERE NULL |
| Compensation on absorb | Could fire | Must not (Ok path) |

## Ops signals

- `RUST_LOG` warn when `absorbed_legacy_collisions > 0`
- Diagnostic duplicate entities query (from #374) still useful for SPEC-083 backlog
- No new migration required for P0 (behavior-only)

## Capacity

Absorb removes need to pin ingest concurrency for this error. Keep pool sizes / statement timeouts as configured; this is not a timeout-class defect.
