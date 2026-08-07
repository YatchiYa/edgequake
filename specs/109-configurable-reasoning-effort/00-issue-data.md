# 00 — Issue Data (Reasoning Effort)

> Source: product / partner identified issue for next version.  
> Cross-ref: [00-why.md](00-why.md) · [04-finding-register.md](04-finding-register.md)

## Original statement

> ## Reasoning effort of models
>
> Identified issue: GPT-5 mini, used for LLM Vision and entity extraction, consumes a significant portion of the token budget for reasoning, to the detriment of output. Reasoning effort is not currently configurable; this will be added in next version.

## English extract

| Signal | Value | Interpretation |
|--------|-------|----------------|
| Models named | GPT-5 mini | Reasoning family; image+text capable |
| Workloads | LLM Vision + entity extraction | `vlm` + `extract` roles |
| Symptom | Reasoning consumes token budget | Internal CoT vs visible/structured output |
| Harm | Output detriment | Truncated / empty extract JSON; thin captions |
| Ask | Make reasoning effort configurable | Product surface + wire-through |
| Timing | Next version | This pack is the implementation SSOT |

## Code reality (pre-SPEC-109)

| Fact | Location | Implication |
|------|----------|-------------|
| Extract sets `reasoning_effort: Some("none")` when `model_accepts_reasoning_effort` | `edgequake-pipeline/.../completion_options.rs` | Intent exists; not configurable |
| Native OpenAI chat builder does **not** call `.reasoning_effort(...)` | `edgequake-llm/.../providers/openai.rs` | Intent never reaches OpenAI Chat Completions |
| OpenAI-compatible / Ollama / LM Studio / NVIDIA / xAI / Anthropic paths already map the field | `edgequake-llm` providers | Gap is **native OpenAI**, not the trait |
| `gpt-5-mini` API rejects `'none'` | Vendor 400 (OpenClaw #62967 class) | Hardcode is unsafe without clamp |
| `gpt-5.4-mini` / `gpt-5.4-nano` support `none` (default) | [OpenAI model pages](https://developers.openai.com/api/docs/models/gpt-5.4-mini) | Floor effort = `none` for 5.4 mini/nano |
| Query / workspace / tenant have no effort field | API + WebUI | Partner cannot tune without rebuild |
| Query UI sends `temperature` but backend ignores it | WebUI vs `QueryRequest` | Precedent: client knobs must be first-class on API |

## What we do **not** know from the statement alone

- Exact deployment model slug (`gpt-5-mini` vs `gpt-5.4-mini` / nano)
- Whether they hit native OpenAI or an OpenAI-compatible proxy
- Observed `completion_tokens` vs empty-body rate
- Whether vision and extract use the same model card

SPEC-109 designs for **all** those variants via role config + clamp registry; measurements after Wave 4 fill proof artifacts under [`measurements/`](measurements/).
