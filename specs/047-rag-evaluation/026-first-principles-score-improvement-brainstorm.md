# 026 — Deep First-Principles Score Improvement Plan (post Medium ablation)

**Status:** Acc #5 DONE ([035](./035-acc5-w3-arith-v2-assessment.md)) · Acc/F1 SOTA remains Acc #2 (F1 still short)  
**Acc #2:** Acc **0.562** · F1 **0.480** · ChartEx **0.286** · Chart long **0.571**  
**Acc #5 W3-v2:** Acc **0.562** · F1 **0.457** · ChartEx **0.286** · Chart long **0.643** · **1251 hit**  
**Acc #4:** Acc 0.545 · F1 0.429 · Chart long 0.643 · soft W3 failed derived counts  

**FP chain:** densify ❌ → year-span ✅ gate → soft W3 ❌ → W3-v2 MUST+example **partial** (1251) · Acc recovered to #2 · F1 still −0.023.  

**Verdict:** Acc #2 Acc/F1 reference until F1 ≥ 0.480. Next: operand retrieval for 541/128 + wrong-arith guard / deterministic %×N · years Gen quote. Honesty: Acc↑ = Gen, not W1 pixels.  

**Implemented:** W1 stack · listmem · year-span · W3-arith-v2 (`grounding.rs`) · densify reverted · protocol `026-listmem-2026-07-15`  

**Not yet:** deterministic %×N tool · Indonesia 198 W2 · F1 ≥ Acc #2  

**Cross-ref:** [032](./032-post-acc3-fp-derived-counts-w3-arith.md) · [034](./034-acc4-w3-arith-assessment.md) · [035](./035-acc5-w3-arith-v2-assessment.md)

---



## 0. Executive verdict (one screen)

```text
┌────────────────────────────────────────────────────────────────────────────────┐
│  STRONGER VISION ALONE FAILED THE W1 GATE.                                     │
│  Chart a_in_e stayed 0.40 under mistral-medium-3-5. Acc/F1 +~0.03 is noise +   │
│  Table/Unans — NOT Chart representation solved.                                │
│                                                                                │
│  Acc/F1 ceiling = answerable soft scores. Dominant failure =                   │
│    zero + page_hit@5 + WRONG pred  (~35/75)                                    │
│  Not: retrieve miss (~5–10) · false refusal|hit (~6) · harness empty.          │
│                                                                                │
│  Hit+wrong mass is Table + Pure-text + Figure FIRST, Chart second.             │
│  Many zeros are FORMAT/EXTRACT (lists, units, language) not missing digits.    │
│                                                                                │
│  Next waves (ordered):                                                         │
│    W1a denser Pass A + crop telemetry → W1b numeric fail-closed specialize     │
│    → W1c tables as first-class → W2 mm page_start → W3 quote-from-context Gen  │
│    → W4 extract/format ANLS hygiene (harness+prompt)                           │
│  Do NOT: ban NA · chase TeleMM · Medium-only scale-up · fusion-first.          │
└────────────────────────────────────────────────────────────────────────────────┘
```

---



## 1. Score physics (what Acc and F1 actually are)

Official MMLongBench soft scoring (`tools/bench047/bench047/mmlongbench_eval_score.py`):


\mathrm{Acc}=\frac{1}{N}\sum_i s_i
\qquad
R=\frac{\sum_{g\neq\mathrm{NA}} s_i}{g\neq\mathrm{NA}}
\qquad
P=\frac{\sum_{g\neq\mathrm{NA}} s_i}{p\neq\mathrm{NA}}
\qquad
F1=\frac{2RP}{R+P}



| Symbol | Meaning for EdgeQuake                                                                         |
| ------ | --------------------------------------------------------------------------------------------- |
| s_i    | Soft score in [0,1] (exact / float / ANLS / list)                                             |
| R      | Mean soft Acc on **answerable** golds (≈ answerable Acc)                                      |
| P      | Answerable score mass / count of **predicted answerable**                                     |
| F1     | Harmonic mean — rises when answerable scores rise **and** you do not spray wrong non-NA preds |


**Implication:** Raising F1 ≠ separate product from Acc. Same forward channel: put correct answerable content into context, then pick it cleanly. False refusal hurts R; guessing on unanswerable hurts P.

---



## 2. Empirical facts (locked chart-8 · same physics)



### 2.1 Headlines


