# Lens — Database Expert

## Verdict

**No schema migration.** SPEC-144 is a frontend framework and network-boundary
upgrade. Persistence, AGE graph, pgvector, and chunk page lineage are untouched.

## What does not change

| Store | Status |
|-------|--------|
| Postgres schemas | Unchanged |
| Chunk `page_start` / `page_end` | Unchanged (SPEC-135) |
| Document markdown markers | Unchanged (SPEC-083 X-13) |
| KV / vector stores | Unchanged |
| Auth tokens in DB | Unchanged (cookie mirror is client/FE only) |

## Read paths that must keep working (no DB change)

1. Document detail → markdown + PDF (SPEC-143).
2. Query SSE → backend stream (proxy compression invariant).
3. Health / ready rewrites in dev (no DB role).

## Non-goals

- New tables for navigation shells or cache tags.
- Persisting Instant Navigations preferences.
- Migrations for Next.js version metadata.

## Integrity

Ops check remains: backend health + existing migrations. No Flyway/SQL PR
for this spec.

## Cross-refs

- Acceptance: [10-acceptance.md](../10-acceptance.md)
- SPEC-143 DB lens: no migration there either
