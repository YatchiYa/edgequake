# 02 — Cross-ref matrix

| Ref | Role for SPEC-131 |
|-----|-------------------|
| [#379](https://github.com/raphaelmansuy/edgequake/issues/379) | Trigger: omit-temperature env + Responses API |
| SPEC-109 | Reasoning effort resolve / clamp / Mistral omit pattern |
| SPEC-103 | LLM response cache — Responses must not break cache key honesty |
| SPEC-126 | Provider prompt-cache keys; P1 Responses parity or explicit defer |
| SPEC-045 / SPEC-057 | `failure_class` + recommended_action; unknown is a bug signal |
| SPEC-123 | Env/config honesty — what operators set must match what runs |
| SPEC-117 | Extraction caps — orthogonal; still uses CompletionOptions |
| AGENTS.md env table | Document new knobs after implement |
| `.env.example` | Operator-facing examples |
| `edgequake-llm` sibling | Wire transport SSOT (Chat Completions + Responses) |
| Open Responses | Cross-vendor Responses shape reference |
| Bedrock Mantle docs | GPT-5.6 Responses-only; `store` retention default |

```ascii
  SPEC-109 effort clamp ──► reasoning_effort Option
         │
         ▼
  SPEC-131 OMIT_REASONING_EFFORT ──► force None (new)
         │
  SPEC-131 OMIT_TEMPERATURE ──► resolve_effective_temperature
         │
         ▼
  CompletionOptions ──► LLMProvider
         │
         ├─ SPEC-126 prompt_cache_key (chat path today)
         └─ SPEC-131 ApiFormat::Responses (new mapper)
                │
                ▼
         failure ──► SPEC-045 classify ──► llm_unsupported_param (new)
```

## Doc ↔ code anchors

| Concern | Path |
|---------|------|
| Temperature gate | `edgequake-pipeline/.../extractor/temperature.rs` |
| Extract options | `edgequake-pipeline/.../extractor/completion_options.rs` |
| Title temperature | `edgequake-api/.../title_generator.rs` |
| VLM hardcoded temp | `edgequake-pdf/.../figure_filter.rs` |
| Role reasoning env | `edgequake-core/src/llm_roles.rs` |
| Failure classify | `edgequake-tasks/src/ingestion_reliability.rs` |
| OpenAI chat wire | `edgequake-llm/src/providers/openai.rs` |
| Compat chat wire | `edgequake-llm/src/providers/openai_compatible.rs` |
| Factory / env | `edgequake-llm/src/factory.rs` |
| Capability registry | `edgequake-llm/src/reasoning_capabilities.rs` |
| Copilot /responses skip | `edgequake-llm/src/providers/vscode/` |
| Product chat facade | `edgequake-api` `/api/v1/chat/completions` (out of scope) |

## Related specs (read, do not fork)

| Spec | Borrow |
|------|--------|
| [109](../109-configurable-reasoning-effort/) | Effort hierarchy + clamp-before-send |
| [045](../045-fix-ingestion-errors/) | Typed failure_class + permanent vs retry |
| [123](../123-env-config-priority/) | Resolved config honesty |
| [126](../126-provider-kv-cache/) | Prompt cache key semantics |
| [129](../129-touchd_document_faill/) | Doc-pack structure template |

## Cross-refs

- Code as-is: [03-code-as-is.md](03-code-as-is.md)
- Target: [04-target-architecture.md](04-target-architecture.md)
- Acceptance: [09-acceptance.md](09-acceptance.md)
