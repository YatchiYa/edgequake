# 02 — Cross-ref matrix

## Spec ↔ law ↔ code

| LAW | Spec anchor | Code symbol / file | Env / API | Tests (Wave E) |
|-----|-------------|--------------------|-----------|----------------|
| 113-1 | This pack | `ollama.rs` `resolve_think` / `is_thinking_model` | — | Path contracts |
| 113-2 | [#369](https://github.com/raphaelmansuy/edgequake/issues/369) | `/api/show` + `/api/tags` `capabilities` | Ollama HTTP | Wiremock show/tags |
| 113-3 | Auto semantics | `resolve_think` Auto branch | — | Omit when Unknown/No |
| 113-4 | Discovery DRY | `discovery/providers/ollama.rs` + new resolver | — | Shared parse unit |
| 113-5 | [SPEC-109](../109-configurable-reasoning-effort/) | `CompletionOptions.reasoning_effort` | role env / UI | Explicit effort + No → omit |
| 113-6 | Cache | provider-local cache map | — | Host/model isolation |
| 113-7 | Probe fail | resolver timeout / empty caps | optional legacy env | Omit + no panic |
| 113-8 | Product UI | `handlers/models_search.rs` `supports_thinking` | `/api/v1/models*` | Catalog matches chat |

## File map (code is law)

### edgequake-llm (fix home)

| Path | Role today | SPEC-113 change |
|------|------------|-----------------|
| [`src/providers/ollama.rs`](../../../edgequake-llm/src/providers/ollama.rs) | `is_thinking_model` substring; Auto → `think: true` | Capability-gated Auto; deprecate substring as SSOT |
| [`src/reasoning_capabilities.rs`](../../../edgequake-llm/src/reasoning_capabilities.rs) | Ollama branch: `m.contains("qwen")` etc. | Gate static effort map on live capability **or** remove Ollama name match; Unknown → `None` |
| [`src/discovery/providers/ollama.rs`](../../../edgequake-llm/src/discovery/providers/ollama.rs) | Already parses `thinking` from `/api/tags` | Extract shared `parse_ollama_capabilities(&[String])`; warm resolver cache |
| **New** `src/providers/ollama_capabilities.rs` (proposed) | — | `ThinkingSupport` + show/tags fetch + TTL cache |
| Tests in `ollama.rs` | Assert `qwen3:*` **is** thinking via name | Flip: VL / non-cap fixtures **must not** Auto-send `think` |

### EdgeQuake monorepo (consume + prove)

| Path | Role today | SPEC-113 change |
|------|------------|-----------------|
| [`edgequake/Cargo.toml`](../../edgequake/Cargo.toml) | `edgequake-llm = "0.10.4"` | Bump to release that contains Waves A–E |
| [`handlers/models_search.rs`](../../edgequake/crates/edgequake-api/src/handlers/models_search.rs) | Surfaces `supports_thinking` | Contract: discovery value, not rename folklore |
| [`services/reasoning_effort_resolve.rs`](../../edgequake/crates/edgequake-api/src/services/reasoning_effort_resolve.rs) | Role hierarchy | No name checks; rely on provider clamp |
| Query / chat / VLM paths | Pass `reasoning_effort` | Regression e2e with mocked Ollama |

## Decision flow (normative)

```text
                    reasoning_effort set?
                     /              \
                   yes               no (Auto)
                    │                  │
                    ▼                  ▼
            clamp vocabulary    CapabilityResolver.lookup(model)
                    │                  │
                    ▼             Yes / No / Unknown
         CapabilityResolver            │
                    │         ┌────────┼────────┐
                    ▼         ▼        ▼        ▼
              Yes → map     think    omit     omit
              No  → omit    true             (+ warn)
              Unk → omit
```

## Env vars (proposed)

| Variable | Status | Meaning |
|----------|--------|---------|
| `OLLAMA_HOST` | Existing | Base URL for show/tags/chat |
| `EDGEQUAKE_OLLAMA_THINK_CAPABILITY` | **Proposed** | `auto` (default) \| `force_on` \| `force_off` \| `legacy_name` |
| `EDGEQUAKE_OLLAMA_CAPABILITY_TTL_SECS` | **Proposed** | Cache TTL (default e.g. 300) |
| `EDGEQUAKE_OLLAMA_CAPABILITY_TIMEOUT_MS` | **Proposed** | Show/tags probe timeout (default e.g. 2000) |

`legacy_name` restores pre-113 substring behavior for emergency rollback only (LAW-113-7 escape hatch).

## Related specs

| Spec | Relevance |
|------|-----------|
| [SPEC-109](../109-configurable-reasoning-effort/) | Effort hierarchy, clamp module, UI Auto |
| [SPEC-109 §7 Ollama](../109-configurable-reasoning-effort/03-provider-capability-matrix.md) | Wire `think`; “Unsupported → omit” — **extend** with live capability |
| edgequake-llm discovery docs | “No Heuristics” — chat must obey same law |
| [SPEC-112](../112-connection-pool/) | Pack structure / brutal honesty pattern |

## External references

- Issue: https://github.com/raphaelmansuy/edgequake/issues/369
- Ollama show capabilities: https://docs.ollama.com/api-reference/show-model-details
- Ollama thinking capability discovery: https://github.com/ollama/ollama/issues/10966
- Capability enum: https://github.com/ollama/ollama/blob/main/types/model/capability.go
- PR introducing show capabilities: https://github.com/ollama/ollama/pull/10066
