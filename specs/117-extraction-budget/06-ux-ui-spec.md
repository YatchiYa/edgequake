# 06 — UX / UI Spec

## Surfaces

| Surface | Control |
|---------|---------|
| `/workspace` overview | `WorkspaceExtractBudgetCard` |
| Reconfigure / create wizard | Dedicated `extract-budget` step (after Chunking, before Extraction) |
| Document upload API | Optional fields (no required WebUI upload UI in v1) |

## Card behavior

| Action | Result |
|--------|--------|
| Inherit | Clear metadata keys |
| Custom | Persist both ints |
| Preset chip | Set 40 / 100 + Custom mode |
| Save | PATCH workspace; toast; future-only copy |

## Copy (locked)

- Title: **Extract budget**  
- Inherit: “Use fleet defaults (usually 40 entities / 100 records per LLM response)”  
- Custom: “Cap entities and total records per extraction response”  
- Preset: **Match LightRAG (40/100)**  
- Honesty: “Applies to future ingestions. Rebuild knowledge graph to reprocess existing documents.”  
- Density: “Adaptive chunking + high budget can inflate entity mentions — see Chunking.”

## Wizard

Dedicated `WorkspaceExtractBudgetStep` (reuses card via `draft-extract-budget` SSOT). Included in review + payload builder.
