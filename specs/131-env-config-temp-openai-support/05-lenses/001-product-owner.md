# Lens 001 — Product Owner

## Stake

Operators on Bedrock Mantle (and similar OpenAI-compatible gateways) must run production ingest without waiting for EdgeQuake model-list patches, and must unlock Responses-only models (GPT-5.6) without a second product.

## Jobs to be done

| Persona | Job |
|---------|-----|
| Platform operator | Point `OPENAI_BASE_URL` at Mantle; set omit/format env; ingest succeeds |
| Acc / eval owner | Pin format + omit in Acc env without code forks |
| Support | Read `failure_class=llm_unsupported_param` and know the fix knob |
| End user (Documents UI) | See Failed with actionable reason, not opaque “unknown” |

## Outcomes (ship order)

| Phase | Outcome | Business value |
|-------|---------|----------------|
| **P0** | Omit temperature/effort via env; classify unsupported params | Unblocks Gemma 4 / Grok 4.3 today; stops 121-doc batch burns |
| **P1** | `API_FORMAT=responses` for OpenAI + openai_compatible | Unlocks GPT-5.6 Mantle; aligns with OpenAI direction |
| **P2** | Docs + Acc pins | Operator self-serve; no tribal knowledge |

## Acceptance (product language)

1. With omit-temp=true, Mantle Gemma/Grok extract does not 400 on temperature.
2. With format=responses, GPT-5.6-class model completes extract/query JSON.
3. Chat Completions remains default — no surprise format flip.
4. EdgeQuake-as-provider chat API unchanged.
5. Docs list the three env vars with Mantle examples.

## Explicit non-goals

- Auto-detecting every Responses-only model without operator config
- Rewriting the WebUI provider wizard in P0
- Hosted web_search / MCP agent loops in v1

## Risks / mitigations

| Risk | Mitigation |
|------|------------|
| Omit-temp lowers creativity/control | Document tradeoff; default remains send when legal |
| `store:true` accidental retention on Bedrock | LAW-131-7 force `store:false` |
| Acc regressions | Default format chat; Acc only pins when testing Mantle |

## Cross-refs

- WHY: [../00-why.md](../00-why.md)
- Acceptance: [../09-acceptance.md](../09-acceptance.md)
- UX: [004-ux-ui.md](004-ux-ui.md)
