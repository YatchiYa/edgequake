# SPEC-096 — Cross-Ref Matrix

> **Cross-refs**: [Findings](01-finding-register.md) · [Laws](00-first-principles.md) · [E2E matrix](04-e2e-test-matrix.md) · [Lenses](lenses/README.md)

| Code / artifact | Law | Finding | Lens | Test ID |
|-----------------|-----|---------|------|---------|
| `json_language_instruction(language)` helper (new) | L2, L4 | F-352-01/02 | fullstack, rust | `spec096_json_prompt_includes_language` |
| `json_extraction_prompt(text, schema, language)` | L2, L4 | F-352-01 | fullstack | `spec096_json_prompt_includes_language` |
| `json_gleaning_prompt(..., language)` | L2 | F-352-02 | fullstack | `spec096_gleaning_prompt_includes_language` |
| `LLMExtractor::with_language` + field | L2, I | F-352-03 | fullstack | `spec096_llm_extractor_language_builder` |
| `GleaningExtractor` passes language to prompt | L2 | F-352-02 | fullstack | `spec096_gleaning_prompt_includes_language` |
| `IngestionPipelineOptions.extraction_language` | L3, D | F-352-04 | fullstack | `spec096_pipeline_factory_wires_language` |
| `build_ingestion_pipeline` → `with_language` | L3 | F-352-04 | fullstack | `spec096_pipeline_factory_wires_language` |
| `SUPPORTED_LANGUAGES` allowlist enforce | L1, L3 | F-352-05 | PO, API | `spec096_api_rejects_unsupported_language` |
| SOTA `get_examples()` omit when ≠ English | L2 | F-352-07 | rust | `spec096_sota_omits_examples_non_english` |
| SOTA `{language}` interpolation (guard) | L2 | F-352-06 | rust | existing SOTA language tests + guard |
| `apply_extraction_language_metadata` | L1, L3, L5 | F-352-08 | database, fullstack | `spec096_workspace_metadata_roundtrip` |
| `CreateWorkspace*` / `UpdateWorkspace*` + `extraction_language` | L1 | F-352-09 | API, frontend | `spec096_api_create_update_get_language` |
| Workspace response exposes `extraction_language` | L1 | F-352-09 | API, UI | `spec096_api_create_update_get_language` |
| `resolve_extraction_language(ws, env)` | L3 | F-352-10/11 | fullstack | `spec096_resolve_language_precedence` |
| `EDGEQUAKE_EXTRACTION_LANGUAGE` | L3 | F-352-10 | PO, ops | `spec096_resolve_language_precedence` |
| Orchestrator → pipeline options language | L3, D | F-352-11 | fullstack | `spec096_e2e_ingest_prompt_language` |
| `WorkspaceExtractionLanguageCard` | L1, L5 | F-352-12 | ux, ui, frontend | `spec096_ui_workspace_language_select` |
| Create-workspace language field | L1 | F-352-12 | ux, frontend | `spec096_ui_create_workspace_language` |
| OpenAPI schema field | L1 | F-352-13 | API | `make codegen-openapi-refresh` + contract |
| `.env.example` + AGENTS env table | L3 | F-352-13 | PO | doc review |
| Feature registry / FEAT id | L1 | F-352-13 | PO | `docs/features.md` |
| Spec pack `specs/096-multi-language-extraction/` | — | all | all | doc existence |

## API contract (normative)

| Field | Location | Type | Notes |
|-------|----------|------|-------|
| `extraction_language` | Create / Update request | `Option<String>` | Omit = leave unchanged (update) or resolve default (create) |
| `extraction_language` | Workspace response | `Option<String>` | Effective configured value if set; `null` means inherit env/default |
| Clear override | Update with `""` or `"none"` | — | Remove metadata key (mirror vision model clear pattern) |

## UI testids (normative)

| Element | `data-testid` |
|---------|----------------|
| Language card | `workspace-extraction-language-card` |
| View mode value | `ws-extraction-language-value` |
| Edit select | `ws-extraction-language-select` |
| Create form select | `create-workspace-extraction-language` |
| Future-only hint | `extraction-language-future-only-hint` |

## External refs

- Issue: https://github.com/raphaelmansuy/edgequake/issues/352  
- LightRAG language: https://github.com/HKUDS/LightRAG/blob/main/docs/ProgramingWithCore.md  
- TrustGraph: https://docs.trustgraph.ai/guides/non-english-languages/  
- Entity types pattern: workspace `entity_types` (SPEC-085 / #216)  
- Sibling UX: [SPEC-086 ingestion UX](../086-improve-ingestion-ux/) · [SPEC-032 workspace config](../032-*)  
