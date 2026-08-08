# 06 — Edge cases

| ID | Case | Risk | Mitigation | Test |
|----|------|------|------------|------|
| EC-01 | Model id contains `qwen3` but caps lack `thinking` (#369) | Hard fail every request | Capability gate; Auto omit | T-113-06, 10, 24 |
| EC-02 | Alias without family substring but caps include `thinking` | Auto would under-think if name-only | Capability Yes → allow Auto think | T-113-07 |
| EC-03 | `/api/show` timeout / 5xx | Stall or panic | Timeout → Unknown → omit | T-113-12 |
| EC-04 | Old Ollama without `capabilities` field | Empty/missing → Unknown | Omit; optional `legacy_name` | T-113-05, 15 |
| EC-05 | `/api/tags` has caps but show disagrees | Wrong think decision | Prefer show for active model; log conflict | Wave B docs |
| EC-06 | Explicit `reasoning_effort=high` on non-thinking model | Client sends think → 400 | Map to omit + warn | T-113-09 |
| EC-07 | Explicit `none` / `minimal` on thinking model | Still inject true from Auto bug class | Explicit none always omits | T-113-08 |
| EC-08 | Model switched at runtime same provider instance | Stale Yes cache → false send | Key by model; invalidate on `set_model` / rebuild | T-113-13/14 |
| EC-09 | Multi-host (local + cloud) | Cross-host cache poison | Key includes host | T-113-14 |
| EC-10 | Ollama Cloud auth on `/api/show` | 401 → Unknown | Forward API key; omit on fail | M + docs |
| EC-11 | Streaming vs non-stream | Gate only on one path | Both call same pre-request resolve | T-113-22 |
| EC-12 | Embedding model id mistakenly used for chat | Weird errors | Out of scope; still must not send think from name | — |
| EC-13 | Caps say thinking but template stale (upstream) | Ollama may still error | Catch “does not support thinking”; once demote cache to No; retry omit (optional Wave B+) | Edge harden |
| EC-14 | Caps omit thinking but model emits `<think>` in content | UX confusion | Do not send `think` param; parsing of content tags unchanged | Existing parse tests |
| EC-15 | `force_on` escape hatch | Operator bricks VL | Document debug-only; default `auto` | Ops |
| EC-16 | Concurrent first-request stampede | N show calls | Singleflight / mutex per model key | Wave B |
| EC-17 | `reasoning_capabilities` still name-matches `qwen` | Clamp thinks levels exist | Wave A4 cleanup | T-113-18 |
| EC-18 | UI shows thinking supported, chat omits | Trust break | LAW-113-8; catalog from same caps | T-113-19 |
| EC-19 | VL instruct vs thinking variants (upstream split) | Wrong folklore guidance | Docs: trust `ollama show` capabilities | Ops + lenses |
| EC-20 | Case / tag variants (`QWEN3-VL:Latest`) | Heuristic casefold already; caps use exact model pull name | Probe with exact configured model string | U |
| EC-21 | Empty model name | Probe nonsense | Config error before chat | Existing builder tests |
| EC-22 | SPEC-109 structured extract + Ollama VL | Effort `none` + false Auto historically | Explicit none + No support both omit | Pipeline contract |

## ASCII — decision under uncertainty

```text
                 ┌──────────────┐
                 │ Probe result │
                 └──────┬───────┘
            Yes / No / Unknown / Error
                 │
     ┌───────────┼───────────┬────────────┐
     ▼           ▼           ▼            ▼
  Auto+Yes    Auto+No    Auto+Unk      Error
  think:true   omit       omit         omit
                                      (log)
```

## ASCII — #369 repro vs fix

```text
  BEFORE (0.10.4)
  name=qwen3-vl:* ──contains qwen3──► think:true ──► Ollama 400

  AFTER (SPEC-113)
  name=qwen3-vl:* ──show caps──► no "thinking" ──► omit ──► Ollama 200
```
