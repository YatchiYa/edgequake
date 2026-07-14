# SPEC-048 — Transparent Ingestion Progress UX

**Status:** draft complete (research + lenses + contract 2026-07-11)  
**Scope:** Documents page — make long-running ingest feel honest, predictable, and controllable  
**Law:** code is law · First Principles · DRY/SOLID across FE↔BE progress contracts  
**Companion:** [047 ingest battle plan](../047-rag-evaluation/016-ingest-speed-reliability-battle-plan.md)  
**Canvas:** `spec048-ingestion-progress-ux` (IDE canvas beside chat)

---

## Document map

| # | File | Purpose |
|---|------|---------|
| 000 | [This index](./000-index.md) | Map + verdict |
| 001 | [5 WHYs + First Principles](./001-five-whys-first-principles.md) | Why the UX fails; axioms |
| 002 | [Code-is-law inventory](./002-code-is-law-inventory.md) | Symbols, paths, dual vocabularies |
| 003 | [Lens — Product Owner](./003-lens-product-owner.md) | Jobs-to-be-done, outcomes, anti-goals |
| 004 | [Lens — UX](./004-lens-ux.md) | Mental models, anxiety, information scent |
| 005 | [Lens — UI Designer](./005-lens-ui-designer.md) | Visual hierarchy, density, motion |
| 006 | [Lens — Frontend](./006-lens-frontend.md) | Hooks, stores, transport fan-out |
| 007 | [Lens — Backend](./007-lens-backend.md) | Status writers, WS gaps, SSOT |
| 008 | [Lens — Full Stack](./008-lens-fullstack.md) | End-to-end contract + failure modes |
| 009 | [Screens (ASCII)](./009-screens-ascii.md) | Current vs target screen wireframes |
| 010 | [Components + navigation](./010-components-navigation-ascii.md) | Component tree, nav, surfaces |
| 011 | [State machines](./011-state-machines.md) | Document / upload / pipeline transitions |
| 012 | [Target UX contract](./012-target-ux-contract.md) | Normative FE↔BE progress contract |
| 013 | [Acceptance + cross-ref](./013-acceptance-criteria-crossref.md) | Testable gates, ID matrix |
| 014 | [Implementation roadmap](./014-implementation-roadmap.md) | Phased P0–P3 work |
| 015 | [Implementation plan](./015-implementation-plan.md) | DRY/SOLID work packages + screenshot matrix |
| e2e | [screenshots/ANALYSIS.md](./e2e/screenshots/ANALYSIS.md) | Playwright visual analysis |

---

## One-sentence verdict

Users see **three competing progress stories** (upload stepper, banner, table badge) fed by **three incomplete transports** (poll KV, PDF SSE, partial WS) over **two status vocabularies** (`status` vs `current_stage`) — so “Pipeline Busy” can coexist with “Completed” rows while the real stage is buried in free-text `stage_message`.

---

## Non-goals

- Redesign the whole Documents IA or workspace switcher
- Replace React Query / WebSocket stack wholesale
- Acc/F1 RAG quality work (SPEC-047)
- Dark-mode / brand refresh for its own sake

---

## Success metric (product)

| Metric | Baseline (today) | Target |
|--------|------------------|--------|
| User can answer “what is happening now?” in ≤3s | Often needs Details dialog + guess | Banner + row show same stage SSOT |
| False “Pipeline Busy” with 0 active docs | Occurs (task lag / `is_busy`) | **0** in acceptance tests |
| Stage vocabulary count (user-facing) | 2+ (legacy + unified + upload FSM) | **1** (`UnifiedStage` + admission `queued`) |
| Live chunk/merge on WS | Missing | Present for PDF + text tracks |
