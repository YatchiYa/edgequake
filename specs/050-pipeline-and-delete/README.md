# SPEC-050 — Pipeline UX Parity & Deletion Progress

> **Status:** In-progress  
> **Branch:** `feat/spec047-vision-ingest-spec048-progress`  
> **Cross-refs:** SPEC-048 (stage stepper), SPEC-017 (three-layer progress), SPEC-006 (cascade delete), FEAT0012 (progress reporting)

## Index

| Document | Lens | Summary |
|----------|------|---------|
| [01-product-owner.md](01-product-owner.md) | Product Owner | Why, business value, 5 WHYs |
| [02-ux-designer.md](02-ux-designer.md) | UX Designer | User flows, decision trees, feedback loops |
| [03-ui-designer.md](03-ui-designer.md) | UI Designer | Component states, visual language, ASCII mockups |
| [04-fullstack-developer.md](04-fullstack-developer.md) | Full-Stack Dev | API contracts, data flow, component tree |
| [05-sre.md](05-sre.md) | SRE | Reliability, observability, blast radius |
| [06-database-expert.md](06-database-expert.md) | DB Expert | Cascade correctness, transaction safety, index coverage |
| [07-complexity-expert.md](07-complexity-expert.md) | O(n) Expert | Complexity analysis, bottlenecks, SLOs |
| [08-implementation-plan.md](08-implementation-plan.md) | All | DRY/SOLID plan, task breakdown, edge cases |
| [e2e/](e2e/) | QA | Playwright E2E test specs and results |
| [screenshots/](screenshots/) | QA | UI screenshots proving implementation |

## First Principles

1. **A user must always know what will happen before a destructive action**  
   → Show impact before delete confirms.

2. **A user must always see real-time progress on any long operation**  
   → Delete and reprocess must surface staged feedback identically.

3. **Feedback granularity must match operation scope**  
   → Single delete → row-level visual. Bulk delete → per-document counter + overall bar.

4. **Failure must never leave the system in a partially-visible unknown state**  
   → If delete partially fails, the row must clearly show partial state, not disappear.

5. **Re-process parity**  
   → Re-process triggers the same ingestion pipeline. Its progress display must be identical to first-time ingestion.
