# SPEC-096 — First Principles

> **Cross-refs**: [WHY](00-why.md) · [Roadmap](03-implementation-roadmap.md) · [SPEC-017 DRY/SOLID](../017-dry-and-solid-audit/) · [SPEC-085 entity_types](../085-fix-security/)  
> **External**: [LightRAG SUMMARY_LANGUAGE](https://github.com/HKUDS/LightRAG/) · [TrustGraph non-English](https://docs.trustgraph.ai/guides/non-english-languages/) · [AutoSchemaKG multilingual](https://github.com/HKUST-KnowComp/AutoSchemaKG/blob/main/example/multilingual_processing.md)

---

## Laws

| Law | Statement |
|-----|-----------|
| **LAW-L1** | Output language is an **explicit workspace contract**, never inferred from document text or entity type labels. |
| **LAW-L2** | One prompt SSOT for JSON extractors (primary + gleaning): a single shared language-instruction block; no duplicated wording. |
| **LAW-L3** | Config resolution is a **single pure function**: `workspace.metadata.extraction_language` → `EDGEQUAKE_EXTRACTION_LANGUAGE` → `"English"`. |
| **LAW-L4** | JSON **schema keys stay English** (`entities`, `name`, `description`, …). Language applies only to human-readable string **values**. |
| **LAW-L5** | Changing `extraction_language` never silently rewrites existing graph nodes; it applies to **future ingestions / reprocess** only. |
| **LAW-L6** | Entity-type **presets** are language-aware via one shared catalog (canonical English key → localized UPPERCASE tokens). Language and `EntityExtractionSchema` stay separate modules (ISP); the UI applies a coordinated remap when the selection matches a known preset. Custom/mixed type lists are never silently rewritten. |

---

## First principles (decomposition)

### 1. What is “extraction language”?

It is the natural language the LLM must use when writing:

- entity `name` and `description`
- relationship `description` (and free-form `type` when not strict)
- keywords / summaries if those prompts are later wired

It is **not**:

- the source document’s language (may differ; mixed corpora exist)
- the UI locale (`i18n` for chrome)
- the JSON wire format language (LAW-L4)
- automatic translation of an already-built graph (LAW-L5)

### 2. Why workspace scope?

A knowledge graph is a **shared vocabulary**. Per-document auto-detect would fragment entity names across languages for the same real-world referent (e.g. `北京` vs `Beijing`). AutoSchemaKG and LightRAG treat language as an explicit operator choice for consistency. EdgeQuake mirrors that with workspace metadata (same pattern as `entity_types`).

### 3. Why not put language inside `EntityExtractionSchema`?

| Concern | `EntityExtractionSchema` | `extraction_language` |
|---------|--------------------------|------------------------|
| Responsibility | Which types + strict/permissive | Which natural language for string values |
| Change cadence | Domain ontology | Locale / market |
| Consumers | Type normalization / OTHER folding | Prompt wording only |

**ISP / SRP:** keep them orthogonal in the pipeline. Schema stays types-only; language stays on extractors/options. **LAW-L6** coordinates *preset tokens* in the WebUI via a shared catalog — that is presentation/config sync, not stuffing language into `EntityExtractionSchema`.

### 4. Why omit English few-shots when language ≠ English?

SOTA prompts embed English examples. Showing English exemplars while instructing “write in Chinese” confuses models (format following over language following). v1 **omits** few-shots for non-English rather than maintaining N translated corpora (YAGNI / DRY). JSON extractors today are largely zero-shot with schema sections — add an explicit language section only.

### 5. Proper nouns

Retain the SOTA rule: if translating a proper noun would create ambiguity, keep the original form. Encode once in the shared language-instruction block (LAW-L2).

---

## SOLID / DRY mapping

| Principle | Application |
|-----------|-------------|
| **S** | `resolve_extraction_language` owns precedence; prompt module owns wording; `apply_extraction_language_metadata` owns JSONB write; UI card owns language presentation; `entity-type-catalog` owns localized type tokens. |
| **O** | New languages = extend `SUPPORTED_LANGUAGES` + catalog columns + UI labels; extractors unchanged. |
| **L** | All production extractors that emit natural language (`LLMExtractor`, `GleaningExtractor`, and SOTA if re-enabled) honor the same resolved language. |
| **I** | Language is **not** stuffed into `EntityExtractionSchema`. Separate builder/`IngestionPipelineOptions` field. Preset localization is a FE catalog concern (LAW-L6). |
| **D** | Pipeline depends on a resolved `&str` / `String`, not on HTTP DTOs or workspace structs. |
| **DRY** | One allowlist (`SUPPORTED_LANGUAGES`); one metadata key (`extraction_language`); one language instruction helper; one resolution function; one entity-type catalog for preset localization. |

---

## Resolution algorithm (normative)

```text
fn resolve_extraction_language(workspace_meta, env) -> String:
  if workspace_meta.extraction_language is Some(non-empty):
    return canonicalize(value)   # must be in SUPPORTED_LANGUAGES or 400 at API
  if env EDGEQUAKE_EXTRACTION_LANGUAGE is Some(non-empty):
    return canonicalize(value)
  return "English"
```

**Canonicalization (v1):** trim; match case-insensitively against `SUPPORTED_LANGUAGES`; store/return the canonical title-case form from the allowlist (`"chinese"` → `"Chinese"`). Reject unknown at API boundary with `400`. Invalid env values log warn and fall through to `"English"` (ops safety; do not crash the server).

---

## Complexity / surface budget

| Surface | Before | After |
|---------|--------|-------|
| Prompt params | `text`, `schema` | `text`, `schema`, `language` |
| Workspace metadata keys | entity_types*, models, … | + `extraction_language` |
| DB migrations | — | **None** (JSONB) |
| UI cards on workspace page | Entity Types, models | + Extraction Language (one select) |

---

## Non-goals (v1)

Documented for PO / UX lenses; do not expand scope without a new SPEC:

1. Per-document language override or auto-detect.
2. Fully translated few-shot libraries per language.
3. Batch-translating existing AGE nodes on language change.
4. Forcing a multilingual embedding model (ops advice only).
5. Replacing English JSON keys with localized keys.

---

## Inheritance

| Spec | Relationship |
|------|----------------|
| SPEC-017 | DRY/SOLID audit — prompt SSOT pattern |
| SPEC-085 / #216 | Workspace `entity_types` metadata pattern to mirror |
| SPEC-032 | Workspace model config + rebuild UX pattern |
| SPEC-051 | Reprocess semantics for LAW-L5 |
| LightRAG port | `{language}` in SOTA prompts — revive for JSON path |
