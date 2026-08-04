# 02 — Cross-Reference Matrix (SPEC-107)

Laws: [01](01-first-principles.md). Symptoms: [00-issue-data.md](00-issue-data.md). Engineering detail: [SPEC-104 matrix](../104-fix-datalayer/02-cross-ref-matrix.md).

| Partner | SQLSTATE | Law | Smoking gun (0.22.0) | Correct identity | Code status | E2E-107 | Release if unfixed |
|---------|----------|-----|----------------------|------------------|-------------|---------|-------------------|
| E1 | `42703` | I1, I2 | INV-D2 `WHERE id::text` | `workspaces.workspace_id` | **Closed ≥0.24.0** | 01 | Log spam; INV-D2 blind |
| E2 | `42P01` | I1 | `graph_name = "edgequake"` | `eq_eq_default_graph` | **Closed ≥0.24.0** | 02 | INV-C blind |
| E3 | INV-03 | I2 | indexed, no chunk body | `public.chunks` \| KV | Monitor closed; **data residual** | 03 | CRITICAL hourly; RAG holes |
| E4 | `23505` | I3 | plain INSERT tenants | `ON CONFLICT (slug)` | **Closed ≥0.24.0** | 04 | Retry storms |

## Call graph (why volume matches)

```ascii
 AppState::new_postgres
        │
        ▼
 InspectorConfig::for_namespace("default")   -- post-104: eq_eq_default_graph
        │
        ├─▶ inspect() at boot
        └─▶ spawn_hourly_monitor() every 3600s
                 ├─ INV-D2  → was E1 (~N tables/hour)
                 ├─ INV-C   → was E2 (1×/hour)
                 └─ INV-03  → E3 alarm (1×/hour if ≥10 orphans)
```

Tenant create is request-path (not hourly) → E4 burst of 6 matches double-submit window.

## SPEC-104 doc map (DRY)

| Partner | SPEC-104 doc |
|---------|--------------|
| E1 | [03-issue-01-workspaces-pk.md](../104-fix-datalayer/03-issue-01-workspaces-pk.md) |
| E2 | [04-issue-02-age-graph-name.md](../104-fix-datalayer/04-issue-02-age-graph-name.md) |
| E3 | [05-issue-03-inv03-chunk-drift.md](../104-fix-datalayer/05-issue-03-inv03-chunk-drift.md) |
| E4 | [06-issue-04-tenant-slug-race.md](../104-fix-datalayer/06-issue-04-tenant-slug-race.md) |