| Metric           | Small vision `P0_mm_ite` | Medium vision `P0_mm_ite_vision_medium` | Δ / read                            |
| ---------------- | ------------------------ | --------------------------------------- | ----------------------------------- |
| Acc              | 0.4154                   | 0.4430                                  | +0.028 · near noise + Table         |
| F1               | 0.2464                   | 0.2616                                  | +0.015 · same story                 |
| Chart Acc        | **0.227**                | 0.182                                   | **↓** — Medium did not help Chart   |
| Table Acc        | 0.193                    | **0.250**                               | ↑ Tables benefited more than Charts |
| Unans Acc        | 0.714                    | **0.786**                               | ↑ Helps F1 precision                |
| page_hit@5 (ans) | 0.773                    | 0.773                                   | Flat — retrieve not the lever       |
| Chart a_in_e     | **0.40**                 | **0.40**                                | **W1 gate FAIL**                    |
| overall a_in_e   | ~0.48                    | 0.48                                    | Flat                                |
| FR (ans)         | 11/75 (0.15)             | 15/75 (0.20)                            | Worse under Medium                  |
| FR | hit@5       | 6                        | 6                                       | Stable secondary                    |
| zero+hit+wrong   | **35**                   | **36**                                  | Dominant, unchanged                 |
| zero+miss page   | 10                       | 5                                       | Mild retrieve gain                  |




### 2.2 Causal split of answerable zeros (Small)

Among answerable n=75: perfect=17 · FR=11 · zero_wrong=45.


| Mode                      | n      | FP reading                                                |
| ------------------------- | ------ | --------------------------------------------------------- |
| zero + hit@5 + wrong pred | **35** | Context present; wrong pick / wrong format / wrong digits |
| zero + miss page          | 10     | Real W2 minority                                          |
| FR | hit@5                | 6      | Secondary Gen/refusal                                     |


**Hit+wrong by primary evidence source (Small):**


| Primary source | n hit+wrong | Note                               |
| -------------- | ----------- | ---------------------------------- |
| Table          | **11**      | Largest multimodal slice           |
| Pure-text      | **10**      | Not only a vision problem          |
| Figure         | 8           | Labels / callouts                  |
| Chart          | 4           | Still critical for Chart Acc slice |
| Layout / ?     | 2           | Tail                               |




### 2.3 Failure taxonomy (examples from predictions)

These are **different diseases** — one ticket cannot fix all:


| Cluster                    | Example (gold → pred)                                        | Root class           |
| -------------------------- | ------------------------------------------------------------ | -------------------- |
| Wrong digit / wrong series | `7→4`, `92%→48%`, `73.0→29.0`                                | **W1 / Gen pick**    |
| Partial page recall        | gold pages `[3,20]` but only page 3 retrieved                | **W2 dilution**      |
| Format / ANLS              | list gold vs bullet/`-` pred; `83672770` vs `83,672.77 lacs` | **Extract/format**   |
| Language drift             | English gold vs French pred (Medium)                         | **VLM language pin** |
| Near-miss string           | names list truncated; `41` vs `41 tweets`                    | **Extract hygiene**  |
| False NA                   | answerable → `Not answerable` with hit                       | **W3 Gen**           |


**Falsified hypothesis (025):** “Stronger Pass A/B VLM ⇒ Chart a_in_e ≥ 0.50.”  
**Surviving hypothesis:** Representation quality is limited by **prompt density, crop geometry, specialize fail-closed, and table path** — model capacity is not the binding constraint on this fixture.

---



## 3. First-principles axioms (score edition)


| ID  | Axiom                          | Score consequence                                               |
| --- | ------------------------------ | --------------------------------------------------------------- |
| FP1 | Information only flows forward | Query LLM cannot invent chart/table cells absent from chunks    |
| FP2 | Measure the bottleneck         | Gate on Chart/Table `a_in_e`, then Chart/Table Acc, then Acc/F1 |
| FP3 | One causal change              | Never ship Medium + denser prompt + crop + grounding in one run |
| FP4 | Honesty > Acc inflation        | Unans Acc ≥ ~0.70; never ban NA to juice Acc/F1                 |
| FP5 | Soft score ≠ semantic success  | Format failures look like Acc zeros — instrument separately     |
| FP6 | Slice honesty                  | Chart Acc can fall while Acc rises (Medium) — report all        |
| FP7 | Code is law                    | Change symbols below; do not invent a parallel ingest           |


---



## 4. End-to-end information flow (code is law)

