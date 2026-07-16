# EdgeQuake Accuracy Improvement Plan — First Principles

**Status:** Working plan  
**Date:** 2026-07-16  
**Requested path:** `specs/055-accuracy-improve-plan/`  
**Canonical SPEC ID:** TBD — `SPEC-055` is already used by
`specs/055-release-plan/`; do not register this document as a second SPEC-055.
**Baseline:** SPEC-047 Phase B CORE @40, protocol `026-listmem-2026-07-15`

---

## 1. Executive decision

The next accuracy program should optimize the information path in this order:

```text
PDF
  → W1 representation: is the answer-bearing evidence preserved?
  → W2 retrieval: is the complete evidence set selected?
  → W3 composition: does the model produce the right typed answer?
  → W4 verification: is the answer supported, correctly formatted, and safe?
```

The @40 evidence does **not** justify another broad prompt-densification or
larger-vision-model campaign. The largest measured error mass is downstream:

1. **117 answerable zero-score questions had the gold page in top-5**, so the
   primary near-term opportunity is answer composition, evidence selection
   within a hit page, arithmetic/list handling, and calibrated refusal.
2. **70 answerable zero-score questions missed the gold page at top-5**, so the
   second opportunity is page-level, multi-granular, cross-page retrieval.
3. **26 unanswerable questions received wrong non-abstaining answers**, so
   indiscriminately suppressing `Not answerable` would improve recall while
   damaging precision and F1.
4. Representation is still incomplete, but the full-n Wave-1 gates pass:
   Chart long `0.600` and Table long `0.585`. W1 changes must therefore be
   targeted at measured Figure/Table/Layout misses, not generic verbosity.

**Program objective:** raise honest CORE @40 from Acc `0.4581` / F1 `0.3564`
to at least Acc `0.500` / F1 `0.400`, while retaining full ingest, unanswerable
accuracy, latency bounds, and Chart/Table representation gates.

This requires about **16.6 additional full-credit equivalents over 397
questions**. It does not require solving every error class.

---

## 2. Baselines and comparability

### 2.1 Locked EdgeQuake baselines

| Evaluation | Docs | Acc | F1 | Validity |
|---|---:|---:|---:|---|
| Chart-8 Acc #2, fig-as-chart | 8 | `0.562` | `0.480` | July 2026 Acc/F1 reference |
| Chart-8 Acc #5, W3-arith-v2 | 8 | `0.562` | `0.457` | Acc tie; F1 below Acc #2 |
| CORE @40, Acc #5 stack | 40 | `0.4581` | `0.3564` | 40/40 ingested; 397/397 scored |

These are different fixtures. Chart-8 is a targeted causal smoke; CORE is a
stratified scale test. A CORE improvement does not automatically replace the
Chart-8 reference, and a Chart-8 improvement does not prove broad CORE lift.

**Sequencing note (fixture-dependent):** On chart-8, representation / chart
digit fidelity remains the primary Acc bottleneck (`a_in_e` / Chart Acc). On
CORE @40, the largest absolute correctable mass is W3/W4 (evidence present +
page hit + wrong/refusal). This plan follows the CORE budget for the scale
program; chart-8 product work stays representation-first and is tracked
separately. Do not collapse the two into one causal run.

### 2.2 External references

- The official MMLongBench-Doc benchmark contains long, multimodal documents,
  cross-page questions, and unanswerable questions. Its paper identifies
  perceptual errors, hallucinated evidence, incomplete evidence, irrelevant
  answers, extractor errors, and reasoning errors as major failure modes.
- The official GPT-4o LVLM F1 of approximately `44.9%` is a difficulty
  reference only. That setup consumes page screenshots and is not directly
  comparable to EdgeQuake's RAG adaptation.
- MHier-RAG reports Acc `52.3%` / F1 `46.0%` with ten parent pages on the full
  benchmark. Its ablations support parent-page retrieval, cross-page summary
  retrieval, multimodal evidence, and a bounded evidence set. This is an
  architectural reference, not a directly comparable EdgeQuake target.
- HiEvi-RAG (July 2026 preprint) reports gains from hierarchical question
  decomposition, visual page retrieval, evidence-page verification, and
  iterative memory. Its own ablations attribute the largest average loss to
  removing evidence verification, followed by decomposition and iterative
  reasoning. It is a frontier hypothesis source, not production proof.

