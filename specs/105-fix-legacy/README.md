# SPEC-105 — Fix Legacy Table Use

> **Status:** implemented · **Inherits:** [SPEC-091](../091-simplify-data-layer/) · [SPEC-104](../104-fix-datalayer/) · upgrade soak [SPEC-093](../93-migration-assessment/)  
> **Hard requirement:** next product version upgrades fleets on **≤ v0.22.0** (mig ≤105, KV SSOT) without data loss or false-orphan monitors mid-cutover.

## Laws (summary)

| Law | Statement |
|-----|-----------|
| L1 | Empty leftovers DROP; non-empty → SPEC-091 confirm-drop (125/126/131) |
| L2 | Unknown `EDGEQUAKE_VECTOR_BACKEND` → TypedEmbeddings |
| L3 | Era-aware via **census SSOT** — dual while legacy tables exist |
| L4 | One `legacy_store_census` for boot / migrate / inspect |
| L5 | Ladder: expandable → confirm-drop → **migration 142** assert |
| L6 | Mid-upgrade must not false-CRITICAL KV-era healthy docs (SPEC-104 EC-16) |

## ≤0.22 upgrade ladder

```ascii
 ≤0.22 (≤105) → roll write-stop binary → migrate expandable
   → migrate --confirm-drop (125/126/131) → migration 142 → steady typed SSOT
```

## Docs

| File | Role |
|------|------|
| [00-why](00-why.md) | Residuals + upgrade hazard |
| [01-first-principles](01-first-principles.md) | LAW-L1..L6 |
| [02-gap-register](02-gap-register.md) | Code gaps |
| [03-implementation-plan](03-implementation-plan.md) | Waves |
| [04-e2e-test-matrix](04-e2e-test-matrix.md) | E2E-105 |
| [05-edge-cases](05-edge-cases.md) | Mid-upgrade ECs |
| [06-post-assessment](06-post-assessment.md) | Grades |
