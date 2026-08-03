## RCA + status (SPEC-106) — fixed in **v0.24.1**

**Still present on product pin v0.24.0** (not only the reported 0.12.11). **Fixed and shipping in v0.24.1.**

### Root cause
Relationship merge calls `GraphStorage::get_edges_for_nodes_batch` **before** edge upsert. The AGE SQL joined raw `graphid` values:

```sql
JOIN vids src ON src.vid = e.start_id
JOIN vids tgt ON tgt.vid = e.end_id
```

Apache AGE does not register a usable `=` operator for `ag_catalog.graphid`, so PostgreSQL raises:

`operator does not exist: ag_catalog.graphid = ag_catalog.graphid` (SQLSTATE `42883`)

This surfaces as `1 knowledge-graph merge error(s) during persist` / `Knowledge graph persist failed …`.

### Why #214 did not fully fix this
[#214](https://github.com/raphaelmansuy/edgequake/issues/214) (v0.12.1) applied the `::text` cast pattern to **`get_nodes_with_degrees_batch`** (graph viz / degrees). The **persist** path `pg_get_edges_for_nodes_batch` was left behind and shipped through v0.24.0.

### Fix (LAW-G1) — landed in v0.24.1
Cast adjacency via `::text` (same SSOT as degrees / M072 text indexes):

```sql
JOIN vids src ON src.vid_text = e.start_id::text
JOIN vids tgt ON tgt.vid_text = e.end_id::text
```

- Spec pack: `specs/106-kg-persist-bug/`
- E2E: `e2e_spec106_graphid_edges_batch` (AGE Postgres + source guard) — **3/3 pass**
- CI: wired in `postgres-integration` (PostgreSQL AGE Graph Tests) + `spec091-data-layer`

### Upgrade
Pull / run image `ghcr.io/raphaelmansuy/edgequake:0.24.1` (and matching frontend/postgres tags). No schema migration required for this patch.
