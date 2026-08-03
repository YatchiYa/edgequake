# SPEC-096 — Finding Register

> **Cross-refs**: [Laws](00-first-principles.md) · [Cross-ref matrix](02-cross-ref-matrix.md) · [GH-352](issues/GH-352-extraction-language.md)

| ID | Finding | Status | Law | Primary locus |
|----|---------|--------|-----|---------------|
| F-352-01 | `json_extraction_prompt` has no language parameter; English-only instructions | OPEN→W1 | L2, L4 | `edgequake-pipeline/src/prompts/json_prompts.rs` |
| F-352-02 | `json_gleaning_prompt` likewise has no language parameter | OPEN→W1 | L2 | `json_prompts.rs` + `extractor/gleaning.rs` |
| F-352-03 | `LLMExtractor` has no language field / `with_language` builder | OPEN→W1 | L2, I | `extractor/llm.rs` |
| F-352-04 | `build_ingestion_pipeline` hardcodes `LLMExtractor` without language | OPEN→W1 | L3 | `ingestion_pipeline.rs` (~L181–182) |
| F-352-05 | `SUPPORTED_LANGUAGES` used only in tests; not enforced at API | OPEN→W2 | L1, L3 | `prompts/mod.rs` |
| F-352-06 | SOTA `{language}` + `with_language` exist but production path unused | OBSERVE | L2 | `prompts/entity_extraction.rs`, `extractor/sota.rs` |
| F-352-07 | SOTA English few-shots always injected even when language ≠ English | OPEN→W1 | L2 | `entity_extraction.rs` `get_examples()` |
| F-352-08 | No workspace metadata key `extraction_language` | OPEN→W2 | L1, L3, L5 | `workspace_service_impl/helpers.rs`, requests/responses |
| F-352-09 | Create/Update workspace API DTOs omit language | OPEN→W2 | L1 | `handlers/workspaces_types/requests.rs`, core `multitenancy/requests.rs` |
| F-352-10 | No `EDGEQUAKE_EXTRACTION_LANGUAGE` env fallback | OPEN→W2 | L3 | `.env.example`, config resolution helper |
| F-352-11 | Orchestrator / ingestion path never reads language into pipeline options | OPEN→W2 | L3, D | `edgequake-core` orchestrator ingestion |
| F-352-12 | WebUI workspace page has Entity Types card but no language control | OPEN→W3 | L1, L5 | `workspace-entity-types-card.tsx`, workspace pages |
| F-352-13 | OpenAPI / features docs do not document extraction language | OPEN→W4 | L1 | OpenAPI, `docs/features.md`, AGENTS env table |
| F-352-14 | No contract/e2e proving language reaches the prompt | OPEN→W4 | all | new tests under pipeline + api + webui e2e |
| F-352-15 | Localized `entity_types` workaround is incomplete (descriptions still English) | DOC | L1 | issue #352 additional context |

Legend: OPEN→Wn = fix in wave n · OBSERVE = document; wire if path re-enabled · DOC = documented behavior · GUARD = regression.

---

## Severity / user impact

| ID | Impact |
|----|--------|
| F-352-01…04 | Blocks all non-English KG quality in production |
| F-352-08…12 | Blocks self-serve configuration without rebuild |
| F-352-06…07 | Misleading “feature exists” in SOTA code |
| F-352-13…14 | Docs/test gaps allow silent regression |
| F-352-15 | False confidence from entity_types-only workaround |

---

## Dependency order

```
F-352-01/02 (prompt SSOT)
    → F-352-03 (LLMExtractor)
    → F-352-04 (factory)
    → F-352-11 (orchestrator resolve)
    → F-352-08/09/10 (API + env + metadata)
    → F-352-12 (UI)
    → F-352-13/14 (docs + e2e)
F-352-07 (few-shot omit) parallel with W1
F-352-06 observe; ensure SOTA still interpolates language if selected later
```
