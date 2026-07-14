# 019 — Query Improvement Plan (First Principles)

**Status:** Q1 grounding **landed** · post-smoke execution moved to [020](./020-post-q1-first-principles-improvement-plan.md)  
**Scope:** Query path only (retrieve → assemble → generate). Ingest representation is a **dependency**, not this plan’s implementation surface.  
**Evidence (pre→post Q1):** Acc 0.384→**0.436** · `page_hit@5` 0.76→0.73 · Chart 0.14→0.18 · Unanswerable 0.69→**0.81** · Pure-text 0.27→**0.19** (over-refusal tax)  
**Peers:** [001](./001-first-principles.md) · [014](./014-ingest-query-pipeline-first-principles.md) · [017](./017-lightrag-vs-edgequake-query-pipeline-assessment.md) · [018](./018-quality-speed-improvement-plan.md) · **[020](./020-post-q1-first-principles-improvement-plan.md)**  
**Canvas:** [spec047-query-first-principles](/Users/raphaelmansuy/.cursor/projects/Users-raphaelmansuy-Github-03-working-edgequake/canvases/spec047-query-first-principles.canvas.tsx) · [post-Q1](/Users/raphaelmansuy/.cursor/projects/Users-raphaelmansuy-Github-03-working-edgequake/canvases/spec047-post-q1-first-principles.canvas.tsx)

### Implementation status (2026-07-11)

| Ticket | Status | Evidence |
|--------|--------|----------|
| Q1.1 page/modality headers | ✅ | `context_format.rs` + `to_context_string`; `e2e_spec047_query_grounding` |
| Q1.2 grounding instructions | ✅ | `grounding.rs` wired into text + vision prompts |
| Q1.3 chunk budget floor (40%) | ✅ | `TruncationConfig.min_chunk_budget_ratio`; env `EDGEQUAKE_MIN_CHUNK_BUDGET_RATIO` |
| Q2.1 Mix RRF ablation profile | ✅ | `profiles.py` → `P1_mix_rrf` |
| Q4.2 OpenAPI mode semantics | ✅ | `query_types.rs` QueryRequest.mode docs |
| Q4.1 / Q4.3 UI modes | ✅ | prior turn |

---

## 0. One-screen law

```text
  Question
     │
     ▼
  ┌──────────────┐   if empty / wrong pages     ┌──────────────┐
  │  RETRIEVE    │ ───────────────────────────▶ │ FIX INDEX /  │
  │  (recall)    │                              │ ARMS / SCOPE │
  └──────┬───────┘                              └──────────────┘
         │ page_hit high, Acc low
         ▼
  ┌──────────────┐   if evidence present but     ┌──────────────┐
  │  GROUND      │   LLM cannot use it         │ FIX PROMPT / │
  │  (context)   │ ───────────────────────────▶ │ BUDGET / CITE│
  └──────┬───────┘                              └──────────────┘
         │ grounded, still wrong short answer
         ▼
  ┌──────────────┐   if answer_in_evidence low   ┌──────────────┐
  │  GENERATE    │ ───────────────────────────▶ │ FIX INGEST   │
  │  (answer)    │   (gold never in markdown)   │ (015 / 014)  │
  └──────────────┘                              └──────────────┘
```

**Master axiom:** *You cannot prompt your way out of a missing page, and you cannot fuse your way out of invisible evidence.*

---

## 1. First principles (query-specific)