---

## 3. Measured @40 state

### 3.1 Headline

| Metric | @40 |
|---|---:|
| Acc | `0.4581` |
| F1 | `0.3564` |
| Ingest coverage | `1.000` |
| Questions | `397` |
| Perfect scores | `176` |
| Zero scores | `213` |
| Partial scores | `8` |
| Answerable questions | `278` |
| Answerable Acc | `0.321` |
| Unanswerable questions | `119` |
| Unanswerable Acc | `0.778` |
| page_hit@5, answerable | `0.698` |
| page_recall@5, answerable | `0.615` |
| False-refusal rate | `0.295` |
| False-refusal given page_hit@5 | `0.170` |
| Empty-answer rate | `0.068` |

### 3.2 Weak slices

| Slice | n | Acc | First interpretation |
|---|---:|---:|---|
| Cross-page | — | `0.239` | incomplete evidence sets / composition |
| List | `43` | `0.205` | retrieval coverage + answer-set formatting |
| Figure, multi-label | `112` | `0.225` | figure semantics or answer composition |
| Layout, multi-label | `37` | `0.183` | structure lost or not selected |
| Chart, multi-label | `58` | `0.293` | visual values + derived answers |
| Integer | `123` | `0.293` | operand retrieval, arithmetic, extraction |
| Table, multi-label | `101` | `0.440` | best visual slice, but W1 margin is thin |

### 3.3 Zero-score error budget

| Mutually useful class | Count | Maximum Acc headroom if all fixed |
|---|---:|---:|
| Answerable + page_hit@5 + zero | `117` | `+0.295` |
| └─ false refusal + page_hit@5 | `33` | `+0.083` |
| └─ wrong non-NA + page_hit@5 | `84` | `+0.212` |
| Answerable + page miss@5 + zero | `70` | `+0.176` |
| Unanswerable + wrong non-NA + zero | `26` | `+0.065` |

These are **oracle upper bounds**, not forecasts. They establish priority:
W3/W4 before W2, then targeted W1.

### 3.4 Statistical caution

A naive question-level bootstrap gives a broad approximate Acc 95% interval of
`[0.409, 0.507]`. Questions within one document are correlated, so decision
gates must use a **paired document-cluster bootstrap**, not independent
question bootstrapping and not overlap/non-overlap of two marginal intervals.

---

## 4. First-principles constraints

### 4.1 Laws

1. **No evidence, no answer:** generation cannot recover facts absent from the
   indexed representation and retrieved context.
2. **Relevant is not sufficient:** a topically related page can still lack a
   required operand, row, comparison target, or cross-page hop.
3. **More context is not monotonically better:** distractors reduce answer
   accuracy. MHier-RAG reports an optimum near ten pages, followed by decline.
4. **Abstention is a precision/recall decision:** banning refusal raises
   answerable recall but damages unanswerable precision and F1.
5. **Gold metadata is evaluation-only:** production code must never consume
   gold evidence pages, answer formats, or answers.
6. **One causal variable per experiment:** W1 changes require re-ingest; W2/W3
   changes should first run query-only against the same immutable workspace.
7. **Full ingest or invalid:** CORE scores require coverage `1.0` (stage=`core` defaults to `BENCH047_REQUIRE_FULL_INGEST=1`; opt out with `=0`).
8. **A score lift must survive slices:** no headline win that regresses
   unanswerable accuracy, Chart/Table fidelity, or latency beyond its gate.

### 4.2 Non-goals

- Do not tune directly against the 40 CORE gold answers.
- Do not add more chart prompt prose without a measured missing field.
- Do not change provider, retrieval, prompt, and extractor in one run.
- Do not use a larger model as the first explanation for a pipeline failure.
- Do not compare EdgeQuake CORE scores directly to LVLM leaderboard scores.
- Do not implement an agentic architecture before cheaper causal levers fail.
- Do not optimize @20/@25; those historical checkpoints had partial ingest.

---

## 5. Target ladder

