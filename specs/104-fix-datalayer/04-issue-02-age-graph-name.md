# 04 — Issue #2: Relation `edgequake."Node"` does not exist (42P01)

**Crit:** High · **Volume:** 24 (hourly) · **Law:** LAW-I1 · **E2E:** E2E-104-02

## Symptom (prod)

StorageInspector INV-C CTE joins `edgequake."Node"`. AGE stores nodes under `{graph_name}."Node"` where `graph_name = eq_{table_prefix}_graph` (e.g. `eq_eq_default_graph`).

## Why V22 has it

```ascii
 PostgresAGEGraphStorage::with_pool
   prefix = config.table_prefix()        -- "eq_default"
   graph  = format!("eq_{}_graph", ...) -- "eq_eq_default_graph"
        │
        │   (correct SSOT)
        │
 InspectorConfig::default()
   graph_name: "edgequake".into()         -- LEGACY / WRONG
        │
        ▼
 INV-C / INV-04 SQL: edgequake."Node"
        │
        ▼
 42P01 undefined_table  (caught → warn + skip)
        │
        ▼
 CQRS drift monitor is blind; hourly CRITICAL path still runs INV-03
```

Call sites using `InspectorConfig::default()`:

- `state/postgres.rs` (boot + hourly)
- `handlers/admin.rs` (inspect / repair)

## V23 residual

**Unfixed.** SPEC-091 schema-qualified `public.documents` in places but left AGE graph hardcoded.

## Remediation

1. `InspectorConfig::for_namespace(ns)` builds `graph_name` / kv / vectors from `PostgresConfig::table_prefix()` (same formula as storage).
2. `Default` → `for_namespace("default")` → `eq_eq_default_graph`.
3. Admin/boot pass workspace namespace when multi-ws inspect is added (edge: [09-edge-cases.md](09-edge-cases.md) EC-05 **RESIDUAL**).

## Fix status (2026-08-03)

**Closed for default workspace.** Grade A−. Residual: multi-graph fleets (EC-05). Migration impact: **none**.

## Ops note

```sql
SELECT name FROM ag_catalog.ag_graph;
-- expect eq_eq_default_graph (and per-workspace graphs), not "edgequake"
```
