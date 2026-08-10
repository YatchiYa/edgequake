# 03 — Implementation roadmap (SPEC-114)

## Dependency ASCII

```ascii
W0 Spec pack
    │
    ▼
W1 API metadata/DTO/normalize ──────┐
    │                               │
    ▼                               │
W2 Pipeline prompt + enforce <──────┘
    │
    ▼
W3 Preset catalog parity ──► W4 Hybrid UI (selectors + preview)
    │                               │
    └──────────► W5 Wizard/payload/cards ◄──┘
                        │
                        ▼
                   W6 E2E gates
                        │
                        ▼
                   W7 v2 stub (docs only)
```

## Waves

| Wave | Deliverable | DoD |
|------|-------------|-----|
| **W0** | Spec pack authored | README links all docs; laws + findings registered |
| **W1** | `relation_types` (+ strict, `kg_schema_preset`) persist | Round-trip GET/PUT; empty = free-form; max 50 |
| **W2** | Pipeline relation prompt + enforce | Strict remap / permissive pass-through tests |
| **W3** | `kg-schema-presets` entity+relation; General ≡ Rust | Unit tests; comment drift fixed |
| **W4** | `RelationTypeSelector` + dual panels + `KgSchemaPreview` | Component tests; a11y; wizard density |
| **W5** | Draft/payload/diff/review + workspace cards | Create + reconfigure persist; rebuild hint |
| **W6** | Playwright + Rust e2e | All F-114-* gated |
| **W7** | Typed-edges v2 design stub | Docs only (superseded by 114b) |
| **W7a / 114b** | `relation_edges` + TypedEdgeEditor + enforce | Laws 9–13 gated |
| **W7b** | React Flow schema canvas | Deferred — product approval |

## Wave detail

### W1 — API

- Refactor `normalize_entity_types` → `normalize_type_list` (alias keep).
- `apply_relation_types_metadata` / `apply_relation_types_strict_metadata` / `apply_kg_schema_preset_metadata`.
- Extend Create/Update/Response DTOs (core + API).
- OpenAPI refresh if required by repo gates.

### W2 — Pipeline

- Extend schema: `relation_types`, `relation_strict`.
- `from_workspace_metadata` reads new keys (LAW-114-3 empty = free-form).
- Prompt section + `enforce_relation_type` (fallback `RELATED_TO` or first listed).
- Factory wiring unchanged call site shape where possible.

### W3 — Presets

- New/extended catalog: each domain has `entityTypes` + `relationTypes`.
- Align General entities with Rust defaults.
- Detect preset including relations; set `kg_schema_preset`.

### W4 — UI

- Extract shared type-list chip/bulk primitives if needed (DRY).
- `RelationTypeSelector` (no colors).
- `KgSchemaPreview` read-only mini schema.
- Extraction step composition.

### W5 — Wizard

- `WizardDraft` fields; payload builders; diff keys; review impact.
- Workspace relation card + preset badge.
- Prefill from workspace on reconfigure.

### W6 — Gates

- See [04-e2e-test-matrix.md](04-e2e-test-matrix.md).

### W7 — v2 stub

- See [issues/ISSUE-typed-edges-v2.md](issues/ISSUE-typed-edges-v2.md).
