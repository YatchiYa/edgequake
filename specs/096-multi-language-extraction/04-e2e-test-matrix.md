# SPEC-096 — E2E / Contract Test Matrix

> **Cross-refs**: [Roadmap](03-implementation-roadmap.md) · [Edge cases](05-edge-cases.md) · [Cross-ref](02-cross-ref-matrix.md)

| Test ID | Kind | Layer | Assertion |
|---------|------|-------|-----------|
| `spec096_json_prompt_includes_language` | unit | pipeline | Prompt for `Chinese` contains language instruction + `"Chinese"`; JSON key section still English |
| `spec096_gleaning_prompt_includes_language` | unit | pipeline | Gleaning prompt includes same language instruction SSOT |
| `spec096_json_prompt_default_english` | unit | pipeline | Default / `"English"` still valid; instruction present or English-explicit |
| `spec096_llm_extractor_language_builder` | unit | pipeline | `with_language("Japanese")` stored and used in `build_prompt` |
| `spec096_pipeline_factory_wires_language` | unit | pipeline | `IngestionPipelineOptions { extraction_language: "Korean", .. }` reaches extractor prompt |
| `spec096_sota_omits_examples_non_english` | unit | pipeline | SOTA system prompt for `Chinese` has no English few-shot entity rows; still has `{language}` |
| `spec096_resolve_language_precedence` | unit | core/pipeline | workspace > env > English; empty workspace falls through; bad env → English + warn |
| `spec096_canonicalize_language` | unit | core/pipeline | `"chinese"` → `"Chinese"`; `"ZH"` rejected unless in allowlist |
| `spec096_api_create_update_get_language` | contract/e2e | api | Create with `extraction_language=Chinese`; GET returns it; Update to `Japanese`; clear with `""`/`none` |
| `spec096_api_rejects_unsupported_language` | contract | api | `extraction_language=Klingon` → HTTP 400 |
| `spec096_workspace_metadata_roundtrip` | e2e PG | core/storage | Metadata JSONB key persists across restart/read |
| `spec096_e2e_ingest_prompt_language` | e2e | api+pipeline | Mock LLM capture: Chinese workspace ingest prompt contains language instruction |
| `spec096_ui_workspace_language_select` | Playwright | webui | Edit workspace → select Chinese → save → reload shows Chinese |
| `spec096_ui_create_workspace_language` | Playwright | webui | Create workspace with language French → detail shows French |
| `spec096_ui_future_only_hint` | Playwright | webui | Hint / toast visible that existing docs need reprocess |
| `spec096_entity_type_catalog_french_general` | unit | webui | General preset → French tokens include `PERSONNE` / `ORGANISATION` |
| `spec096_entity_type_catalog_custom_no_remap` | unit | webui | Custom list unchanged by `remapPresetTypes` |
| `spec096_ui_entity_types_follow_language` | Playwright | webui | S06: select French → chips show French preset tokens; S07: English restores English preset |
| `spec096_openapi_has_extraction_language` | contract | api | OpenAPI schema includes `extraction_language` on workspace create/update/response |

## Run (after W1–W4 land)

```bash
# Pipeline units
cargo test -p edgequake-pipeline --lib json_extraction_prompt
cargo test -p edgequake-pipeline --lib spec096

# Core / API
cargo test -p edgequake-core --lib resolve_extraction_language
cargo test -p edgequake-api --test contract_spec096_extraction_language
cargo test -p edgequake-api --features postgres --test e2e_spec096_extraction_language

# OpenAPI
make codegen-openapi-refresh
cargo test -p edgequake-api --test spec027_api_contract

# WebUI
cd edgequake_webui && pnpm exec playwright test e2e/spec096-extraction-language.spec.ts
```

## Proof script (optional, W4)

`specs/096-multi-language-extraction/e2e/run_spec096_proof.sh` should:

1. Assert backend healthy.
2. Create workspace with `extraction_language=Chinese` via curl.
3. GET workspace and jq-assert field.
4. Optionally trigger mock ingest and grep backend log / captured prompt for language instruction.
5. Print PASS/FAIL summary to `e2e/artifacts/RUN_NOTES.md`.

## Coverage vs laws

| Law | Tests |
|-----|-------|
| L1 | API reject, UI select, metadata roundtrip |
| L2 | JSON + gleaning prompt units, SOTA omit |
| L3 | resolve precedence |
| L4 | prompt still has English JSON keys |
| L5 | UI future-only hint; no auto graph rewrite test (assert absence of migration) |
