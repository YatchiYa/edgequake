# 023 — First Principles: Next Improvements After MV-18/19

**Status:** ASSESSMENT (2026-07-11) · no Acc heuristics  
**Peers:** [015](./015-modality-aware-vision-improvement-plan.md) · [022](./022-reassessment-2026-07-11.md) · artifact [`smoke-post-mv18-full-chart`](./e2e/artifacts/smoke-post-mv18-full-chart/)  
**Canvas:** `spec047-next-from-mv18.canvas.tsx`  
**Law:** Information only flows forward · measure the bottleneck · one causal change per experiment · honesty > Acc inflation · code is law

---

## 0. Where we are (physics, not vibes)

| Metric | Pre-MV18 | Post-MV18/19 | Post-MV24 | Gate |
|--------|----------|--------------|-----------|------|
| Chart `answer_in_evidence` | ~0.32 | **0.409** (n=22) | **0.409** | G-A ≥ **0.50** — FAIL |
| Chart Acc | 0.136 | **0.182** | **0.182** | Flat |
| Overall Acc | 0.427 | **0.423** | **0.433** | Small ↑ (page_hit) |
| page_hit@5 | 0.76 | 0.72 | **0.80** | R healthy |
| Unanswerable Acc | 0.833 | 0.786 | 0.738 | Protect ≥ ~0.70 |

**MV-24 result:** Crops fired on **8/8** docs (hi-res ink crops). Chart a_in_e **unchanged** — perception budget alone is not the remaining omission bottleneck. Prefer **MV-26/27/28** next (routing, specialize soft-fail, page-local dump; a_in_doc 0.55 − a_in_e 0.41).

**Chart miss taxonomy (n=22):**

| Mode | Count | Meaning |
|------|-------|---------|
| Hit | 9 | Scalar %/float on correct page — pipeline works |
| Total omission | ~10 | Gold string never in evidence-page markdown |
| Wrong page | 3 | Number in doc, not on gold `evidence_pages` |

---

## 1. Irreducible axioms (apply before any ticket)

| ID | Axiom | Implication |
|----|-------|-------------|
| FP1 | Information only flows forward | Improve what vision **writes**; query prompts cannot fix missing numbers |
| FP2 | Measure the bottleneck | Gate on Chart `answer_in_evidence` (+ Acc same direction); not Acc alone |
| FP3 | One causal change per experiment | Crop ≠ DPI ≠ routing ≠ query fusion |
| FP4 | Honesty > Acc inflation | Never invent unreadables; never ban “Not answerable” |
| FP5 | Evidence page is the fidelity atom | Tail `<!-- multimodal-chunks -->` does **not** count for G-A |
| FP6 | Code is law | Extend Pass A/B SSOT; don’t fork a parallel “chart pipeline” |
| FP7 | Gold pages are eval-only | Never inject `evidence_pages` into retrieve |

---

## 2. Code diagnosis (SSOT map)

```text
PDF
 └─ Pass A  vision_prompts::RAG_PAGE_VISION_SYSTEM_PROMPT
            VisionPdfConverter (.system_prompt)          # dump readable numbers
            write_page_png_assets (FULL PAGE, max 2000px) # MV-21 drawing
 └─ Pass B  classify → should_specialize_as_chart?
            chart_analysis_messages → key_values + data_table_md
            FAIL-OPEN → keep weak classify on error
 └─ Render  inline: # name / **Type:** Chart / description   (ON page — fidelity sees this)
            append: [Chart Name]… under multimodal-chunks     (OFF page — fidelity IGNORES)
 └─ Chunk   atomic_blocks preserve chart/table regions
 └─ Query   modality_retrieve (naive/FTS only) · page_hit@5 ~0.72
```

### Root limits after MV-18/19 (causal, not speculative)

1. **Whole-page PNG @ `max_rendered_pixels: 2000`** — chart marks are often illegible; VLM lawfully omits (`vision.rs`).  
2. **No chart-region crop (MV-24 unimplemented)** — literature (DePlot / CharTool / AgentsCamp) converges on **crop → table**, not full-page caption.  
3. **Weak drawing caption (`"Page N"`)** — `context_suggests_chart` rarely fires; Screenshot/Other misroutes skip dense specialize (`vision_markdown.rs`, `prompts.rs`).  
4. **Specialize fail-open** — JSON/empty extract keeps classify prose without `key_values` (`image_specialize.rs`).  
5. **Gold answer shape** — counts (`872`), lists, multi-hop need **compute over extracted table**, not more retrieval.  
6. **Wrong-page landings** — numbers exist elsewhere in doc; fidelity page-scoped; crops + page-local dump help.

### What is already solid (do not reopen)

- Pass A number-dump prompt + Pass B schema (`data_table_md`)  
- Drawing tags + ite analyze path  
- Atomic chart/table chunking  
- Fail-open modality filter (retrieval assist, not Acc patch)  
- Honest refusal / grounding

---

## 3. External research (aligned with first principles)

