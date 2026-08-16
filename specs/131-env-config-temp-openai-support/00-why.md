# 00 — Why SPEC-131

## Trigger

Intake: [`zz-raw.md`](zz-raw.md) — GitHub [#379](https://github.com/raphaelmansuy/edgequake/issues/379).

Operators on AWS Bedrock Mantle (OpenAI-compatible) hit mass ingest failure when the extract model rejects `temperature: 0`. Separately, GPT-5.6-series models on Mantle are **Responses-only** and cannot be used at all because EdgeQuake only speaks Chat Completions upstream.

## Product WHY

```ascii
  Operator: “I pointed EDGEQUAKE_LLM_MODEL at Gemma 4 / Grok 4.3
             on Mantle — why did 121 docs fail extraction?”
  Operator: “I need gpt-5.6-luna on Bedrock — why won’t EdgeQuake call it?”
       │
       ▼
  Today (two axes):
       Axis A — Parameter surface drift
         Hardcoded temperature omit list (gpt-5 / o*)
         Misses Gemma / Grok / future families
         Some call sites hardcode Some(0.0) anyway
              │
              ▼
         HTTP 400 unsupported_value → failure_class=unknown
              │
       Axis B — Transport binding
         Product LLM = Chat Completions only
         Mantle GPT-5.6 = Responses API only
              │
              ▼
         Model selectable in env, unusable on wire
```

## Five WHYs

1. **Why did ingest fail?** Upstream returned `unsupported_value` for `temperature`.
2. **Why was temperature sent?** Extract prefers `0.0` via `effective_temperature_for_model`; Gemma/Grok are not on the omit list; VLM paths may hardcode `Some(0.0)`.
3. **Why isn’t every rejecting model on the list?** The set of models that forbid overrides is open-world and grows with every provider release.
4. **Why can’t operators disable the field today?** There is no env/policy knob; only code patches or model renames.
5. **Root cause (A):** EdgeQuake treats temperature as a universal Chat Completions knob and encodes exceptions as a lagging substring catalog instead of an operator-controlled omit policy.

**Parallel root cause (B):** EdgeQuake binds product completion semantics to one HTTP transport (`/v1/chat/completions`). Platforms that expose the same model only on `/v1/responses` are unreachable regardless of `CompletionOptions` correctness.

## Job to be done

> When I run EdgeQuake against OpenAI-compatible gateways (including Bedrock Mantle), I can omit unsupported sampling/effort fields via env without waiting for a model-list patch, and I can select Responses API transport for models that are not available on Chat Completions — without changing EdgeQuake’s own server chat facade.

## Success criteria

1. `EDGEQUAKE_LLM_OMIT_TEMPERATURE=true` → no `temperature` on upstream Chat Completions for extract, query, title, VLM.
2. `EDGEQUAKE_LLM_OMIT_REASONING_EFFORT=true` → no `reasoning_effort` on the wire (after role resolve).
3. Existing model-gate omit for gpt-5 / o* still works when env unset.
4. `EDGEQUAKE_LLM_API_FORMAT=responses` → `POST …/responses` with `store:false`; structured JSON parity for extract/query.
5. Temperature / unsupported_value errors classify as `llm_unsupported_param` with actionable recommended_action.
6. Setup docs list the three env knobs; e2e wiremock matrix in [08-test-protocol.md](08-test-protocol.md) passes.

## Non-goals (product)

- Replacing EdgeQuake `/api/v1/chat/completions` **server** endpoint
- Dropping Chat Completions default before Responses parity is proven
- Hosted tools / Conversations API / `previous_response_id` multi-turn in v1
- Auto-detecting Responses-only models without operator format config (v1)

## Cross-refs

- Laws: [01-first-principles.md](01-first-principles.md)
- Code: [03-code-as-is.md](03-code-as-is.md)
- Plan: [07-implementation-plan.md](07-implementation-plan.md)
- Edges: [10-edge-cases.md](10-edge-cases.md)
- Issue: [#379](https://github.com/raphaelmansuy/edgequake/issues/379)
