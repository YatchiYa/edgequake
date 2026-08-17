# 00 — Why (SPEC-102)

## Symptom

Complex knowledge graphs are hard to analyze: many entity types collapse to the same gray, duplicate palettes drift, and admins cannot assign domain colors that persist across sessions/devices.

## Evidence

- Hardcoded `ENTITY_TYPE_COLORS` in `edgequake_webui/src/lib/graph/label-utils.ts` incomplete vs Rust `default_entity_types()` (`CREATURE`, `METHOD`, `CONTENT`, `DATA`, `ARTIFACT`, `NATURALOBJECT`, `OTHER` → DEFAULT gray).
- Divergent maps: `use-graph-expansion.ts`, `graph-search.tsx`, `label-search.tsx`.
- Workspace `metadata` stores `entity_types` but no color map; settings store only `colorBy` mode.
- SPEC-030 F-A11Y-05: color is sole type differentiator — custom colors must keep legend labels.

## Five WHYs

1. **Why are networks hard to distinguish?** Nodes of different types share similar/identical fills.  
2. **Why?** Defaults miss many types; customs cannot be set.  
3. **Why no customs?** Coloring is frontend-only; no API field.  
4. **Why frontend-only?** Historical SSOT in `label-utils` never extended to workspace metadata.  
5. **Root cause:** No workspace-scoped color override contract + DRY violation (multiple palettes) + incomplete defaults.

## Causal ASCII

```ascii
Hardcoded incomplete palette
        + duplicate TYPE_COLORS copies
        + no metadata.entity_type_colors
                │
                ▼
Unknown / domain types → DEFAULT gray
Inconsistent accents across search vs graph
No admin persist across devices
                │
                ▼
Complex graph analysis fails (symptom)
```