```text
PDF bytes
  │
  ├─ Pass A  VisionPdfConverter::convert
  │     RAG_PAGE_VISION_SYSTEM_PROMPT          [vision_prompts.rs]
  │     fig/table PNG assets
  │     chart_residual_candidate_pages         [chart_crop.rs]  ← SKIPS if fig OR table exists
  │     <!-- edgequake-page:N --> + <drawing/>
  │
  ├─ Pass B  analyze_multimodal_images (opts=ite)
  │     classify → specialize_image_analysis   [image_specialize.rs]
  │     CHART/TABLE prompts                    [prompts.rs]
  │     merge_pass_a_dump_when_sparse          [soft — only if ZERO digits]
  │
  ├─ Index   enrich_processed_text_with_mm_chunks
  │     append_mm_chunks_to_text @ DOC END     [chunks.rs]  ← wrong page_start risk
  │     PageAwareChunking → embed
  │
  ├─ Query   hybrid + document_scope
  │     page_hit@k from chunk page_start
  │     grounding_instructions                 [grounding.rs]
  │
  └─ Score   extract short answer → soft s_i → Acc / F1
```



### 4.1 Code levers ↔ score physics


| Stage         | Symbol / file                                   | Failure it can fix                                    | Cannot fix alone                  |
| ------------- | ----------------------------------------------- | ----------------------------------------------------- | --------------------------------- |
| Pass A prompt | `vision_prompts.rs`                             | Missing digit tables in page MD                       | Wrong Gen pick among dense digits |
| Residual crop | `chart_crop.rs::chart_residual_candidate_pages` | Pass B never sees plot when loose ImageXObject exists | Wrong OCR of visible number       |
| Specialize    | `image_specialize.rs` + `prompts.rs`            | Prose-only Chart extract                              | Format ANLS in harness            |
| Soft merge    | `merge_pass_a_dump_when_sparse`                 | Empty specialize                                      | Wrong digits already in Pass A    |
| mm page stamp | `chunks.rs`                                     | W0/Gen page grounding                                 | Missing numbers                   |
| Grounding     | `grounding.rs`                                  | FR | hit; cluttered pick                              | Absent numbers                    |
| Extract       | `bench047/extract.py`                           | Format zeros                                          | Representation                    |




### 4.2 Known skip / soft paths (attack surface)

1. **Crop skip law:** residual crops only when `no_fig && no_table` — mis-bounded “figure” ImageXObject ⇒ no tight chart crop for Pass B.
2. **Numeric soft-fail:** specialize retries/merges only when description has **zero** digits — wrong digit dumps pass the gate.
3. **Mm sidecar at doc end:** inherits last page’s `page_start` under naive split.
4. **Language:** Chart prompt says “requested language”; Medium produced French on English gold → Acc zero despite semantic match.

---



## 5. Deep brainstorm — idea space (then kill)



### 5.1 Representation (W1) — keep / kill


| Idea                                                       | Keep?                           | Why                                                                                     |
| ---------------------------------------------------------- | ------------------------------- | --------------------------------------------------------------------------------------- |
| Stronger VLM again (Large)                                 | Later                           | Medium already failed a_in_e gate; cost ≠ proven lift                                   |
| Denser Pass A chart/table dump + forced GFM schema         | **Yes**                         | Binding channel; prompts already ask for tables — enforce + verify                      |
| Always emit chart-region crop even if fig exists           | **Yes if telemetry shows miss** | Code hole real; unproven Acc causation until coverage metric                            |
| Dual-pass: structure then detail (Twin-T style)            | Research                        | CVPR’26 chart/table specialist — optional later expert model, not Wave 1                |
| OCR engine sidecar (MinerU/etc.) for tables                | Maybe                           | Literature: structure-preserving OCR can match VLMs on tables; adds pipeline complexity |
| Fail-closed specialize when key_values empty / density low | **Yes**                         | Stronger than “any digit”                                                               |
| Correctness check: re-read crop vs key_values              | Hard                            | Expensive; Wave 2+                                                                      |
| Pixel estimate when labels missing                         | **No**                          | Violates FP4 / invent ban                                                               |
| Ban residual crop area gates                               | **No**                          | Causes junk crops / cost                                                                |




### 5.2 Retrieval (W2) — keep / kill


