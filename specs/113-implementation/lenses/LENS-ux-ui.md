# LENS — UX / UI (SPEC-113)

## User-visible failure today

```text
  User selects ollama / qwen3-vl:*
  Query → error toast / Failed document
  Message often opaque ("provider error" / network-ish)
  User renames model (power-user escape) — product shame
```

## UX principles

1. **Honesty** — If the model cannot think, do not imply Thinking is on (LAW-113-8).
2. **Progressive disclosure** — Auto remains default; advanced effort control only when `supports_thinking`.
3. **Recoverable copy** — Prefer: “This Ollama model does not support thinking mode; retrying without it.” (if Wave B+ auto-retry lands).
4. **No rename rituals** — Never document `ollama cp` as the primary UX; runbook-only until fix ships.

## Surfaces (SPEC-109 aligned)

| Surface | Behavior after fix |
|---------|-------------------|
| Query sheet effort | Hide/disable when `supports_thinking=false` |
| Workspace role LLM | Same gate per role model |
| PDF vision effort | VL models often non-thinking — default omit is correct |
| Models catalog search | Filter `requires_thinking` must use live caps |

## Microcopy (draft)

| State | Copy |
|-------|------|
| Caps No | “Thinking not supported by this model” |
| Caps Unknown | “Thinking availability unknown — using standard completion” |
| Caps Yes + Auto | “Thinking: Auto” |
| Provider 400 think | “Model rejected thinking mode — check Ollama capabilities” |

## Anti-patterns

- Badge “Qwen3 = Reasoning” from name alone  
- Forcing users through Settings to “turn off thinking” when we injected it silently  
- Silent success after alias rename without explaining root cause in release notes
