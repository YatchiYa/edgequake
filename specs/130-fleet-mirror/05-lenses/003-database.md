# Lens 003 — Database Expert

## Constraint

No DDL required for SPEC-130. `public.relationships` arbiter and FKs already exist (migrations 130/fleet). Application must stop discarding `RETURNING id`.

## Autocommit vs UoW

```ascii
  TODAY
  sink  pool.execute(INSERT)     → autocommit
  await barrier
  mirror pool.query(SELECT)      → sees committed rows IF ids match

  TARGET (preferred)
  sink  INSERT … RETURNING id    → map in process memory
  mirror upsert by id            → no SELECT-by-name needed
  (optional later: same sqlx::Transaction for sink+embed — not required if map is correct)
```

Visibility races are secondary once identity is retained. Optional same-transaction coupling is a hardening follow-up, not the MVP.

## Verify SQL (operator / e2e)

```ascii
  -- After a Failed typed mirror (pre-fix leftover spine):
  SELECT e.name, r.relation_type, r.id, r.source_id, r.target_id, r.created_at
  FROM relationships r
  JOIN entities e1 ON e1.id = r.source_id
  JOIN entities e2 ON e2.id = r.target_id
  WHERE e1.name = 'MELISSA_BOTHA' AND e2.name = 'FLAT_4';

  -- Duplicate-name check (deterministic miss class):
  SELECT name, count(*), array_agg(id ORDER BY created_at)
  FROM entities WHERE workspace_id = $ws
  GROUP BY name HAVING count(*) > 1;
```

## Decisions

| Decision | Choice |
|----------|--------|
| DDL | None |
| RLS | Not a factor when role bypasses RLS (reporter verified) |
| ON CONFLICT | Must RETURNING id of existing row |
| Entity lookup in batch sink | Prefer aligning with EntityNameIndex oldest-wins **or** eliminate need via UUID map (map preferred) |

## Cross-refs

- Laws: [../01-first-principles.md](../01-first-principles.md)
- Edges: [../10-edge-cases.md](../10-edge-cases.md)
- Parent: [../../098-data-access-hardening/](../../098-data-access-hardening/)
