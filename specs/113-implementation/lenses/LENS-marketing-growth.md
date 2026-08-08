# LENS — Marketing & Growth (SPEC-113)

## Narrative risk

Local-first GraphRAG lives or dies on **“bring your Ollama models.”** A substring heuristic that bricks popular VL tags converts demos into support threads and GitHub bugs — exactly #369.

## Positioning after fix

| Avoid | Prefer |
|-------|--------|
| “We auto-enable thinking for Qwen3” | “We honor Ollama’s per-model capabilities” |
| “Smart defaults for reasoning models” | “Safe Auto: never send unsupported `think`” |
| Shame the reporter’s workaround | Credit the alias proof; ship the real fix |

## Growth loops touched

```text
  HuggingFace / Ollama library VL model
        → pull
        → select in EdgeQuake
        → FIRST chat must succeed
        → upload PDF demo
```

First-chat failure at the VL step kills the activation funnel for multimodal evaluators.

## Release communication (when Waves ship)

- Changelog: “Ollama `think` gated on `/api/show` capabilities (#369)”  
- Migration: none; remove alias workarounds optional  
- Do not claim fixed Ollama VL template quirks — separate upstream story

## Credibility asset

Publish the one-screen ASCII from the README + link to ops runbook. Engineers share diagrams; marketers share “works with your local models without renaming.”
