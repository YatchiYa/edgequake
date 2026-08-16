# Lens 002 — Full Stack Developer

## Stake

Two repos change in lockstep: EdgeQuake product (options + classify + docs) and sibling `edgequake-llm` (wire strip + Responses mapper). Keep DRY: one resolver, one mapper, no per-call-site inventiveness.

## Implementation map

```ascii
  edgequake (product)
    temperature.rs          → resolve_effective_temperature
    completion_options.rs   → use resolver
    title_generator.rs      → use resolver
    figure_filter.rs        → STOP hardcoding Some(0.0); use resolver
    effort resolve paths    → apply OMIT_REASONING_EFFORT
    ingestion_reliability   → LlmUnsupportedParam
    .env.example / AGENTS   → document knobs

  edgequake-llm
    api_format.rs (new)     → parse EDGEQUAKE_LLM_API_FORMAT
    env_truthy helpers      → OMIT_* shared
    responses_map.rs (new)  → request/response/SSE
    openai.rs               → branch on ApiFormat; wire strip
    openai_compatible.rs    → same
    factory.rs              → fail loud on invalid format
    tests/wiremock          → E2E-131-*
```

## SOLID checklist

- [ ] Resolver not duplicated in llm crate model lists
- [ ] Responses mapper not copy-pasted between OpenAI and openai_compatible
- [ ] Classifier tokens single enum variant + `from_token`
- [ ] No change to product `/api/v1/chat/completions` handler shape

## Integration points

| Concern | Approach |
|---------|----------|
| Workspace LLM roles | Unchanged; omit is **server env** fleet policy |
| Prompt cache SPEC-126 | Chat path unchanged; Responses: forward `prompt_cache_key` if accepted else document P1.1 |
| Hybrid embedding | Orthogonal — embeddings stay embed endpoint |
| Streaming query UI | Map Responses text deltas to existing `StreamChunk` |

## Boot / fail loud

Invalid `EDGEQUAKE_LLM_API_FORMAT` → provider factory / server start error with allowed values. Do not silently treat as chat.

`API_FORMAT=responses` on Ollama-native provider → ignore with debug log (native path) **or** document “only applies to OpenAI/openai_compatible”; prefer ignore+log for Acc mock safety.

## Test ownership

| Layer | Owner crate |
|-------|-------------|
| Resolver unit | edgequake-pipeline |
| Classifier unit | edgequake-tasks |
| Wiremock body | edgequake-llm (+ api e2e if needed) |
| Source contract (no bare Some(0.0) LLM) | edgequake-api or pipeline contract test |

## Cross-refs

- Target: [../04-target-architecture.md](../04-target-architecture.md)
- Plan: [../07-implementation-plan.md](../07-implementation-plan.md)
- Tests: [../08-test-protocol.md](../08-test-protocol.md)
