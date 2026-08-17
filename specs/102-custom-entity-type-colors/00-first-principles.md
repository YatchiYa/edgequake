# 00 — First Principles (SPEC-102)

## Axioms

1. **Color encodes type identity** — same type → same color everywhere in entity-type mode.  
2. **Workspace owns presentation policy** — colors travel with the workspace like `entity_types`.  
3. **Defaults are honest fallbacks** — customs override; never invent random colors per session.  
4. **One resolver** — presentation components never own private palettes.  
5. **Validate at the boundary** — reject invalid hex before persistence.  
6. **Evidence beats vibes** — every finding maps to a gate.

## Laws

| Law | Statement |
|-----|-----------|
| **LAW-102-1** | Single resolver — `resolveEntityTypeColor(type, overrides)` is the only path to entity-type fills in WebUI. |
| **LAW-102-2** | Workspace SSOT — persist `metadata.entity_type_colors` via create/update workspace; surface on `WorkspaceResponse`. |
| **LAW-102-3** | Hex contract — only `#RGB` / `#RRGGBB` (case-insensitive); normalize keys to UPPERCASE; max 50 entries; empty map clears key. |
| **LAW-102-4** | Mode gate — custom/default type colors apply only when `colorMode === "entity-type"`; community mode unchanged. |
| **LAW-102-5** | DRY — delete duplicate `TYPE_COLORS` / Tailwind type maps; re-export from `label-utils` for compat. |
| **LAW-102-6** | SOLID — pure color module ≠ picker UI ≠ API adapter; selector props optional (`colors` / `onColorsChange`). |
| **LAW-102-7** | A11y v1 — every swatch paired with type label/text; shapes deferred (SPEC-030 F-A11Y-05). |
| **LAW-102-8** | CI is proof — every F-102-* has unit, Playwright, or Rust gate; non-regress graph CLS / entity-type-selector. |

## DRY / SOLID

| Principle | Application |
|-----------|-------------|
| **DRY** | One defaults map + one resolver; one `apply_entity_type_colors_metadata` helper. |
| **SRP** | Resolver validates/resolves; picker collects; workspace mutation persists. |
| **OCP** | New default type = extend defaults map; consumers unchanged. |
| **DIP** | UI depends on hook/resolver, not hard-coded hex tables. |
| **ISP** | Selector color props optional — read-only / type-only callers unaffected. |
| **LSP** | Memory + postgres workspace services share same request/response shape. |

## Inheritance (do not break)

| Prior | Constraint |
|-------|------------|
| SPEC-085 / #216 | `entity_types` normalize + persist path unchanged |
| SPEC-013 | Workspace update entity_types e2e remains green |
| SPEC-101 | Wizard shell / extraction step composition unchanged |
| SPEC-100 | Graph CLS reserved slots remain |
| SPEC-030 | Legend labels remain; do not rely on color alone |
