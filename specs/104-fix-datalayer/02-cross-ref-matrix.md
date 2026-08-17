# 02 — Cross-Reference Matrix (SPEC-104)

Laws: [01-first-principles.md](01-first-principles.md). Symptoms: [00-issue-data.md](00-issue-data.md). Assessment: [13-fix-assessment.md](13-fix-assessment.md).

| Issue | Law | Prod volume | Pre-fix smoking gun | Correct identity | Fix status | E2E | Release risk if unfixed |
|-------|-----|-------------|----------------------|------------------|------------|-----|-------------------------|
| #1 workspaces.id | I1, I2 | 2304 / ~24h | INV-D2 `WHERE id::text` | `workspaces.workspace_id` | **Closed** | E2E-104-01 | Log spam; INV-D2 blind |
| #2 edgequake.Node | I1 | 24 / ~24h | `graph_name = "edgequake"` | `eq_{table_prefix}_graph` | **Closed** (default ns) | E2E-104-02 | INV-C / INV-04 blind |
| #3 INV-03 | I2, LAW-D6 | 24 / ~24h | KV `LIKE '{id}-chunk-%'` | `public.chunks.document_id` | **Closed** post-091 | E2E-104-03 | False green post-125 |
| #4 tenants slug | I3 | 6 burst | plain INSERT | `ON CONFLICT (slug)` | **Closed** | E2E-104-04 | Client retry storms |
| #5 node counts | I4 | 4 burst | `@>` without GIN / over budget | M038 GIN + SPEC-089 | **Observable** | E2E-104-05 | Soft list gaps |

## Migration coupling (no new migs)

| Phase | SPEC-091 | SPEC-104 |
|-------|----------|----------|
| Schema 106–141 | Required for typed SSOT | None |
| Mig 125 drop KV | Guards need chunks coverage | INV-03 becomes meaningful **only if** this binary ships |
| Boot LD-15 | Unchanged | Unchanged |
| Wrong order: 104 binary on KV-era | — | EC-16 false INV-03 |

## Call graph (monitor path, post-fix)

```ascii
 AppState::new_postgres
        │
        ▼
 InspectorConfig::for_namespace("default")
   graph = eq_eq_default_graph
        │
        ├─▶ StorageInspector::inspect()  (startup)
        │         ├─ M038 GIN schema check
        │         ├─ INV-03  (public.chunks)
        │         ├─ INV-C   (JOIN {graph}."Node")
        │         └─ INV-D2  (workspace_id, UUID tables)
        │
        └─▶ spawn_hourly_monitor() ── every 3600s ──▶ same inspect()
```
