# SPEC-047 smoke — 2026-07-15T02:24:05Z

> EdgeQuake RAG adaptation of MMLongBench-Doc; not comparable to the LVLM leaderboard without caveats. Official LVLM GPT-4o F1≈44.9% is a difficulty reference only. Provider stack: mistral-small-latest + mistral-embed (Postgres).

## Verdict
- valid: `True`
- Overall Acc: **0.4154** (n_scored=117)
- Overall F1: **0.2464**
- Docs: 8 | Questions: 117 | Ingest skip: 0
- Ingest coverage: 1.00
- Profile: `P0_mm_ite` mode=`hybrid` process_options=`ite` query_workers=2 ingest_workers=4

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
- page_hit@1: 0.37333333333333335
- page_hit@3: 0.7066666666666667
- page_hit@5: 0.7733333333333333
- page_hit@10: 0.8133333333333334
- page_recall@5: 0.6551111111111111
- mean_n_chunk_sources: 17.2
- mean_arm_local_chunks: 7.1891891891891895
- mean_arm_global_chunks: 7.545454545454546
- mean_arm_naive_chunks: 18.986666666666668

## Refusal diagnostics (020 A2)
- n_answerable: 75
- false_refusal_rate: 0.1467 (n=11)
- false_refusal_given_page_hit@5: 0.1034 (n=6 / 58)

## Arm-gate diagnostics (020 B1/B2)
- n_with_arm_diag: 117
- arms_gated_rate: 0.8547008547008547
- planned_graph_rate: 1.0 (planned_naive_only=0.0)
- arm_graph_present_rate: 0.9914529914529915 (productive chunks; empty local ≠ gate)
- naive_only_rate: 0.008547008547008548 (productive; prefer planned_* for B2 honesty)
- arm_local_present_rate: 0.9829059829059829
- arm_global_present_rate: 0.15384615384615385

## Slices
- Single-page Acc: 0.3488
- Cross-page Acc: 0.1389
- Unanswerable Acc: 0.7143

### By evidence source
- Chart: Acc=0.2273 (n=22)
- Figure: Acc=0.2381 (n=21)
- Generalized-text (Layout): Acc=0.3618 (n=11)
- Pure-text (Plain-text): Acc=0.2548 (n=26)
- Table: Acc=0.1927 (n=24)

### By document type
- Academic paper: Acc=0.2500 (n=16)
- Administration/Industry file: Acc=0.5544 (n=18)
- Financial report: Acc=0.3529 (n=17)
- Research report / Introduction: Acc=0.3819 (n=54)
- Tutorial/Workshop: Acc=0.6667 (n=12)


## vs LVLM SOTA (July 2026 reference) — READ CAVEATS

**Task identity:** this EdgeQuake run is a **RAG adaptation** on the chart-8 smoke fixture
(8 docs / 117 Qs, hybrid retrieve + Small LLM).
Official MMLongBench-Doc leaderboard scores are **page-screenshot LVLMs on ~1082 questions**.
Numbers are **difficulty references**, not a same-protocol ranking.

