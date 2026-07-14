# 002 — MMLongBench-Doc Deep Dive (Understand Before You Measure)

**Cross-ref:** [001 First Principles](./001-first-principles.md) · [003 Protocol](./003-fair-evaluation-protocol.md) · Upstream: [GitHub](https://github.com/mayubo2333/MMLongBench-Doc) · [Paper](https://arxiv.org/abs/2407.01523) · [Homepage](https://mayubo2333.github.io/MMLongBench-Doc/) · [HF dataset](https://huggingface.co/datasets/yubo2333/MMLongBench-Doc)

---

## 1. What the benchmark is

| Fact | Value (README / homepage, 2025 refresh) |
|------|----------------------------------------|
| Documents | **135** real PDFs |
| Questions | **1,091** expert-annotated |
| Avg length | **~47.5 pages**, **~21,214** text tokens |
| Domains | **7** document-type families |
| Cross-page Qs | **~33.0%** |
| Unanswerable Qs | **~22.5%** (“Not answerable”) |
| Evidence sources | Pure-text, table, chart, image, layout, … |
| Data license | **CC BY-NC 4.0** (research / non-commercial) |
| Code license | Apache-2.0 |

Sample schema (official):

```json
{
  "doc_id": "Independents-Report.pdf",
  "doc_type": "Research report / Introduction",
  "question": "What's the percentage of people who are democrats and voted in the last election compared to the entire population in 2018?",
  "answer": "18.29%",
  "evidence_pages": "[3, 5]",
  "evidence_sources": "['Pure-text (Plain-text)']",
  "answer_format": "Float"
}
```

Download:

```python
from datasets import load_dataset
samples = load_dataset("yubo2333/MMLongBench-Doc/data")["train"]
```

PDFs live under upstream `./data/documents/` (also via HF; total dataset ~662 MB).

---

## 2. What the *original* system under test does

Official LVLM protocol (paper §4 / MATHVISTA-style):

```text
  PDF ──render──▶ page PNG screenshots
                         │
                         ▼
              LVLM freestyle long answer
                         │
                         ▼
              GPT-4o short-answer extractor
                         │
                         ▼
              rule-based eval_score() → Acc + F1
```

**Implication for EdgeQuake:** stuffing all pages as images into Mistral Small is *not* our product path and may hit API image-count limits (Mistral Vision docs: **max 8 images per request**). RAG adaptation is the honest product evaluation.

Published reference point (LVLM, not RAG): GPT-4o overall F1 ≈ **44.9%** (homepage / README). Use only as a *difficulty* anchor, never as a same-task baseline.

---

## 3. Official scoring (must preserve)

Upstream `eval/eval_score.py` (authoritative):

| `answer_format` | Rule |
|-----------------|------|
| `Int` | Exact int equality (pred coerced via float→int) |
| `Float` | Clean %/$ ; allow ×100 / ÷100 ; `isclose` rel_tol=0.01 |
| `Str` / `None` | Exact match for “hard” patterns (URL, email, date, phone, …); else **ANLS** (Levenshtein, threshold 0.5) |
| List-like | Length must match; element-wise exact or ANLS |

Aggregates (`eval_acc_and_f1`):

- **Accuracy** = mean score over evaluated samples  
- **Generalized F1** balances answerable vs predicted-answerable (unanswerable = `"Not answerable"`)  
- Slices: single-page / cross-page / unanswerable / evidence source / document type  

Harness **must vendor or submodule** this file (or a byte-compatible port) and record its git SHA in the scorecard.

---

## 4. Official answer extraction

Upstream `eval/extract_answer.py` calls OpenAI Chat Completions (default `gpt-4o`) with a fixed prompt to turn long analysis into a short prediction.

Fair options for SPEC-047 (pick one; record in scorecard):

| Mode | Extractor | When to use |
|------|-----------|-------------|
| `official` | GPT-4o | Default for comparability with literature |
| `mistral_judge` | `mistral-small-latest` | All-Mistral cost path; report separately |
| `dual` | both | Smoke/core: measure extractor sensitivity |

Never change the **rule scorer**. Only the extractor may vary, and only if labeled.

---

## 5. Why this benchmark stresses EdgeQuake

| Stress | EdgeQuake surface |
|--------|-------------------|
| Long PDFs | Vision PDF pipeline (`pdf_processing.rs` / pdf2md) + chunking + cost |
| Charts / images | Vision quality → markdown fidelity → retrieval |
| Tables | Structure preservation in markdown |
| Cross-page | Hybrid / graph multi-hop retrieval |
| Unanswerable | Refusal behavior; hallucination rate |
| Short gold answers | Forces precise generation, not vague summaries |

Paper finding relevant to RAG: many LVLMs underperform **OCR+LLM** baselines. EdgeQuake’s vision ingest is effectively a high-quality parse step — expect text/table slices to be stronger than chart/image unless vision is excellent.

---

## 6. Dataset hygiene for EdgeQuake

1. Cache under `~/.cache/edgequake/bench047/mmlongbench-doc/` (or `EDGEQUAKE_BENCH_CACHE`) — **not in git**.  
2. Verify PDF checksums against a manifest generated at download time.  
3. Map `doc_id` → local path 1:1; never rename files.  
4. Keep Q&A JSON immutable; smoke/core subsets are **filters**, not edits.  
5. Respect NC license in CI (research runners only; no commercial packaging of the PDFs).

---

## 7. Known pitfalls (fairness traps)

1. **Page index conventions** differ across forks (0- vs 1-based). Prefer official JSON `evidence_pages` as strings-of-lists; do not invent page filters for primary runs.  
2. **Updated Q&A (Sep 2025)** — pin dataset revision / commit; record in scorecard.  
3. **VLMEvalKit integration** exists upstream; do not mix VLMEvalKit LVLM runners with EdgeQuake RAG scores.  
4. **Average 47 pages** × vision = expensive. Smoke stratification is mandatory.  
5. **Unanswerable labeling** must survive extraction (`pred == "Not answerable"`).

---

## 8. Minimal mental model for implementers

```text
  For each question q on document d:

    ensure d ingested & status=Completed
    a_long = EdgeQuake.query(q, mode=hybrid)
    a_short = Extractor(q, a_long)
    score   = eval_score(gt=q.answer, pred=a_short, type=q.answer_format)

  Then Acc / F1 / slices → scorecard.json
```

Next: [003 Fair Evaluation Protocol](./003-fair-evaluation-protocol.md).
