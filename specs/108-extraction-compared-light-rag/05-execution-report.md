# 05 — Execution Report

> Filled 2026-08-04. Primary artifacts: [measurements/SUMMARY.md](measurements/SUMMARY.md).

## What ran

| Step | Result |
|------|--------|
| `geometry_probe.py` | A/B/C stride N for S1-md, S2-medical-one, S-synth-200kb; PDF byte pins for S1 |
| `mention_sim.py` | Mock M vs U for partner envelope + S2 scales |
| `cargo test … adaptive_sizes_match_lightrag_thresholds` | **ok** (thresholds 1200/800/600) |
| Live EQ HTTP ingest (JWT) | Backend up but auth required — **not** re-ingested this pack |
| Acc ingest-audit unique U | Reused `20260721T010844Z` (fair pins medical) |

## Geometry scoreboard

| Sample | A (fair 1200) | B (product) | B/A | Pin B |
|--------|--------------:|------------:|----:|------:|
| S1-md | 8 | 12 | 1.50 | 800 |
| S-synth-200kb | 21 | 41 | 1.95 | 600 |
| S2-medical-one | 159 | 317 | **1.99** | 600 |

S1 PDF (~1.1 MB): product pin **600** vs fair/LR **1200** (chunk N after text extract).

A and C match under identical 1200/100 heuristic (LAW-X3 geometry).

## Partner number fit

```text
Partner M = 12367
min_N ≥ ceil(12367/40) = 309
S2 product N = 317   ← same order
mock envelope M = 12360, U ≈ 3709 (M/U ≈ 3.3)
```

## Fair unique graph (not vanity)

Acc audit (fair ingest, medical corpus):

| | Unique nodes | Edges |
|--|-------------:|------:|
| LightRAG | 3580 | 5325 |
| EdgeQuake | 3950 | 3927 |

EQ slightly denser; **not** an order-of-magnitude over-extract vs LR under fair pins.

## Mode honesty

- Geometry: **geometry-only-stride** (word/CJK heuristic; not full RecursiveCharacterSplitter).
- M/U: **mock-extract** (illustrates LAW-X1; not live LLM yields).
- Unique U EQ↔LR: **live Acc historical** audit under fair pins.

## Commands to reproduce

```bash
python3 specs/108-extraction-compared-light-rag/measurements/geometry_probe.py
python3 specs/108-extraction-compared-light-rag/measurements/mention_sim.py
cd edgequake && cargo test -p edgequake-pipeline adaptive_sizes_match_lightrag_thresholds --lib
```