| ID | Principle | Operational meaning |
|----|-----------|---------------------|
| **Q1** | Information only flows forward | Query cannot invent chart numbers lost at vision |
| **Q2** | Classify the failure before the fix | Retrieval miss ≠ grounding miss ≠ generation miss ≠ representation miss |
| **Q3** | Measure the bottleneck with existing fields | Prefer `page_hit@k`, arm chunk counts, `context_empty`, answer_in_evidence over Acc alone |
| **Q4** | Dense + sparse + rank is the 2026 baseline | EdgeQuake already has BM25+dense; do not remove it to “match LightRAG” |
| **Q5** | Context is a scarce budget | Entities/rels that do not help the answer are tax on chunks |
| **Q6** | Grounding requires addressable evidence | Pages, modalities, and citations must be **visible to the LLM**, not only in HTTP `sources[]` |
| **Q7** | Mode semantics must be honest | UI/API labels must match EdgeQuake arms (hybrid ≠ LightRAG hybrid) |
| **Q8** | Honesty > Acc inflation | Keep “Not answerable”; protect unanswerable Acc |
| **Q9** | One causal change per experiment | Locked profile; no mid-run provider/mode swaps |
| **Q10** | Code is law | Every ticket cites a symbol that exists |

### Five WHYs (compressed)

| Why | Because | Therefore |
|-----|---------|-----------|
| Why Acc low with page_hit 0.76? | LLM does not see page/modality headers | **Ground before fuse** |
| Why Chart Acc ~0.14? | Numbers often absent from evidence markdown | **Ingest (015), not more arms** |
| Why hybrid floods naive (19 chunks)? | Round-robin + 3 arms | Ablate **mix RRF** vs hybrid |
| Why false refusal? | Missing numerics + strict grounding prompt | Fix evidence, don’t ban refusal |
| Why not “add GraphRAG”? | Hybrid baseline not fully converted to Acc | Graph only after grounding + representation |

---

## 2. Failure taxonomy (diagnose first)

Assign every miss to **exactly one primary class** (use diagnostics + fidelity):

| Class | Signal | Lawful response |
|-------|--------|-----------------|
| **R — Retrieval** | `page_hit@5` low; wrong docs; empty context | Scope, arms, BM25/RRF, modality filter, keyword quality |
| **G — Grounding** | `page_hit` high; LLM says “no info” / wrong page | `to_context_string` headers, chunk budget floor, citation format |
| **Gen — Generation** | Evidence in prompt; short-answer extract fails | Prompt / response_type / extractor (last resort) |
| **Rep — Representation** | `answer_in_evidence` low on gold pages | Hand off to [015](./015-modality-aware-vision-improvement-plan.md) / ingest |
| **Scope — Dilution** | Workspace-wide retrieve; cross-doc pollution | `--document-scope` (bench); product doc filter |

**Smoke reading (pre-Q1):** high-ish page_hit + low Chart Acc ⇒ mix of **G** (prompt) and **Rep** (charts). Do **not** start with PPR or more graph hops.

**Smoke reading (post-Q1):** Acc↑ mainly via refusal honesty; Pure-text↓ + false_refusal≈0.33 ⇒ **G-cal** (calibrate, don’t ban); Chart still **Rep**. Next plan: [020](./020-post-q1-first-principles-improvement-plan.md).

---

## 3. Query call graph (anchors)

```text
POST /api/v1/query | /chat/completions
  → query_execution / query_execute
  → QueryEngine::run_query_pipeline
       prepare: keywords ∥ embeddings (query / ll / hl)
       retrieve: naive | local | global | hybrid | mix | bypass
       postprocess: document_filter → rerank → prune → balance_context
       generate: context.to_context_string() → prompt.rs → LLM
```

| Stage | Critical symbols |
|-------|------------------|
| Mode / arms | `modes/{naive,local,global,hybrid,mix}.rs`, `mix_weights.rs` |
| Fusion | `fusion.rs`, `hybrid_merge.rs` |
| Sparse | `sparse_retrieval.rs`, Postgres FTS |
| Modality | `modality_retrieve.rs` |
| Budget | `truncation.rs::balance_context` |
| Prompt context | **`context.rs::to_context_string`** (no `page_start` today) |
| Answer | `prompt.rs::generate_answer*` |

---

## 4. Decision tree (what to build next)