| Milestone | Acc | F1 | Supporting gates |
|---|---:|---:|---|
| Baseline B0 | `0.458` | `0.356` | page_hit@5 `0.698` |
| M1 — composition | `≥0.480` | `≥0.375` | no W1/retrieval regression |
| M2 — evidence retrieval | `≥0.500` | `≥0.400` | page_hit@5 `≥0.750` |
| M3 — cross-page | `≥0.525` | `≥0.425` | cross-page Acc `≥0.320` |

M1 requires roughly 9 additional full-credit equivalents; M2 requires 17; M3
requires 27. Targets are cumulative and must be confirmed on the untouched
CORE @40 fixture after passing development and Chart-8 gates.

---

## 6. Evaluation firewall — Wave 0

**Purpose:** make every later claim attributable and reproducible.

### W0.1 Immutable manifests

- Pin fixture file, question dataset revision, code SHA, model identifiers,
  prompt hashes, extraction hashes, and workspace ID.
- Record whether each run is fresh-ingest, query-only, or extractor-only.
- Fail closed if 40/40 docs or 397/397 questions are not present.
- Keep Chart-8 as the fast regression fixture; use CORE @40 only at wave gates.

**Code:**

- `tools/bench047/bench047/run.py`
- `tools/bench047/bench047/score.py`
- `tools/bench047/bench047/fidelity.py`
- `tools/bench047/bench047/protocol.py`
- `tools/bench047/scripts/run_phase_b_core.sh`

### W0.2 Per-question failure ledger

Add generated diagnostics to each prediction:

- `representation_present`: answer needle or audited semantic evidence on gold
  page; evaluation-only.
- `page_hit@1/3/5/10` and full page recall.
- `context_sufficient`: offline judge label; never provided to generation.
- `answerability_prediction`, `answer_confidence`, and refusal reason.
- `answer_contract`: inferred type, cardinality, units, requested operation.
- candidate answer, normalized answer, verifier decision.
- exact retrieved page IDs, chunk IDs, modalities, arm, rank, and rerank score.

Classify every miss as the earliest failing wave:

```text
W1: required evidence absent from indexed gold page
W2: evidence represented but incomplete in retrieved set
W3: context sufficient but candidate answer wrong/incomplete
W4: candidate semantically right but verification/normalization wrong
```

### W0.3 Statistics

- Use paired per-question deltas and document-cluster bootstrap 95% CIs.
- Report effect size, not only pass/fail.
- Run at least three query repetitions when provider sampling is uncontrolled.
- Report median and worst repeat.
- Preserve per-slice sample counts; do not interpret tiny slices as stable.

### Wave 0 exit

- [ ] One command regenerates the locked B0 report.
- [ ] Failure attribution covers at least 95% of zero-score questions.
- [ ] Run manifests are hash-complete.
- [ ] Partial ingest produces `valid=false`.
- [ ] No benchmark gold metadata enters production request payloads.

---

## 7. Wave 1 — W3/W4 answer composition and verification

**Why first:** 117 zero-score answerable questions already hit a gold page.

### W1.1 Typed answer contract

Before generation, infer a contract from the **question only**:

```text
kind: integer | float | string | list | boolean | date | span
cardinality: one | exact_n | all_matching
units: optional
operation: extract | compare | count | difference | ratio | percent_of | list
```

The generator returns a structured candidate:

```json
{
  "answer": "...",
  "kind": "integer",
  "support_chunk_ids": ["..."],
  "operands": [{"value": "36%", "chunk_id": "..."}, {"value": "1503", "chunk_id": "..."}],
  "operation": "percent_of",
  "confidence": 0.0
}
```

The user-facing response remains backward compatible. The structure is an
internal intermediate representation.

**Important:** infer from the question; never pass benchmark `answer_format`.

**Code:**

- existing policy: `edgequake-query/src/grounding.rs`
- existing prompts: `edgequake-query/src/engine_impl/prompt.rs`
- new focused module: `edgequake-query/src/answer_contract.rs`
- API orchestration: `edgequake-api/src/services/query_generation.rs`

### W1.2 Deterministic grounded operations

Move arithmetic from prompt-only behavior to a small, audited executor:

- `percent_of(p, n) = round(p/100 × n)`
- difference, sum, ratio, year-span expansion
- unit preservation and compatible-unit checks
- operand provenance required for every operation
- reject ambiguous operand pairings
- no operation if a required operand is absent

This addresses the observed `541`, `128`, `1251`, and related integer errors
without teaching arbitrary code execution.

