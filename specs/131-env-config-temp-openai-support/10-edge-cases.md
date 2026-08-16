# 10 — Edge cases

> Each row: risk → mitigation → test.

| ID | Edge case | Mitigation | Test |
|----|-----------|------------|------|
| EC-131-01 | Gemma/Grok not on heuristic list | Env omit (LAW-131-2) | U-131-01, E2E-131-01, LIVE-131-A |
| EC-131-02 | VLM hardcodes `Some(0.0)` | Resolver at call site + wire strip | E2E-131-07, E2E-131-01 VLM path |
| EC-131-03 | OpenAI quirk skips only ≈1.0 | Omit env strips entirely; do not send 1.0 as stand-in | E2E-131-01 |
| EC-131-04 | Operator wants temp=0 on supporting model | Leave omit unset; gate allows Some | U-131-04 |
| EC-131-05 | Omit-temp increases variance vs temp=0 | Document quality tradeoff in AI lens / setup | docs review |
| EC-131-06 | `OMIT_REASONING_EFFORT` vs SPEC-109 role desired | Env wins after resolve | E2E-131-03 |
| EC-131-07 | Mistral Large already omits effort | Omit-env redundant; still None | existing e2e109 + E2E-131-03 |
| EC-131-08 | Invalid `API_FORMAT` | Fail loud | U-131-06 |
| EC-131-09 | format=responses on Ollama native | Ignore + debug log (or N/A) | unit factory |
| EC-131-10 | format=responses on Mock | Ignore; deterministic answers | Acc smoke |
| EC-131-11 | Base URL `/v1` vs `/openai/v1` | Document both AWS variants; join `/responses` to configured base | docs + wiremock path assert |
| EC-131-12 | Azure Responses support gaps | P1.1 or loud unsupported if format=responses | unit/azure skip |
| EC-131-13 | Responses `output` starts with reasoning item | Parse message / `output_text` only for content | E2E-131-08 |
| EC-131-14 | Multimodal VLM images | Map to Responses `input_image` parts | unit mapper + optional e2e |
| EC-131-15 | Streaming semantic events | Map `output_text.delta`; ignore unknown event types | E2E-131-05 |
| EC-131-16 | Forgot `store:false` | Hardcode in mapper; wiremock assert | E2E-131-04 |
| EC-131-17 | Bedrock retains data if store true | LAW-131-7; ops callout | E2E-131-04 |
| EC-131-18 | Prompt-cache key on Responses | Forward if accepted; else P1.1 defer documented | AI lens + honest assessment |
| EC-131-19 | Copilot responses-only models | Follow-up: format=responses instead of skip | note in plan; not P0 |
| EC-131-20 | Tools on Responses | v1: chat tools path may stay chat-only; document | honest assessment |
| EC-131-21 | Temperature 400 classified unknown | New permanent class | U-131-05, E2E-131-06 |
| EC-131-22 | Retry storm on permanent param error | `is_permanent: true` | unit permanent |
| EC-131-23 | Concurrent extract workers inherit env | Process env shared — OK | N/A |
| EC-131-24 | Workspace wants different format than fleet | Out of v1 (env fleet-level) | PO non-goal |
| EC-131-25 | `n` parallel completions | Responses removed `n` — EdgeQuake never depended | N/A |
| EC-131-26 | json_schema strict extract under Responses | Map `text.format` json_schema | E2E-131-04 |
| EC-131-27 | Query temperature unset today | Omit-env no-op; still OK | smoke |
| EC-131-28 | Title temp 0.3 with omit | Becomes None | unit title options |
| EC-131-29 | Acc pin omit accidentally globally | Acc scenarios pin explicitly; product default false | Acc review |
| EC-131-30 | Partial provider (OpenAI ok, compat lag) | Shared mapper forced in both | code review + E2E both providers |

## Priority mitigations for P0

Must close before claiming P0 done: **EC-131-01, 02, 03, 21, 22**.

## Priority mitigations for P1

**EC-131-08, 11, 13, 15, 16, 17, 26.**

## Cross-refs

- Tests: [08-test-protocol.md](08-test-protocol.md)
- Honest: [11-honest-assessment.md](11-honest-assessment.md)