| Idea                        | Keep?   | Why                                            |
| --------------------------- | ------- | ---------------------------------------------- |
| Fix mm `page_start`         | **Yes** | Correctness + Gen `page=`                      |
| Hybrid → Mix RRF            | Later   | page_hit@5 already 0.77; Acc bulk is hit+wrong |
| Boost modality=chart chunks | Maybe   | After numbers exist                            |
| Increase top-k              | Low     | Dilution risk; page_hit@1 already weak         |
| Gold evidence_pages leak    | **No**  | Protocol cheat                                 |




### 5.3 Generation / extract (W3–W4) — keep / kill


| Idea                                                                | Keep?    | Why                             |
| ------------------------------------------------------------------- | -------- | ------------------------------- |
| Quote-matching grounding when sparse numeric + hit                  | **Yes**  | Attacks wrong-with-hit after W1 |
| Ban “Not answerable”                                                | **No**   | Kills Unans Acc / F1 honesty    |
| Stronger query LLM                                                  | After W1 | Digits must exist first         |
| Extract prompt: strict JSON short answer, English-only, strip units | **Yes**  | Many zeros are format           |
| Official GPT-4o extractor ablation                                  | Optional | Isolate harness vs system       |
| Language pin on Pass B (`en`)                                       | **Yes**  | Medium French pred              |




### 5.4 Ops / measurement — keep / kill


| Idea                                       | Keep?   | Why                                     |
| ------------------------------------------ | ------- | --------------------------------------- |
| Crop coverage telemetry in ingest.jsonl    | **Yes** | Prove crop hypothesis                   |
| Fidelity by Chart **and Table** gate ≥0.50 | **Yes** | Medium showed Table moves independently |
| Format-failure classifier in bench047      | **Yes** | Separate ANLS/list fails from W1        |
| Full 135-doc before Chart a_in_e moves     | **No**  | Wastes money                            |
| TeleMM Acc as success                      | **No**  | Different task                          |


---



## 6. Ranked engineering waves (data-adjusted)



### Wave 0 — Already done (do not redo)


| Ticket                                 | Result                                 |
| -------------------------------------- | -------------------------------------- |
| `ite` multimodal                       | Live; valid smokes                     |
| MV-18/19 denser prompts / crops        | Chart a_in_e ~0.32→0.40                |
| EQ-047-W1-vision Medium ablation [025] | **Gate FAIL** — Chart a_in_e flat 0.40 |




### Wave 1 — Representation density (primary Acc/F1)

**Goal:** Chart `a_in_e` ≥ **0.50** and Table `a_in_e` ≥ **0.55** on chart-8 fidelity (n≥15 Chart / n≥10 Table).


| ID                           | Ticket                                                                                                                      | Code locus                                      | FP3 experiment                             |
| ---------------------------- | --------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------- | ------------------------------------------ |
| **EQ-047-W1-dense-A**        | Harden Pass A: require GFM data table for every chart; multi-panel explicit; reject invent; English output pin              | `vision_prompts.rs` · `VisionPdfConverter`      | Small vision only vs locked Small baseline |
| **EQ-047-W1-crop-telemetry** | Log per page: fig/table/residual crop fired + bbox area                                                                     | `chart_crop.rs` · `vision.rs` · ingest metadata | Measure before Acc claim                   |
| **EQ-047-W1-crop-expand**    | If Chart-miss pages show fig-present + residual-skip, allow residual **alongside** fig (ink-gated)                          | `chart_residual_candidate_pages`                | Ablation only after telemetry              |
| **EQ-047-W1-dense-B**        | Specialize fail-closed: require `key_values.len()≥K` or `data_table_md` non-empty; retry once; else keep Pass A dump loudly | `image_specialize.rs` · `prompts.rs`            | Density gate ≠ correctness gate            |
| **EQ-047-W1-table**          | First-class table specialize path (`t`): wide tables, multi-page continuity, unit columns                                   | `prompts.rs` TABLE_* · analyzer                 | Tables dominate hit+wrong                  |


**Gate to exit Wave 1:** Chart a_in_e ≥ 0.50 · Table a_in_e ≥ 0.55 · Chart Acc ↑ vs 0.227 · page_hit@5 ≥ 0.75 · Unans Acc ≥ 0.70.

### Wave 2 — Index honesty (secondary Acc, primary diagnostics)


