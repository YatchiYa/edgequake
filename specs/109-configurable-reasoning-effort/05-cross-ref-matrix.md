# SPEC-109 — Cross-Reference Matrix

| Finding / UC | Law | Code / surface (today → target) | E2E |
|--------------|-----|----------------------------------|-----|
| F1 native OpenAI drop | R2, R7 | `edgequake-llm/.../openai.rs` → set `reasoning_effort` on chat + stream + tools paths | **E2E-109-01** |
| F2 hardcoded `"none"` | R3, R5 | `completion_options.rs` → `lowest_for_structured_output` or resolved extract effort + clamp | **E2E-109-02**, **E2E-109-03** |
| F3 no UX/API | R6, R8 | `RoleLlmConfig`, tenant/server defaults, `QueryRequest`, PDF options, WebUI | **E2E-109-04**, **E2E-109-06**, **E2E-109-07**, **E2E-109-08** |
| F4 no registry | R3, R4 | New `edgequake-llm/src/reasoning_capabilities.rs`; delete/replace boolean-only gate | **E2E-109-02**, **E2E-109-05** |
| F5 asymmetric providers | R7 | Audit anthropic/ollama/xai/nvidia/lmstudio/openai_compatible; keep maps; add clamp at edge | Provider unit tests |
| F6 QueryRequest gap | R6, R8 | `query_types.rs` + stream twin + OpenAPI | **E2E-109-04**, **E2E-109-07** |
| F7 effective config | R8 | `config_resolution.rs` + settings UI panel | **E2E-109-06** |
| F8 catalog capabilities | R4, R8 | Models API payload from registry | **E2E-109-08** (UI options) |
| F9 cache key | R1 adjacent | SPEC-103 hash includes effort when `Some` | Unit under query cache |
| F10 doc drift | — | Pipeline comments, `docs/configuration.md`, CHANGELOG | Review gate |
| UC-EXTRACT | R5 | Workspace `llm_roles.extract.reasoning_effort` → extractor options | **E2E-109-03** |
| UC-VLM | R5 | Vision env/workspace + `vision_reasoning_effort` | Contract + upload e2e |
| UC-QUERY | R5 | Auto omit; request override wins | **E2E-109-04** |
| UC-KEYWORD / SUMMARY | R5 | Same resolver as extract floor | Unit resolve tests |
| Mistral Large omit | R3 | Registry empty → omit | **E2E-109-05** |
| gpt-5-mini `none`→`minimal` | R3 | Registry clamp | **E2E-109-02** |

## Hierarchy vs code owners

| Layer | Owner crate | Symbol / endpoint |
|-------|-------------|-------------------|
| Compiled defaults | `edgequake-core` | `compiled_default(role)` in resolver |
| Env | `edgequake-core` / api boot | `EDGEQUAKE_*_REASONING_EFFORT` |
| Server DB | `edgequake-core` + api | `ServerLlmDefaults`, `PATCH /settings/llm-defaults` |
| Tenant seed | `edgequake-core` + api | Tenant defaults on workspace create |
| Workspace role | `edgequake-core` | `RoleLlmConfig.reasoning_effort` |
| Request | `edgequake-api` | `QueryRequest`, `PdfUploadOptions` |
| Clamp + wire | `edgequake-llm` | `reasoning_capabilities`, providers |

## Dependency order (must not invert)

```text
Wave 0: edgequake-llm (F1 + F4)  ──publish/bump──►  Wave 1: resolve + extract (F2)
                                              └──►  Wave 2: API fields (F3/F6/F7)
                                              └──►  Wave 3: WebUI (F3/F8)
                                              └──►  Wave 4: proof makefile + Playwright
```
