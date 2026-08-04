# 00 — Issue Data (Partner Email)

> Source: partner message to Raphaël Mansuy (Quantalogic prod).  
> Image / env (from same incident class as SPEC-104): `ghcr.io/raphaelmansuy/edgequake:0.22.0`, `quantalogic-prod-db`.  
> Engineering dump with fifth timeout issue: [SPEC-104 00-issue-data](../104-fix-datalayer/00-issue-data.md).

## Original (French, abridged)

Quatre classes d’erreurs dans les logs PROD (dernières 24h) :

1. `column "id" does not exist` sur la table `workspaces` (**~2300** occurrences)
2. `relation "edgequake.Node" does not exist` (**24** occurrences) — CTE `prefixes` / `probes` / `hits` joignant `edgequake."Node"` sur `source_ids`
3. SPEC-021 P-D1 hourly invariant — **CRITICAL** INV-03: 20 indexed documents have no KV chunks (SAGA failure?)
4. `duplicate key value violates unique constraint "tenants_slug_key"` (**6** occurrences) on `INSERT INTO tenants (... slug ...)`

Question: *Est-ce que ce sont des erreurs que tu constates toi aussi ?* Offre de session détail.

## English extract

| # | SQLSTATE / signal | Volume / 24h | Hot path |
|---|-------------------|--------------|----------|
| E1 | `42703` undefined_column `workspaces.id` | ~2300 | StorageInspector INV-D2 (hourly × ~N tables) |
| E2 | `42P01` undefined_table `edgequake."Node"` | 24 | StorageInspector INV-C CTE (1×/hour) |
| E3 | INV-03 CRITICAL drift | 24 (hourly log) | Indexed docs without chunk body |
| E4 | `23505` unique_violation `tenants_slug_key` | 6 (burst) | `POST` tenant create / retry |

## Sample query (E2) — partner paste

```sql
WITH prefixes AS MATERIALIZED (
  SELECT prefix, ord
  FROM unnest($1::text[]) WITH ORDINALITY AS t(prefix, ord)
),
probes AS MATERIALIZED (
  SELECT p.prefix, p.ord, (p.prefix || gs.i::text) AS chunk_id
  FROM prefixes p
  CROSS JOIN generate_series(0, $2::int - 1) AS gs(i)
),
hits AS MATERIALIZED (
  SELECT pr.prefix, pr.ord, v.id
  FROM probes pr
  INNER JOIN edgequake."Node" v
    ON ((ag_catalog.agtype_to_json(v.properties))::jsonb -> 'source_ids')
       @> to_jsonb(pr.chunk_id)
)
SELECT p.prefix, count(DISTINCT h.id)::BIGINT AS cnt
FROM prefixes p
LEFT JOIN hits h ON h.prefix = p.prefix
GROUP BY p.prefix, p.ord
ORDER BY p.ord
```

## Sample INSERT (E4)

```sql
INSERT INTO tenants (tenant_id, name, slug, is_active, metadata, settings, created_at, updated_at)
VALUES ($1, $2, $3, $4, $5, '{}'::jsonb, $6, $7)
```

## Acknowledgement in thread

> Ok je prends le point
