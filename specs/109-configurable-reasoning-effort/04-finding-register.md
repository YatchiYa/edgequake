# SPEC-109 — Finding Register

> Updated after implementation waves 0–4 (2026-08-05).

| ID | Finding | Severity | Verdict | Primary path |
|----|---------|----------|---------|--------------|
| **F1** | Native OpenAI provider builds Chat Completions **without** setting `reasoning_effort`, even when `CompletionOptions.reasoning_effort` is `Some` | **P0** | **Closed** | `edgequake-llm` `providers/openai.rs` + Azure — `apply_reasoning_effort` on chat/tools/stream |
| **F2** | Extraction hardcodes `reasoning_effort: "none"` whenever `model_accepts_reasoning_effort` — not configurable; unsafe for models that reject `none` (e.g. `gpt-5-mini`) | **P0** | **Closed** | `extraction_completion_options_with_effort` + `clamp` / `lowest_for_structured_output` |
| **F3** | No tenant / workspace / server / query / vision request field or WebUI control for reasoning effort | **P0** | **Closed** | API DTOs + `ReasoningEffortSelect` (query sheet, server card, explainability) |
| **F4** | No shared capability registry: EdgeQuake uses ad-hoc `model_accepts_reasoning_effort` boolean; cannot express per-model allowed sets or clamp | **P0** | **Closed** | `edgequake-llm::reasoning_capabilities` |
| **F5** | OpenAI-compatible / Ollama / LM Studio / NVIDIA / xAI / Anthropic already forward or map effort — behavior **asymmetric** vs native OpenAI | **P1** | **Closed** (all builders clamp via shared registry; Anthropic uses `output_config.effort`; OpenRouter forwards) | Various providers + `clamp_options_reasoning_effort` |
| **F6** | `QueryRequest` lacks `reasoning_effort` (and already ignores client `temperature`) — precedent that UI knobs must be first-class on API | **P1** | **Closed** | `query_types` + chat + engine `QueryRequest` wire-through |
| **F7** | Effective-config / explainability does not report reasoning effort or clamp | **P2** | **Closed** | `reasoning_roles` on `/config/effective` + WebUI panel |
| **F8** | Models catalog / WebUI cannot discover supported efforts → risk of illegal selects | **P1** | **Closed** | `ModelCapabilitiesResponse.reasoning_effort` |
| **F9** | LLM response cache keys (SPEC-103) do not mention effort — two efforts could collide on same prompt if effort later wired into query path | **P2** | **Closed** | `hash_query_prompt_with_effort` |
| **F10** | Comments in `sota.rs` / CHANGELOG assume `"none"` always works for gpt-5-mini/nano — documentation drift | **P2** | **Closed** | configuration.md + CHANGELOG + `.env.example` |

## Detail notes

### F1 — Native OpenAI drop — fixed

`apply_reasoning_effort` maps clamped strings → `async_openai::types::ReasoningEffort` on chat / tools / stream builders. E2E-109-01 covered in llm crate tests.

### F2 — Hardcoded none — fixed

Desired effort resolved then clamped; unset structured roles use `lowest_for_structured_output` (`minimal` for mini, `none` for 5.4-nano).

### F3–F8 — Surfaces + catalog — fixed

See roadmap waves 2–3 and proof `make spec109-reasoning-effort-proof`.

## Non-findings

| Item | Note |
|------|------|
| Trait field missing | `CompletionOptions.reasoning_effort` already existed |
| Embeddings need effort | Out of scope |
| Responses API / thinking-trace UI | Out of scope v1 |
| crates.io publish of edgequake-llm | **Done** — `0.10.4` on crates.io; EdgeQuake path patch removed |
