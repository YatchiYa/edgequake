# 01 — First principles (LAW-113)

> **Cross-refs:** [WHY](00-why.md) · [Issue data](00-issue-data.md) · [SPEC-109](../109-configurable-reasoning-effort/) · Ollama [show](https://docs.ollama.com/api-reference/show-model-details) · [capability.go](https://github.com/ollama/ollama/blob/main/types/model/capability.go)

## Laws

### LAW-113-1 — Code is law

Issue prose is a hypothesis. The SSOT for this defect is:

| Symbol | Location |
|--------|----------|
| `OllamaProvider::is_thinking_model` | `edgequake-llm/src/providers/ollama.rs` |
| `OllamaProvider::resolve_think` | same |
| `reasoning_capabilities::capabilities("ollama", …)` | `edgequake-llm/src/reasoning_capabilities.rs` |
| `OllamaDiscovery::fetch_from_api` | `edgequake-llm/src/discovery/providers/ollama.rs` |

If narrative and code disagree, **code wins** until patched. EdgeQuake monorepo consumes the crate; fix lands in `edgequake-llm`, then dep bump.

### LAW-113-2 — Capability is truth; name is not

```text
  model id string  ──►  heuristic contains("qwen3")  ──►  FALSE POSITIVE risk
                              │
                              ✗ forbidden as SSOT for wire `think`

  Ollama capabilities[]  ──►  "thinking" ∈ list  ──►  may send `think`
                              │
                              ✓ SSOT for Auto / clamp eligibility
```

Ollama publishes capabilities on `/api/show` (and, on modern builds, `/api/tags`). That is the authoritative answer to “does this **local** model artifact support the Thinking API?” — not the human-readable tag name. Aliasing proof in #369 is the existence proof of LAW-113-2.

### LAW-113-3 — Asymmetric failure cost (omit ≻ false-positive send)

| Action | Non-thinking model | Thinking model |
|--------|--------------------|----------------|
| **Omit** `think` | Works | Usually works (provider/model default) |
| Send `think: true` | **Hard error** (`does not support thinking`) | Works |
| Send `think: false` / effort `none` | Often works; some variants ignore | Disables or maps per template |

Corollary for **Auto** (`reasoning_effort` unset): when capability is **unknown**, **omit** `think`. Do **not** invent `think: true` from a substring. False-positive send is a P0 product outage; false-negative omit is degraded reasoning, not a brick wall.

### LAW-113-4 — One capability SSOT (DRY)

Discovery already parses `"thinking"` from `/api/tags` into `ModelCapabilities.supports_thinking`. Chat must not maintain a parallel folklore list.

```text
  ┌─────────────────────┐
  │ CapabilityResolver  │  ← single parser for tags/show JSON
  └─────────┬───────────┘
            │
     ┌──────┴──────┐
     ▼             ▼
 Discovery UI   resolve_think / clamp
 (catalog)      (wire Auto path)
```

Forbidden: `is_thinking_model` substring OR-chain as production SSOT. Allowed: temporary **opt-in** legacy heuristic behind an explicit env flag for pre-capabilities Ollama only (see Wave B).

### LAW-113-5 — Explicit user intent wins (SPEC-109)

Config hierarchy from SPEC-109 still applies. If the operator sets `reasoning_effort` to a think-on level:

1. Resolve capability.
2. If **not** thinking-capable → **omit** `think` + warn (never 400 the user by sending unsupported `think`).
3. If thinking-capable → map effort → Ollama `think` wire value (bool / level string).

Auto remains: thinking-capable → product may default `think: true` (today’s intent for true thinking models); non-capable → omit.

### LAW-113-6 — Cache with identity keys; invalidate on model identity change

Capability answers are per `(host, model_name)` (and ideally digest if available). Cache must not leak across hosts or renamed aliases incorrectly:

- `qwen3-vl:8b` and alias `vl-instruct-8b` are **different** cache keys (correct: alias may be probed independently).
- TTL + explicit invalidate on model switch / provider rebuild.

### LAW-113-7 — Fail soft on probe errors

`/api/show` failure (timeout, 404, old server without field, cloud quirks) must not crash chat. Policy:

```text
  probe OK + thinking     → allow Auto think
  probe OK + !thinking    → never send think
  probe FAIL / missing    → omit think  (LAW-113-3)
  env LEGACY_HEURISTIC=1  → optional name fallback (documented escape hatch)
```

### LAW-113-8 — Product surfaces must not lie

If chat omits `think` because capability says no, models catalog / search (`supports_thinking`) must agree. UI “Thinking” toggles (SPEC-109) should disable or hide when capability is false — not show enabled and then 500.

---

## SOLID / DRY application

| Principle | Application |
|-----------|-------------|
| **S** | `CapabilityResolver` owns fetch/cache/parse; `resolve_think` owns effort→wire mapping only |
| **O** | New sources (`/api/tags` bulk warm, `/api/show` precise) extend resolver; do not fork second Auto path in API crate |
| **L** | Thinking-capable and non-capable models remain interchangeable `LLMProvider` callers — no caller-side name checks |
| **I** | Narrow `ThinkingSupport { Unknown, Yes, No }` — do not force chat through full discovery registry |
| **D** | `resolve_think` depends on capability trait/helper, not hardcoded substrings |
| **DRY** | One JSON→`supports_thinking` parse shared with `OllamaDiscovery`; delete duplicate folklore in `reasoning_capabilities` Ollama branch |

## Relationship to SPEC-109

SPEC-109 LAW-R3/R4: clamp in `reasoning_capabilities`; never send illegal effort.  
SPEC-113 adds: for Ollama, **eligibility to send `think` at all** is a **live capability**, not a static name registry. Static registry may still map effort vocabulary for known families **after** capability says Yes.
