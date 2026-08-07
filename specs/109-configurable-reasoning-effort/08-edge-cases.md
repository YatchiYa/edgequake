# SPEC-109 — Edge Cases

| ID | Case | Expected behavior |
|----|------|-------------------|
| **EC-01** | Model does not support reasoning | Omit field; UI shows N/A; desired config retained for when model changes |
| **EC-02** | Desired `none` on `gpt-5-mini` | Clamp to `minimal`; `clamped=true` in effective config |
| **EC-03** | Desired `xhigh` on model without `xhigh` | Nearest-lower (e.g. `high`); never 400 |
| **EC-04** | Desired `max` on Anthropic Sonnet | Clamp away from `max` if unsupported |
| **EC-05** | Empty string / whitespace env | Treat as unset |
| **EC-06** | Unknown effort string `"ultra"` | Warn; treat as unset or clamp to medium-if-present — **prefer unset** (do not guess up) |
| **EC-07** | Temperature + reasoning models | Keep existing temperature gating (`effective_temperature_for_model`); effort orthogonal |
| **EC-08** | Streaming query override | Stream request carries same `reasoning_effort` as non-stream |
| **EC-09** | Keyword role unset | Inherit compiled lowest for keyword model; do not inherit query Auto omit |
| **EC-10** | Chat completions path | Same policy as query role |
| **EC-11** | SPEC-103 cache | Hash must include effective effort when `Some`; changing effort busts cache |
| **EC-12** | Acc / cold bench | Pin `EDGEQUAKE_EXTRACT_REASONING_EFFORT` / VLM to floor; document in Acc runbooks |
| **EC-13** | Provider switch mid-workspace | Re-clamp on each call for **current** model; stored desired string may clamp differently |
| **EC-14** | OpenAI-compatible proxy strips unknown fields | Still send; if proxy drops, F1-class defect on proxy — not EQ |
| **EC-15** | Concurrent roles different efforts | Extract `minimal` and query `high` in same process — no global static |
| **EC-16** | Migration: existing workspaces | Missing metadata → compiled defaults; no forced DB migration required if metadata-only |
| **EC-17** | ServerFirst vs EnvFirst | Effort fields obey existing `EDGEQUAKE_CONFIG_PRIORITY` merge helper |
| **EC-18** | Vision without LLM (pure pdfium) | No effort control; hide UI |
| **EC-19** | Gleaning multi-pass | Same extract effort for all gleaning calls unless future per-pass config |
| **EC-20** | Responses API / pro mode | Out of scope v1 — do not partially implement |

## Failure modes to avoid

1. Fix OpenAI forward **without** clamp → `gpt-5-mini` production 400s.  
2. UI hardcodes effort enum without catalog → illegal options.  
3. Global process-wide effort static → role crosstalk.  
4. Silent drop of request override when workspace set.