```text
IF answer_in_evidence(Chart) low
  → STOP query work on charts; run 015 / fidelity (Rep)
ELSE IF page_hit@5 < 0.70 (scoped)
  → R: document scope, Mix RRF, naive weight, BM25 depth, modality filter
ELSE IF page_hit@5 ≥ 0.70 AND Acc low / false refusal high
  → G: page headers in prompt, chunk budget floor, cite-by-page instructions
ELSE IF grounded Acc still low on multi-hop only
  → optional graph science (PPR / communities) — Phase E
ELSE
  → latency / cost: arm gate tuning, cache, skip dead arms
```

---

## 5. Phased plan (query-first)

### Phase Q0 — Instrument (keep honest)

| Ticket | Change | Gate |
|--------|--------|------|
| Q0a | Every prediction already has `retrieval` / `page_hit@k` (W0) — keep required | scorecard `ops.retrieval` present |
| Q0b | Log primary failure class per miss (R/G/Gen/Rep) in harness notes | triage table in SUMMARY |
| Q0c | Arm hit-rates already in stats — publish in smoke SUMMARY | mean_arm_* visible |

**Exit:** triage table for last smoke (no code change required if fields exist).

### Phase Q1 — Grounding (highest leverage query-only)

*Why:* Converts existing page hits into usable LLM evidence (Q6).

| # | Ticket | Symbol | Gate | Effort |
|---|--------|--------|------|--------|
| **Q1.1** | Inline `[page=N]` + modality in chunk headers | `context.rs::to_context_string` | Acc↑ on page_hit hits; unanswerable ≥0.65 | S |
| **Q1.2** | Prompt: prefer cited pages; refuse if no supporting chunk | `prompt.rs` | false refusal ↓ without unanswerable collapse | S |
| **Q1.3** | Chunk budget floor (≥40% of total after buffer) | `truncation.rs` | mean chunk tokens ↑ | M |
| **Q1.4** | Optional: demote entity/rel blocks when intent=Factual | `balance_context` + intent | factual Acc ↑ | M |

**Reject:** “Never say Not answerable.”

### Phase Q2 — Retrieval alignment (mode & fusion)

*Why:* Bench locks `hybrid` round-robin while production default is Mix RRF (017 naming trap).

| # | Ticket | Symbol | Gate | Effort |
|---|--------|--------|------|--------|
| **Q2.1** | Ablation profile `P1_mix_rrf` vs `P0 hybrid` | `profiles.py`, `mix.rs` | Acc / page_hit delta table | S |
| **Q2.2** | If Mix wins → lock bench047 default to `mix` | `000-index`, runbook | Acc↑ held on re-run | S |
| **Q2.3** | Hybrid fusion = RRF option for factual/chart | `hybrid_merge.rs`, env | Chart page_hit@1 ↑ | S |
| **Q2.4** | Intent gate audit: false Factual→naive-only on relational Qs | `mix_weights::intent_arm_mask` | misroute rate | S |
| **Q2.5** | Keep BM25 on; ablate only as labeled experiment | `sparse_retrieval.rs` | do not ship BM25-off as default | — |

### Phase Q3 — Precision stage (rerank)

