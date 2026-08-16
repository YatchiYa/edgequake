# Labeled publish peer — `EQ_LLM_CACHE_WARM_v1`

**Not Acc headline.** Acc SSOT remains [`publish/latest/`](../../latest/) (`medical-mid-20260815T110218Z`).

## What this peer proves

Warm Mix **product latency** with EQ `EDGEQUAKE_LLM_CACHE=1` and LR `enable_llm_cache=True` (matched cache-aided protocol):

| Layer | EdgeQuake | LightRAG | Claim |
|-------|-----------|----------|-------|
| Query p50 | **82 ms** | 993 ms | EQ **0.083×** (warm wall win) |
| EQ stages p50 | kw **0** · embed **0** · retrieve **56** · gen **0** | — | Retrieve-only after fill |
| Acc (point) | 0.792 | 0.773 | Statistical **tie** (CI includes 0) — **not Acc Beat** |

## Physics (first principles)

1. Pass 1 fills EQ `public.llm_cache` + engine embed LRU.  
2. Pass 2 measures warm repeats.  
3. Fix: Acc Mix workspace inject reuses engine embed LRU when `(name, model, dim)` match — without this, warm EQ still paid ~2s mistral-embed RTT.

## Reproduce

```bash
export BENCH001_EQ_WORKSPACE_ID=<warm-full-corpus-ws>
make bench001-medical-mid-eq-llm-cache-warm
```

## Artifacts

- [BUSINESS_REPORT.md](./BUSINESS_REPORT.md)
- [EXEC_SUMMARY.txt](./EXEC_SUMMARY.txt)
- [SUMMARY.md](./SUMMARY.md)
- [scorecard.json](./scorecard.json)
- Archive: [`medical-mid-20260815T132034Z`](../../../history/medical-mid-20260815T132034Z/)
