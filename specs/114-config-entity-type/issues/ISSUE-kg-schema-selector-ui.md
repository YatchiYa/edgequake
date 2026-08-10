# ISSUE — KG schema selector UI

**Findings:** F-114-05, F-114-07, F-114-08  
**Wave:** W4–W5  
**Laws:** LAW-114-7, LAW-114-6

## Goal

Hybrid Extraction step: dual panels (entity + relation) + mini Visual schema preview; wizard payload/diff/cards.

## Work

1. `RelationTypeSelector` (chips, add, bulk, strict; no colors).  
2. `KgSchemaPreview` read-only.  
3. Compose in `workspace-extraction-step.tsx` with domain summary.  
4. Extend `WizardDraft`, payloads, diff, review rebuild hint.  
5. Workspace relation types card + preset badge.  
6. Prefill on reconfigure.

## Acceptance

- Create + reconfigure round-trip.  
- Preview updates when lists change.  
- Empty relations show free-form copy.  
- Mobile: stacked panels.
