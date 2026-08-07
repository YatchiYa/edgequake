# SPEC-109 — Implementation Roadmap

> Spec pack is Wave −1 (this directory). Code waves below are normative for implementers.
> Status: waves 0–4 implemented 2026-08-05 (`edgequake-llm` **0.10.4** on crates.io).

## Wave 0 — `edgequake-llm` (blocking)

**Repo:** `/Users/raphaelmansuy/Github/03-working/edgequake-llm`

- [x] Add `src/reasoning_capabilities.rs` (+ re-export from `lib.rs`)
  - `capabilities(provider, model)`
  - `clamp_reasoning_effort(...)`
  - `lowest_for_structured_output(...)`
  - Unit table: `gpt-5-mini` + `none` → `minimal`; `gpt-5.4-nano` + `none` → `none`; `mistral-large` → omit; `gpt-4.1-mini` → omit
- [x] Native `providers/openai.rs`: set `reasoning_effort` on **all** chat paths (complete, chat, stream, tools) via async-openai builder when `options.reasoning_effort` is `Some` **after caller clamp** (provider may also clamp defensively)
- [x] Audit Anthropic / Ollama / xAI / NVIDIA / LM Studio / OpenAI-compatible: apply clamp at request build or document that callers must clamp
- [x] Docs: `docs/providers.md` + OpenAI section — wire field + registry
- [x] Bump crate version (e.g. `0.10.4` or next semver)
- [x] Tests: serialization assert JSON contains `"reasoning_effort"` / Anthropic `output_config.effort`

**EdgeQuake:** path patch removed; depends on crates.io `edgequake-llm = "0.10.4"`.

## Wave 1 — Core resolution + extract

**Crates:** `edgequake-core`, `edgequake-pipeline`

- [x] Extend `RoleLlmConfig` with `reasoning_effort: Option<String>`
- [x] Add `resolve_role_reasoning_effort(...)` beside `resolve_role_llm` (LAW-R6)
- [x] Extend `ServerLlmDefaults` with `reasoning_effort` + `reasoning_by_role`
- [x] Tenant / workspace seed fields (mirror vision defaults pattern)
- [x] Replace `extraction_completion_options` hardcode:
  - Input: model + optional desired effort from resolver
  - Output: clamped `CompletionOptions` using `lowest_for_structured_output` when desired unset for extract/summary
- [x] Wire extract / sota / gleaning / summary / keyword paths through resolver
- [x] Vision PDF path: resolve VLM effort into vision completion options (`ReasoningEffortInjectProvider`)
- [x] Deprecate or thin-wrap `model_accepts_reasoning_effort` → registry

## Wave 2 — API + OpenAPI

**Crate:** `edgequake-api`

- [x] `QueryRequest` / stream request: `reasoning_effort: Option<String>`
- [x] Pass override into query engine completion options
- [x] `PdfUploadOptions.vision_reasoning_effort`
- [x] `GET/PATCH /settings/llm-defaults` schema
- [x] Workspace / tenant create-update DTOs
- [x] `GET /config/effective` per-role desired/effective/source/clamped
- [x] Models catalog: expose `reasoning_effort.supported` / `lowest_structured`
- [x] `make codegen-openapi-refresh` + `cargo test -p edgequake-api --test spec027_api_contract`
- [x] Echo effort on query stats when set

## Wave 3 — WebUI

**App:** `edgequake_webui`

- [x] Types: workspace / tenant / query settings / API clients
- [x] Workspace page: per-role effort select (filtered by model capabilities)
- [x] `/settings`: fleet default + per-role advanced
- [x] Tenant wizard: seed control
- [x] `query-settings-sheet`: Auto + filtered efforts; include in request body
- [x] PDF upload advanced: vision effort (dropzone select when Vision parser + multipart)
- [x] Effective-config panel: show clamp badge
- [x] i18n strings (EN minimum)

## Wave 4 — Proof + docs

- [x] `make spec109-reasoning-effort-proof` (unit + contract suite; optional live OpenAI behind flag)
- [x] Playwright: E2E-109-08 (`make spec109-e2e` + [`measurements/e2e/screenshots/`](measurements/e2e/screenshots/README.md))
- [x] Update `edgequake/docs/configuration.md`, `.env.example`, AGENTS env table
- [x] CHANGELOG entry
- [x] Fill [`measurements/`](measurements/) with proof logs
- [x] Mark findings F1–F8 closed in [04](04-finding-register.md)

## Definition of Done

1. Native OpenAI request JSON includes `reasoning_effort` when configured (**E2E-109-01**).  
2. `gpt-5-mini` + desired `none` never yields vendor 400; effective `minimal` (**E2E-109-02**).  
3. Unconfigured extract uses lowest supported for the model; workspace override changes options (**E2E-109-03**).  
4. Query request override beats workspace query role (**E2E-109-04**).  
5. Mistral Large omits field (**E2E-109-05**).  
6. Effective config reports effort + source (**E2E-109-06**).  
7. OpenAPI contract green (**E2E-109-07**).  
8. Playwright workspace save + query sheet override (**E2E-109-08**).  
9. No second metadata tree; `RoleLlmConfig` remains SSOT for role overrides.  
10. Acc docs recommend pinning structured roles to lowest effort.

## Risk register

| Risk | Mitigation |
|------|------------|
| F1 fixed without F2 clamp | Ship Wave 0 clamp + OpenAI forward **together** |
| UI offers illegal values | Catalog-driven options only |
| Cache collisions | Include effort in SPEC-103 hash when present |
| Provider doc churn | Registry tests + matrix maintenance rule in [03](03-provider-capability-matrix.md) |
| Path vs crates.io lag | **Resolved** — `0.10.4` published; patch removed |
