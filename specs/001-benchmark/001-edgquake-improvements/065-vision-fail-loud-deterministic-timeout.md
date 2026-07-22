# 065 — Explicit Vision: fail loud + deterministic size-scaled timeout

## Verdict (First Principles)

**Neither “always higher timeout” nor “load-adaptive timeout” alone.** Use a **layered policy**:

| Layer | Rule | Why |
|-------|------|-----|
| **Intent** | Workspace/upload **explicit Vision** → **fail closed** (no silent EdgeParse) | User chose quality path; silent downgrade is a lie |
| **Budget** | Timeout = `f(page_count, provider_class)` only | Same PDF → same budget → no flake |
| **Heal** | Missing `page_count` → extract from bytes; if still unknown → **assume 50 pages** | Unknown must not collapse to 120s floor |
| **Implicit** | Default/auto Vision may still EdgeParse-fallback with lineage | Availability for unconfigured workspaces |

## Rejected (flaky / wrong incentives)

- **Load-adaptive timeout** (shrink under queue pressure) → non-deterministic Acc/CI flakes
- **Try 30s then fallback** races → quality depends on load
- **Silent EdgeParse after explicit Vision** → UI says Vision, corpus is EdgeParse
- **Blind “raise default to 30m”** without page scale → either waste or still short for 100+ pages

## AI engineering alignment

Industry practice for PDF/VLM pipelines: **deterministic routing + escalation with audit trail**, not mid-run adaptive deadlines. Escalation is for **implicit** paths; **explicit** choices fail loud so operators can retry, switch model, or switch backend deliberately.

## Implementation map

- `edgequake-pdf::should_fallback_to_edgeparse(..., backend_explicit)` — explicit → never
- `vision_outer_timeout_secs` + `UNKNOWN_PAGE_COUNT_VISION_BUDGET_ASSUMPTION = 50`
- `PdfUploadOptions::apply_workspace` + retry path applies workspace
- Heal `page_count` from PDF bytes before budget math

## Override

`EDGEQUAKE_VISION_TIMEOUT_SECS` still wins when set (ops escape hatch).
