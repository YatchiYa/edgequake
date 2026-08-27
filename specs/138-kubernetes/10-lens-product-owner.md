# 10 — Lens: Product Owner

> **Cross-refs:** [WHY](00-why.md) · [Matrix](02-cross-ref-matrix.md) · [Hub](README.md)

## Outcome

Operators can deploy EdgeQuake on Kubernetes with one Makefile target and verify LLM traces appear in Langfuse — the same debugging experience as Docker dev, on a cluster.

## Success metrics

| Metric | Target |
|--------|--------|
| Cold install time (kind) | < 15 min (Langfuse stores slow-start) |
| Trace visible in Langfuse | < 30s after query |
| One-command proof | `make spec138-kubernetes-proof` exit 0 |

## Honesty rules

- Langfuse receives **traces**, not application stdout logs.
- kind profile uses **mock LLM** — not representative of production LLM quality.
- Bundled Langfuse stores are for dev/E2E, not production HA.

## Non-goals

- Replacing Langfuse Cloud for SaaS customers
- In-cluster Ollama as product default
