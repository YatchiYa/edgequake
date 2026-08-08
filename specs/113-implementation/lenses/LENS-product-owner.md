# LENS — Product Owner (SPEC-113)

## Problem in product language

Operators pick a **local multimodal model** (often Qwen3-VL) for PDF / vision / cheap local chat. EdgeQuake silently adds `think: true` because the **name looks like a reasoning family**. Ollama rejects the call. The product looks broken; the workaround is renaming the model — an unacceptable onboarding story.

## “Done” means

| Outcome | Law |
|---------|-----|
| Any Ollama model whose caps lack `thinking` works under Auto | 113-2, 113-3 |
| True thinking models still get Auto think when caps say Yes | 113-5 |
| UI capability chips match chat behavior | 113-8 |
| Partners unblock today via runbook without a release | 07-ops |
| #369 closed with measurement proof, not folklore | brutal honesty |

## Non-goals this pack

- Guaranteeing VL models produce great extraction quality.
- Owning Ollama template bugs for `think:false` on some VL tags.
- Shipping a marketing claim “we support every GGUF on the internet.”

## Acceptance narrative

```text
  Before:  qwen3-vl:* → every chat dies; rename model to survive
  After:   caps drive think; Auto omits when unsupported
           rename workaround optional nostalgia
```

## Priority call

Ship **Wave A + E** before polish. Capability-gated Auto stops the bleeding; cache/UI honesty harden trust.
