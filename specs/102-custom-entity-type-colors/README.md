# SPEC-102 — Custom Entity Type Colors

> **Product pin**: EdgeQuake v0.22.0+  
> **Status**: Waves 0–5 complete  

> **Inherits**: SPEC-013/#216 entity_types persist · SPEC-085 custom entity config · SPEC-030 F-GR / F-A11Y-05 · SPEC-100 graph CLS · SPEC-101 wizard extraction step  
> **Peers**: FEAT-0005 Custom Entity Configuration · `label-utils` / graph renderer

## Start here

1. [00-why.md](00-why.md) — Five WHYs + causal ASCII  
2. [00-first-principles.md](00-first-principles.md) — LAW-102-1…8 + SOLID/DRY  
3. [01-finding-register.md](01-finding-register.md) — F-102-*  
4. [02-cross-ref-matrix.md](02-cross-ref-matrix.md) — code ↔ law ↔ test  
5. [03-implementation-roadmap.md](03-implementation-roadmap.md) — Waves 0–5 + DoD  
6. [04-e2e-test-matrix.md](04-e2e-test-matrix.md) — gates  
7. [05-edge-cases.md](05-edge-cases.md) — EC register  
8. Issues → [`issues/`](issues/)  
9. Lenses → [`lenses/`](lenses/)

## Scope (locked)

| In | Out |
|----|-----|
| Workspace-scoped `metadata.entity_type_colors` | Per-user / localStorage-only colors |
| Single FE resolver + expanded defaults | Community palette customization |
| EntityTypeSelector + graph legend color edit | Node shape encoding (SPEC-030 F-A11Y-05 shapes) |
| Hex `#RGB` / `#RRGGBB` validation (API + UI) | Storing colors on AGE graph nodes |
| E2E + unit + Rust persist gates | Degree-based coloring completion |

## Locked decisions

1. **Workspace SSOT** — `entity_type_colors` in workspace metadata JSONB; anyone who can `PUT` workspace can edit.  
2. **Override > default > DEFAULT** — customs merge over `ENTITY_TYPE_COLORS`; unknown → `#94a3b8`.  
3. **One resolver** — delete duplicate `TYPE_COLORS` maps.  
4. **Entity-type mode only** — community coloring unchanged.  
5. **No migration** — metadata JSONB only.  
6. **A11y v1** — swatch + type label; shapes deferred.  
7. **CI is proof** — every F-102-* maps to a unit, Playwright, or Rust gate.

## Surfaces

| Surface | Role |
|---------|------|
| `lib/graph/entity-type-colors.ts` | Pure resolver + defaults + hex validation |
| `EntityTypeSelector` | Chip swatches + color picker (create/reconfigure) |
| `entity-type-filter-list` / legend | In-graph edit + reset |
| `graph-renderer` | Applies resolved color when `colorMode === entity-type` |
| `PUT/GET /api/v1/workspaces/{id}` | Persist / read `entity_type_colors` |

## Target composition

```ascii
Workspace.metadata.entity_type_colors
        │
        ▼
useEntityTypeColors() ──► resolveEntityTypeColor(type, overrides)
        │                         │
        │                         ├─► graph-renderer (entity-type mode)
        │                         ├─► legend / filter list
        │                         └─► search / browser / context menu
        │
EntityTypeSelector / legend picker ──► debounced PUT workspace
```

## Verification

```bash
cd edgequake_webui && bun test src/lib/graph/
cd edgequake_webui && pnpm exec playwright test e2e/spec102-entity-type-colors.spec.ts
cargo test -p edgequake-core --lib apply_entity_type_colors
cargo test -p edgequake-api --test e2e_spec013_github_issues spec102
```