| Source | Finding | EdgeQuake mapping |
|--------|---------|-------------------|
| **DePlot** (plot→table then LLM reason) | Separate **extract** from **reason** | Keep Pass B table dump; add crop; optional query-time compute over table |
| **ExChart / ExChart-Bench (2026)** | MLLMs fail precise value recovery without labels; progressive coord→table | Don’t invent; improve perception (crop/DPI) before Acc prompts |
| **CharTool (2026)** | Crop tool + code compute | MV-24 crops + post-extract arithmetic for count/gap golds |
| **Chart-MRAG / AgentsCamp** | Full-page weak for dense charts; crop+caption or crop+VLM re-read | MV-24; optional Phase D retrieval-time re-read **after** G-A |
| **BigData Boutique MM-RAG 2026** | Caption for search; **data extract** for numeric QA | We chose extract — stay the course |
| **MMLongBench-Doc** | Chart slice is hard; unanswerable must stay | Protect Unans Acc |

**Reject from research fashion:** ColPali-only Acc chase, ban-NA, gold-page retrieve, “describe chart better” without table schema.

---

## 4. Lawful improvement queue (ranked)

Score = **causal clarity × measurability × no Acc heuristics**.

| # | Ticket | Change | Gate | Effort |
|---|--------|--------|------|--------|
| **1** | **EQ-047-MV-24** ✅ Chart-region crops | Pass A–gated hi-res re-render + deterministic ink-bbox crop; drawing override + specialize-time fail-open crop | Chart a_in_e ↑ toward **0.50** on frozen 22 | L |
| **2** | **EQ-047-MV-25** Render fidelity | Raise DPI / `max_rendered_pixels` for pages with chart candidates (budgeted) | More `**Data table:**` / `key_values` on gold pages | S |
| **3** | **EQ-047-MV-26** ✅ Routing hygiene | Feed Pass A body + nearby headings into specialize context; route Screenshot/Other when page has chart-like tables; log specialize fail rate | ↑ `drawing_chart` hits; ↓ fail-open rate | S |
| **4** | **EQ-047-MV-27** ✅ Specialize retry / soft-fail | One repair with stricter “JSON+key_values only”; if still empty, keep Pass A table (not weak classify) | Fewer omission misses | S |
| **5** | **EQ-047-MV-28** ✅ Fidelity-page integrity | Ensure specialize markdown replaces drawing **inside** page marker; viewer `![…](assets/…)`; optional duplicate `data_table_md` into Pass A section | Shrink a_in_doc − a_in_e gap | S |
| **6** | **EQ-047-MV-17** Number-token recall metric | Bench: gold numeric tokens ⊂ evidence-page markdown | Early signal without Acc noise | S |
| **7** | **EQ-047-MV-29** Table-native compute (post G-A) | For count/list golds: deterministic ops over extracted markdown tables at **query** (cite table cells) | Chart Acc ↑ when a_in_e already true | M |
| **8** | Query W2 (only after G-A) | Truncation / page_hit for Chart-hit rows | `retrieval_miss_given_rep_ok` ↓ | S |

**Parallel (orthogonal, not Chart Acc):** B3 Mix ablation · L-B2 lineage telemetry.

---

## 5. Explicit rejects (flaky heuristics)

- Ban / soft-ban “Not answerable”  
- Gold `evidence_pages` in retrieve  
- Acc-only system-prompt patches at query  
- Expanding `query_prefers_chart_modality` to chase Acc before G-A  
- Generic captioning without `key_values` / `data_table_md`  
- Assuming appended `[Chart Name]` mm-chunks satisfy G-A (no page marker)  
- Mid-run provider swaps / leaderboard cosplay  

---

## 6. Experiment protocol (locked)

```text
Workspace twin of be4c40a9 (or fresh soft-resume)
Fixture: smoke_chart_doc_ids_v1.txt
One change per run
Primary: Chart answer_in_evidence (n=22) + mm_probe (Chart/KeyValues/DataTable)
Secondary: Chart Acc, overall Acc, Unans Acc, page_hit@5
Stop and reassess when G-A passes (≥0.50) before starting query-lane Acc work
```

---

## 7. Recommended next 48h

1. ~~**Spec + spike MV-24**~~ ✅ Landed + Acc gate run (`smoke-post-mv24-chart-crops`).  
2. ~~**Re-ingest chart fixture**~~ ✅ Crops fired 8/8; **G-A FAIL** (a_in_e flat 0.409).  
3. ~~**MV-26** routing hygiene + **MV-27** specialize soft-fail~~ ✅ (Pass A caption context + soft-fail dump).  
4. ~~**MV-28** page-local dump + viewer images~~ ✅ (`![…](assets/…)` + mm-assets GET + AuthenticatedMarkdownImage).  
5. **Re-ingest Acc gate** on `smoke_chart_doc_ids_v1` after MV-26/27/28; target Chart a_in_e ≥ 0.50.  
6. **Do not** start B3 Mix as the Chart Acc lever (orthogonal).  

---

## Citation

Ma et al., MMLongBench-Doc, arXiv:2407.01523.  
Liu et al., DePlot, ACL Findings 2023.  
ExChart (CHI’26 materials): https://exchart.github.io/  
CharTool, arXiv:2604.02794.
