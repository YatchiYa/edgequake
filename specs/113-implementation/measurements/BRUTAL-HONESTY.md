# BRUTAL-HONESTY — SPEC-113 (post residual-risk fix 2026-08-08)

> First principles: **code is law**. Claims below are checked against crates.io `0.10.6`, local HEAD, live Ollama, and git state — not intent.

## Verdict (one screen)

```text
  P0 root cause (#369 Auto think from name)  →  FIXED in edgequake-llm 0.10.5+
  Residual llm correctness (Unknown TTL + live env) → FIXED in 0.10.6
  Live proof of causal class                 →  YES (alias + curl + crate tests)
  Product honesty (LAW-113-8)                →  YES (effort UIs gated; fail-soft if unknown)
  OpenAPI SSOT                               →  YES (codegen-openapi-refresh)
  Released llm to origin / tagged            →  YES (crates.io 0.10.6 + tag v0.10.6)
  EdgeQuake pin + UI on shared branch        →  YES after this commit/push
  Product tag v0.24.4                        →  NO (intentionally out of this train)
  Reporter's exact qwen3-vl blob tested      →  NO  (synthetic alias = same mechanism)
```

## Laws scorecard

| Law | Status | Evidence / gap |
|-----|--------|----------------|
| LAW-113-1 Code is law | **Pass** | Default Auto path uses `resolve_think_for_request` → show/tags capabilities; substring only under `legacy_name` |
| LAW-113-2 Capability is truth | **Pass** | Live: `qwen3-fake-vl:test` caps lack thinking → Auto omits; raw `think:true` 400s |
| LAW-113-3 Omit ≻ false send | **Pass** | Unknown/5xx → omit (wiremock T-113-12); live granite/alias omit |
| LAW-113-4 One SSOT (DRY) | **Pass** | Shared `capabilities_include_thinking`; discovery uses it; OpenAPI regenerated from Rust |
| LAW-113-5 Explicit intent | **Pass** | Explicit high + No → omit+warn; none always omit (unit) |
| LAW-113-6 Cache identity | **Pass** | Keyed `(host, model)`; Unknown uses short TTL (5s), Yes/No keep normal TTL |
| LAW-113-7 Fail soft | **Pass** | Probe fail → Unknown → omit; mode read from env at resolve (override only for tests) |
| LAW-113-8 Surfaces must not lie | **Pass** | `ReasoningEffortSelect.thinkingSupported`; query / server / onboarding / workspace / PDF vision gated; `undefined` fails soft |

## What we can claim (with evidence)

| Claim | Evidence |
|-------|----------|
| crates.io has `edgequake-llm` **0.10.6** with Unknown short TTL + live env mode | Published; tag `v0.10.6` |
| EdgeQuake workspace pin is `0.10.6` from registry (no path patch) | `Cargo.toml` + `Cargo.lock` checksum |
| Wiremock: VL caps → outbound chat omits `think` | `e2e113-gates.txt`, T-113-10 |
| Wiremock: thinking caps → Auto sends `think:true` | T-113-11 |
| Live: name has `qwen3`, caps lack thinking → Auto chat works | `qwen3-fake-vl:test` = `ollama cp granite4`; T-113-23 PASS |
| Live: thinking model Auto works | `deepseek-r1:1.5b`, `thinking_tokens=Some(63)` |
| `legacy_name` still bricks the alias | Env contrast FAIL with 400 — proves escape hatch = old bug |
| Catalog + OpenAPI expose `supports_thinking` | utoipa → snapshot → `schema.d.ts` |
| Effort UIs hide ladder when `supports_thinking === false` | vitest + call-site wiring |
| #369 closed with measurement pointers | Issue closed 2026-08-08 |

## What we must **not** claim

| Overclaim | Why not |
|-----------|---------|
| “Product cut v0.24.4 / Acc re-run done” | This train pins llm + merges honesty; **no** `make version-bump` / Acc gate |
| “We tested the reporter’s exact `qwen3-vl-*` weights” | Synthetic alias proves the **name-vs-capability** mechanism; not their blob |
| “Full EdgeQuake HTTP query e2e through Axum” | Tests hit `OllamaProvider` + discovery DTO mapping — **not** `/api/v1/query` with a running API server |
| “Caps Yes can never 400” | EC-13 (stale template) retry/demote **not** implemented; upstream can still reject |
| “CI already gates T-113-* on every PR” | Tests exist; not proven green on GitHub Actions for every matrix cell |

## Residual risks (ordered) — after 0.10.6 train

1. **No product tag v0.24.4** — Partners who only pull Docker/product tags may lag until the next cut; source pin `0.10.6` is enough for git consumers.
2. **EC-13 stale-template 400** — Capability Yes can still hard-fail if upstream template rejects `think`; no demote/retry yet.
3. **Exact reporter weights untested** — Mechanism proven; blob-specific quirks unknown.

Cleared from prior audit: uncommitted pin, LAW-113-8 partial UI, Unknown full-TTL lie, mode frozen at `build()`, OpenAPI hand-patch drift.

## Honesty bar for #369 (re-check)

| Bar | Met? |
|-----|------|
| 1. VL-class omit `think` (wiremock or live) | **Yes** (both) |
| 2. Thinking-class Auto may send `think` | **Yes** (wiremock + live) |
| 3. Alias workaround not required for class | **Yes** (for capability-gated client) |
| 4. Measurements under this folder | **Yes** |

Closing #369 was **justified for the P0 wire bug**. Residual product honesty + llm cache/mode + OpenAPI SSOT + git hygiene for the pin are addressed in this residual-risk train. Remaining honesty gap is **product tag**, not wire correctness.

## First-principles bottom line

The defect was never “Qwen3-VL is broken.” It was **using the wrong truth source** (name ⊂ folklore) for a wire parameter with asymmetric fail cost. The fix restores the correct SSOT (Ollama `capabilities`), ships it as `0.10.6`, and aligns catalog/UI/OpenAPI so surfaces do not offer a think ladder the model cannot honor.
