# 00 — Why (SPEC-100)

Hard refresh and soft refetch on dashboard screens insert conditional panels (chunk progress, reprocess strips, admin cards) after first paint. Users feel “bouncy” UI — Cumulative Layout Shift (CLS).

Documents already fixed this (SPEC-099 F-099-17). Other routes still:

- `return null` → tall card (Pipeline chunk progress, Settings admin)
- Spinner → fixed `h-64` list without matching skeleton
- Full-page skeleton on soft refetch when cache exists
- Breadcrumb appearing only at depth ≥2 shoves `main`

## Goal

Same CLS budget and reservation playbook on every `(dashboard)` route, with shared primitives and CI gates.
