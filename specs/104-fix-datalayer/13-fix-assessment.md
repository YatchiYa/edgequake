# 13 — Post-Fix Assessment v3 (SPEC-104 A+)

> **As-of:** 2026-08-03 · after A+ first-principles pass ([14-harden-notes.md](14-harden-notes.md) § A+).  
> **Method:** re-grade against LAW-I1..I4 and EC-01..18 with honesty — no overclaim.

## Executive verdict

```ascii
 BEFORE A+                   AFTER A+                      STILL OPEN (non-goals)
 ┌──────────────────┐     ┌──────────────────┐     ┌──────────────────┐
 │ Conflict only in │     │ Error::Conflict  │     │ 57014 under load │
 │ HTTP handler     │────▶│ in core + mem+PG │     │ with GIN (cap.)  │
 │ INV-01 split_part│     │ document_id join │     │ INV-C default ns │
 │ best-effort      │     │ + safe SQL idents│     │ only (cost)      │
 └──────────────────┘     └──────────────────┘     └──────────────────┘
 Ship: YES as **0.24.0**. Domain policy + DE hygiene closed; #5 remains capacity.
```

| Issue | Grade v2 | Grade v3 (A+) | Notes |
|-------|----------|---------------|-------|
| #1 workspaces.id | A | **A+** | Naming SSOT + INV-D2; no residual |
| #2 AGE graph name | A | **A+** | Helpers + multi-graph GIN; INV-C scope explicit |
| #3 INV-03 | A− | **A+** | Dual presence + safe idents |
| #4 tenant slug | A | **A+** | Conflict at **service** layer (PG + in-memory); HTTP maps 409 |
| #5 node counts | B+ | **B+** | Unchanged — capacity / SPEC-089, not naming |

**Overall:** incident defects #1–#4 meet A+ data-engineering bar (SSOT, fail-visible, natural-key policy at domain boundary, identifier allowlist). #5 stays B+ by design.

---

## 1. First principles applied (A+)

| Principle | Application |
|-----------|-------------|
| **LAW-I1 SSOT** | Relation/graph names only via `PostgresConfig::{age_graph_name,bare_*}` |
| **LAW-I3 natural key** | Slug get-or-create atomic; identity clash = `Error::Conflict` in core (not handler-only) |
| **Fail visible** | INV-01 never silent green; missing stores → schema Warning/Critical |
| **Safe dynamic SQL** | `require_safe_sql_ident` allowlist before any `{kv}`/`{vec}` interpolation |
| **Prefer typed joins** | INV-01 uses `document_id` column when present; else id/`split_part` fallback |
| **SRP** | Domain conflict in `WorkspaceService`; API only maps `CoreError → ApiError` |
| **DRY** | Same conflict policy in PG path and in-memory path |

## 2. What A+ closed vs v2 residuals

| Residual (v2) | Resolution |
|---------------|------------|
| Tenant 409 handler-only | `Error::Conflict` in `pg_create_tenant` + in-memory; `From` → HTTP 409 |
| INV-01 legacy best-effort | Prefer `document_id` column match when column exists |
| Identifier interpolation | `require_safe_sql_ident` on INV-01/INV-03 dynamic tables |

## 3. Migration impact (unchanged stance)

- **Still zero new SQL migrations.**
- Deploy any 0.23+ binary with A+ harden; no schema gate.

## 4. Edge-case scorecard

| Status | IDs |
|--------|-----|
| CLOSED | EC-01..04, 06..12, 14..18 |
| PARTIAL | EC-05 (GIN all graphs; INV-C still default) |
| OPS | EC-13 (57014 with GIN = capacity) |

## 5. E2E

| ID | Status |
|----|--------|
| E2E-104-01..06 | Source/unit contracts (incl. Conflict + safe ident) |
| E2E-104-07..10 | PG when `DATABASE_URL` set; 09 expects service-layer Conflict |
| issue331/336 | Unchanged capacity evidence |

## 6. Honest residuals (do not mark done)

1. **57014** on large graphs despite GIN — SPEC-089 batch/timeout product limit.
2. **INV-C** entity_count drift still uses configured default graph only (cost tradeoff).

## 7. Ship checklist

- [x] Naming helpers + inspector wire
- [x] INV-03 dual / INV-01 typed + document_id prefer
- [x] Tenant atomic + **service-layer** Conflict → 409
- [x] Safe SQL identifier allowlist
- [x] Multi-graph GIN
- [x] Contracts updated (A+)
- [x] Staging inspect: zero 42703/42P01 since MARKER; INV-03 only true orphans ([measurements/](measurements/))
- [x] Release notes: duplicate slug same name → 200; different name → 409 ([`CHANGELOG.md`](../../CHANGELOG.md) **0.24.0**)
