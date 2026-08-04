# SPEC-108 measurements SUMMARY

**Date:** 2026-08-04  
**Modes:** geometry-only (stride) + mock-extract (M vs U) + Acc historical unique-node audit

## Geometry (H2)

| Sample | A N @1200 | B N (product) | B pin | B/A |
|--------|----------:|--------------:|------:|----:|
| S1-md (LightRAG paper gold) | 8 | 12 | 800 | **1.50** |
| S-synth-200kb | 21 | 41 | 600 | **1.95** |
| S2-medical-one (~1MB text) | 159 | 317 | 600 | **1.99** |
| S1 PDF bytes only | pin 1200 | pin **600** | 600 | (text N after parse) |

A ≡ C under 1200/100 (same heuristic).

**Partner envelope:** M=12367 ⇒ min N ≥ **309**. S2 product arm N=**317** lands in that band.

## M vs U mock (H1)

| Scenario | M | U | M/U |
|----------|--:|--:|----:|
| Partner envelope N=309 y=40 reuse=0.7 | 12360 | 3709 | **3.33** |
| S2 product N=317 y=30 reuse=0.55 | 9510 | 4122 | 2.31 |

Code SSOT: `stats.rs` sums mentions; merger collapses to unique `EntityId`.

## Fair unique graph (H4 monitor) — Acc audit

Source: `specs/001-benchmark/e2e/artifacts/ingest-audit/20260721T010844Z/SUMMARY.md` (fair Acc pins, medical corpus):

| Side | Unique nodes U | Edges |
|------|---------------:|------:|
| LightRAG | 3580 | 5325 |
| EdgeQuake | 3950 | 3927 |

Jaccard ≈ 0.45; EQ slightly denser but **same order** as LR — not 12k vanity.

## Verdict line

Partner “12k entities” is explained by **H1 (M on card)** amplified by **H2 (adaptive → more chunks)**. Fair unique U is Acc-scale ~4k for multi-doc medical, not 12k.
