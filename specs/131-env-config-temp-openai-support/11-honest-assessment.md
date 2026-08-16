# 11 — Honest assessment

## What we know for certain (code)

1. Temperature omit is a **substring allowlist**, not an operator policy — Gemma/Grok fail analytically.
2. OpenAI provider only skips temperature when ≈1.0; it will send `0.0` if options say so.
3. VLM figure-filter can bypass the pipeline gate with hardcoded `Some(0.0)`.
4. There is **no** `/v1/responses` client; Copilot skips Responses-only models.
5. Temperature `unsupported_value` classifies as **`unknown`** today.

## What we know from external docs (Aug 2026)

1. OpenAI recommends Responses for new projects; Chat Completions remains supported.
2. Bedrock Mantle serves OpenAI-compatible Responses; GPT-5.6 family is Responses-oriented on Mantle.
3. Mantle/`store` defaults to **true** with multi-day retention — EdgeQuake must opt out.
4. Many GPT-5.x / reasoning models reject temperature overrides; effort is the control plane (SPEC-109 kinship).

## What we have not live-proven in this spec pack

- Actual Mantle HTTP against Gemma 4 / Grok 4.3 / GPT-5.6 from this workspace (no AWS credentials assumed).
- Exact base path string every region/account uses (`/v1` vs `/openai/v1`).
- Whether every openai_compatible gateway implements Open Responses identically.

## Confidence

| Claim | Confidence |
|-------|------------|
| P0 omit-env fixes #379 temperature axis | **High** (code path clear) |
| Classifier improvement reduces operator confusion | **High** |
| P1 Responses unlocks GPT-5.6 Mantle | **Medium-High** (docs strong; live gated) |
| Zero regressions on default chat path | **High** if format default preserved + wiremock |
| Prompt-cache parity on Responses day-1 | **Medium** — may defer P1.1 |

## Residual risks

- Shared mapper bugs on multimodal / tools.
- Acc accidentally pinning omit/format.
- Operators forgetting restart after env change.

## Verdict

SPEC-131 is the right product response to #379: **policy over catalogs** for parameters, **config over hard-bind** for transport, **honest failure_class** for triage. Implementation should ship **P0 first** (unblocks current Mantle Chat Completions failures) then **P1 Responses**.

## Cross-refs

- WHY: [00-why.md](00-why.md)
- Acceptance: [09-acceptance.md](09-acceptance.md)
- Edges: [10-edge-cases.md](10-edge-cases.md)
