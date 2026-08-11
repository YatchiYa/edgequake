# 00 — Why SPEC-117

## Trigger

Per-response extract caps (**40 ents / 100 rows**) ship fleet-wide (SPEC-001/054, LightRAG parity). Gaps:

| Approach | Gap |
|----------|-----|
| Fleet env only | Acc/dev/partner clash; cannot tune one workspace |
| Document gleaning / `chunk_options` | Caps not exposable on upload API |
| WebUI | Silent on K — ops must know env names |
| Hard FIFO truncate | Order bias; no automatic gleaning continue when truncated |

SPEC-116 productized **geometry \(N\)**. This SPEC productizes **budget \(K\)** and improves selection/recovery under the budget.

## Non-goals

- Change fleet default 40/100 or Acc publication pins  
- Global “max entities in workspace” quota  
- DEG-RAG post-graph denoising  
- Auto-rebuild on save  
- Schema migration (metadata JSON only)

## Phase 5 (shipped)

Relation-aware hard truncate (LAW-117-8): prefer relation-bearing entities by degree under \(K\); Acc/LR FIFO via `EDGEQUAKE_EXTRACT_CAPS_SELECTION=fifo`.

## Success

1. Workspace can **Inherit** or set explicit ents/records with LightRAG 40/100 preset.  
2. Document upload can override caps (wins last).  
3. Resolve is one SSOT in `extract_caps.rs` (LAW-117-6).  
4. Truncation is observable; gleaning continue recovers when capped + gleaning left.  
5. Edge cases validated; UX matches Chunking card pattern.
