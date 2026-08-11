# SPEC-116 — Workspace Adaptive Chunking

> **Mission:** Productize LightRAG-fair chunk geometry as a **workspace** policy (not fleet env only), with exceptional UX and DRY pipeline SSOT.  
> **Trigger:** SPEC-108 / SPEC-115 — adaptive ON densifies KG vs LightRAG fixed 1200/100.

## Short verdict

| Mode | Effective geometry |
|------|-------------------|
| **Inherit** (default) | Fleet env (`EDGEQUAKE_ADAPTIVE_CHUNKING` + size/overlap) |
| **Adaptive** | Force ON → 1200/800/600 by text bytes |
| **Fixed** | Force OFF → workspace size/overlap (defaults **1200/100**) |
| Preset | “Match LightRAG (Acc fair)” → Fixed 1200/100 |

Precedence: **document `chunk_options` > workspace > fleet env**.

## Document map

```ascii
 00-why
   → 01-first-principles (LAW-116-1..7)
   → 02-cross-ref-matrix
   → 03-code-as-is
   → 04-target-architecture
   → 05-lenses/ (PO, fullstack, DB, UX, front, growth, llm-power, extract-budget)
   → 06-ux-ui-spec
   → 07-implementation-plan
   → 08-test-protocol
   → 09-acceptance
   → 10-llm-power-first-principles   ← research (N vs y(model))
   → 11-research-evidence-aug-2026   ← citations through Aug 2026
   → 12-extract-budget-first-principles ← per-chunk K=40/100
   → 13-extract-budget-brainstorm    ← deep design options + phases
```

## Status board

| ID | Item | Status |
|----|------|--------|
| D1 | Doc pack (00–09 + lenses) | This tree |
| D2 | LLM power × graph research (10–11 + lens 007) | This tree |
| D3 | Extract budget brainstorm (12–13 + lens 008) | This tree |
| I1 | `ChunkingPolicy` SSOT | Implementation |
| I2 | Workspace metadata + API | Implementation |
| I3 | Worker inject policy | Implementation |
| I4 | WebUI card + wizard | Implementation |
| T1 | Contract + e2e + Playwright | Implementation |

## Ops fair pin (fleet, unchanged)

```bash
export EDGEQUAKE_ADAPTIVE_CHUNKING=0
export EDGEQUAKE_CHUNK_SIZE=1200
export EDGEQUAKE_CHUNK_OVERLAP=100
```

Workspace Fixed mode productizes the same pin without process-wide env.
