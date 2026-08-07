# SPEC-109 — Provider Capability Matrix

> **LAW-R3 / LAW-R4**: Clamp in `edgequake-llm::reasoning_capabilities`.  
> Cite vendor docs; when docs and live API disagree, **live API 400 text wins** and the registry must be patched.

## 1. Unified product vocabulary

| Effort | Meaning (product) |
|--------|-------------------|
| `none` | Prefer zero / minimal internal reasoning; maximize output budget |
| `minimal` | Fastest reasoning tier on families that lack `none` |
| `low` | Light reasoning |
| `medium` | Balanced (common provider default) |
| `high` | Deep reasoning |
| `xhigh` | Extended (OpenAI 5.4+ / 5.6 class) |
| `max` | Maximum (OpenAI 5.6 / some Anthropic Opus) |

**Omit** (`None` in Rust) = do not send the field; provider uses its model default.

## 2. Clamp algorithm (normative)

```text
fn clamp(provider, model, desired: Option<&str>) -> Option<String>:
  caps = supported(provider, model)
  if caps is None or empty:
    return None                    # non-reasoning: omit always
  if desired is None:
    return None                    # Auto / omit
  if desired in caps:
    return Some(desired)
  # map common aliases then nearest-lower in ordered scale:
  # none < minimal < low < medium < high < xhigh < max
  return Some(nearest_lower(desired, caps))
    # special case: desired=none, caps has minimal but not none → "minimal"
    # if desired below all caps → caps.lowest
```

`lowest_for_structured_output(model)` = first of `none`, then `minimal`, then `low` present in `caps`.

## 3. OpenAI (native Chat Completions)

