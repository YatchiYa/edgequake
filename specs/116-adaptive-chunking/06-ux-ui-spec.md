# 06 — UX / UI Spec

## Placement

After Extraction Language card on `/workspace` (and deeplink twin). Wizard: **dedicated `chunking` step** (before extraction) on create-tenant, create-workspace, first-run, and reconfigure — reuses `WorkspaceChunkingCard` via `WorkspaceChunkingStep`.

## States

| State | UI |
|-------|-----|
| View | Badge: Inherit / Adaptive / Fixed · 1200/100 |
| Edit Inherit | Mode select; Acc-fair chip available |
| Edit Adaptive | Mode + threshold explainability line |
| Edit Fixed | Mode + size + overlap number inputs |
| Acc-fair click | Sets Fixed + 1200 + 100 |

## Copy (EN defaults)

- Title: `Chunking`
- Description: `How documents are split into chunks before entity extraction.`
- Future hint: `Applies to future document ingestions. Use Rebuild Knowledge Graph to re-chunk existing documents.`
- Acc chip: `Match LightRAG (Acc fair)`
- Adaptive help: `Adaptive sizing uses 1200, 800, or 600 tokens by document size (same thresholds as fleet adaptive).`

## Test IDs

- `workspace-chunking-card`
- `chunking-mode-select`
- `chunking-acc-fair-chip`
- `chunking-size-input` / `chunking-overlap-input`
- `chunking-future-only-hint`

## A11y

Labeled controls; Acc chip `type="button"`; invalid Fixed pair → inline `role="alert"` before save.
