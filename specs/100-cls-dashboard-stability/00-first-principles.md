# 00 — First Principles (SPEC-100)

| Law | Statement |
|-----|-----------|
| **LAW-100-1** | Reserve geometry before async chrome paints (`minHeight` + matching skeleton). |
| **LAW-100-2** | Soft refresh must not unmount primary content (`placeholderData` / `isInitialLoading`). |
| **LAW-100-3** | Never `return null` for late tall admin/auth/live-work chrome — use reserved slot. |
| **LAW-100-4** | Shared dashboard chrome (breadcrumb band) has a stable height on every route. |
| **LAW-100-5** | Page shells that own scroll use `h-full min-h-0 overflow-clip`. |
| **LAW-100-6** | CI is proof — every F-100-* maps to a unit or Playwright gate. |
| **LAW-100-7** | Do not change product semantics — layout reservation only (inherit SPEC-099/098/091). |
