# SPEC-113 — Ollama thinking capability gate (Issue #369)

> **Trigger:** [#369](https://github.com/raphaelmansuy/edgequake/issues/369) — `OllamaProvider::is_thinking_model()` substring match (`qwen3`, …) auto-sends `think: true`, breaking non-thinking variants (e.g. many `qwen3-vl-*`).  
> **Method:** First principles — **code is law** — capability from Ollama `/api/show`|/api/tags is SSOT; name is not.  
> **Audience:** Engineering (fix train in `edgequake-llm`) + operators (runbook now).  
> **Ship vehicle:** `edgequake-llm` patch release → EdgeQuake workspace dep bump (post Waves A–F).

## One-screen verdict

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│  INCIDENT — False-positive Ollama `think` injection                          │
│    Symptom: chat/query fails: "does not support thinking"                    │
│    Repro proof: ollama cp qwen3-vl:* → alias without "qwen3" fixes instantly │
│                                                                              │
│  CODE FACTS (edgequake-llm 0.10.4 / local HEAD)                              │
│    resolve_think Auto: if is_thinking_model(name) → think:true               │
│    is_thinking_model: contains("qwen3"|deepseek-r1|qwq|…)                    │
│    Discovery ALREADY parses capabilities[].thinking from /api/tags           │
│    reasoning_capabilities Ollama branch also name-matches "qwen"             │
│                                                                              │
│  FIRST PRINCIPLE                                                             │
│    Capability is truth. Unknown → omit think (asymmetric fail cost).         │
│                                                                              │
│  FIX TRAIN — Waves A–F (04-fix-plan)                                         │
│    A gate Auto  B cache/escape  C DRY  D product honesty  E tests  F release │
└──────────────────────────────────────────────────────────────────────────────┘
```

## Status board

| Thread | Severity | Present on 0.10.4? | Fix needed? |
|--------|----------|--------------------|-------------|
| Auto `think:true` from name substring | P0 | **Yes** | Capability gate |
| Discovery vs chat capability split brain | P0 | **Yes** | DRY resolver |
| `reasoning_capabilities` Ollama `contains("qwen")` | P1 | **Yes** | Remove/gate |
| Catalog `supports_thinking` unused by chat Auto | P1 | **Yes** | Wire / align |
| Ops workaround documented | — | **This pack** | Runbook |

## Document map

```ascii
 00-why / 00-issue-data
   → 01-first-principles (LAW-113-*)
   → 02-cross-ref-matrix
   → 03-root-cause
   → 04-fix-plan (DRY / SOLID waves A–F)
   → 05-e2e-test-matrix
   → 06-edge-cases
   → 07-ops-runbook
   → measurements/ (BRUTAL-HONESTY + future gates)
   → lenses/ (PO, fullstack, DB, UX/UI, front, marketing)
```

## Locked decisions

| Decision | Choice |
|----------|--------|
| Truth source | Ollama `capabilities` (`thinking`); code in `ollama.rs` + discovery |
| Auto when Unknown | **Omit** `think` (LAW-113-3) |
| Name heuristic | Not default SSOT; optional `legacy_name` escape only |
| Fix home | `edgequake-llm` first; EdgeQuake bumps dep |
| VL special-cases | **Forbidden** as primary fix (`-vl` suffix lists rot) |
| SPEC-109 | Keep effort hierarchy; eligibility gated by capability |
| Docs vs code | Waves A–F implemented in `edgequake-llm` 0.10.5 + EdgeQuake pin (2026-08-08) |

## Start here

1. [00-why.md](00-why.md)  
2. [00-issue-data.md](00-issue-data.md) + [measurements/BRUTAL-HONESTY.md](measurements/BRUTAL-HONESTY.md)  
3. [01-first-principles.md](01-first-principles.md)  
4. [03-root-cause.md](03-root-cause.md)  
5. [04-fix-plan.md](04-fix-plan.md)  
6. [07-ops-runbook.md](07-ops-runbook.md) (partner can act now)  
7. Lenses: [lenses/README.md](lenses/README.md)

## Cross-spec anchors

| Spec | Relevance |
|------|-----------|
| [SPEC-109](../109-configurable-reasoning-effort/) | `reasoning_effort`, clamp module, Auto UX |
| [SPEC-109 matrix § Ollama](../109-configurable-reasoning-effort/03-provider-capability-matrix.md) | Wire `think`; extend with live caps |
| [SPEC-112](../112-connection-pool/) | Pack structure / honesty pattern |
| edgequake-llm discovery | “No Heuristics” — chat must match |

## Implementation status

Waves A–F landed: capability-gated Auto in `edgequake-llm` 0.10.5; EdgeQuake catalog/UI honesty; wiremock gates under `measurements/`.

## Out of scope

- Patching Ollama server templates for VL think toggles  
- Renaming upstream model library tags
