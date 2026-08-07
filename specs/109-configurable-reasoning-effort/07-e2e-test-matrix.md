# SPEC-109 — E2E Test Matrix

## Gates

| Test ID | Kind | Assert | Mechanism |
|---------|------|--------|-----------|
| **E2E-109-01** | unit / contract (`edgequake-llm`) | When `CompletionOptions.reasoning_effort = Some("low")`, native OpenAI Chat Completions JSON **includes** `"reasoning_effort":"low"` (chat + stream builders) | Serialize request builder / httpmock capture |
| **E2E-109-02** | unit (`edgequake-llm`) | `clamp("openai","gpt-5-mini", Some("none")) == Some("minimal")`; `clamp(...,"gpt-5.4-nano", Some("none")) == Some("none")` | Registry table tests |
| **E2E-109-03** | contract (`edgequake-pipeline` / api) | Extract options use resolver: default = lowest supported; workspace `llm_roles.extract.reasoning_effort="low"` (if supported) appears in options after clamp | Mock LLM capturing `CompletionOptions` |
| **E2E-109-04** | api / query contract | `QueryRequest.reasoning_effort` overrides workspace query role; `Auto`/omit inherits | Mock provider + workspace fixture |
| **E2E-109-05** | unit | `mistral-large-latest` → `reasoning_effort` omitted even if desired `high` | Registry + extraction_completion_options |
| **E2E-109-06** | api contract | `GET /api/v1/config/effective` returns per-role `desired`, `effective`, `source`, `clamped` | HTTP test against test app |
| **E2E-109-07** | OpenAPI | New fields present on Query, Workspace update, LLM defaults, PDF options schemas | `spec027_api_contract` + codegen refresh |
| **E2E-109-08** | Playwright | Workspace: set extract effort, reload, value persists; Query sheet: set override, intercepted POST body contains field | `edgequake_webui/e2e/` |
| **E2E-109-09** | unit (query cache) | Cache key differs when effort differs (SPEC-103 adjacency) | `edgequake-query` cache hash test |
| **E2E-109-10** | optional live | Real OpenAI `gpt-5-mini` with clamped `minimal` returns non-empty structured extract on fixture chunk | `#[ignore]` + `OPENAI_API_KEY` |

## Proof commands (target)

```bash
# Wave 0 — sibling crate
cd ../edgequake-llm && cargo test reasoning_capabilities --lib
cd ../edgequake-llm && cargo test openai --lib

# EdgeQuake (after bump / path patch)
cd edgequake && cargo test -p edgequake-llm --lib   # if vendored; else dependency tests in CI of llm crate
cd edgequake && cargo test -p edgequake-pipeline --lib completion_options
cd edgequake && cargo test -p edgequake-core --lib llm_roles
cd edgequake && cargo test -p edgequake-api --test spec027_api_contract
cd edgequake && cargo test -p edgequake-api --test contract_spec109_reasoning_effort   # to add
cd edgequake && cargo test -p edgequake-query --lib cache::

# Makefile aggregator
make spec109-reasoning-effort-proof

# UI
cd edgequake_webui && pnpm exec playwright test e2e/reasoning-effort.spec.ts
```

## Fixture pins

| Fixture | Value |
|---------|-------|
| Structured default model A | `gpt-5-mini` → expect effective `minimal` |
| Structured default model B | `gpt-5.4-nano` → expect effective `none` |
| Non-reasoning | `gpt-4.1-mini` → expect omit |
| Rejecting | `mistral-large-latest` → expect omit |
| Query override | workspace query `medium`, request `low` → send `low` if supported |

## Exit criteria

All non-optional gates green in CI. Optional live (**E2E-109-10**) documented under [`measurements/`](measurements/) when run.
