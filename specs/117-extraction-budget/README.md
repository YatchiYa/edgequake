# SPEC-117 — Workspace Extract Budget

> **Mission:** Productize per-response extract caps (entities/records) as a **workspace** policy with document API override, plus pipeline ranking + truncation→gleaning continue.  
> **Trigger:** Caps exist fleet-only (`extract_caps.rs` 40/100); partners need lower/higher K without Acc env fights; FIFO truncate needs recovery.

## Short verdict

| Mode | Effective K / R |
|------|-----------------|
| **Inherit** (default) | Fleet `EDGEQUAKE_MAX_EXTRACTION_*` or **40/100** |
| **Explicit** | Workspace `extract_max_entities` / `extract_max_records` |
| Preset | “Match LightRAG (40/100)” |
| Document API | Optional per-upload override — **wins last** |

Precedence: **document > workspace > fleet env > 40/100**.

Pipeline v1 extras: prompt **rank highest-value first**; if hard-truncated and gleaning remains → **continue** for additional ents.

## Document map

```ascii
 00-why
   → 01-first-principles (LAW-117-1..8)
   → 02-cross-ref-matrix
   → 03-code-as-is
   → 04-target-architecture
   → 05-lenses/ (PO, fullstack, DB, UX, front, growth)
   → 06-ux-ui-spec
   → 07-implementation-plan
   → 08-test-protocol
   → 09-acceptance
```

## Status board

| ID | Item | Status |
|----|------|--------|
| D1 | Doc pack | This tree |
| I1 | `ExtractionCaps` resolve SSOT | Implementation |
| I2 | Prompt rank + glean continue | Implementation |
| I3 | Workspace metadata + API | Implementation |
| I4 | Document admission override | Implementation |
| I5 | WebUI card + wizard | Implementation |
| T1 | Contract + e2e + Playwright | Implementation |

## Fleet defaults (unchanged)

```bash
# Optional overrides — product default remains 40 / 100
export EDGEQUAKE_MAX_EXTRACTION_ENTITIES=40
export EDGEQUAKE_MAX_EXTRACTION_RECORDS=100
```

Workspace Inherit keeps **K values** Acc/env-aligned until explicit.

## Acc / dual-SUT (SPEC-001)

Product hard truncate defaults to **relation_aware** (LAW-117-8). Acc must pin LightRAG-parity FIFO in one place:

```bash
export EDGEQUAKE_EXTRACT_CAPS_SELECTION=fifo
export EDGEQUAKE_MAX_EXTRACTION_ENTITIES=40
export EDGEQUAKE_MAX_EXTRACTION_RECORDS=100
```

Forced by `tools/bench001/bench001/acc_env.py` (`PUBLICATION_ENV`) and `start_acc_backend.py`. Doctor fails if selection ≠ `fifo` after pins apply.