| ID                           | Ticket                                                              | Code locus                                |
| ---------------------------- | ------------------------------------------------------------------- | ----------------------------------------- |
| **EQ-047-W2-mm-page**        | Stamp `page_start` from drawing/asset page when appending mm chunks | `chunks.rs` · page-aware chunker          |
| **EQ-047-W2-modality-boost** | Optional: prefer chart/table modality chunks in hybrid merge        | query fusion — **only after Wave 1 gate** |




### Wave 3 — Generation under dense context


| ID                  | Ticket                                                                                                      | Code locus                |
| ------------------- | ----------------------------------------------------------------------------------------------------------- | ------------------------- |
| **EQ-047-W3-quote** | When page_hit and numeric context present: instruct quote exact cell/value; still allow honest NA if absent | `grounding.rs`            |
| **EQ-047-W3-lang**  | Force Pass B / query extract language = English for SPEC-047                                                | specialize + `extract.py` |


**Gate:** FR overall ↓ without Unans Acc ↓ · wrong-with-hit ↓ among Chart/Table.

### Wave 4 — Soft-score hygiene (F1/Acc without new facts)


| ID                       | Ticket                                                                                     | Code locus             |
| ------------------------ | ------------------------------------------------------------------------------------------ | ---------------------- |
| **EQ-047-W4-extract**    | Short-answer extract: strip units when gold is bare number; list as JSON array; no bullets | `bench047/extract.py`  |
| **EQ-047-W4-format-tag** | Tag predictions `format_fail_suspected` when ANLS would pass after normalize               | diagnostics            |
| **EQ-047-W4-judge**      | Optional P0_official_extractor GPT-4o vs Mistral — isolate harness                         | profile already exists |




### Wave 5 — Only if Wave 1–3 stall

- Stronger vision **again** (Large) as labeled ablation  
- Structure-preserving OCR sidecar for tables  
- Cross-page multi-hop (Cross-page Acc 0.14)  
- Mix RRF vs hybrid

---



## 7. F1-specific improvement map

```text
                    ┌── raise answerable soft scores (R)
                    │     = Wave 1 digits + Wave 3 correct pick + Wave 4 format
   F1 ──────────────┤
                    │
                    └── raise precision (P)
                          = fewer wrong non-NA preds
                          = keep Unans Acc high (do not ban NA)
                          = cut confident wrong digits (Wave 1+3)
```


| Lever                | Effect on F1 | Evidence                         |
| -------------------- | ------------ | -------------------------------- |
| Chart/Table a_in_e ↑ | **Primary**  | Ceiling on answerable scores     |
| Cut FR               | Helps R      | Medium raised FR → fought F1     |
| Unans Acc ↑          | Helps P      | Medium Unans 0.71→0.79 helped F1 |
| Format normalize     | Cheap F1/Acc | List/unit zeros                  |
| Ban NA               | **Harms**    | Classic Acc cosplay              |


---



## 8. Measurement protocol (battle-tested)



### 8.1 Fixture & pins

- Fixture: `smoke_chart_doc_ids_v1.txt` (8 docs / 117 Q)  
- Profile default for Acc chain: `P0_mm_ite` (Small+Small+ite+dscope)  
- Ablations labeled in scorecard `pins.vision_model` / `profile_id`  
- Never overwrite prior SUMMARY without snapshot dir



### 8.2 Mandatory gates before Acc/F1 storytelling


| Gate                  | Threshold                                 | On fail                                            |
| --------------------- | ----------------------------------------- | -------------------------------------------------- |
| `valid`               | true                                      | Fix ops                                            |
| ingest_coverage       | ≥ 0.90                                    | Retry / lower concurrency                          |
| fidelity `gateable`   | true (full answerable n)                  | Re-run `bench047 fidelity` without `--max-samples` |
| Chart **a_in_e_long** | ≥ **0.50**                                | Stay in Wave 1 (raw a_in_e is not the gate)        |
| Table **a_in_e_long** | ≥ **0.55**                                | Stay in W1-table                                   |
| page_hit@5            | ≥ 0.75                                    | Check W2 / scope                                   |
| Unans Acc             | ≥ 0.70                                    | Revert Gen/extract                                 |
| ΔAcc claim            | ≥ +0.05 **and** Chart **exclusive** Acc ↑ | Else “noise / list-normalize / slice trade”        |


**Protocol law (**`026-hardened-2026-07-15`**):**

