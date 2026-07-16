# SPEC-047 smoke — 2-doc `ite` validate — 2026-07-15T01:57:08Z

> EdgeQuake RAG adaptation of MMLongBench-Doc; not comparable to the LVLM leaderboard without caveats. Official LVLM GPT-4o F1≈44.9% is a difficulty reference only. Provider stack: mistral-small-latest + mistral-embed (Postgres).

## What this run is

**Purpose:** prove vision PDF ingest + multimodal Pass B (`process_options=ite`) is wired and produces a **valid** scorecard — not a locked Acc baseline.

| Pin | Value |
|-----|--------|
| Fixture | `smoke_validate_2_doc_ids` (2 PDFs) |
| Docs | `05-03-18-political-release.pdf` (chart-heavy) · `measuringsuccessonfacebooktwitterlinkedin-…pdf` (figure-heavy) |
| Profile | `P0_mm_ite` · hybrid · `--document-scope` |
| Vision | Pass A: `pdf_parser_backend=vision` · `mistral-small-latest` |
| Multimodal | Pass B: `process_options=ite` (`i` images/charts/figures, `t` tables, `e` equations) + `VLM_PROCESS_ENABLE=true` |
| Workspace | `2ae2d8ea-885b-4e25-ae7a-3f2987866321` |
| Snapshot | `smoke-validate2-ite-20260715-015708-complete/` |

**Do not read** `smoke-pre-validate2-ite-20260715-015126/` — that archive is the **failed** first attempt (`PARTIAL_INGEST`, Acc 0) caused by Postgres pool timeouts / 500s on status polls after PDFs had already finished converting.

## Verdict
- valid: `True`
- Overall Acc: **0.5357** (n_scored=24)
- Overall F1: **0.2857**
- Docs: 2 | Questions: 24 | Ingest skip: 0
- Ingest coverage: 1.00
- Profile: `P0_mm_ite` mode=`hybrid` process_options=`ite` query_workers=2 ingest_workers=2
- Empty-answer rate: **0.00** · page_hit@5: **1.00**

## Explanation (how to read this)

1. **`valid=true` means the pipeline worked** — both docs ingested, all 24 questions scored, no empty RAG answers. It does **not** mean “ready for SOTA / full leaderboard.”

2. **Acc ≈ 0.54 looks high vs chart-8 (~0.43) because n=24 is tiny and skewed.** This fixture over-weights unanswerable (Unans Acc 0.90) and one tutorial-style doc. Treat Acc here as a **smoke of process**, not the locked Acc physics number.

3. **`ite` is activated and retrieval is healthy under document-scope:**
   - Task payload had `multimodal_process_options=ite`
   - `context_empty_rate=0` and `page_hit@5=1.0` on answerable Qs → gold pages are retrieved
   - False refusal 14% (2/14) with page hit still present → generation/refusal, not “never found the page”

4. **Chart is still the quality gap** (Acc **0.14**, n=7) even with `ite` on. Figures fare better (**0.48**). That matches the SPEC-047 W1 story: Pass B helps enablement/retrieval of visuals, but chart **numbers in markdown** remain the Acc bottleneck (see fidelity work in 015).

5. **Cross-page stays hard** (Acc 0.18) while single-page is 0.62 — expected on a 2-doc hybrid RAG smoke.

### Compared to related runs

| Run | n Qs | valid | Acc | page_hit@5 | Notes |
|-----|------|-------|-----|------------|--------|
| This validate-2 `ite` | 24 | true | 0.54 | 1.00 | Process OK; Acc noisy |
| Failed pre-archive | 0 | false | 0 | — | Ops/pool failure |
| Locked chart-8 smoke | 117 | true | ~0.43 | ~0.85 | Acc baseline for progression |

## How to read this score (standard gates)
- **valid=true** means ops gates passed (ingest + non-empty answers). It is not “beats GPT-4o.”
- **Acc** = mean official short-answer score; **F1** balances answerable vs predicted-answerable.
- **LVLM GPT-4o F1≈44.9%** is a difficulty reference only (page-screenshot task ≠ RAG).
- Prefer slice gaps (chart / cross-page / unanswerable) over a single headline number.
- **page_hit@k** (W0): gold `evidence_pages` ∩ retrieved chunk `page_start` — retrieval law, not Acc.
- **false_refusal** (020 A2): answerable gold ∧ pred≈Not answerable; slice by page_hit@5.

## Retrieval diagnostics (W0)
- n_answerable_with_diag: 14
- document_scope: `True`
- context_empty_rate: 0.0000
- page_hit@1: 0.6428571428571429
- page_hit@3: 1.0
- page_hit@5: 1.0
- page_hit@10: 1.0
- page_recall@5: 0.8571428571428571
- mean_n_chunk_sources: 18.714285714285715
- mean_arm_local_chunks: 5.571428571428571
- mean_arm_global_chunks: 9.0
- mean_arm_naive_chunks: 20.0

## Refusal diagnostics (020 A2)
- n_answerable: 14
- false_refusal_rate: 0.1429 (n=2)
- false_refusal_given_page_hit@5: 0.1429 (n=2 / 14)

## Arm-gate diagnostics (020 B1/B2)
- n_with_arm_diag: 24
- arms_gated_rate: 0.9583333333333334
- planned_graph_rate: 1.0 (planned_naive_only=0.0)
- arm_graph_present_rate: 1.0 (productive chunks; empty local ≠ gate)
- naive_only_rate: 0.0 (productive; prefer planned_* for B2 honesty)
- arm_local_present_rate: 1.0
- arm_global_present_rate: 0.041666666666666664

## Slices
- Single-page Acc: 0.6190
- Cross-page Acc: 0.1818
- Unanswerable Acc: 0.9000

### By evidence source
- Chart: Acc=0.1429 (n=7)
- Figure: Acc=0.4762 (n=6)
- Pure-text (Plain-text): Acc=0.0000 (n=4)
- Table: Acc=0.2500 (n=4)

### By document type
- Research report / Introduction: Acc=0.4167 (n=12)
- Tutorial/Workshop: Acc=0.6548 (n=12)

## Next lever
Raise **Chart Acc / Chart answer-in-evidence** (SPEC-047 015 modality-aware vision). Do not ban “Not answerable” or chase LVLM F1 on this 2-doc Acc.

## Citation
Ma et al., MMLongBench-Doc, arXiv:2407.01523 / NeurIPS 2024 D&B.
https://github.com/mayubo2333/MMLongBench-Doc
