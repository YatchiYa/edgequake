# SPEC-096 — WHY (Five WHYs)

> **Cross-refs**: [README](README.md) · [Laws](00-first-principles.md) · [GH-352 study](issues/GH-352-extraction-language.md)  
> **Issue**: https://github.com/raphaelmansuy/edgequake/issues/352  
> **Product pin**: EdgeQuake v0.22.0+

---

## Symptom (reporter)

Users who upload non-English documents (Chinese, Japanese, Korean, …) cannot control the language of extracted entity names, types, keywords, or descriptions. Output is often English or mixed-language even when:

1. The source document is non-English.
2. Workspace `entity_types` are set to localized labels (e.g. `["人物", "组织", "地点"]`).
3. The LLM model itself is multilingual (Qwen, etc.).

There is no environment variable, API field, or workspace UI to set extraction output language. The only workaround is hoping localized entity type names bias the model — unreliable and does not control descriptions/keywords.

---

## Five WHYs

### WHY 1 — Why is extraction output English / mixed?

Because the production JSON extraction prompt (`json_extraction_prompt` in `json_prompts.rs`) contains **only English instructions** and never tells the LLM which natural language to use for names and descriptions.

### WHY 2 — Why doesn’t the existing `{language}` placeholder work?

Because that placeholder lives only in the **SOTA / tuple** prompt templates (`entity_extraction.rs`). Production uses `LLMExtractor` (JSON), which never interpolates language. `SUPPORTED_LANGUAGES` is referenced only in unit tests.

### WHY 3 — Why isn’t `SOTAExtractor.with_language()` used?

Because `build_ingestion_pipeline` in `ingestion_pipeline.rs` hardcodes:

```rust
LLMExtractor::new(llm.clone()).with_entity_schema(entity_schema.clone())
```

No language is passed. `SOTAExtractor` is effectively dead for the production path.

### WHY 4 — Why can’t users configure language at all?

Because workspace create/update API and metadata helpers only persist `entity_types` / `entity_types_strict` (and model overrides). There is no `extraction_language` field, no `EDGEQUAKE_EXTRACTION_LANGUAGE` env, and no WebUI control. Config surface was never built when JSON extractors replaced the LightRAG-ported SOTA path.

### WHY 5 — Why does LightRAG parity matter here?

LightRAG exposes `SUMMARY_LANGUAGE` / `addon_params.language` so operators can force entity/relation output language without forking prompts ([LightRAG README](https://github.com/HKUDS/LightRAG/); [ProgramingWithCore](https://github.com/HKUDS/LightRAG/blob/main/docs/ProgramingWithCore.md)). EdgeQuake ported the `{language}` prompt text but dropped the config wire when shipping `LLMExtractor` as default — a **parity regression**, not an intentional product choice.

**Root cause:** Language is a first-class extraction contract in prompt templates and industry peers, but the production factory, JSON prompts, API, and UI never received the wiring. Dead code looks like a feature; users experience an English-only graph.

---

## Causal ASCII

```
  Non-English document upload
           |
           v
  Workspace entity_types=["人物",…]   ← weak hint only
           |
           v
  build_ingestion_pipeline
           |
           v
  LLMExtractor (JSON) ──► json_extraction_prompt(text, schema)
           |                         |
           |                         v
           |              English-only instructions
           |              NO language parameter
           v
  LLM returns mixed/English names + descriptions
           |
           v
  Graph nodes stored ──► query/UI show wrong language

  SOTA path (unused):
  SOTAExtractor.with_language("Chinese")
           |
           v
  entity_extraction.rs "{language}"  ◄── dead for production
```

```
  FIXED shape
           |
           v
  UI / API / env set extraction_language
           |
           v
  resolve_extraction_language(ws → env → English)
           |
           v
  build_ingestion_pipeline(..., language)
           |
           v
  LLMExtractor.with_language + gleaning
           |
           v
  json_*_prompt(..., language)  ← LAW-L2 SSOT
           |
           v
  Names/descriptions in target language
  (JSON keys stay English — LAW-L4)
```

---

## What already exists / what does not

| Artifact | Exists? | Production use? |
|----------|---------|-----------------|
| `{language}` in SOTA prompts | Yes | No (SOTA unused) |
| `SUPPORTED_LANGUAGES` | Yes | Tests only |
| `SOTAExtractor::with_language` | Yes | No |
| `json_extraction_prompt` language arg | **No** | N/A |
| `LLMExtractor` language field | **No** | N/A |
| Workspace `extraction_language` | **No** | N/A |
| Env `EDGEQUAKE_EXTRACTION_LANGUAGE` | **No** | N/A |
| WebUI language selector | **No** | N/A |

---

## External peer evidence

| System | Mechanism | Takeaway for EdgeQuake |
|--------|-----------|------------------------|
| [LightRAG](https://github.com/HKUDS/LightRAG/) | `SUMMARY_LANGUAGE` / `addon_params.language` | Env + runtime knob; display names like `Chinese` |
| [TrustGraph non-English guide](https://docs.trustgraph.ai/guides/non-english-languages/) | Edit extraction prompts to emit target language | Prompt text is the control surface |
| [AutoSchemaKG multilingual](https://github.com/HKUST-KnowComp/AutoSchemaKG/blob/main/example/multilingual_processing.md) | Explicit language metadata; no silent auto-detect for graph consistency | Workspace-scoped explicit language (LAW-L1) |
