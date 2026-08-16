# SPEC-126 — Provider KV / Prompt Cache

**Status:** IMPLEMENTED  
**Product pin:** default **on** (`EDGEQUAKE_PROMPT_CACHE`)  
**Distinct from:** [SPEC-103](../103-llm-cache/) response cache (skip the LLM call)

## First principles (no heuristics)

| Law | Rule |
|-----|------|
| LAW-126-1 | Two layers: SPEC-103 skips generation; SPEC-126 still generates and reuses provider KV. |
| LAW-126-2 | Wire format is chosen by the **provider type**, never by parsing `provider.name()` or model strings. |
| LAW-126-3 | Stable bytes first, dynamic last. Changing retrieval/query must not sit in the cached prefix. |
| LAW-126-4 | GPT-5.6 `prompt_cache_options` / `prompt_cache_breakpoint` are sent on **`OpenAIProvider` (Native constructor, including official-OpenAI proxies)** and **Azure OpenAI**. `OpenAIProvider::compatible` and `OpenAICompatibleProvider` send `prompt_cache_key` only. No host-name sniffing. |
| LAW-126-5 | Model support is learned from a structured `error.param` 400 (`prompt_cache_options` / `prompt_cache_breakpoint`): unknown → try; match → remember false and retry without. A 200 does not record “supported”. No version arithmetic, no URL fingerprint lists. |
| LAW-126-6 | Acc leaves SPEC-126 **on**. Pin only SPEC-103 off. |
| LAW-126-7 | OpenRouter: `cache_control` + `prompt_cache_key` + `session_id` (sticky routing). Bedrock Converse: `cachePoint` after system blocks. |

## One-screen verdict

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│  Honor each vendor's prompt/KV cache. Still generate. Cheaper prefill.       │
│  Native OpenAI/Azure: key + GPT-5.6 explicit breakpoints (API-learned).      │
│  OpenAI::compatible / Mistral / NVIDIA: prompt_cache_key.                    │
│  Anthropic: cache_control + TTL. OpenRouter: cache_control + session_id.     │
│  Bedrock Converse: cachePoint. Gemini/Ollama: layout. Mix context in user.   │
│  Acc: leave ON (not a skip-generation Beat). SPEC-103 stays pinned off.      │
└──────────────────────────────────────────────────────────────────────────────┘
```

## Consequences

| Layer | Default | What it does | Acc |
|-------|---------|--------------|-----|
| SPEC-103 `EDGEQUAKE_LLM_CACHE` | ON | Skip LLM on exact keyword/answer hit | Pin **off** |
| SPEC-126 `EDGEQUAKE_PROMPT_CACHE` | ON | Reuse provider KV for identical prefixes | **Leave on** |

- Cached input is billed cheaper (Mistral ~10%; Anthropic reads ~10% with 1.25×/2.0× write; OpenAI GPT-5.6 writes 1.25×, reads 0.1×).
- First request is a cache **write**; later matching prefixes hit.
- TTFT drops; decode time does not.
- Mix answers keep the same facts; only message roles change (instructions first). Mix KV win is **small** unless Mix system instructions exceed the vendor min tokens (OpenAI 1024, Anthropic 512–4096, Gemini 2048–4096). Extract/glean is the high-volume win.
- OpenAI ~15 RPM per `prompt_cache_key`. Extract uses one `eq:extract:{provider}:{model}` key; burst ingest may miss routing. Do not partition keys with PII.
- Disable: `EDGEQUAKE_PROMPT_CACHE=0`. Anthropic/Bedrock TTL: `EDGEQUAKE_PROMPT_CACHE_TTL=5m|1h`.
- CI checkouts sibling `edgequake-llm` for `[patch.crates-io]`. After publishing **0.10.8**, bump the workspace pin and drop the patch.
- E2E: `cargo nextest run -p edgequake-api --test e2e_spec126_prompt_cache`. Live: `EDGEQUAKE_LIVE_PROMPT_CACHE=1 cargo test --manifest-path ../edgequake-llm/Cargo.toml --test e2e_spec126_prompt_cache live_openai -- --ignored`.

## Prompt layout (SOTA Aug 2026)

Stable bytes first, dynamic last:

1. Extract / glean / keywords: system template, user = chunk or query
2. Mix answer: system = instructions; user = `---Context---` + query
3. Keys: `eq:{role}:{provider}:{model}` — no PII
4. GPT-5.6 native/Azure: explicit breakpoint on the last leading system/developer block; `prompt_cache_options.mode=explicit` so the changing user turn is not a cache write

