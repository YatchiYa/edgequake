# SPEC-096 — Implementation Roadmap

> **Cross-refs**: [Laws](00-first-principles.md) · [Findings](01-finding-register.md) · [E2E](04-e2e-test-matrix.md) · [Edge cases](05-edge-cases.md)

---

## Wave 0 — Specs (this pack)

- [x] WHY, laws, findings, cross-ref, edge cases, issue study, lenses, README

---

## Wave 1 — Pipeline prompt SSOT (F-352-01…04, F-352-07)

**Goal:** Production JSON extractors honor a language string; DRY instruction block; SOTA few-shot omit.

1. [ ] Add `json_language_instruction(language: &str) -> String` in `prompts/json_prompts.rs` (or small `language.rs` sibling) covering:
   - Entire natural-language output in `{language}`
   - Proper-noun retention when translation is ambiguous
   - Explicit: JSON keys remain English (LAW-L4)
2. [ ] Extend `json_extraction_prompt` and `json_gleaning_prompt` with `language: &str`; inject instruction block.
3. [ ] Add `language: String` + `with_language` on `LLMExtractor`; default `"English"`.
4. [ ] Thread language through `GleaningExtractor` (field or from wrapped extractor / builder).
5. [ ] Add `extraction_language: String` to `IngestionPipelineOptions` (default English); factory calls `with_language`.
6. [ ] SOTA: when `language != "English"`, return empty examples from `get_examples()` (or skip `{examples}` section).
7. [ ] Unit tests: prompt contains language name; English default; gleaning parity; SOTA omit.

**DoD W1**

- [ ] Mock LLM path shows Chinese in prompt when `language = "Chinese"`.
- [ ] No duplication of language wording between primary and gleaning.
- [ ] `EntityExtractionSchema` unchanged (ISP).

---

## Wave 2 — API / Core / Env (F-352-05, F-352-08…11)

**Goal:** Config surfaces + single resolve function.

1. [ ] `canonicalize_extraction_language` / `resolve_extraction_language` in pipeline or core helper (LAW-L3). Export allowlist check.
2. [ ] `apply_extraction_language_metadata` next to entity_types helpers; empty/`none` clears key.
3. [ ] Add `extraction_language: Option<String>` to core + API Create/Update/Response DTOs.
4. [ ] Validate allowlist on create/update → `400` with clear message.
5. [ ] Wire orchestrator ingestion: resolve language → `IngestionPipelineOptions`.
6. [ ] Document `EDGEQUAKE_EXTRACTION_LANGUAGE` in `.env.example`.
7. [ ] `make codegen-openapi-refresh` (or project equivalent) + API contract tests.

**DoD W2**

- [ ] Round-trip create → get → update → get for `Chinese`.
- [ ] Precedence: workspace overrides env overrides English.
- [ ] Invalid language rejected; bad env does not crash (warn → English).

---

## Wave 3 — WebUI (F-352-12)

**Goal:** Discoverable workspace configuration beside Entity Types.

1. [ ] Types: `Workspace.extraction_language?: string | null`.
2. [ ] API client: pass field on create/update.
3. [ ] New `WorkspaceExtractionLanguageCard` (view + edit select from `SUPPORTED_LANGUAGES` list mirrored in FE constants).
4. [ ] Wire into `/w/[slug]/workspace` and dashboard workspace edit; create-workspace form.
5. [ ] Future-only hint + toast on change (reprocess / rebuild knowledge graph) — mirror entity types / LLM change UX (LAW-L5).
6. [ ] i18n keys under `workspace.extractionLanguage.*`.
7. [ ] Playwright: select Chinese, save, reload shows value.

**DoD W3**

- [ ] User can set language without touching Rust source.
- [ ] `data-testid`s match [cross-ref matrix](02-cross-ref-matrix.md).

---

## Wave 4 — Docs + E2E proof (F-352-13…14)

1. [ ] `docs/features.md` FEAT entry + AGENTS.md env table row.
2. [ ] Operator note: multilingual embeddings improve retrieval (TrustGraph advice) — docs only.
3. [ ] Land all test IDs from [04-e2e-test-matrix.md](04-e2e-test-matrix.md).
4. [ ] Optional Playwright proof script under `specs/096-multi-language-extraction/e2e/`.
5. [ ] Close GH-352 with link to SPEC-096 + PR.

**DoD W4**

- [ ] All matrix gates green.
- [ ] Issue #352 closable with evidence.

---

## Wave 5 — Language-aware entity type presets (LAW-L6)

**Goal:** When Extraction Language changes, preset-backed Entity Type chips remap to localized UPPERCASE tokens from one catalog.

1. [x] Add FE `entity-type-catalog.ts` (canonical English → per-language tokens) + helpers `localizeTypes` / `remapPresetTypes` / `detectCanonicalPreset`.
2. [x] Refactor `ENTITY_PRESETS` to `getPresetTypes(key, language)`.
3. [x] Wire language change on workspace pages + create flows: remap only when current types match a known preset; custom lists unchanged + hint.
4. [x] Pass `extractionLanguage` into `EntityTypeSelector` so preset buttons insert language-correct tokens.
5. [x] Playwright S06/S07 + analyzed screenshots; update FEAT-096 docs.

**DoD W5**

- [x] French language + General preset shows French tokens (e.g. `PERSONNE`).
- [x] Custom type lists are never auto-rewritten.
- [x] Screenshots under `specs/096-.../e2e/screenshots/` analyzed PASS.

---

## Definition of Done (product)

- [ ] Non-English workspace can set `extraction_language` via UI and API.
- [ ] Resolved language appears in production JSON extraction prompts.
- [ ] Changing language does not mutate existing nodes until reprocess (LAW-L5).
- [ ] Preset entity types follow extraction language when selection is preset-backed (LAW-L6).
- [ ] DRY/SOLID laws L1–L6 hold (review against [00-first-principles.md](00-first-principles.md)).
- [ ] Documentation and OpenAPI updated.

---

## Suggested file touch list

| Wave | Paths                                                                                                                                                           |
| ------| -----------------------------------------------------------------------------------------------------------------------------------------------------------------|
| W1   | `edgequake-pipeline/src/prompts/json_prompts.rs`, `extractor/llm.rs`, `extractor/gleaning.rs`, `ingestion_pipeline.rs`, `prompts/entity_extraction.rs`          |
| W2   | `edgequake-core/.../helpers.rs`, `types/multitenancy/requests.rs`, `edgequake-api/.../requests.rs` + responses, orchestrator ingestion, `.env.example`, OpenAPI |
| W3   | `edgequake_webui/src/components/workspace/workspace-extraction-language-card.tsx`, workspace pages, API types, i18n                                             |
| W4   | `docs/features.md`, AGENTS.md, tests, `specs/096/.../e2e/`                                                                                                      |
| W5   | `edgequake_webui/src/constants/entity-type-catalog.ts`, `entity-presets.ts`, `entity-type-selector.tsx`, workspace pages, Playwright S06/S07                     |

---

## Out of scope (do not schedule in W1–W5)

- Per-document language / auto-detect
- Translated few-shot libraries
- Graph rewrite on language change (existing AGE node types)
- Auto-translating arbitrary custom types via LLM
- Embedding model auto-switch
