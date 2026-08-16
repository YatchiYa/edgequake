# 09 — Acceptance

> Maps [#379](https://github.com/raphaelmansuy/edgequake/issues/379) AC → SPEC-131 proofs.

| #379 AC | SPEC proof | Status (spec) |
|---------|------------|---------------|
| `OMIT_TEMPERATURE=true` prevents temperature; ingest succeeds Gemma/Grok Mantle | E2E-131-01 + LIVE-131-A; U-131-01 | Specified |
| `API_FORMAT=responses` routes to `/v1/responses` | E2E-131-04 | Specified |
| Extraction + query equivalent structured JSON under Responses | E2E-131-04/05/08 + LIVE-131-B | Specified |
| ChatGPT 5.6-series works on Bedrock Responses mode | LIVE-131-B | Specified (gated) |
| Configuration documented in setup guide | WP-9 `.env.example` + AGENTS + setup | Specified |

## Additional SPEC acceptance (beyond issue)

| ID | Criterion |
|----|-----------|
| A-131-01 | Env omit supersedes model gate (LAW-131-2) |
| A-131-02 | Heuristic gate still omits gpt-5 / o* when env unset (LAW-131-11) |
| A-131-03 | VLM figure-filter no longer bypasses resolver (LAW-131-3) |
| A-131-04 | Wire strip defense when options still `Some` (LAW-131-4) |
| A-131-05 | `store:false` always on Responses (LAW-131-7) |
| A-131-06 | `llm_unsupported_param` permanent + actionable (LAW-131-8) |
| A-131-07 | Default `API_FORMAT=chat_completions` (LAW-131-5) |
| A-131-08 | Product `/api/v1/chat/completions` facade unchanged (LAW-131-9) |
| A-131-09 | Invalid format fails loud at boot/factory |
| A-131-10 | `OMIT_REASONING_EFFORT` omits effort field |

## Definition of Done (implementation)

1. P0 merged: omit + classifier + call sites + E2E-131-01/02/03/06/07  
2. P1 merged: Responses + E2E-131-04/05/08  
3. P2: docs updated; #379 comment links this pack  
4. Honest assessment updated when LIVE-131 run

## Cross-refs

- Tests: [08-test-protocol.md](08-test-protocol.md)
- Edges: [10-edge-cases.md](10-edge-cases.md)
- Intake: [zz-raw.md](zz-raw.md)
