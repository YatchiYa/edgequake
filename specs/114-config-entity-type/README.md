# SPEC-114 — Visual KG Type & Relation Configuration

> **Product pin**: EdgeQuake v0.24.3+  
> **Status**: Waves W0–W7 complete (v1); **SPEC-114b** typed edges (wave A) in progress / shipping  

> **Inherits**: SPEC-013/#216 entity_types persist · SPEC-085 custom entity config · SPEC-096 extraction language · SPEC-101 wizard extraction step · SPEC-102 entity type colors  
> **Peers**: FEAT-0005 Custom Entity Configuration · LightRAG entity extraction schema

## Start here

1. [00-why.md](00-why.md) — Five WHYs + causal ASCII  
2. [00-first-principles.md](00-first-principles.md) — LAW-114-1…8 + SOLID/DRY  
3. [01-finding-register.md](01-finding-register.md) — F-114-*  
4. [02-cross-ref-matrix.md](02-cross-ref-matrix.md) — code ↔ law ↔ test  
5. [03-implementation-roadmap.md](03-implementation-roadmap.md) — Waves W0–W7 + DoD  
6. [04-e2e-test-matrix.md](04-e2e-test-matrix.md) — gates  
7. [05-edge-cases.md](05-edge-cases.md) — EC-114-*  
8. Issues → [`issues/`](issues/)  
9. Lenses → [`lenses/`](lenses/)

## One-screen verdict

```ascii
+------------------------------------------------------------------+
|  PROBLEM: Workspace KG vocabulary is half-configured             |
|  - Entity types: presets/chips/strict (OK but flat UX)           |
|  - Relation types: free-form LLM only (NOT configurable)         |
|  - FE General preset ≠ Rust default_entity_types()               |
+------------------------------------------------------------------+
|  SOLUTION (phased):                                              |
|  v1 Dual allowlists + domain presets (entity+relation)           |
|     + hybrid UX (dual panels + mini schema preview)              |
|  v2 Typed edges Source--REL-->Target + expandable canvas         |
+------------------------------------------------------------------+
|  SSOT: workspaces.metadata JSONB (no new SQL columns)            |
|  APPLY: future ingestions; rebuild/reprocess for existing graph  |
+------------------------------------------------------------------+
```

## Locked decisions

| Decision        | Choice                                                                                |
| -----------------| ---------------------------------------------------------------------------------------|
| Schema model    | **Phased** — v1 dual allowlists; **114b** `relation_edges[]`; canvas later            |
| UX surface      | **Hybrid** — dual panels + honest typed-edge editor (no fake preview pairing)         |
| Persistence     | Workspace `metadata` JSONB (SPEC-096 / SPEC-102 pattern)                              |
| Scope of apply  | Future ingestions only; honest rebuild hint on review                                 |
| Empty relations | Absent/empty `relation_types` ⇒ free-form (backward compatible)                       |
| Preset identity | Optional `kg_schema_preset`; FE General ≡ Rust defaults                               |

## Scope

| In (v1 + 114b) | Out (114b / later) |
|----------------|--------------------|
| `relation_types` + strict + `relation_edges` | Full React Flow canvas (W7b) |
| Domain presets: entity + relation + curated edges | OWL/RDF/SHACL export |
| Dual panels + `TypedEdgeEditor` + honest preview | Auto-infer schema from documents |
| Pipeline prompt + enforce labels + endpoints | Per-user schemas / AGE node storage |
| Wizard create/reconfigure + workspace cards | Multi-ontology versioning |
| E2E + unit + Rust gates | |

## Metadata contract (v1)

| Key | Semantics |
|-----|-----------|
| `entity_types` | existing allow-list |
| `entity_types_strict` | existing (absent ⇒ true) |
| `relation_types` | **new** `string[]`; empty/absent ⇒ free-form |
| `relation_types_strict` | **new** bool; absent ⇒ true when list non-empty |
| `kg_schema_preset` | **new** optional preset id for UX honesty |
| `relation_edges` | **114b** `{source,relation,target}[]`; empty/absent ⇒ unconstrained endpoints |
| `entity_type_colors` | unchanged (SPEC-102) |

## Target composition

```ascii
Domain preset card
        │
        ├─► entity_types[] (+ strict, colors)
        └─► relation_types[] (+ strict)
                │
                ▼
        PUT /workspaces/{id}  →  metadata JSONB
                │
                ▼
        ExtractionSchema (entity + relation)
                │
                ├─► JSON prompts (GUIDANCE / STRICT)
                └─► enforce_*_type on parse

WebUI: EntityTypeSelector ∥ RelationTypeSelector → KgSchemaPreview
```

## Document map

| Doc | Role |
|-----|------|
| [00-why.md](00-why.md) | Symptom → Five WHYs |
| [00-first-principles.md](00-first-principles.md) | Laws + DRY/SOLID |
| [01-finding-register.md](01-finding-register.md) | F-114-* |
| [02-cross-ref-matrix.md](02-cross-ref-matrix.md) | Traceability |
| [03-implementation-roadmap.md](03-implementation-roadmap.md) | Waves |
| [04-e2e-test-matrix.md](04-e2e-test-matrix.md) | Gates |
| [05-edge-cases.md](05-edge-cases.md) | EC register |
| [lenses/](lenses/) | PO / Fullstack / DB / UX / Design / Growth |
| [issues/](issues/) | Workstream slices |

## Verification (summary)

```bash
# Docs: every F-114-* appears in 02-cross-ref-matrix.md
cargo test -p edgequake-core --lib normalize_type_list
cargo test -p edgequake-pipeline --lib enforce_relation_type
cargo test -p edgequake-api --test e2e_spec114_relation_types
cargo test -p edgequake-api --test e2e_spec114_extraction_schema -- --test-threads=1
cargo test -p edgequake-pipeline --test e2e_spec114_gleaning_relations
cd edgequake_webui && bun test src/constants/ src/components/shared/
cd edgequake_webui && pnpm exec playwright test e2e/spec114-kg-schema.spec.ts

# Live Mistral / Ollama extract (opt-in; pins mistral-small-latest / qwen3.6:35b-a3b)
export MISTRAL_API_KEY=...
make spec114-e2e-mistral-extract
ollama pull qwen3.6:35b-a3b && make spec114-e2e-ollama-extract
make spec114-e2e-live-extract
```

See [measurements/README.md](measurements/README.md) for CI vs live runbook.

## Cross-spec anchors

- [SPEC-013 entity extraction](../013-fix-issues-05-2026/entity_extraction/)
- [SPEC-096 multi-language](../096-multi-language-extraction/)
- [SPEC-101 reconfigure wizard](../101-workspace-reconfigure-wizard/) (if present) / wizard shell
- [SPEC-102 colors](../102-custom-entity-type-colors/)
- [SPEC-099 UX](../099-ux-ui-improvement/) progressive disclosure patterns
