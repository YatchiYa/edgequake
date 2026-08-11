# 09 — Acceptance

## Pass

- [x] Doc pack complete under `specs/117-extraction-budget/`  
- [x] Inherit default preserves env/40/100 Acc **K** behavior  
- [x] Workspace explicit ints round-trip API + UI  
- [x] Document override wins over workspace  
- [x] Invalid pairs → 400  
- [x] Prompt ranking language present  
- [x] Truncation + gleaning continue when applicable  
- [x] Relation-aware hard truncate (LAW-117-8); Acc FIFO via env pin  
- [x] Acc SSOT pins `EDGEQUAKE_EXTRACT_CAPS_SELECTION=fifo` (`acc_env` + Acc backend)  
- [x] OpenAPI + web types refreshed  
- [x] Contract + pipeline e2e + Playwright green  
- [x] No schema migration  

## Fail

- Soft-only caps without hard truncate  
- UI invents resolve math  
- Changing workspace K silently rewrites old graph without Rebuild  
- Fleet defaults changed without Acc decision
- Blind FIFO-only truncate with no Acc pin escape hatch when product ships relation-aware
