# 007 — ML Scientist Lens

**Cross-ref:** [002](./002-benchmark-deep-dive-mmlongbench.md) · [003](./003-fair-evaluation-protocol.md) · [011](./011-complementary-benchmarks-methodology.md) · [012](./012-acceptance-criteria-and-scorecard.md)

---

## 1. Primary metrics (locked)

| Metric | Definition | Role |
|--------|------------|------|
| Acc | Mean of per-question `eval_score` | Overall correctness |
| F1 | Generalized F1 from upstream (answerable vs pred-answerable) | Balances hallucination on unanswerable |
| Slice Acc | single-page / cross-page / unanswerable / source / doc_type | Diagnostic |

**Headline number for SPEC-047:** Overall **F1** on the declared stage, with Acc beside it.

---

## 2. Stratification (why smoke is not random 10)

Random docs under-sample rare chart/image and unanswerable cases. Smoke fixture must maximize **coverage of failure modes**, not minimize variance of the mean.

Report for every stage:

- n questions, n docs  
- % cross-page, % unanswerable  
- distribution of `answer_format`  
- distribution of evidence sources  

If smoke distribution diverges wildly from full, note it in SUMMARY (selection bias disclaimer).

---

## 3. Statistical honesty

| Practice | Rule |
|----------|------|
| Single run | Report as point estimate; do not invent ±CI without repeats |
| Repeats | Optional 3× smoke with different workspaces → mean±std of F1 |
| Extractor sensitivity | Dual extractor on smoke; report ΔF1 |
| Multiple comparisons | Ablations are exploratory unless pre-registered in 009 |
| Leaderboard compare | Forbidden without task-equivalence banner |

Noise band (initial heuristic, revise after repeats): smoke F1 ±0.03 absolute is “same”; larger swings need investigation.

---

## 4. Confounders to control

1. **Ingest quality** vs **retrieval** vs **generation** — use ablations P5/P6.  
2. **Extractor model** — keep constant when comparing EdgeQuake versions.  
3. **Dataset revision** — Sep 2025 Q&A updates.  
4. **Prompt changes** in EdgeQuake query system prompt — hash it.  
5. **Partial ingest** — never impute zeros for failed docs; exclude + report coverage.

---

## 5. Experimental register (pre-register before full)

| Exp ID | Question | Primary contrast | Stage |
|--------|----------|------------------|-------|
| E1 | Does hybrid beat naive on cross-page? | P0 vs P1 | core |
| E2 | Does vision beat text parse on chart/image? | P0 vs P5 | smoke→core |
| E3 | How much headroom if retrieval perfect? | P0 vs P6 | smoke |
| E4 | Extractor sensitivity | official vs mistral_judge | smoke |

---

## 6. Complementary scientific program

MMLongBench-Doc alone cannot answer:

- multi-doc corpus retrieval (need MultiHop-RAG / UniDoc-Bench)  
- pure embedding quality (need MTEB / BEIR)  
- graph necessity (need GraphRAG-Bench)  

See [011](./011-complementary-benchmarks-methodology.md).

---

## 7. ML acceptance

- [ ] Scorecard includes slice table  
- [ ] Selection bias note for smoke/core  
- [ ] Experiments E1–E4 runnable  
- [ ] No silent imputation  

Next: [008 Product/SRE](./008-product-sre-lens.md).
