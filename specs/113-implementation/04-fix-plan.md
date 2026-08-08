# 04 — Fix plan (DRY / SOLID)

> **Status:** Implemented (edgequake-llm 0.10.5 + EdgeQuake pin, 2026-08-08).  
> **Laws:** [01-first-principles.md](01-first-principles.md) · **Tests:** [05-e2e-test-matrix.md](05-e2e-test-matrix.md) · **Edges:** [06-edge-cases.md](06-edge-cases.md)  
> **Home repo:** `edgequake-llm` → then bump EdgeQuake workspace pin.

## Wave overview

```text
  Wave A (P0)  Capability-gated resolve_think (stop the bleeding)
       │
       ▼
  Wave B (P0)  Cache + probe UX + legacy escape hatch
       │
       ▼
  Wave C (P1)  DRY with discovery + reasoning_capabilities cleanup
       │
       ▼
  Wave D (P1)  Product honesty (catalog / UI / docs)
       │
       ▼
  Wave E      Gates (ship with A–D; do not defer)
       │
       ▼
  Wave F      Release: edgequake-llm crate → EdgeQuake dep bump
```

---

## Wave A — Capability-gated Auto (P0)

### A1 — Introduce `ThinkingSupport` + resolver API (SOLID-S/I)

**Where:** new module e.g. `edgequake-llm/src/providers/ollama_capabilities.rs` (or `ollama/capabilities.rs`).

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThinkingSupport {
    Yes,
    No,
    Unknown,
}

#[async_trait]
pub trait OllamaThinkingCapability: Send + Sync {
    async fn thinking_support(&self, model: &str) -> ThinkingSupport;
}
```

**Parse rule (DRY with discovery):** `"thinking"` ∈ `capabilities` array (case-sensitive per Ollama constant) ⇒ `Yes`; array present without it ⇒ `No`; missing field / transport error ⇒ `Unknown`.

### A2 — Prefer `/api/show` for the active model (LAW-113-2)

```text
  POST {host}/api/show  { "model": "<id>" }
        │
        ▼
  JSON.capabilities  →  ThinkingSupport
```

Docs: [Show model details](https://docs.ollama.com/api-reference/show-model-details).

### A3 — Rewrite Auto branch of `resolve_think` (LAW-113-3/5)

**Problem today:** `resolve_think` is **sync** and calls `is_thinking_model`.

**Do:**

1. Ensure capability is resolved **before** building `ChatRequest` (async path in `complete` / `chat` / stream).
2. Pass `ThinkingSupport` into a pure sync mapper:

```text
  fn map_think(effort, support) -> Option<Value>
    explicit none/off     → None (omit)
    explicit on/level     → if support == No { None + warn } else { wire value }
                            if support == Unknown { None + warn }  // fail soft
    Auto (effort unset)   → if support == Yes { Some(true) } else { None }
```

3. Delete production use of substring `is_thinking_model` (keep behind `legacy_name` only if Wave B ships escape hatch).

### A4 — Stop treating name-matched `qwen*` as registry-capable (LAW-113-4)

In `reasoning_capabilities.rs` Ollama branch: **do not** return `Some(supported levels)` solely from `m.contains("qwen")`. Options (pick one in impl; prefer first):

- **Preferred:** Ollama static registry returns `None` always; live capability + effort vocabulary table keyed only after `ThinkingSupport::Yes`.
- **Acceptable interim:** keep effort vocabulary for known thinking families but **never** allow Auto send without live `Yes`.

---

## Wave B — Cache, performance, escape hatch (P0/P1)

### B1 — TTL cache keyed by `(host, model)` (LAW-113-6)

Avoid `/api/show` on every token request. Warm on provider build / first chat / `list_models`.

### B2 — Bulk warm from `/api/tags` (optional fast path)

Modern Ollama may include `capabilities` on tags (already used by discovery). Use tags to seed cache; show remains source of truth on miss / conflict.

### B3 — Env escape hatches (LAW-113-7)

| Mode | Behavior |
|------|----------|
| `auto` (default) | Capability probe |
| `force_off` | Never send `think` |
| `force_on` | Always send `think: true` (debug only; expect breakage) |
| `legacy_name` | Old substring heuristic |

Log once per process when non-`auto`.

### B4 — Timeouts

Probe must not stall chat forever (`EDGEQUAKE_OLLAMA_CAPABILITY_TIMEOUT_MS`). On timeout → `Unknown` → omit.

---

## Wave C — DRY / SOLID cleanup (P1)

### C1 — Shared capability parse helper

Extract from discovery + resolver:

```rust
pub fn capabilities_include_thinking(caps: &[impl AsRef<str>]) -> bool
```

### C2 — Remove or quarantine `is_thinking_model`

- Unit tests that assert `qwen3-vl` **is** thinking via name → rewrite to capability fixtures.
- Mark `#[cfg]` / `legacy_name` only.

### C3 — Single warn metric / trace field

`ollama.think_decision{support,effort,sent}` for ops (no PII).

---

## Wave D — Product honesty (P1)

### D1 — Models catalog

Ensure `supports_thinking` from discovery stays authoritative; after llm bump, add contract that chat Auto will not send `think` when discovery says false (integration with wiremock).

### D2 — UX (SPEC-109 surfaces)

When selected Ollama model has `supports_thinking=false`, disable Thinking / effort controls or show “not supported by this model” (see lenses).

### D3 — Docs

Update `docs/operations/configuration.md` + edgequake-llm Ollama provider docs: capability-gated think; remove “qwen3 always thinks” folklore.

---

## Wave E — Tests (mandatory)

See [05-e2e-test-matrix.md](05-e2e-test-matrix.md). No merge without green gates for A–C at minimum.

---

## Wave F — Release train

```text
  1. Land Waves A–E on edgequake-llm
  2. cargo publish edgequake-llm X.Y.Z  (or git path pin for monorepo CI)
  3. Bump edgequake workspace edgequake-llm
  4. Run EdgeQuake: cargo test -p edgequake-api --lib + targeted e2e
  5. Close #369 with before/after curl proof in measurements/
```

---

## SOLID / DRY checklist for implementers

| Check | Pass criterion |
|-------|----------------|
| One capability SSOT | Chat Auto and discovery share parse + prefer live API |
| No substring SSOT | `contains("qwen3")` not on default path |
| Async boundary clear | Probe async; map pure sync |
| Fail soft | Unknown → omit `think`; chat still works |
| Explicit intent | Effort set + No capability → omit, not 400 |
| Escape hatch | `legacy_name` documented, not default |
| Dep bump | EdgeQuake pin moves only after llm gates green |

## Non-goals for first code train

- Fixing Ollama VL template / `think:false` ignored quirks (upstream).
- Rewriting SPEC-109 config hierarchy.
- Embedding a static list of all VL model ids.