1. Acc/F1 = official MMLongBench soft-score (vendored `eval_score`) — keep.
2. Do **not** gate on raw Chart a_in_e (short golds like `"6"` inflate hits).
3. Cross-run fidelity compares require **same** `n_answerable_audited` (all answerable).
4. Report multi-label Chart Acc **and** exclusive Chart Acc (`len(sources)==1`).
5. Attribute Acc Δ: `list_gold` / `unanswerable` / `other_answerable` — list mass ≠ W1 win.
6. Store `pred_raw` alongside `pred` (W4 normalize transparency).



### 8.3 New diagnostics to add (instrumentation tickets)

1. `crop_coverage`: pages with chart-like content vs residual fired
2. `specialize_numeric_density`: mean `#key_values` per Chart item
3. `format_fail_rate`: normalize-then-ANLS rescues
4. Causal ledger in SUMMARY: zero+hit+wrong / FR|hit / miss counts (auto)
5. ~~short-needle a_in_e~~ → **shipped** as `a_in_e_long` + `short_needle_fp_suspect`



### 8.4 Commands

```bash
# Baseline Acc chain
make bench047-smoke

# After Wave 1 code change (still Small vision unless labeled)
python3 -m bench047.cli fidelity smoke
# Gate Chart/Table a_in_e_LONG before celebrating Acc (never --max-samples for gates)

# Compare with attribution
python3 -m bench047.cli report specs/.../smoke-chart8-026-dense-... \
  --compare specs/.../smoke-chart8-ite-sota-...

# Medium vision only as labeled ablation (already proven insufficient alone)
make bench047-smoke-vision-medium
```

---



## 9. SOLID / DRY implementation rules


| Principle | Rule                                                                                                      |
| --------- | --------------------------------------------------------------------------------------------------------- |
| SRP       | Pass A prompt ≠ Pass B specialize ≠ Gen grounding ≠ harness extract — separate PRs                        |
| OCP       | New profiles for ablations (`P0_mm_ite_*`); never silently retarget locked Acc chain                      |
| DIP       | Bench depends on `BenchProfile` pins, not scattered env strings                                           |
| DRY       | Vision model constants in `profiles.py`; prompts SSOT in `vision_prompts.rs` / `prompts.rs`               |
| Tests     | Unit: crop eligibility · numeric density gate · mm page stamp · extract normalize · doctor vision catalog |
| E2E       | Existing `e2e_spec047_vision_drawing_pipeline` + chart-8 smoke gate                                       |


---



## 10. Risk register


| Risk                                         | Mitigation                                                   |
| -------------------------------------------- | ------------------------------------------------------------ |
| Denser prompts → longer pages → cost/timeout | Cap table rows; concurrency pins; fail-closed stale PDFs     |
| Crop expansion → junk crops                  | Ink area gates; telemetry first                              |
| Gen “always quote” → Unans Acc drop          | Keep NA allowed; gate Unans Acc                              |
| Format normalize hides real errors           | Tag suspected format_fail; don’t auto-boost Acc in scorecard |
| Language pin breaks multilingual docs        | SPEC-047 English-only pin; product multilingual separate     |
| Acc ↑ Chart ↓ (Medium pattern)               | Slice gates mandatory                                        |


---



## 11. Success definition (product)

**Minimum shippable improvement (chart-8):**

1. Chart **a_in_e_long** ≥ 0.50 · Table **a_in_e_long** ≥ 0.55 (full-n fidelity, gateable=true)
2. Chart **exclusive** Acc ≥ 0.30 · Table exclusive Acc ≥ 0.30 (also report multi-label)
3. Acc ≥ 0.48 **or** F1 ≥ 0.32 with Unans Acc ≥ 0.70
4. page_hit@5 ≥ 0.75 · valid=true
5. Paired Acc Δ attribution shows material `other_answerable` (not only `list_gold`) if claiming W1

**Not success:** Acc +0.02 with flat a_in_e_long; Chart exclusive Acc flat/regression; TeleMM Δ storytelling; celebrating Acc when list_gold mass dominates Δ.

---



## 12. Suggested sprint order (concrete)

