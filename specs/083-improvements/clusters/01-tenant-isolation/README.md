# Cluster 01 — Tenant isolation (RLS + WebSocket)

> **Sprint**: 1  
> **Laws**: LAW-1, LAW-3, LAW-4  
> **Defects**: S-01…S-06, X-37, C-24, X-23

---

## WHY

Historically RLS looked like isolation but was inert (autocommit GUC). That path is **FIXED** (S-03…S-06, S-01/S-02, X-37, C-24): production uses `with_rls_transaction` / `with_optional_pg_rls` (session, identity, PDF lineage, conversations). Residual risk is operational drift (FORCE/GUC namespaces) — keep Cluster 01 RLS e2e green.

## ROOT CAUSE (historical — now mitigated)

```
  set_config(..., is_local=true) in autocommit
       --> GUC dies with statement
       --> policies see NULL
       --> only app WHERE remains (forgotten WHERE = leak)

  Mitigated by: BEGIN → set_tenant_context → work → COMMIT
  API SSOT: with_optional_pg_rls (no acquire_rls_connection call-sites)
```

## SOLUTION (DRY: `TenantContext` + `with_rls_transaction`) — landed

| Step | Action | Status |
|------|--------|--------|
| 1 | `with_rls_transaction` / `with_optional_pg_rls` | FIXED |
| 2 | Unify GUC to `app.current_*` | FIXED (S-05) |
| 3 | FORCE RLS + `document_originals` | FIXED (S-04/S-06) |
| 4 | Fail-closed policies | FIXED |
| 5 | WS identity + track ownership | FIXED (S-01/S-02) |
| 6 | Exhaustive `matches_track_id` (C-24) | FIXED |
| 7 | Lagged → notify/disconnect (X-23) | see register |
| 8 | Document `IsolationPolicy` (X-37) | FIXED |

## EDGE CASES

| EC | Case | Mitigation |
|----|------|------------|
| EC-T1 | Pool connection reuse | Always clear context on release |
| EC-T2 | API-key auth without workspace | Fail closed or bind default workspace |
| EC-T3 | Missing Origin on WS | Prod reject (S-10) |
| EC-T4 | Lag under global bus | Per-tenant broadcast or early filter |

## E2E

`e2e_ws_tenant_a_never_sees_tenant_b`, `e2e_cancel_foreign_track_id_404`, `e2e_rls_guc_visible_on_following_insert`, `e2e_document_originals_cross_workspace_denied`, `e2e_null_tenant_row_invisible`
