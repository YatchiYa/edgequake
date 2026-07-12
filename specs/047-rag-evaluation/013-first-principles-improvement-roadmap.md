# 013 — First-Principles Improvement Roadmap (Code Is Law)

**Cross-ref:** [001](./001-first-principles.md) · [004](./004-ai-engineer-lens.md) · [005](./005-expert-developer-lens.md) · [012](./012-acceptance-criteria-and-scorecard.md) · smoke [`SUMMARY`](./e2e/artifacts/smoke/SUMMARY.md)

**Status:** actionable · code-anchored · **Acc chain superseded by [022](./022-reassessment-2026-07-11.md)** (2026-07-11)  
**Baseline (historical 10-doc):** P0 smoke Acc **0.45** / F1 **0.29** (valid)  
**Locked chart-fixture HEAD:** Acc **~0.43** · Unans **≥0.81** · Chart **~0.14** → next lever **015**  
**Law:** Every workstream names **files + symbols** that exist today. No vibes. No Acc inflation.

---

## 0. One-line strategy

> Make the right pages and structures **retrievable in code**; keep refusal when they are not.

Interactive board: Cursor canvas `spec047-first-principles-improve.canvas.tsx`.

**Follow-on:** modality-aware vision plan → [015](./015-modality-aware-vision-improvement-plan.md).

---

## 1. First principles (do not violate)

| ID | Principle | Implication |
|----|-----------|-------------|
| FP1 | **Information only flows forward** | If vision/chunking drops a chart number, no prompt can honestly recover it |
| FP2 | **Measure the bottleneck** | Instrument `page_hit@k` / empty-context **before** changing retrieval or prompts |
| FP3 | **One causal change per experiment** | Locked profile; ablations labeled; no mid-run provider swaps |
| FP4 | **Honesty > Acc inflation** | Do not ban “Not answerable”; protect unanswerable Acc (~0.89) |
| FP5 | **Task identity** | Optimize RAG adaptation metrics; never chase LVLM leaderboard as same task |
| FP6 | **Fail closed** | Empty answers / crashes → INVALID, not a “model quality” story |
| FP7 | **Code is law** | Roadmap tickets cite paths; diagnostics read API fields that the engine already emits |

---

## 2. End-to-end law (call graph)

```text
PDF upload
  tools/bench047/bench047/client.py::upload_pdf
  → edgequake-api handlers/pdf_upload/upload.rs::upload_pdf_document
  → processor/pdf_processing.rs (SSOT worker)
  → edgequake-pdf VisionPdfConverter | EdgeParsePdfConverter
       injects <!-- edgequake-page:N -->
  → PageAwareChunking (chunker/page_aware.rs)  # never spans pages
  → chunk_storage.rs  # KV+vector metadata: page_start, page_end, document_id
  → entity extraction + graph

Query (bench047 P0)
  client.py::query  → POST /api/v1/query {query, mode=hybrid, …}
  → query_execute.rs → query_pipeline.rs
       prepare → retrieve → postprocess → generate
  → hybrid.rs::query_hybrid_with_vector_storage
       Box::pin(local ∥ global ∥ naive) via arm_timed.rs
       → hybrid_merge.rs::merge_hybrid_contexts (round-robin | RRF)
  → helpers.rs::build_chunk_from_result  # page_start from vector metadata
  → context.rs::QueryContext::to_context_string  # pages NOT inlined in prompt
  → prompt.rs::generate_answer_with_provider
  → source_reference_builder.rs  # HTTP sources[].page_start

Score
  extract.py → mmlongbench_eval_score.py
  diagnostics.py  # page_hit@k vs gold evidence_pages (offline only)
```

**Naming law:** codebase uses `page_start` / `page_end` (1-indexed `u32`), **not** `page_id`.

---

## 3. Smoke evidence (why these workstreams)

| Observation | Count / rate | Causal reading |
|-------------|--------------|----------------|
| False “Not answerable” on answerable Qs | ~50% | Context empty or wrong → refusal is *correct given context* |
| Long answer says “no information” | 35 / 91 answerable | Retrieval miss, not extractor quirk |
| Misses by evidence source | Figure 24, Chart 21, Table 16 | Representation + retrieve of visual structure |
| Unanswerable Acc | 0.89 | Keep this; do not “fix” Acc by forcing answers |
| Chart Acc | 0.05 | Highest-leverage representation gap |
| Workspace-wide retrieve | 10 docs / smoke | Cross-doc dilution confounder (W2 `--document-scope`) |