*Why:* 2026 pattern = hybrid recall → rerank precision ([Cohere rerank](https://docs.cohere.com/docs/reranking-with-cohere.mdx); hybrid RAG practice).

| # | Ticket | Symbol | Gate | Effort |
|---|--------|--------|------|--------|
| **Q3.1** | Bind real cross-encoder when key present | API rerank provider | slice Acc / nDCG proxy | M |
| **Q3.2** | Candidate depth: retrieve k=50–100, rerank to 20 | `rerank_top_k`, naive candidate_k | page_hit@5↑ or Acc↑ | M |
| **Q3.3** | Fail-open if reranker down (already) — keep | — | no INVALID from rerank alone | — |

### Phase Q4 — UX / honesty (already started)

| # | Ticket | Status |
|---|--------|--------|
| **Q4.1** | Surface all 6 modes + tooltips in WebUI | Done (2026-07-11) |
| **Q4.2** | OpenAPI note: EQ hybrid ≠ LightRAG hybrid | Open |
| **Q4.3** | Default UI mode = `mix` | Done |

### Phase Q5 — Hand-off to ingest (when Rep dominates)

Do **not** implement in query crate. Trigger [015](./015-modality-aware-vision-improvement-plan.md) / [018 Phase B](./018-quality-speed-improvement-plan.md) when:

- Chart `answer_in_evidence` flat after Q1–Q2  
- Or fidelity audit shows gold absent from page markdown  

### Phase Q6 — Optional science (only if Q1–Q3 stall on multi-hop)

| Ticket | When |
|--------|------|
| PPR / bipartite chunk pick default for Exploratory | Cross-page Acc flat after grounding |
| Community reports | Exploratory Acc flat |
| Late-interaction page embeddings | Caption-and-index plateaus |

---

## 6. Experiment protocol (non-negotiable)

1. **One change** per `bench047-smoke` (or labeled ablation pair).  
2. Locked profile in `tools/bench047/bench047/profiles.py`.  
3. Always report: Acc, F1, Unanswerable Acc, Chart Acc, `page_hit@5`, context_empty_rate.  
4. Fail closed on empty answers.  
5. No gold `evidence_pages` in retrieve (oracle only if labeled `oracle_*`).

```text
Order (query lane):
  Q1.1 page headers  →  Q2.1 mix vs hybrid  →  Q1.3 budget floor  →  Q3.1 rerank bind
Parallel (ingest lane when Rep):
  015 chart prompts / fidelity
```

---

## 7. Scoreboard targets (query lane)

| Gate | Metric | Target | Floor |
|------|--------|--------|-------|
| GQ-1 | Acc (smoke, scoped) | ≥ 0.48 after Q1+Q2 | valid=true |
| GQ-2 | `page_hit@5` | ≥ 0.80 | — |
| GQ-3 | Chart Acc | ≥ 0.25 **query-only**; ≥ 0.30 needs 015 | Unanswerable ≥ 0.65 |
| GQ-4 | False “no info” on page_hit hits | ↓ ≥ 30% relative | — |
| GQ-5 | Query p95 | ≤ +10% vs baseline after Q3 | — |

---

## 8. Anti-patterns

| Anti-pattern | Why it fails Q-principles |
|--------------|---------------------------|
| Ban “Not answerable” | Violates Q8; inflates Acc |
| Turn off BM25 to match LightRAG | Violates Q4; throws away EQ advantage |
| Add arms before page headers | Violates Q2/Q6 |
| Prompt-only Acc patches | Violates Q1 when Rep is the class |
| Equate hybrid≡LightRAG hybrid | Violates Q7 |
| Mid-run model swap | Violates Q9 |

---

## 9. Definition of done (this plan)

- [x] Q1.1–Q1.3 merged; smoke Acc↑ (0.384→0.436) with triage → G-cal + Rep remaining  
- [ ] Q2.1 published Acc table: `mix` vs `hybrid` → tracked in [020](./020-post-q1-first-principles-improvement-plan.md) B3  
- [ ] Bench default mode justified by that table (update 000-index if mix wins)  
- [x] OpenAPI / docs state EQ mode semantics (Q4.2)  
- [x] Unanswerable Acc never sacrificed (0.69→0.81)  

**Relationship to 018:** 018 is the full quality+speed backlog (ingest+query). **019 is the query decision system** — use it to choose *which* 018 tickets to pull next.

---

## 10. Immediate next actions

**Done (Q1):** headers + grounding prompt + chunk budget; Acc 0.384→0.436.

**Next:** follow [020](./020-post-q1-first-principles-improvement-plan.md) — A1 calibrated refusal → B1 arm-gate honesty → B3 Mix ablation ∥ 015 for Chart.
