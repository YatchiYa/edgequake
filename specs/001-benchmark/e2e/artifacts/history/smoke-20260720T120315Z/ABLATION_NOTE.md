# Ablation — 044 B5 placeholder provenance + a1fp

**Archive:** `smoke-20260720T120315Z`  
**Workspace:** `8e990410-43b5-44f4-9f56-87bd154570ce` (fresh force-ingest; prior Acc peer `2a7bcb2f-…` frozen)  
**Profile:** `A1FP_p2b_rr_cer_fact_protect_bm25_v1_lrlike_arms_v2`  
**Ingest profile:** `B5_placeholder_provenance_v1` (md + glean=1 · no FAQ · chunk 1200/100)

## One confound

Relation-endpoint **placeholder** nodes inherit the relation’s `source_chunk_id` into `source_chunk_ids` / `source_ids` (and enrich existing zero-chunk stubs). Query pack unchanged (`a1fp`).

## Ingest hygiene ([audit 20260720T120040Z](../../ingest-audit/20260720T120040Z/))

| Signal | Prior warm `2a7bcb2f` | B5 WS |
|--------|----------------------:|------:|
| EQ zero-chunk rate | 0.076 (345 stubs) | **0.0** |
| UNKNOWN empty stubs | 345 | **0** |
| Mean chunks/entity | 2.228 | **2.373** |
| age_over_vectors | 1.082 | 1.077 |

## Acc (n=40)

| | EQ Acc | Fact ER | Sum ER | ctx | recall |
|--|-------:|--------:|-------:|----:|-------:|
| Prior peer `a1fp` T095809Z | 0.775 | 0.85 | 0.86 | 0.500 | 0.926 |
| **B5 + a1fp** T120315Z | **0.801** | **0.85** | 0.863 | **0.519** | 0.926 |
| LR (this run) | 0.782 | 0.90 | 0.983 | 0.519 | 0.966 |

Δ Acc vs LR **+0.019** · 95% CI **[-0.064, +0.100]** — still includes 0 → **no Beat claim**.

## Probe

`Medical-0002d2de` (`bone cancers`): phrase still ✗ in Mix — SELECT law unchanged (037–042). B5 is ingest hygiene / Acc ceiling, not Mix SELECT.

## Call

**PROMOTE** B5 workspace as Acc Fact peer candidate (hygiene PASS ∧ Acc ≥ 0.755 ∧ Fact ER ≥ 0.83 ∧ ctx ≥ 0.50). Keep prior `2a7bcb2f` frozen for A/B. Do not claim Beat.