**Wire (v1):** top-level `reasoning_effort` on Chat Completions  
([Reasoning guide](https://developers.openai.com/api/docs/guides/reasoning) — Responses uses nested `reasoning.effort`; EdgeQuake v1 stays on Chat Completions because `OpenAIProvider` uses `async-openai` chat).

**Guide values (model-dependent):** `none` · `minimal` · `low` · `medium` · `high` · `xhigh` · `max`.

| Model / family | Supported (registry pin) | Default (docs) | Structured floor | Citation |
|----------------|--------------------------|----------------|------------------|----------|
| `gpt-5` (base 2025-08) | `minimal`, `low`, `medium`, `high` | medium-ish | `minimal` | [gpt-5 model page](https://developers.openai.com/api/docs/models/gpt-5): “minimal, low, medium, and high” |
| `gpt-5-mini` | `minimal`, `low`, `medium`, `high` | medium (omit) | **`minimal`** (`none` **illegal**) | API 400 class ([OpenClaw #62967](https://github.com/openclaw/openclaw/issues/62967)); community matrix; model page confirms reasoning tokens |
| `gpt-5-nano` | `minimal`, `low`, `medium`, `high` | medium (omit) | `minimal` | Same generation as mini (pin; verify on bump) |
| `gpt-5.1` | `none`, `low`, `medium`, `high` | model-dependent | `none` | Community + guide progression (no `minimal` on 5.1+) |
| `gpt-5.4-mini` | `none`, `low`, `medium`, `high`, `xhigh` | **`none`** | `none` | [gpt-5.4-mini](https://developers.openai.com/api/docs/models/gpt-5.4-mini): “none (default), low, medium, high and xhigh” |
| `gpt-5.4-nano` | `none`, `low`, `medium`, `high`, `xhigh` | **`none`** | `none` | [gpt-5.4-nano](https://developers.openai.com/api/docs/models/gpt-5.4-nano) |
| `gpt-5.4` / `gpt-5.5` | include `none` … `xhigh` (confirm on model page at impl) | often `medium` for 5.5 | `none` if listed | [Reasoning guide](https://developers.openai.com/api/docs/guides/reasoning) |
| `gpt-5.6` (+ terra/luna/sol) | `none`, `low`, `medium`, `high`, `xhigh`, `max` | `medium` if omit | `none` | [Model guidance](https://developers.openai.com/api/docs/guides/latest-model) |
| `o1` / `o3` / `o4*` | typically `low`, `medium`, `high` (no `none` on early o-series) | medium | `low` | Historical o-series; registry must pin per slug |
| Non-reasoning (`gpt-4.1*`, `gpt-4o*`) | ∅ | — | omit | Do not send field |

### Critical acceptance

| Case | Desired | Effective send |
|------|---------|----------------|
| `gpt-5-mini` + structured default | `none` | **`minimal`** |
| `gpt-5.4-nano` + structured default | `none` | **`none`** |
| `gpt-4.1-mini` + any | `low` | **omit** |

**Implementation debt:** `edgequake-llm` `providers/openai.rs` must call `request_builder.reasoning_effort(...)` (async-openai 0.34 already has the field). **E2E-109-01**, **E2E-109-02**.

## 4. OpenAI-compatible gateway

**Wire:** JSON field `reasoning_effort` (already in `openai_compatible.rs`).  
**Clamp:** Use OpenAI registry when `provider=openai` or model id matches `gpt-*` / `o*`; otherwise passthrough with warn, or gateway-specific row if known.

## 5. Mistral

| Model | Behavior | Wire |
|-------|----------|------|
| `mistral-small*` (adjustable) | Accepts reasoning_effort; practical docs emphasize `high` vs `none` | Top-level `reasoning_effort` |
| `mistral-medium-3*` / `medium-3-5` | Adjustable | Same |
| `mistral-large*` / Magistra / Codestral | **Reject** (API 3051) | **Omit always** |

Docs: [Mistral Reasoning](https://docs.mistral.ai/capabilities/reasoning).  
Registry: keep SPEC-047 rule — Large never sends. For Small/Medium, map product scale: unsupported mid-tiers → `high` if “on”, `none` if floor (document mapping in registry comments).

## 6. Anthropic

| Item | Rule |
|------|------|
| Wire | Map `reasoning_effort` → `output_config.effort` (Claude 4.6+ stable; Opus 4.5 may need beta header — follow existing `anthropic.rs`) |
| Typical values | `low`, `medium`, `high`; `max` on select Opus |
| Product `none` / `minimal` | Map to `low` (nearest) |
| Docs | [Effort](https://platform.claude.com/docs/en/build-with-claude/effort) |

Audit Wave 0: ensure mapping already present; add clamp to avoid illegal `max` on Sonnet.

## 7. Ollama

| Item | Rule |
|------|------|
| Wire | Existing map `reasoning_effort` → `think` levels (`high`/`medium`/`low`/`max` …) in `ollama.rs` |
| Unsupported | Omit `think` |
| Clamp | Map product vocabulary onto Ollama’s think enum; unknown → omit |

## 8. xAI

| Item | Rule |
|------|------|
| Wire | `reasoning.effort` / preserved `reasoning_effort` per `xai.rs` (grok-4.3+ first-class) |
| Values | Commonly `none` / `low` / `medium` / `high` |
| Older grok-4 | May reject — filter path already exists; keep + clamp |

## 9. NVIDIA (OpenAI-compatible)

| Item | Rule |
|------|------|
| Wire | Top-level `reasoning_effort` (DeepSeek / Nemotron thinking) |
| Values | Provider-specific; often `low`/`medium`/`high`/`max` |
| Docs | `edgequake-llm/docs/providers/nvidia/` |

## 10. LM Studio

| Item | Rule |
|------|------|
| Wire | `reasoning_effort` on `/v1/chat/completions`; `none` strips thinking fields |
| Values | `low` / `medium` / `high` / `none` |

## 11. Registry API shape (Rust)

```rust
pub struct ReasoningCapabilities {
    pub supported: &'static [&'static str],
    pub default_when_omitted: Option<&'static str>, // informational
}

pub fn capabilities(provider: &str, model: &str) -> Option<ReasoningCapabilities>;
pub fn clamp_reasoning_effort(provider: &str, model: &str, desired: Option<&str>) -> Option<String>;
pub fn lowest_for_structured_output(provider: &str, model: &str) -> Option<String>;
```

Expose the same data through EdgeQuake models catalog for UI (see [02](02-use-cases-and-surfaces.md) §4).

## 12. Maintenance rule

On every `edgequake-llm` release that adds a GPT-5.x / Claude / Mistral slug:

1. Update this matrix row + registry unit tests.  
2. Run `E2E-109-02`-class clamp fixtures.  
3. Note doc URL + date in CHANGELOG.