**Stop condition:** if an arithmetic tool improves known examples but regresses
integer or unanswerable slices on Chart-8, rollback the operation router rather
than adding prompt exceptions.

### W1.3 List/set composer

List Acc is only `0.205`.

- Detect `all`, `which N`, comparisons, ordered lists, and set-valued questions.
- Retrieve/retain one evidence item per candidate row/entity.
- Normalize whitespace and punctuation only after semantic composition.
- Preserve ordering only when the question requires it.
- Verify requested cardinality; do not silently return a singleton for a list.
- Emit partial lists only when explicitly labeled incomplete in internal state;
  benchmark extraction still receives the short final answer.

### W1.4 Evidence sufficiency and calibrated refusal

Do not globally discourage `Not answerable`. Split the 82 false refusals:

- 33 have page_hit@5: candidate W3 refusal errors.
- the remainder mostly need W2 evidence recovery.

Add a set-level verifier:

```text
Supported   → answer
Refuted     → retry candidate once or abstain
Insufficient→ expand retrieval once; then abstain
```

Signals:

- support coverage for candidate claims/operands;
- contradiction;
- missing requested cardinality;
- retrieval score spread/uncertainty;
- answer confidence;
- cross-page hop completeness.

Google's sufficient-context work supports using context sufficiency plus
confidence rather than relevance alone. SURE-RAG supports set-level
Supported/Refuted/Insufficient decisions and warns that retrieval uncertainty
is important. Start with a prompted shadow judge and calibration dataset; do
not train a verifier until the shadow labels are audited.

**Code:**

- new: `edgequake-query/src/evidence_sufficiency.rs`
- extend diagnostics, not transport semantics:
  `edgequake-query/src/query_reliability.rs`
- integrate after candidate generation in
  `edgequake-api/src/services/query_generation.rs`

### Wave 1 experiments

| ID | Single change | Fast fixture | Promotion gate |
|---|---|---|---|
| G1 | typed contract only | Chart-8 + fixed CORE error set | list/int Acc up; no F1 loss |
| G2 | grounded operation executor | derived-count cases | all operands cited; no invented math |
| G3 | list composer | list slice | paired list Acc `+0.05` absolute |
| G4 | sufficiency shadow judge | 213 zeros + matched hits | audited precision/recall report |
| G5 | selective answer/retry | full query-only CORE | F1 `+0.015`; UNA Acc loss `<0.01` |

### Wave 1 exit

- [ ] CORE paired Acc delta CI lower bound `>0`.
- [ ] Acc reaches `≥0.480` or the wave is stopped.
- [ ] F1 does not regress.
- [ ] Unanswerable Acc remains `≥0.770` or within `-0.01` paired.
- [ ] No W1 metric changes in query-only runs.
- [ ] p95 query latency increase `≤25%`.

---

## 8. Wave 2 — W2 hierarchical evidence retrieval

**Why second:** 70 answerable zeros miss the gold page at top-5; cross-page Acc
is `0.239`, and page recall@5 is only `0.615`.

### W2.1 Page-first candidate layer

EdgeQuake already stores `page_start` and `modality` on chunks and formats them
for grounding. Promote page to a retrieval unit:

1. retrieve a high-recall chunk pool;
2. aggregate chunk scores by `(document_id, page_start)`;
3. build a page candidate with text, visual descriptions, table/chart blocks,
   headings, and neighboring-page metadata;
4. rerank pages;
5. select chunks from the best pages under a diversity budget.

This is incremental: it reuses current chunk/vector storage before introducing
a new visual index.

**Code:**

- retrieval entry:
  `edgequake-query/src/engine_impl/query_entry/query_pipeline.rs`
- chunk retrieval:
  `edgequake-query/src/engine_impl/modes/chunk_retrieval.rs`
- reranking:
  `edgequake-query/src/engine_impl/reranking.rs`
- metadata:
  `edgequake-query/src/context_format.rs`
- new: `edgequake-query/src/page_candidates.rs`

### W2.2 Candidate generation before reranking

Current reranking only sees returned chunk content. Improve recall first:

- dense + BM25 + graph + modality arms produce a larger candidate pool;
- deduplicate by chunk and page;
- rerank top 50 candidates to top 20 chunks / top 8–10 pages;
- retain at least one candidate per detected sub-query;
- record pre-rerank recall and post-rerank precision.

