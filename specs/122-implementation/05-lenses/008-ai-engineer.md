# Lens 008 — AI Engineer

## Stake

Bulk wall clock is dominated by model calls: vision pages, entity extract per chunk, embeddings. Provider choice (Ollama vs Mistral) changes both latency and safe concurrency.

## Cost model

```ascii
  T_doc ≈ T_convert(PDF?) + Σ_chunks (T_extract + T_embed) + T_graph
  T_bulk ≈ (Σ T_doc) / effective_parallelism

  effective_parallelism ≤ min(tenant, provider_budget, extract, embed, vision)
```

## Provider notes

| Provider | Parallelism reality | Guidance |
|----------|---------------------|----------|
| Ollama local | Often `OLLAMA_NUM_PARALLEL=1`; queue beyond that | Match EdgeQuake extract/tenant to 1 unless VRAM proven |
| Mistral API | Parallel OK until rate limits | Arm B measure; backoff on 429 |
| Vision models | Page-tax heavy | Dominates PDF arms (H2) |

References: [Ollama FAQ — concurrent requests](https://docs.ollama.com/faq).

## Quality vs speed

1. Gleaning / higher extract caps improve recall but multiply calls.
2. Noisy PDF markdown (H3) multiplies chunks — quality defect with throughput symptom.
3. Do not silently drop entities to “go faster” without product approval.

## Experiments (measurement)

1. Arm A vs Arm B same fixtures → docs/min.
2. PDF vs text token-normalized → H2/H3.
3. Optional: raise extract to 2 on Ollama with `OLLAMA_NUM_PARALLEL=2` — only if VRAM OK; record regression.

## Cross-refs

- PDF hypothesis: [../12-pdf-quality-hypothesis.md](../12-pdf-quality-hypothesis.md)
- Repro: [../10-reproduction.md](../10-reproduction.md)
