# SPEC-047 smoke — 2026-07-11T12:18:44Z

> EdgeQuake RAG adaptation of MMLongBench-Doc; not comparable to the LVLM leaderboard without caveats. Official LVLM GPT-4o F1≈44.9% is a difficulty reference only. Provider stack: mistral-small-latest + mistral-embed (Postgres).

## Verdict
- valid: `True`
- Overall Acc: **0.4229** (n_scored=117)
- Overall F1: **0.2321**
- Docs: 8 | Questions: 117 | Ingest skip: 0
- Ingest coverage: 1.00
- Profile: `P0_mm_ite` mode=`hybrid` process_options=`ite` query_workers=2

## How to read this score
- **valid=true** means ops gates passed (ingest + non-empty answers). It is not “beats GPT-4o.”
- **Acc** = mean official short-answer score; **F1** balances answerable vs predicted-answerable.
- **LVLM GPT-4o F1≈44.9%** is a difficulty reference only (page-screenshot task ≠ RAG).
- Prefer slice gaps (chart / cross-page / unanswerable) over a single headline number.
- **page_hit@k** (W0): gold `evidence_pages` ∩ retrieved chunk `page_start` — retrieval law, not Acc.
- **false_refusal** (020 A2): answerable gold ∧ pred≈Not answerable; slice by page_hit@5.

## Retrieval diagnostics (W0)
- n_answerable_with_diag: 75
- document_scope: `True`
- context_empty_rate: 0.0000
- page_hit@1: 0.38666666666666666
- page_hit@3: 0.64
- page_hit@5: 0.72
- page_hit@10: 0.8266666666666667
- page_recall@5: 0.6464444444444445
- mean_n_chunk_sources: 16.85333333333333
- mean_arm_local_chunks: 10.81081081081081
- mean_arm_global_chunks: 12.0
- mean_arm_naive_chunks: 19.386666666666667

## Refusal diagnostics (020 A2)
- n_answerable: 75
- false_refusal_rate: 0.2267 (n=17)
- false_refusal_given_page_hit@5: 0.1111 (n=6 / 54)

## Arm-gate diagnostics (020 B1/B2)
- n_with_arm_diag: 117
- arms_gated_rate: 0.8974358974358975
- planned_graph_rate: 1.0 (planned_naive_only=0.0)
- arm_graph_present_rate: 1.0 (productive chunks; empty local ≠ gate)
- naive_only_rate: 0.0 (productive; prefer planned_* for B2 honesty)
- arm_local_present_rate: 0.9914529914529915
- arm_global_present_rate: 0.1111111111111111

## Slices
- Single-page Acc: 0.2944
- Cross-page Acc: 0.1389
- Unanswerable Acc: 0.7857

### By evidence source
- Chart: Acc=0.1818 (n=22)
- Figure: Acc=0.2313 (n=21)
- Generalized-text (Layout): Acc=0.1818 (n=11)
- Pure-text (Plain-text): Acc=0.2163 (n=26)
- Table: Acc=0.1927 (n=24)

### By document type
- Academic paper: Acc=0.3750 (n=16)
- Administration/Industry file: Acc=0.4444 (n=18)
- Financial report: Acc=0.3529 (n=17)
- Research report / Introduction: Acc=0.4005 (n=54)
- Tutorial/Workshop: Acc=0.6548 (n=12)

## Citation
Ma et al., MMLongBench-Doc, arXiv:2407.01523 / NeurIPS 2024 D&B.
https://github.com/mayubo2333/MMLongBench-Doc

## Comparison vs locked lineage (same fixture / dscope)

| Metric | Lineage | MV-18/19 full | Δ |
|--------|---------|---------------|---|
| Acc | 0.427 | **0.423** | ≈noise |
| F1 | 0.225 | **0.232** | +0.007 |
| Unanswerable Acc | 0.833 | 0.786 | −0.047 |
| Pure-text Acc | 0.192 | **0.216** | +0.024 |
| Chart Acc | 0.136 | **0.182** | **+0.046** |
| page_hit@5 | 0.760 | 0.720 | −0.040 |
| Chart answer_in_evidence | ~0.32–0.36 | **0.409** (n=22) | **+Rep** |

**Verdict:** Representation moved (Chart a_in_e 0.41; Chart Acc 0.18). Overall Acc flat — expected until Chart a_in_e clears G-A (≥0.50) / G-B (≥0.60). Tickets MV-18/19 landed; next Acc lever still denser chart extract / Pass A quality, not fusion.
