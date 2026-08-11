# 07 — Implementation Plan

## Phases

1. Docs (this tree) — done  
2. Pipeline SSOT + ranked prompt + glean continue — done  
3. Workspace metadata + API + in-memory — done  
4. Document admission + prepare/factory — done (text/file/batch)  
5. OpenAPI + web types + payload — done  
6. Card + wizard — done  
7. Tests — done (contract + pipeline e2e + Playwright)  
8. Relation-aware hard truncate (LAW-117-8) — done  

## Edge-case matrix

| Case | Expect |
|------|--------|
| Inherit + env 40/100 | Unchanged |
| WS 60/150, doc omit | 60/150 |
| Doc 20/50 over WS 60 | 20/50 |
| entities > records | 400 |
| entities ≤ 0 | 400 |
| Clear inherit | Keys removed |
| Truncate + gleaning=0 | Cap only |
| Truncate + gleaning≥1 | Continue with prior names |
| Multi-tenant | Isolated metadata |
| Prompt uses resolved K | Not stale env-only |
| Null JSON keys | Treated as inherit |
| Late bridges under K | Kept (relation-aware); dropped under `=fifo` |
| Rel weight under total cap | Higher weight kept first |

## DRY / SOLID checklist

- [x] Single `resolve` / `resolve_for_ingestion`  
- [x] Single metadata apply helper (`extract_budget_metadata`)  
- [x] Prompt builders accept caps (JSON + SOTA + glean continue)  
- [x] In-memory mirrors Postgres path  
- [x] Multipart + text share `parse_document_extract_caps`  
- [x] Prepare uses `ExtractionCaps::from_value` (no ad-hoc pair parse)
- [x] Single `apply_extraction_caps(_with_strategy)` selection SSOT
- [x] Acc FIFO opt-in via env (no duplicate truncate paths)