Evaluate the existing BM25/default reranker against the available neural
cross-encoder configuration. Do not assume reranking is helpful: off-the-shelf
visual/verifier models can degrade retrieval without alignment.

### W2.3 Query decomposition for eligible questions

Apply only to questions classified as comparative, list, multi-entity,
multi-period, or likely cross-page:

1. decompose into at most 3 atomic evidence requests;
2. retrieve for root and children;
3. union/deduplicate pages;
4. rerank against the root question;
5. require evidence diversity across children.

Question-decomposition research shows recall gains but also added noise, hence
the required root-query reranker and eligibility gate.

**New module:** `edgequake-query/src/query_decomposition.rs`

### W2.4 Neighbor and parent expansion

When a page is selected:

- include same-page chunks;
- optionally add `page-1/page+1` for continuation tables/captions;
- add the nearest section heading/summary;
- preserve page boundaries and provenance;
- enforce a page budget to prevent context flooding.

### W2.5 Visual late-interaction shadow index

ColPali-style page-image retrieval is promising for visually rich pages and
layout cues, but it is a larger dependency and storage change. Run it as a
shadow retriever after the metadata-based page layer:

- index page screenshots as multi-vector embeddings;
- measure page Recall@K on the fixed fixture;
- fuse only if it adds unique gold pages beyond text/BM25;
- require latency/storage/cost accounting.

Do not merge this into production until it beats the cheaper page candidate
layer on paired retrieval metrics.

### Wave 2 experiments

| ID | Single change | Primary metric | Gate |
|---|---|---|---|
| R1 | larger candidate pool | pre-rerank page recall@20 | `+0.05` |
| R2 | page aggregation/rerank | page_hit@5 | `≥0.735` |
| R3 | conditional decomposition | cross-page page recall@10 | `+0.07` |
| R4 | neighbor/parent expansion | cross-page Acc | `+0.04` |
| R5 | visual shadow retriever | unique gold-page adds | positive, p95 budget met |

### Wave 2 exit

- [ ] page_hit@5 `≥0.750`.
- [ ] page_recall@5 `≥0.680`.
- [ ] Cross-page Acc `≥0.300`.
- [ ] CORE Acc `≥0.500`, F1 `≥0.400`.
- [ ] Context-empty rate `≤0.03`.
- [ ] p95 query latency increase from B0 `≤50%`.

---

## 9. Wave 3 — W1 targeted representation

W1 already passes its Chart/Table gates, so re-ingest only for measured gaps.

### W3.1 Table as a structured object

Current table prompt returns a free-form `description` containing Markdown.
Introduce a structured schema:

```text
title, headers, rows, units, footnotes, merged-cell lineage, page, continuation
```

Index:

- full Markdown table;
- row-level chunks with repeated headers and units;
- column/value lexical aliases;
- continuation links across pages.

This protects Table long (only `0.585`, close to the `0.55` gate) and supports
lists and cross-page table questions.

**Code:**

- `edgequake-api/src/services/multimodal/prompts.rs`
- new table struct/serializer beside
  `edgequake-api/src/services/multimodal/image_specialize.rs`
- chunk creation:
  `edgequake-api/src/services/multimodal/chunks.rs`

### W3.2 Figure relationship packets

Figure Acc is `0.225` despite relatively stronger figure representation
fidelity. Convert figure analysis into retrievable atomic facts:

- components;
- directed relationships;
- labels/numbers;
- panel identifier;
- caption linkage;
- page and bounding-box lineage.

Avoid more prose. Index atomic relationship statements plus the original
description.

### W3.3 Cross-page topology summaries

Create bounded, deterministic summaries for:

- section → page membership;
- table/chart continuation;
- repeated entity/metric across pages;
- caption → figure/table;
- appendix/reference links.

This is the EdgeQuake-native equivalent of topological cross-page chunks in
MHier-RAG. Build from document structure and lineage first; use an LLM only for
summary text after deterministic links exist.

### W3.4 Representation fail-closed gates

- Every chart/table/figure object carries extraction density diagnostics.
- Sparse retry triggers only on missing required fields, not word count.
- Preserve Pass A when specialization is malformed.
- Fresh-ingest experiments compare the same pages and answer needles.