---

## 4. Flaky heuristics to reject

| Heuristic | Why flaky | Lawful alternative |
|-----------|-----------|--------------------|
| Prompt: “never say Not answerable” | Inflates Acc; kills unanswerable Acc | Fix `page_hit@k` first |
| Feed gold `evidence_pages` into retrieve | Oracle leakage | Offline metric only (`diagnostics.py`) |
| Mix embed dims / models mid-run | Invalidates scorecard | Locked `profiles.py` |
| Tune hybrid weights without arm hit-rates | Blind fusion | Expose arm metadata (W0b) then ablate |
| One-off few-shots for failing Qs | Non-reproducible | Stratified fixture + profile id |
| Acc vs GPT-4o LVLM F1 as same task | Different input modality | Banner in scorecard |

---

## 5. Workstreams (ordered) — code anchors + tickets

### W0 — Observability (DONE in harness · 2026-07-10)

**Done when:** every new prediction row carries a `retrieval` block; scorecard `ops.retrieval` aggregates `page_hit@k`.

| Ticket | Change | Code |
|--------|--------|------|
| **EQ-047-W0a** ✅ | Capture `sources` + compute `page_hit@k` | `tools/bench047/bench047/diagnostics.py` |
| **EQ-047-W0a** ✅ | Wire into query loop | `run.py` + `client.py::query(include_references=True)` |
| **EQ-047-W0a** ✅ | Scorecard + SUMMARY | `score.py::write_summary` / `build_scorecard` |
| **EQ-047-W0a** ✅ | Unit tests | `tools/bench047/tests/test_diagnostics.py` |
| **EQ-047-W0b** ✅ | Expose engine `context_empty`, `arms_run`, per-arm chunk counts on HTTP `QueryStats` | `query_stats_mapper.rs` · `mix.rs`/`hybrid.rs` `attach_arm_metadata` · `types.rs::QueryStats` |
| **EQ-047-W0c** | Optional: emit page markers into `to_context_string()` for grounded citations | `edgequake-query/src/context.rs` |

**Metric definition (law):**

```text
retrieved_pages = ordered unique SourceReference.page_start where source_type=chunk
page_hit@k      = |gold evidence_pages ∩ retrieved_pages[:k]| > 0
page_recall@k   = |gold ∩ top-k| / |gold|
context_empty   = stats.context_empty (engine SSOT via W0b) else sources proxy
```

Gold pages are **never** sent to the API. Re-run query stage to populate diagnostics on old artifacts:

```bash
# after ingest exists
bench047 smoke --query-only --no-resume --api http://localhost:8091
# or scoped ablation (W2 flag, changes Acc — label profile)
bench047 smoke --query-only --no-resume --document-scope --api http://localhost:8091
```

---

### W1 — Representation (charts / figures / tables)

**Hypothesis:** Chart Acc 0.05 is mostly **ingest fidelity**, not fusion.

| Ticket | Change | Code |
|--------|--------|------|
| **EQ-047-W1a** ✅ probe | Audit vision markdown for chart/table number recall on frozen page sample | `fidelity.py` + `fidelity_audit.py` · `bench047 fidelity` · page markers |
| **EQ-047-W1b** ✅ code | Multimodal analyze + chart/figure specialize when `-i` / `P0_mm_ite` | `multimodal/{prompts,image_specialize,analyzer,chunks}.rs` · bench047 `process_options` |
| **EQ-047-W1c** | Table preprocessor quality for wide markdown tables | `pipeline/table_preprocessor.rs` |
| **EQ-047-W1d** | Ablation `P5_text_parse` (edgeparse) vs `P0` vision | `profiles.py::P5_text_parse` |

**Done when:** held-out page fidelity ↑ **and** chart Acc moves on re-score with **same** retrieve profile.

---

### W2 — Retrieval (raise `page_hit@5`)

**Hypothesis:** False refusal tracks `page_hit@5` miss, not prompt tone.