| System | Acc | F1 | Chart Acc | Protocol |
|--------|-----|----|-----------|----------|
| **EdgeQuake P0_mm_ite (this run)** | **0.4154** | **0.2464** | **0.2273** | RAG · 8-doc smoke · dscope · ite |
| TeleMM2.0 (2026-01-05) — official HF SOTA | 0.5609 | 0.5590 | 0.5416 | Full LVLM board |
| GPT-4.1 (2025-04-14) | 0.4974 | 0.5142 | 0.4847 | Full LVLM board |
| GPT-4o (2024-11-20, refreshed board) | 0.4625 | 0.4624 | 0.4315 | Full LVLM board |
| Paper GPT-4o (NeurIPS'24 original report) | — | 0.4490 | — | Full LVLM board |

Sources: [OpenIXCLab/mmlongbench-doc-results](https://huggingface.co/datasets/OpenIXCLab/mmlongbench-doc-results)
(official). Aggregators may list higher single scores (e.g. Qwen / Nemotron ~57–62%) under
third-party protocols — prefer the official Acc/F1 board for citation.

- ΔAcc vs TeleMM2.0 (SOTA Acc): **-0.1455** (not same task)
- ΔF1 vs TeleMM2.0 (SOTA F1): **-0.3126** (not same task)
- ΔF1 vs paper GPT-4o (0.449): **-0.2026** (difficulty ref only)
- Ops: ingest_coverage=1.0 page_hit@5=0.7733333333333333 empty=0.0

## vs prior locked chart-8 (same fixture)

| Run | Acc | F1 | Chart | page_hit@5 | Unans |
|-----|-----|-----|-------|------------|-------|
| This (2026-07-15 ite) | 0.4154 | 0.2464 | **0.2273** | 0.77 | 0.71 |
| Prior (2026-07-14 1530) | **0.4347** | 0.2430 | 0.1818 | **0.85** | **0.79** |

ΔAcc −0.019 vs prior is within smoke noise; Chart Acc moved up (~+0.05). Do not call this a regression until a repeated locked run.

## Fidelity (W1 sample, n=25 answerable)

- answer_in_evidence: **0.48** (Chart **0.40**)
- Representation miss dominates (~13/25). Retrieval-miss-given-rep-ok ≈ 2 → bottleneck is **ingest text**, not hybrid fusion.

## First-principles assessment — how to improve

### Is this the wrong test for EdgeQuake?
**Partly yes for SOTA headlines; no for product truth.**

| Goal | Right measurement |
|------|-------------------|
| Ship a graph RAG that answers from ingested PDFs | This scorecard + page_hit + fidelity (**correct**) |
| Claim “beats TeleMM2.0 / GPT-4o on MMLongBench” | **Wrong** — those models see page screenshots on ~1082 Qs; we retrieve text chunks from 8 docs |
| Difficulty reference | Official LVLM Acc/F1 is fair as a *ceiling/hardness* signal, with the banner caveat |

**Verdict:** keep MMLongBench as a RAG adaptation benchmark; stop ranking against LVLM board Acc as if same protocol.

### Stronger model?
**Yes — but only on the vision / chart extract path first.**

Evidence: Chart Acc 0.23 vs TeleMM Chart 0.54; Chart fidelity 0.40 → gold numbers often never enter markdown. `mistral-small-latest` as Pass A+B VLM is a deliberate cost pin, not a SOTA vision stack.

| Upgrade | Expected effect | When |
|---------|-----------------|------|
| Stronger **vision** for Pass A / chart specialize | ↑ answer_in_evidence → ↑ Chart Acc | **Highest leverage** after gate fidelity |
| Stronger **query** LLM | Modest ↓ false refusal (~10% when page hit) | After W1 moves |
| Stronger embed only | Unlikely Acc leap (page_hit already 0.77) | Low priority |

### Better harness?
**Already good enough for this Acc band; polish, don’t rebuild.**

Working: `valid=true`, empty=0, `ite` on, ingest 8/8, fail-closed ops. Gaps are product ingest quality, not missing CLI flags.

Worthwhile harness work: CI fidelity gate on Chart `answer_in_evidence ≥ 0.50`; Oracle P6 ablation (retrieval ceiling); don’t “ban Not answerable.”

### Better prompt?
**Lowest first-principles leverage alone.**

FP1: if the chart number never hits the index, no query prompt can honestly recover it. Prompt tweaks that forbid “Not answerable” inflate Acc and kill Unans (this run Unans 0.71 already softer than prior 0.79).

Worthwhile only after representation improves: grounded generation when `page_hit` / dense context (cut the 10% false-refusal-given-hit).

### Ranked next moves

1. **W1 — Chart/table numbers into markdown** (modality crops + stronger vision) — gate fidelity Chart ≥0.50  
2. **Measure** page_hit@5 must stay ≥ ~0.75 (don’t trade retrieve for Acc)  
3. **W3 — grounded refusal** when evidence is present  
4. **Stronger query model** ablation labeled in scorecard  
5. **Never** optimize toward TeleMM Acc without labeling LVLM task

## Citation
Ma et al., MMLongBench-Doc, arXiv:2407.01523 / NeurIPS 2024 D&B.
https://github.com/mayubo2333/MMLongBench-Doc
