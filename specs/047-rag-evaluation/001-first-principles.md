# 001 — First Principles (Fair RAG Evaluation)

**Cross-ref:** [000 INDEX](./000-index.md) · [002 Deep Dive](./002-benchmark-deep-dive-mmlongbench.md) · [003 Protocol](./003-fair-evaluation-protocol.md)

---

## 1. What are we measuring?

```text
  Question (from benchmark)
           │
           ▼
  ┌────────────────────┐     ┌────────────────────┐
  │  System under test │     │  Scoring oracle    │
  │  (EdgeQuake RAG)   │────▶│  (official rules)  │
  └────────────────────┘     └────────────────────┘
           │                           │
           ▼                           ▼
     free-form answer          Acc + generalized F1
```

**Axiom A1 — Task identity must be named honestly.**  
MMLongBench-Doc’s published numbers are for **LVLMs that see page screenshots**. EdgeQuake **does not** do that. We measure:

> *Given the same real PDFs and the same questions, how well does EdgeQuake’s ingest→retrieve→generate pipeline produce answers that match the official short references under the official scoring rules?*

Call this **MMLongBench-Doc / RAG adaptation**, never “MMLongBench-Doc LVLM score.”

---

## 2. Irreducible requirements

| ID | Principle | Violation = invalid run |
|----|-----------|-------------------------|
| P1 | **Real documents** | Must use official PDFs from HF/GitHub, not summaries, OCR dumps, or synthetic stubs |
| P2 | **Official questions & answers** | Must use the released Q&A JSON (incl. unanswerable + meta fields) |
| P3 | **Official score calculator** | Must reuse `eval/eval_score.py` semantics (Int/Float/Str/List/ANLS) |
| P4 | **Three-stage eval** | Response → short-answer extraction → rule score ([paper §4.1](https://arxiv.org/abs/2407.01523)) |
| P5 | **Fixed system profile** | Provider, models, query mode, chunk/vision settings pinned in scorecard |
| P6 | **Isolation** | One workspace per run; no cross-contamination from prior docs |
| P7 | **Reproducibility** | Seeded smoke subset, version pins, artifact hash, command log |
| P8 | **Fail closed** | Missing vision, failed ingest, or extractor outage → run marked `INVALID`, not scored as 0 |

---

## 3. Why RAG on a DU benchmark is still valuable

First principles of EdgeQuake value:

1. **Production systems rarely stuff 47-page PDFs as images into a VLM.** They ingest once, index, retrieve.
2. **Cross-page questions (≈33%)** stress multi-hop retrieval — EdgeQuake’s graph + hybrid modes exist for this.
3. **Unanswerable questions (≈22.5%)** stress refusal / hallucination control — a RAG honesty test.
4. **Evidence sources (text/table/chart/image/layout)** expose whether PDF→markdown vision ingest preserves the signal retrieval needs.

So the benchmark is a **stress corpus + gold Q&A**, not a claim that EdgeQuake is an LVLM.

---

## 4. Fairness constraints (do not cheat)

```text
FORBIDDEN                                              REQUIRED
─────────                                              ────────
• Feeding gold evidence_pages into the retriever       • Blind query: question text only
• Truncating PDFs to evidence pages only               • Full PDF ingest as shipped
• Editing questions / answers                          • Exact official strings
• Softening “Not answerable” into guesses              • Explicit unanswerable handling
• Mixing providers mid-run                             • Single locked profile
• Reporting Acc without F1 / slice breakdowns          • Full scorecard schema
• Comparing to GPT-4o LVLM F1 as if same task          • Side-by-side only with caveat banner
```

Optional **oracle ablations** (labeled `oracle_*`, never mixed into primary score):

- `oracle_page_filter`: retrieve only from gold `evidence_pages` → upper bound on generation given perfect retrieval  
- `oracle_chunk`: inject gold text spans if available → generation ceiling  

These diagnose *retrieval vs generation* failure. They are not the headline number.

---

## 5. Progression as a first-class principle

Evaluation quality increases with **coverage**, not with clever prompts alone:

| Stage | Docs | Why it exists |
|-------|------|---------------|
| Smoke | 10 | Prove download → ingest → hybrid query → extract → score works end-to-end |
| Core | ~40 | Enough strata to see text vs chart vs cross-page vs unanswerable gaps |
| Full | 135 | Publishable scorecard; comparable across EdgeQuake versions |

**Axiom A2 — Never skip smoke.** A full run that fails ingest on 30% of PDFs is not a model quality result; it is an ops failure.

---

## 6. Provider first principles (Mistral Small + Embed)

| Layer | Choice | Principle |
|-------|--------|-----------|
| Embed | `mistral-embed` @ 1024-d | One embedding space for ingest + query; never mix dims mid-run |
| LLM | `mistral-small-latest` | Same family for entity extraction and answer generation (cost/quality balance) |
| Vision | `mistral-small-latest` | Same Small model for page understanding during PDF ingest (user intent) |
| Query | `hybrid` | Exercise local + global + naive arms together |

**Axiom A3 — Vision must actually see.** If the client strips images, the run is OCR/text-only and must be labeled as such. Silent degradation is scientific fraud.

---

## 7. What “easy to run / easy to evaluate” means

```text
  Human (or agent) should be able to:

  1. export two API keys
  2. run one make target
  3. open one JSON + one Markdown summary
  4. know in <60s: Acc, F1, smoke/core/full, pass/fail gates
```

If reading results requires a notebook archaeology session, the harness failed its UX requirement ([008](./008-product-sre-lens.md)).

---

## 8. Five Whys (compressed)

1. Why evaluate? → Know if GraphRAG helps on hard long PDFs.  
2. Why MMLongBench-Doc first? → Real multi-modal long PDFs + short deterministic answers + public scoring code.  
3. Why not only LVLM protocol? → EdgeQuake’s product is RAG, not 50-image context stuffing.  
4. Why smoke first? → Cost and failure rate of 135×~47-page vision ingest is high; de-risk plumbing.  
5. Why multiple lenses? → AI quality, code quality, ops cost, and statistical honesty fail independently.

Continue to [002](./002-benchmark-deep-dive-mmlongbench.md).
