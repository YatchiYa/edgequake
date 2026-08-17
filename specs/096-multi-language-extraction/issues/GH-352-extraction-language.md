# GH-352 — Extraction output language cannot be controlled

> **Issue**: https://github.com/raphaelmansuy/edgequake/issues/352  
> **Opened**: 2026-07-30 by [@BlackYooi](https://github.com/BlackYooi)  
> **Label**: `bug`  
> **Owner comment** (2026-07-31): “Thank it will be fixed very soon”  
> **SPEC**: [SPEC-096](../README.md)

---

## Reporter summary

### Bug title

Extraction output language cannot be controlled — `{language}` prompt placeholder is unused by the production JSON extractor.

### Expected

1. `LLMExtractor` accepts a `language` parameter interpolated into the JSON extraction prompt.
2. SOTA `{language}` remains wired when `SOTAExtractor` is selected.
3. User-facing mechanism: env (`EDGEQUAKE_EXTRACTION_LANGUAGE`) and/or workspace metadata `extraction_language`.
4. Few-shot examples translated per language **or omitted** when language ≠ English.

### Actual

- Production path: English-only JSON prompts.
- No API / env / UI config.
- Localized `entity_types` is an incomplete workaround.

### Reproduce (reporter)

1. Deploy with any multilingual LLM.
2. Upload Chinese (or other non-English) document.
3. Create workspace with `"entity_types": ["人物", "组织", "地点"]`.
4. Observe English/mixed entity names and descriptions.

---

## Code-is-law trace (as of v0.22.0)

| Claim in issue | Verified locus | Verdict |
|----------------|----------------|---------|
| `{language}` in SOTA prompts | `prompts/entity_extraction.rs` (~L95, L134, L174) | TRUE |
| JSON prompt no language | `prompts/json_prompts.rs` `json_extraction_prompt(text, schema)` | TRUE |
| `SUPPORTED_LANGUAGES` test-only | `prompts/mod.rs` L67–78 + tests | TRUE |
| `SOTAExtractor::with_language` | `extractor/sota.rs` L75 | TRUE; unused in factory |
| `LLMExtractor` no language | `extractor/llm.rs` struct fields | TRUE |
| Factory hardcodes LLMExtractor | `ingestion_pipeline.rs` ~L181–182 | TRUE |
| Gleaning also English-only | `gleaning.rs` → `json_gleaning_prompt` | TRUE (extension of issue) |
| No workspace API field | `workspaces_types/requests.rs` Create/Update | TRUE |
| Entity types metadata pattern exists | `apply_entity_types_metadata` | TRUE — mirror for language |

---

## Gap classification

| Layer | Gap | SPEC-096 wave |
|-------|-----|---------------|
| Prompt | No language section in JSON SSOT | W1 |
| Extractor | No builder / field | W1 |
| Factory | No wire | W1 |
| Few-shot | English examples pollute non-English SOTA | W1 |
| Core resolve | No pure resolution fn | W2 |
| API / metadata | No field / helper | W2 |
| Env | No fallback | W2 |
| OpenAPI | Undocumented | W2/W4 |
| WebUI | No selector | W3 |
| Tests / docs | Missing | W4 |

---

## Locked SPEC-096 answers to reporter asks

| Reporter ask | Locked decision |
|--------------|-----------------|
| Language on `LLMExtractor` | Yes — `with_language` + prompt arg |
| Plumb SOTA `{language}` | Keep; omit English few-shots when ≠ English |
| Env var | `EDGEQUAKE_EXTRACTION_LANGUAGE` (LightRAG `SUMMARY_LANGUAGE` parity) |
| Workspace field | `extraction_language` in metadata JSONB |
| Few-shots | **Omit** when ≠ English (no translated corpus in v1) |
| Config UI | Workspace Extraction Language card (+ create form) |

---

## Acceptance mapping

| Reporter expected behavior | Acceptance ID | Test ID |
|---------------------------|---------------|---------|
| JSON extractor language param | AC-PO-01 | `spec096_json_prompt_includes_language` |
| SOTA language still works | AC-PO-02 | `spec096_sota_omits_examples_non_english` |
| Env or workspace settable | AC-PO-03 | `spec096_resolve_language_precedence` + API roundtrip |
| Few-shots not confusing | AC-PO-04 | `spec096_sota_omits_examples_non_english` |
| Non-English docs usable without rebuild | AC-PO-05 | UI + API e2e |

---

## Related work

- Workspace entity types UI: `WorkspaceEntityTypesCard` (SPEC-085 / #216)
- Workspace model config rebuild toast: SPEC-032
- Reprocess: SPEC-051 (LAW-L5 — language change → reprocess to refresh graph)
- LightRAG parity: `SUMMARY_LANGUAGE` / `addon_params.language`
