# 03 — Root cause

> **Verdict:** Production chat Auto-path **injects `think: true` from model-name substrings**, while discovery already knows real capabilities. False-positive on any id containing `qwen3` (including many VL variants) → Ollama hard-fails the request.

## Causal chain

```text
  Operator selects Ollama model "qwen3-vl:…"
        │
        ▼
  CompletionOptions.reasoning_effort = None   (Auto / SPEC-109 query default)
        │
        ▼
  OllamaProvider::resolve_think(model, opts)
        │
        ├─ clamp_reasoning_effort("ollama", model, None) → None
        │     (also: static registry may name-match "qwen" — secondary smell)
        │
        ▼
  Auto branch:
    desired.is_none() && is_thinking_model(model)
        │
        ▼
  is_thinking_model: model_lower.contains("qwen3") == true
        │
        ▼
  ChatRequest { think: Some(Bool(true)), … }  ──POST──►  /api/chat
        │
        ▼
  Ollama: capabilities do NOT include "thinking"
        │
        ▼
  HTTP error: "… does not support thinking"
        │
        ▼
  EdgeQuake surfaces failure (query / chat / pipeline)
```

## Why the alias workaround “fixes” it

```text
  ollama cp qwen3-vl:8b  vl-instruct-8b

  weights ──────────── identical
  capabilities ─────── identical
  is_thinking_model ── "vl-instruct-8b".contains("qwen3") == false
                       → Auto omits think → chat succeeds
```

This is not a model fix. It is proof the **predicate is wrong**.

## Irony: discovery already correct

```text
  /api/tags  →  capabilities: ["completion","vision",…]
                    │
                    ▼
  OllamaDiscovery sets supports_thinking = caps.contains("thinking")

  UI / models search can show thinking=false
  while chat Auto still forces think=true from the name.
```

Two brains, one product. LAW-113-4 demands one.

## Secondary defect — static Ollama registry

In `reasoning_capabilities.rs`:

```rust
if p.contains("ollama") {
    if m.contains("deepseek") || m.contains("qwen") || m.contains("r1") || m.contains("think") {
        return Some(ReasoningCapabilities { supported: &["low","medium","high","max"], … });
    }
}
```

`contains("qwen")` is **broader** than `qwen3` and also name-folklore. Even when Auto is fixed, explicit effort + clamp may still treat VL ids as reasoning-capable. Wave A must address **both** entry points.

## Not the root cause

| Hypothesis | Why rejected |
|------------|--------------|
| Ollama bug for this alias case | Alias with same weights works |
| EdgeQuake API forgot to pass effort | Failure is Auto with effort **unset** |
| Only `-vl` suffix is special | Any non-thinking id containing `qwen3` / `deepseek-r1` / … false-positives |
| Discovery broken | Discovery path already capability-based |

## Blast radius

| Surface | How hit |
|---------|---------|
| Query / chat (Ollama) | Auto `think: true` |
| Entity extract / summary / keyword | Role defaults may set effort; clamp + Auto interactions |
| VLM / PDF vision | High chance of `qwen3-vl*` model ids |
| Streaming | Same `resolve_think` before stream/non-stream chat |

## Fix direction (preview)

Replace name SSOT with `CapabilityResolver` (`/api/show` precise, `/api/tags` warm), Auto omit unless `thinking` present — see [`04-fix-plan.md`](04-fix-plan.md).
