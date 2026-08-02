# 02 — Cross-ref matrix (SPEC-100)

| ID | Behavior | Gate | Law |
|----|----------|------|-----|
| F-100-01 | Shared `cls-stability` + `ReservedSlot` | unit: `cls-stability.test.ts` | LAW-100-1 |
| F-100-02 | Breadcrumb: no empty depth≤1 band; `h-9` bar at depth≥2 | Playwright: `spec100-breadcrumb-slot` | LAW-100-4 |
| F-100-03 | TenantGuard overlay when workspace hydrated | unit/manual + layout smoke | LAW-100-3 |
| F-100-04 | Pipeline chunk/active/metrics reservation | Playwright: `spec100-pipeline-cls` | LAW-100-1/2/5 |
| F-100-05 | Document detail progress strip + matched skeleton | Playwright: `spec100-document-detail-cls` | LAW-100-1/2 |
| F-100-06 | Dashboard / Workspace / Query / Graph CLS | Playwright: `spec100-dashboard-cls` (+ peers) | LAW-100-1/2 |
| F-100-07 | Settings / Knowledge / Costs / API Explorer CLS | Playwright: `spec100-settings-cls` (+ peers) | LAW-100-3 |
| F-100-08 | Documents wrappers still pass SPEC-099 | Playwright: `spec099-layout-stability` | LAW-100-7 |
