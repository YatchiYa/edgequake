# Lens 006 — AI Engineer (knowledge pin: August 2026)

## Stake

Parameter legality and transport choice are first-class LLM systems concerns. EdgeQuake must speak the APIs providers actually expose in 2026 — Chat Completions **and** Responses — without baking open-world model catalogs into product code.

## Industry pins (Aug 2026)

| Source | Takeaway for EdgeQuake |
|--------|------------------------|
| [OpenAI migrate to Responses](https://developers.openai.com/api/docs/guides/migrate-to-responses) | Responses is recommended for new work; Chat Completions remains supported; messages→items; `store` defaults on |
| [Responses API blog](https://developers.openai.com/blog/responses-api) | Reasoning state + polymorphic `output[]`; better cache utilization claims |
| [Open Responses](https://www.openresponses.org/specification) | Cross-vendor shape for `/v1/responses` |
| [Bedrock Mantle](https://docs.aws.amazon.com/bedrock/latest/userguide/bedrock-mantle.html) | OpenAI-compatible Responses; **`store` default true (30d)**; set `store:false` to avoid retention |
| [GPT-5.6 on Bedrock](https://aws.amazon.com/blogs/machine-learning/get-started-with-openai-gpt-5-6-sol-terra-and-luna-on-amazon-bedrock/) | Sol/Terra/Luna via Responses on Mantle; `input` / `output_text` |
| AWS IDP GPT-5.x notes | Mantle GPT-5.x often **not** on Converse; temperature/top_p rejected — use `reasoning.effort` |

## First-principles for model params

```ascii
  Sampling knobs (temperature, top_p)     Reasoning knobs (effort)
         │                                        │
         ▼                                        ▼
  Illegal on many 2025–2026 reasoning /     SPEC-109 already clamps;
  “default-only” Chat Completions models    SPEC-131 adds fleet omit
         │
         ▼
  Policy: omit field  >  send preferred 0  >  fake send 1.0
```

Sending `temperature: 1.0` is **not** equivalent to omit on gateways that treat any explicit value as unsupported.

## Responses mapping (AI-critical)

```ascii
  Extract / query need: deterministic-ish structured JSON
       │
       ▼
  Chat: response_format = json_schema | json_object
  Resp: text.format   = json_schema | json_object
       │
       ▼
  Output: concatenate message output_text parts;
          ignore reasoning summary items for product content
          (may log thinking_tokens if usage present)
```

Streaming: prefer semantic `response.output_text.delta` events; map to existing UI token stream. Do not require tool events for v1 extract/query.

## Mantle operator recipe (document in setup)

```text
# Gemma / Grok — Chat Completions, omit temperature
EDGEQUAKE_LLM_PROVIDER=openai
EDGEQUAKE_LLM_MODEL=google.gemma-4-31b
OPENAI_BASE_URL=https://bedrock-mantle.<region>.api.aws/openai/v1   # or …/v1 per AWS pin
EDGEQUAKE_LLM_OMIT_TEMPERATURE=true
EDGEQUAKE_LLM_API_FORMAT=chat_completions

# GPT-5.6 — Responses
EDGEQUAKE_LLM_MODEL=openai.gpt-5.6-luna
EDGEQUAKE_LLM_API_FORMAT=responses
EDGEQUAKE_LLM_OMIT_TEMPERATURE=true   # belt-and-suspenders
# reasoning via existing EDGEQUAKE_*_REASONING_EFFORT (SPEC-109)
```

Base URL path variants (`/openai/v1` vs `/v1`) differ across AWS docs — operators must match the endpoint they use; EdgeQuake appends `/chat/completions` or `/responses` relative to configured base.

## Eval / quality

- Omit-temp may increase output variance vs `temperature:0` on models that **do** support 0 — document for Acc.
- Responses structured extract: same json_schema as chat; wiremock proves field mapping; live Mantle proves model acceptance.
- Do not claim “Responses always smarter” in product UI — internal OpenAI claims are not EdgeQuake SLOs.

## Observability

Span attributes: `llm.api_format`, `llm.omit_temperature`, `gen_ai.request.model`. Never log full prompts with PII in clear beyond existing policy.

## Cross-refs

- Architecture: [../04-target-architecture.md](../04-target-architecture.md)
- SPEC-109: [../../109-configurable-reasoning-effort/](../../109-configurable-reasoning-effort/)
- Edges: [../10-edge-cases.md](../10-edge-cases.md)
