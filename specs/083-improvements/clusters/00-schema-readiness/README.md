# Cluster 00 — Schema readiness (P0 / X-03)

> **Sprint**: 0  
> **Laws**: LAW-2, LAW-8  
> **Defects**: [P0](../../defects/P0.md) PARTIAL · [X-03](../../defects/X-03.md) FIXED · [C-20](../../defects/C-20.md) FIXED  
> **Incident**: [INCIDENT-PROD-DIAGNOSIS.md](../../INCIDENT-PROD-DIAGNOSIS.md)

---

## WHY

Historically production chat/ingest bound exclusively to denormalized `eq_*` columns that may never exist on large AGE graphs. **X-03 FIXED**: boot readiness + COALESCE property fallback + health `eq_id_schema`. **C-20 FIXED**: native upsert contracts assert eq_* arbiters. Residual **P0 PARTIAL**: DDL under load on large AGE graphs can still time out.

## ROOT CAUSE (historical — mitigated for readers)

```
  long query --AccessShare--> blocks ALTER
  ALTER timeout --> no columns
  Mitigated: SchemaReadiness + COALESCE(eq_*, props) + refuse/fallback
  Residual: ops DDL completion on huge graphs (P0)
```

## SOLUTION (DRY: `SchemaReadiness`) — landed for X-03/C-20

| Step | Status |
|------|--------|
| Catalog probe → `eq_id_schema` health | FIXED (X-03) |
| COALESCE fallback + metric | FIXED (X-03) |
| Native upsert eq_* arbiter contracts | FIXED (C-20) |
| Maintenance-window reconcile / large-graph DDL | PARTIAL (P0) |

## EDGE CASES

See incident EC-P0-*; multi-graph partial readiness; NULL backfill mid-flight.

## E2E

`e2e_schema_ready_refuses_traffic*` (adapted), `e2e_degrees_match_property_fallback`, `contract_eq_columns_present_after_reconcile`, `contract_native_upsert_eq_arbiter`
