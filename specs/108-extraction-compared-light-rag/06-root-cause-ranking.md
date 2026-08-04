# 06 — Root-Cause Ranking

> Scored from [05-execution-report.md](05-execution-report.md) + code in [03-code-comparison.md](03-code-comparison.md).

| Rank | ID | Hypothesis | Verdict | Evidence |
|-----:|----|------------|---------|----------|
| 1 | **H1** | Metric illusion — UI stores mentions M, not unique U | **Primary** | `stats.rs` sum; mock M/U≈3.3 on partner envelope; Acc U≈4k not 12k |
| 2 | **H2** | Adaptive geometry inflates N → M | **Confirmed** | B/A ≈ 1.5–2.0; S2 product N=317 fits ≥309 for M=12k |
| 3 | **H3** | Strategy geometry (Pdf vs R) | **Secondary** | PDF auto-strategy differs from MD Recursive; same size still changes N |
| 4 | **H4** | Merge/normalize gap vs LightRAG | **Monitor** | Fair Acc: EQ 3950 vs LR 3580, Jaccard ~0.45 — denser, not 12k-class |
| 5 | **H5** | True over-extract (prompt/glean) | **Not required** | Caps already 40/100 LR-parity; H1+H2 explain partner symptom |

## Causal chain (partner symptom)

```ascii
 large doc bytes
   → adaptive ON → chunk_size 600
   → N ≳ 300
   → LLM × N (≤40 ents)
   → M ≈ 12k written to document card   ← partner sees this
   → merge → U ≪ M in AGE                ← not shown as primary metric
```

## What is **not** broken

- Extract caps (40/100) match LightRAG.
- Fair-pin unique graph density is Acc-comparable to LightRAG.
- Chunker Recursive separators intentionally mirror LR strategy R.

## Follow-ups (out of SPEC-108 code scope)

| If product wants… | Action |
|-------------------|--------|
| Honest UI | Surface U and/or entities/1k chars (SPEC-086) alongside M |
| LR-like vanity | Pin `EDGEQUAKE_ADAPTIVE_CHUNKING=0` + 1200/100 for that workspace |
| Deeper EQ↔LR name gap | Continue Acc ingest Jaccard work (029/054) — separate from partner 12k |

## Decision

**No emergency chunker rewrite.** Answer the partner with LAW-X1 + LAW-X2; offer fair-pin config; optionally productize density display later.