| Ticket | Change | Code |
|--------|--------|------|
| **EQ-047-W2a** ✅ flag | Document-scoped retrieve (ingest `document_id`, **not** gold pages) | `client.py` `document_filter.document_ids` · CLI `--document-scope` · `document_filter_resolver.rs` |
| **EQ-047-W2b** | Diagnose hybrid dilution with arm metrics (needs W0b) | `hybrid.rs` + `hybrid_merge.rs` · env `EDGEQUAKE_HYBRID_FUSION` |
| **EQ-047-W2c** | Compare `P1_naive` vs `P0_primary` on chart/cross-page slices | `profiles.py` · locked ablations |
| **EQ-047-W2d** | Graph investment only if Local/Global beat Naive on cross-page | `local.rs` / `global.rs` / `chunk_retrieval.rs` |

**Done when:** answerable `page_hit@5` ↑; false-refusal ↓; unanswerable Acc ≥ ~0.85.

---

### W3 — Generation (grounded when hit, refuse when miss)

| Ticket | Change | Code |
|--------|--------|------|
| **EQ-047-W3a** | Conditional generation on real `context_empty` (engine already short-circuits) | `prompt.rs::generate_answer_with_provider` |
| **EQ-047-W3b** | Optionally surface page numbers in context string (W0c) so model can cite pages | `context.rs::to_context_string` |
| **EQ-047-W3c** | Dual extractor bias check (F7) only after W1–W2 stable | `extract.py` · `P0_official_extractor` |

**Anti-pattern:** changing `EXTRACT_PROMPT` to forbid “Not answerable”.

---

### W4 — Evaluation discipline

| Ticket | Change | Code / doc |
|--------|--------|------------|
| **EQ-047-W4a** | Smoke → P1–P6 ablations → core | `profiles.py` · [010 runbook](./010-smoke-then-full-runbook.md) |
| **EQ-047-W4b** | Update [012 progression table](./012-acceptance-criteria-and-scorecard.md) every valid run | scorecard JSON |
| **EQ-047-W4c** | Fail-closed gates stay on | `run.py` `EMPTY_ANSWERS` / `QUERY_TOO_FAST` |

---

## 6. Non-flaky success gates

| Gate | Metric | Source of truth | Pass idea |
|------|--------|-----------------|-----------|
| G0 | `ops.retrieval.n_with_retrieval_diag` | scorecard | = n answerable scored |
| G1 | Chart/figure fidelity on held-out pages | W1 sample script | ↑ vs baseline |
| G2 | `page_hit@5` on answerable smoke | `ops.retrieval.page_hit@5` | ↑ vs baseline |
| G3 | Unanswerable Acc | `slices.unanswerable_accuracy` | ≥ ~0.85 |
| G4 | Overall F1 | `metrics.f1` | ↑ only with G1–G3 held |

---

## 7. Implementation status

| Workstream | Status |
|------------|--------|
| W0a harness diagnostics | **Implemented** (`diagnostics.py` + wired) |
| W0b API arm / context_empty stats | **Implemented** (`query_stats_mapper` + arm chunk META) |
| W0 smoke query-only baseline | **Measured** 2026-07-10: `page_hit@5≈0.59` (answerable) |
| W1a fidelity probe | **Implemented** (`bench047 fidelity`); chart answer-in-page ≈0.36 |
| W1 representation (015 A+B) | **Code landed**; G-A **failed** live — `ite` no-op without `<drawing>` refs; Phase C next |
| W2 document-scope flag | **Flag ready** (`--document-scope`); default off for P0 continuity |
| W2 fusion / naive ablations | Open — 16 Qs have answer in markdown but `page_hit@5=false` |
| W3 generation | Open — 10 false refusals despite `page_hit@5=true` |
| W4 discipline | Ongoing |
| Ingest P5 reliability (slim CP / stage honesty / chunk_only) | **Implemented** 2026-07-11 — see [016](./016-ingest-speed-reliability-battle-plan.md) |

---

## 8. What “code is law” forbids in PRs

1. Changing Acc without showing `page_hit@5` delta on the same fixture.
2. Prompt-only PRs that claim to “fix RAG quality.”
3. Using gold `evidence_pages` inside `document_filter` or retrieval.
4. Silent workspace reuse that changes LLM/vision pins (`client.py` already fails closed).
5. Reporting LVLM leaderboard numbers without the RAG banner.
