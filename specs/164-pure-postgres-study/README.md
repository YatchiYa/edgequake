# SPEC-164 — Pure PostgreSQL Graph

> **Status:** **Shipped in v0.35.0** — plain `eq_graph_nodes` / `eq_graph_edges` are the product graph.  
> **Question:** Is Apache AGE still necessary for EdgeQuake GraphRAG?  
> **Answer:** **No.** Measured latency and hop correctness at ≤1M nodes do not need it. Dropping the extension is a migrate question; the default is to leave it installed.  
> **Why it matters:** Hosted Postgres (RDS, Cloud SQL, AlloyDB) ships **pgvector** and does not ship Apache AGE. EdgeQuake can run on that Postgres.  
> **Proof:** study benches ([`reports/PROOF.md`](reports/PROOF.md), 2026-10-11) · product write-up ([`reports/WHY-AGE-IS-OPTIONAL.md`](reports/WHY-AGE-IS-OPTIONAL.md)) · article ([`articles/017-plain-postgres-graph/`](../../articles/017-plain-postgres-graph/)).

## Install on Postgres without Apache AGE

Fresh install needs PostgreSQL 18 (or 16/17), **pgvector**, **pg_trgm**, **btree_gin**, and **uuid-ossp**. The `age` package is optional.

```bash
# Example: official pgvector image (no AGE package in the image)
docker run -d --name eq-no-age \
  -e POSTGRES_PASSWORD=edgequake \
  -e POSTGRES_USER=edgequake \
  -e POSTGRES_DB=edgequake \
  -p 127.0.0.1:54329:5432 \
  pgvector/pgvector:0.8.6-pg18-trixie

export DATABASE_URL='postgres://edgequake:edgequake@127.0.0.1:54329/edgequake'

# Apply SAFE SCHEMA through 206. Migration 171 (DROP AGE) stays pending.
# Non-interactive migrate never drops AGE. Answer the question only when you mean to.
edgequake migrate

# Serve. Boot skips CREATE EXTENSION age when the cutover ledger marks age_absent
# (greenfield without AGE does that in migration 170).
edgequake
```

Pass criteria:

| Check | Expect |
|-------|--------|
| `SELECT EXISTS (SELECT 1 FROM pg_available_extensions WHERE name = 'age')` | `false` on that image |
| `SELECT EXISTS (SELECT 1 FROM pg_extension WHERE extname = 'age')` | `false` after migrate |
| `SELECT status, age_absent FROM eq_graph_cutover WHERE id = 1` | `ready`, `true` |
| `curl -sf "$API/health"` | `status=healthy`, graph storage up |
| Plain node write + `search_nodes` | succeeds |

Upgrade from v0.34.0 (schema 169): [docs/operations/upgrade-to-0.35.0.md](../../docs/operations/upgrade-to-0.35.0.md). Dropping AGE on an existing database is `edgequake migrate --drop-age` after the cutover ledger is `ready`. `--confirm-drop` does not answer that question.

```mermaid
flowchart LR
  pg["Postgres + pgvector"] --> m170["migrate 170 plain tables"]
  m170 --> head["migrate 172–206 indexes"]
  head --> serve["serve on eq_graph_*"]
  m170 -.->|optional| m171["--drop-age only if asked"]
```

## Why AGE is not required (numbers)

| Evidence | Number |
|----------|--------|
| SCALE-M worst plain/AGE latency ratio | **1.087** (degree batch) |
| SCALE-M upsert plain/AGE | **0.286** (plain faster) |
| BFS close on AGE parent heaps vs plain edges | **165.302 ms** vs **0.779 ms** |
| Neighbor-set equality | **50/50** on S, M, Hub, L |
| Fresh train on Postgres without AGE | **203** steps → schema **206** |

Full write-up: [`reports/WHY-AGE-IS-OPTIONAL.md`](reports/WHY-AGE-IS-OPTIONAL.md). Study decision: [09-decision.md](09-decision.md).

## What shipped in v0.35.0

| Piece | Role |
|-------|------|
| Migration **170** | `eq_graph_nodes` / `eq_graph_edges`, mirror triggers, cutover ledger |
| Migration **171** | Optional `DROP EXTENSION age` — migrate asks; default is keep |
| Migrations **172–206** | Request-path indexes and RLS-safe predicates |
| Boot | Skips `CREATE EXTENSION age` when the ledger marks AGE absent |
| Health | `graph_available` when AGE is loaded **or** the plain ledger is ready |

The published `edgequake-postgres` image can still include the AGE library so an existing database can backfill before anyone answers yes to the drop.

## Study archive (reading order)

The measurement study that unlocked this cut:

1. [00-five-whys.md](00-five-whys.md)  
2. [01-first-principles.md](01-first-principles.md)  
3. [09-decision.md](09-decision.md)  
4. [12-v035-migration-plan.md](12-v035-migration-plan.md)  
5. [reports/PROOF.md](reports/PROOF.md)  

Optional re-run of the scratch benches (needs a Postgres that has AGE for ARM-A):

```bash
export SPEC164_PG_CONTAINER="${SPEC164_PG_CONTAINER:-eqdoc-scratch}"
./specs/164-pure-postgres-study/measurements/run_bench.sh
./specs/164-pure-postgres-study/measurements/run_inventory_bench.sh
```