```text
Sprint A (1–2 days)
  [x] W1-dense-A Pass A prompt harden + English pin + unit tests
  [x] W1-crop-telemetry fields in PDF metadata / HTML comment
  [x] W4-extract minimal list/number normalize (low risk)
  [x] chart-8 re-ingest Small · fidelity gate  → GATES FAIL (Chart/Table a_in_e)

Sprint B (depends on A telemetry)
  [x] W1-dense-B specialize density fail-closed + retry
  [ ] W1-table specialize densify
  [x] chart-8 · fidelity + Acc slices  → Acc↑ F1↑ but Chart Acc↓ / a_in_e flat-low

Sprint C (NOW: telemetry shows Chart-miss ∩ residual-skip HIGH)
  [x] W1-crop-expand  ← residual alongside fig (ink-gated); tables still skipped
  [x] W2-mm-page (landed early — correctness fix, low risk)
  [x] crop-expand Acc re-run `smoke-chart8-026-crop-expand-20260715-0535` (valid, n=117)
      Acc 0.506 / F1 0.403 vs dense 0.500 / 0.374 (ΔAcc +0.006 noise: 13↑12↓)
      Chart a_in_e_long **0.214 FAIL** (identical to dense) · Table long 0.412 FAIL
      Chart exclusive Acc 0.286 vs dense 0.143 — **only mover is MMMU extract unwrap**
      (`MMMU` vs `["MMMU"]`), not crop surface. Causal audit stands: ink + fig>chart inject
      + gold-page∩write=∅. Wave 1 Chart gate NOT met.
  [x] W1-coexist — chart drawing+href alongside figs (tables still block);
      stop inject rewrite chart→fig. Unit+contract green.
  [x] W1-fig-as-chart — when alongside ink residual empty, promote
      `page-NNNN-fig-01.png` → `page-NNNN-chart.png` (cap=12). Unit tests green.
      Lit: charts need first-class retrieval units (NAACL'24 chart retrieval;
      RAG-Anything panels as nodes; LFRAG block-level). Full-bleed fig = crop
      the figure, not invent empty residual.
  [x] chart-8 Acc after coexist `…-1547` — Acc↑ F1↑ but Chart a_in_e_long **flat 0.214 FAIL**
      ChartEx↓ = MMMU `["MMMU"]` vs `"MMMU"` only; attribution Acc↑ = list+unans.
      Coexist MD verified (2311 wr=4). See 027 analysis.
  [x] prebuild fig-as-chart binary (cargo build -p edgequake)
  [ ] rebuild/restart + Acc after coexist+fig-as-chart → Chart a_in_e_long gate
  [ ] W1-next: gold-page residual ranking; W1-table densify
  [ ] W3-quote + W3-lang  (FR rose 0.15→0.23 — secondary)
  [ ] chart-8 · wrong-with-hit ↓ + Chart a_in_e_long ≥ 0.50

Sprint D
  [ ] core (~40) if Sprint C Chart gate passes
  [ ] optional Medium re-ablation ON TOP of denser prompts (FP3 labeled)
```

---



## 13. Bottom line

> **To improve Acc and F1, raise answerable soft scores by putting correct chart/table (and text) facts into indexable markdown, then picking them without format loss — while keeping unanswerable honesty.**  
> Medium vision proved **capacity ≠ fidelity** on this fixture. The next binding work is **Wave 1 density + tables + telemetry**, not another model bump, not fusion, and not ban-NA prompting.

---



## Appendix A — Artifact pointers


| Run             | Path                                                        |
| --------------- | ----------------------------------------------------------- |
| Small ite SOTA  | `e2e/artifacts/smoke-chart8-ite-sota-20260715-020211/`      |
| Medium vision   | `e2e/artifacts/smoke-chart8-vision-medium-20260715-024736/` |
| 026-dense       | `e2e/artifacts/smoke-chart8-026-dense-20260715-0348/`        |
| 026 crop-expand | `e2e/artifacts/smoke-chart8-026-crop-expand-20260715-0535/` |
| Live smoke      | `e2e/artifacts/smoke/`                                      |




## Appendix B — Key symbols cheat sheet


| Symbol              | Path                                                        |
| ------------------- | ----------------------------------------------------------- |
| Pass A prompt       | `edgequake-pdf/src/vision_prompts.rs`                       |
| Residual crops      | `edgequake-pdf/src/chart_crop.rs`                           |
| Pass B specialize   | `edgequake-api/src/services/multimodal/image_specialize.rs` |
| Chart/Table prompts | `…/multimodal/prompts.rs`                                   |
| Mm append           | `…/multimodal/chunks.rs`                                    |
| Grounding           | `edgequake-query/.../grounding.rs`                          |
| Soft Acc/F1         | `tools/bench047/bench047/mmlongbench_eval_score.py`         |
| Fidelity            | `tools/bench047/bench047/fidelity.py`                       |