### Wave 3 exit

- [ ] Chart a_in_e_long `≥0.60`.
- [ ] Table a_in_e_long `≥0.62` (raise margin from `0.585`).
- [ ] Figure a_in_e_long does not regress.
- [ ] Table/List/Cross-page Acc each improve on paired samples.
- [ ] Re-ingest cost and failure rate stay within operational budget.

---

## 10. Wave 4 — bounded evidence-driven loop

Only proceed if Waves 1–3 plateau below M3.

```text
classify question
  → retrieve root + eligible sub-queries
  → verify evidence set
  → if insufficient: one bounded expansion
  → compose typed answer
  → verify support and answer contract
  → answer or abstain
```

Limits:

- maximum 3 child queries;
- maximum 2 retrieval rounds;
- maximum 10 pages;
- deterministic token/cost budget;
- no hidden recursive agent loop;
- every expansion reason recorded.

This adopts the useful parts of HiEvi-RAG without requiring GRPO training or an
8×A100 verifier before EdgeQuake proves the basic loop.

### Wave 4 exit

- [ ] M3 target reached or a documented plateau is accepted.
- [ ] Mean extra LLM calls `≤2` per eligible question.
- [ ] p95 latency and cost remain within product SLO.
- [ ] Verifier false-safe/false-abstain rates are audited.

---

## 11. Experiment discipline

### 11.1 Dataset separation

| Set | Purpose | May tune on it? |
|---|---|---|
| Unit/contract fixtures | correctness and edge cases | yes |
| Chart-8 | fast causal/regression smoke | limited |
| CORE development slice | error analysis / threshold calibration | yes, predeclared |
| CORE @40 holdout report | promotion | no answer-level tuning |

If the current CORE @40 has already influenced implementation choices, freeze a
second holdout from the remaining MMLongBench documents before claiming broad
generalization.

### 11.2 Run types

- **W3/W4 change:** same workspace, query-only, same retrieval artifacts first.
- **W2 change:** same workspace, fresh query run; no re-ingest.
- **W1 change:** new workspace, fresh ingest, then query.
- **Extractor/evaluator change:** rescore immutable raw predictions; never
  regenerate answers in the same step.

### 11.3 Promotion template

Every assessment records:

```text
Hypothesis:
Single changed variable:
Expected causal path:
Primary metric:
Guard metrics:
Paired delta + document-cluster 95% CI:
Latency/cost delta:
Slice wins/losses:
Decision: promote | revise | revert
```

### 11.4 Stop and rollback rules

Revert when any occurs:

- paired CORE Acc delta CI includes a material negative effect;
- F1 drops by more than `0.01`;
- unanswerable Acc drops by more than `0.01`;
- Chart/Table W1 gate fails;
- page recall improves but end-to-end Acc drops due to distractors;
- latency exceeds the wave budget without proportional score gain;
- improvement is isolated to a memorized benchmark example;
- a change requires gold answer/evidence metadata at runtime.

---

## 12. Implementation order

### Phase A — one week: measurement and cheap W3 wins

- [ ] A1 Add failure ledger and paired cluster-bootstrap report.
- [ ] A2 Add question-only typed answer contract.
- [ ] A3 Add deterministic grounded operation executor.
- [ ] A4 Add list/set composer.
- [ ] A5 Run G1–G3; retain only causal wins.

### Phase B — one week: refusal and sufficiency

- [ ] B1 Build offline sufficiency labels for a stratified error sample.
- [ ] B2 Run shadow Supported/Refuted/Insufficient judge.
- [ ] B3 Calibrate one retry/abstention decision on development only.
- [ ] B4 Run full query-only CORE gate.

### Phase C — one to two weeks: hierarchical retrieval

- [ ] C1 Page candidate aggregation from existing chunk metadata.
- [ ] C2 Candidate-pool and reranker ablation.
- [ ] C3 Conditional decomposition.
- [ ] C4 Neighbor/parent expansion.
- [ ] C5 Run R1–R4; promote smallest winning stack.

### Phase D — two weeks: targeted representation

- [ ] D1 Structured table objects and row chunks.
- [ ] D2 Figure relationship packets.
- [ ] D3 Cross-page topology summaries.
- [ ] D4 Fresh-ingest Chart-8 then CORE gate.

