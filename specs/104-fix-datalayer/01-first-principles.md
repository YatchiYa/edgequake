# 01 — First Principles (SPEC-104)

> Method: reduce the five production failures to what a **monitor + write path** minimally requires, state axioms, derive enforceable laws (`LAW-I1..I4`). Every remediation in [08-remediation-plan.md](08-remediation-plan.md) traces to a law. Nothing is justified by taste.

## Axioms

1. **A relation name is an identity.** Code that invents a second name for the same relation (e.g. `"edgequake"` vs `eq_eq_default_graph`) creates a second, false universe that only appears as `42P01` / `42703`.
2. **A monitor that swallows errors is worse than no monitor.** Fail-open (`.unwrap_or(true)`, early `return` on missing table) converts integrity defects into silent green health while Postgres logs spam.
3. **A UNIQUE constraint without an idempotent write is a race amplifier.** Concurrent or retried creates that mint a new PK each time collide on the natural key (`slug`) forever.
4. **Work proportional to fleet size must not invent unbounded SQL per request.** Counts and reconciles need indexes, batches, and timeouts — or they become `57014` under load.

## Laws

| Law | Statement | Derived from | Honored on v0.22? | Honored on v0.23 HEAD (pre-104)? |
|-----|-----------|--------------|-------------------|----------------------------------|
| **LAW-I1 — Schema-name SSOT** | Inspector / admin / storage derive relation names from one helper (`PostgresConfig::table_prefix` → `eq_{prefix}_graph`, `workspace_id` PK). No hardcoded `"edgequake"` / `"id"`. | Axiom 1 | Violated (#1, #2) | Violated |
| **LAW-I2 — Fail-visible monitors** | Monitor SQL errors are logged and counted as check failures; they never masquerade as “healthy / no orphans / no drift”. | Axiom 2 | Violated (#1 unwrap_or, #3) | Worse (#3 silent post-125) |
| **LAW-I3 — Idempotent unique writes** | User-facing INSERT targeting a UNIQUE natural key is get-or-create or `ON CONFLICT` with a defined HTTP contract. | Axiom 3 | Violated (#4) | Violated |
| **LAW-I4 — Bounded lineage probes** | Node/edge counts by `source_ids` use child `"Node"`, M038 GIN, batch caps, statement_timeout; missing GIN is a visible schema finding. | Axiom 4 | Partially (SPEC-089) | Same |

## DRY / SOLID map

```ascii
 FIRST PRINCIPLE     DRY                      SOLID                     SSOT
 ┌──────────────┐  ┌────────────────────┐  ┌─────────────────────┐  ┌──────────────────┐
 │ LAW-I1       │─▶│ one naming helper  │  │ DIP: inspector      │─▶│ PostgresConfig   │
 │ schema names │  │ shared w/ storage  │  │  depends on prefix  │  │ table_prefix()   │
 ├──────────────┤  ├────────────────────┤  ├─────────────────────┤  ├──────────────────┤
 │ LAW-I2       │─▶│ one error path for │  │ SRP: monitor reports│─▶│ InspectorReport  │
 │ fail-visible │  │ SQL failures       │  │  truth, not hope    │  │ is health SSOT   │
 ├──────────────┤  ├────────────────────┤  ├─────────────────────┤  ├──────────────────┤
 │ LAW-I3       │─▶│ one create path    │  │ ISP: create vs get  │─▶│ tenants.slug     │
 │ idempotent   │  │ for tenants        │  │  stay explicit      │  │ unique natural   │
 ├──────────────┤  ├────────────────────┤  ├─────────────────────┤  ├──────────────────┤
 │ LAW-I4       │─▶│ one count SQL      │  │ OCP: GIN via mig    │─▶│ idx_node_source_ │
 │ bounded      │  │ (analytics+INV-C)  │  │  038, not ad-hoc    │  │ ids_gin          │
 └──────────────┘  └────────────────────┘  └─────────────────────┘  └──────────────────┘
```

## Root-cause classes (all five issues)

```ascii
 ┌────────────────────┐     ┌─────────────────────┐     ┌──────────────────┐
 │ SSOT violation     │     │ Silent fail-open    │     │ Non-idempotent   │
 │ (inspector ≠       │────▶│ (.unwrap_or / early │────▶│ write + hot      │
 │  storage naming)   │     │  return hide truth) │     │ monitor loops    │
 └────────────────────┘     └─────────────────────┘     └──────────────────┘
        #1 #2                        #1 #3                     #4 #5
```

## What SPEC-091 did and did not fix

| Concern                           | SPEC-091          | SPEC-104           |
| -----------------------------------| -------------------| --------------------|
| Chunk text SSOT → `public.chunks` | Yes (mig 106–141) | INV-03 must follow |
| Boot never migrates (LD-15)       | Yes               | Unchanged          |
| Inspector column / graph names    | **No**            | This spec          |
| Tenant slug races                 | **No**            | This spec          |

## Official anchors (as of 2026)

- PostgreSQL error classes: `42703` undefined_column, `42P01` undefined_table, `23505` unique_violation, `57014` query_canceled — [PostgreSQL Errcodes](https://www.postgresql.org/docs/current/errcodes-appendix.html).
- `INSERT ... ON CONFLICT` — [SQL INSERT](https://www.postgresql.org/docs/current/sql-insert.html).
- Apache AGE graph schemas are per-graph catalogs (`{graph}."Node"`), not a hardcoded `edgequake` schema.
