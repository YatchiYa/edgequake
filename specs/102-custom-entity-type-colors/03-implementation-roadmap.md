# 03 — Implementation roadmap (SPEC-102)

## Waves

| Wave | DoD |
|------|-----|
| **W0** | `entity-type-colors.ts` + unit tests; expanded defaults; `label-utils` re-exports |
| **W1** | Rust create/update/response + `apply_entity_type_colors_metadata` + persist/invalid tests; in-memory parity |
| **W2** | FE types/hook; wire resolver everywhere; delete duplicate palettes; onboarding payload |
| **W3** | Selector color picker + legend edit/reset + wizard/reconfigure wire |
| **W4** | Playwright `spec102-*` gates green |
| **W5** | Cross-ref filled; FEAT-102; CHANGELOG; verify cmds |

## Dependency ASCII

```ascii
W0 resolver
  └─► W1 API persist
        └─► W2 FE wire + DRY
              └─► W3 UI picker / legend
                    └─► W4 e2e
                          └─► W5 closeout
```

## Primary files by wave

| Wave | Files |
|------|-------|
| W0 | `edgequake_webui/src/lib/graph/entity-type-colors.ts`, `*.test.ts`, `label-utils.ts` |
| W1 | `requests.rs` (core + api), `helpers.rs`, `workspace_ops.rs`, `in_memory.rs`, `responses.rs`, `workspace_crud.rs`, API test |
| W2 | `types/workspace.ts`, hook, renderer, expansion, search, onboarding payload |
| W3 | `entity-type-selector.tsx`, `entity-type-filter-list.tsx`, wizards |
| W4 | `e2e/spec102-entity-type-colors.spec.ts` |
| W5 | `docs/features.md`, `CHANGELOG.md`, this pack |

## Exit criteria

- [x] Spec pack authored  
- [x] All F-102-* gated  
- [x] Non-regression green (unit + OpenAPI refresh)  
- [x] FEAT-102 registered  

## Out of scope

Per-user overrides, community custom palettes, AGE node color props, shape encoding, degree coloring.