### Phase E — optional frontier

- [ ] E1 ColPali-style shadow page index.
- [ ] E2 Bounded evidence-driven loop.
- [ ] E3 Compare against the simpler winning stack on score, cost, and latency.

---

## 13. Code ownership map

| Concern | Existing code | Planned extension |
|---|---|---|
| Query orchestration | `edgequake-query/src/engine_impl/query_entry/query_pipeline.rs` | sufficiency/expansion decision |
| Chunk retrieval | `edgequake-query/src/engine_impl/modes/chunk_retrieval.rs` | page candidate pool |
| Chart routing | `edgequake-query/src/modality_retrieve.rs` | multi-modality intent, not chart-only |
| Reranking | `edgequake-query/src/engine_impl/reranking.rs` | page/set reranking diagnostics |
| Context budget | `edgequake-query/src/truncation.rs` | page diversity + sub-query floors |
| Context provenance | `edgequake-query/src/context_format.rs` | page/evidence packet metadata |
| Grounding | `edgequake-query/src/grounding.rs` | typed candidate policy |
| Answer generation | `edgequake-api/src/services/query_generation.rs` | composer + verifier |
| Chart/Figure extraction | `edgequake-api/src/services/multimodal/image_specialize.rs` | figure facts, density telemetry |
| Table extraction | `edgequake-api/src/services/multimodal/prompts.rs` | structured table schema |
| Benchmark | `tools/bench047/bench047/` | failure ledger, paired statistics |

Keep new modules small and single-purpose. Do not turn
`query_pipeline.rs`, `grounding.rs`, or `query_generation.rs` into policy
monoliths.

---

## 14. Risks

| Risk | Mitigation |
|---|---|
| Benchmark overfitting | second holdout; no gold metadata at runtime |
| More recall adds distractors | bounded page budget + verifier/reranker |
| Refusal suppression harms UNA F1 | calibrated selective answer gate |
| Judge agrees with generator errors | different prompt/model; counterfactual audits |
| LLM reranker degrades retrieval | shadow mode and paired Recall/NDCG |
| Visual retriever adds heavy infra | defer until metadata page layer proves limit |
| W1 re-ingest is costly | only after query-only waves; cache immutable assets |
| Provider variance hides effects | repeated runs + paired cluster bootstrap |
| Duplicate SPEC namespace | renumber before canonical registration |

---

## 15. Recommended immediate next experiment

**G1: Typed answer contract + deterministic normalizer, no retrieval or ingest
change.**

Why:

- lowest operational cost;
- directly targets List `0.205`, Integer `0.293`, and hit-page wrong answers;
- query-only causal run on the immutable @40 workspace;
- does not require a new model, index, or database schema.

Pre-register:

```text
Primary: CORE Acc paired delta > 0
Secondary: F1 paired delta > 0
Slices: List +0.05 absolute; Integer +0.03 absolute
Guards: UNA Acc ≥0.768; page_hit metrics identical; p95 +≤10%
Promotion: at least 5 additional full-credit equivalents and no guard failure
```

Then run G2 arithmetic as a separate change. Do not combine G1 and G2 before
their individual effects are known.

---

## 16. Research references

1. MMLongBench-Doc, NeurIPS 2024:
   <https://arxiv.org/html/2407.01523v3>
2. ColPali, ICLR 2025:
   <https://proceedings.iclr.cc/paper_files/paper/2025/hash/99e9e141aafc314f76b0ca3dd66898b3-Abstract-Conference.html>
3. Sufficient Context, ICLR 2025 / Google Research:
   <https://research.google/blog/deeper-insights-into-retrieval-augmented-generation-the-role-of-sufficient-context/>
4. Question Decomposition for RAG, 2025 preprint:
   <https://ar5iv.labs.arxiv.org/html/2507.00355>
5. MHier-RAG, 2025 preprint:
   <https://arxiv.org/html/2508.00579v3>
6. SURE-RAG, 2026 preprint:
   <https://arxiv.org/html/2605.03534v1>
7. HiEvi-RAG, July 2026 preprint:
   <https://arxiv.org/html/2607.04625>

Preprints inform hypotheses; only EdgeQuake's controlled ablations determine
promotion.
