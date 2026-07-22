# 061 — LightRAG-as-law → EdgeQuake First Principles (ideas)

**Status:** Research hub · ideas only · **not** Acc Beat fishing  
**Date:** 2026-07-21  
**Law source:** `/Users/raphaelmansuy/Github/03-working/LightRAG` (+ [RoleSpecificLLMConfiguration.md](https://github.com/HKUDS/LightRAG/blob/main/docs/RoleSpecificLLMConfiguration.md))  
**Measured ceiling:** [059](./059-c1b-latency-ceiling-keyword-embed.md) · [060](./060-c1d-heuristic-keyword-latency.md)  
**Program hub:** [055](./055-post-acc-ceiling-first-principles.md)

---

## 0. Todo (this study)

```text
- [x] Step 1: Map LightRAG mix query path (keyword → embed → arms → merge → rerank → QUERY)
- [x] Step 2: Diff vs EdgeQuake prepare/retrieve/rerank/generate + measured stages
- [x] Step 3: Derive First-Principles levers (what to copy, what EQ already beats, what not to copy)
- [x] Step 4: Rank one-confound product packs (latency first; Acc peer frozen)
```

---

## 1. First principles (decomposition)

| Layer | Question | Law |
|-------|----------|-----|
| **Q0 Cost anatomy** | Where does wall time go? | Under Acc Mistral pins: **generate ≈ keyword ≈ embed ≫ retrieve ≫ BM25-rerank**. CE is already optional. |
| **Q1 Role economics** | Same model for every LLM call? | LightRAG law: **KEYWORD ≠ QUERY ≠ EXTRACT**. Latency-sensitive steps get ultra-fast non-thinking models; answer quality gets the strong model. |
| **Q2 Parallelism** | What can overlap? | EQ already parallelizes keyword‖embed_one and Mix arms. LR is **sequential** on arms and keyword→embed. Do **not** copy LR's sequential arms. |
| **Q3 Fusion identity** | Round-robin vs RRF? | LR mixes by round-robin; EQ default RRF. Acc peer keeps RRF — fusion is an Acc confound, not a free latency win. |
| **Q4 Cache identity** | Repeat queries cheap? | LR caches keywords + answers (`ENABLE_LLM_CACHE`). EQ has keyword cache + embed_one cache; answer cache / TTFT metrics are weaker product surfaces. |
| **Q5 Acc vs latency** | Can Soft Mix fix ≤1.5×? | **No.** Generate alone can exceed 1.5× LR. Soft Mix Acc fishing STOP ([055](./055-post-acc-ceiling-first-principles.md)). |

**Ceiling rule (binding):** Copy LightRAG's *role economics* and *token budgets*, not its *sequential Mix* or *default CE tax*. Measure wall p50 + stage honesty. Never promote Acc from latency packs.

---

## 2. LightRAG mix path (code is law)

```text
aquery → kg_query
  → extract_keywords_only          # role_llm_funcs["keyword"] + keywords cache
  → _perform_kg_search
       · ONE batch embed([query, ll, hl])
       · local THEN global THEN naive   # sequential awaits
       · round-robin merge
  → process_chunks_unified         # rerank default ON (RERANK_BY_DEFAULT)
  → role_llm_funcs["query"]        # answer; stream optional; query cache if non-stream
```

| Law | Where | Implication for EQ |
|-----|-------|--------------------|
| KEYWORD role = ultra-fast non-thinking | `llm_roles.py` + `KEYWORD_LLM_*` env | EQ has workspace `llm_roles.keyword` but **Acc/default still uses Query model** → ~1.8s keyword tax ([059](./059-c1b-latency-ceiling-keyword-embed.md)) |
| KEYWORD prompt = hl/ll JSON only | `operate.py:extract_keywords_only` | EQ adds `query_intent` — useful for Fact BM25; keep, but don't put intent on a slow model |
| Keyword LLM cache | `cache_type="keywords"` | Product: ensure Acc/warm path hits cache; pin cache identity to KEYWORD model |
| Batched query embeds | one `embedding_func(texts, context="query")` | EQ: `embed_one` ‖ keywords then optional 2-vec batch — **two RTTs** when hl/ll ≠ query; batch path uncached |
| Sequential Mix arms | `_get_node_data` then `_get_edge_data` then naive | EQ `tokio::join!` is **strictly better** — keep |
| Round-robin fusion | `_merge_all_chunks` | Optional `MIX_FUSION=lightrag` exists; Acc peer stays RRF |
| Rerank default true | `base.py` / README (~1–2s) | Acc Fact uses CE; latency packs use BM25 — **labeled peers** |
| Token budgets | entity 6k / relation 8k / total 30k | EQ packing must stay LR-shaped for Mix fidelity ([033](./033-denser-graph-mix-packing.md)) |
| QUERY can be stronger / thinking | docs recommend 30B+ thinking for QUERY | Product: split models; Acc fairness may keep same QUERY model as LR for fair Acc |

---

## 3. EdgeQuake today vs LightRAG

| Dimension             | LightRAG                          | EdgeQuake                                           | Winner for product                                             |
| -----------------------| -----------------------------------| -----------------------------------------------------| ----------------------------------------------------------------|
| KEYWORD≠QUERY models  | First-class env (`KEYWORD_LLM_*`) | Workspace metadata only; no process env pin         | **LR law → EQ must close**                                     |
| Keyword ‖ embed       | Sequential                        | Parallel                                            | **EQ**                                                         |
| Mix arms              | Sequential                        | Parallel + arm semaphore                            | **EQ**                                                         |
| Embed batching        | 1 batch query/ll/hl               | Often 1+1 (embed_one + batch)                       | **LR law → unify**                                             |
| Fusion                | Round-robin                       | RRF (default)                                       | Acc: EQ; LR-parity mode: optional                              |
| Intent / Fact protect | No                                | Yes (`query_intent`, Fact BM25/CE)                  | **EQ product** (Acc peer)                                      |
| Heuristic keywords    | No                                | `KEYWORD_MODE=heuristic`                            | Stage✓ / wall✗ ([060](./060-c1d-heuristic-keyword-latency.md)) |
| TTFT / stream metric  | stream flag; no TTFT SLO in core  | stream after full retrieve; no TTFT in `QueryStats` | Both weak → **EQ opportunity**                                 |
| Acc peer honesty      | n/a                               | Split Acc Fact vs L2 Parity                         | **EQ**                                                         |

**Measured (C1b `T013842Z`, Acc pins):** keyword 1782 · embed 2212 · retrieve 539 · rerank 9 · generate **2421** · ratio **3.91×**.  
**C1d:** keyword 0 does **not** move wall — generate (+ retrieve variance) dominates.

---

## 4. Idea portfolio (ranked)

### P0 — Must ship for latency law (product; not Acc promote)

#### Idea A — **Fast KEYWORD role as default product path** (LR law #1)

**Principle:** Keyword extraction is a *routing* call, not an *answering* call. Paying mistral-small twice is a category error.

**Do:**
1. Process env (LR-shaped): `EDGEQUAKE_KEYWORD_LLM_PROVIDER` / `EDGEQUAKE_KEYWORD_LLM_MODEL` / host / key — fallback to workspace `llm_roles.keyword` then Query LLM.
2. Acc backend whitelist + pin in SUMMARY (same lesson as `FACT_CE_SKIP`).
3. Default product recommendation: nano / local non-thinking (e.g. `gpt-5-nano`, Ollama 7–9B, or local vLLM) with thinking **off**.
4. Pack `c1e` = C1b + fast KEYWORD model; success = keyword p50 ≪ 400 ms **and** wall p50 ↓ **and** Acc tax labeled (no Acc promote unless CI clear).

**Why not heuristic alone:** [060](./060-c1d-heuristic-keyword-latency.md) zeroed the stage and **lost** Mix keyword quality (retrieve↑, wall flat).

#### Idea B — **Single-batch query embeddings** (LR law #2)

**Principle:** One network RTT for `{query, ll, hl}` after keywords (or: start query embed early, then only embed missing keyword strings).

**Do:**
1. After keywords resolve, one `embed([query, hl_text, ll_text])` (reuse equal texts — already partially in `compute_with_query_vec`).
2. Cache **batch** embeddings (today only `embed_one` is cached).
3. Optional: speculative `embed_one(query)` ‖ keywords remains; then only embed hl/ll if ≠ query (current), but ensure no third RTT.

**Success:** pure embed p50 ↓ under Acc remote embed; wall moves only if embed was on critical path after keyword shrinks.

#### Idea C — **Generate ceiling honesty + TTFT product** (physics)

**Principle:** Full-answer wall under Acc Mistral can exceed 1.5× LR even with free retrieve/rerank. UX SLO ≠ completion SLO.

**Do:**
1. Add `ttft_ms` / `time_to_first_token_ms` to stream path + harness SUMMARY.
2. Product docs: dual SLO — **TTFT ≤ X** and **completion p50 ≤ Y× LR**.
3. Optional faster QUERY model for product (not Acc fairness) or stream-first UI.
4. Context token trim before generate (already LR-shaped caps) — measure prompt tokens vs generate ms.

**Success:** publishable TTFT win even when completion ratio stays >1.5×.

---

### P1 — High leverage, one confound each

#### Idea D — **Keyword + answer LLM caches as product defaults**

LR: `ENABLE_LLM_CACHE` for keywords and non-stream answers.  
EQ: strengthen keyword cache hit rate under Acc concurrency; optional answer cache for identical Mix queries (invalidate on ingest).  
**Pack:** warm-repeat latency (2nd pass) vs cold — separate from Acc n=40.

#### Idea E — **Rerank economics (local CE or BM25 by intent)**

LR docs: rerank helps quality, costs 1–2s → deploy **local** CE.  
EQ Acc Fact already pays remote CE; latency packs use BM25.  
**Product default matrix:** Fact→BM25 or local CE; Exploratory/Complex→CE protect; never silent CE on latency peer.

#### Idea F — **EXTRACT≠QUERY on ingest (graph quality)**

LR: EXTRACT medium non-thinking; QUERY strong.  
EQ roles exist — pin EXTRACT to a schema-strong model and keep Acc Fact peer on B5 until labeled B10+ ingest.  
**Not** Soft Mix Acc; this is graph identity / L2.

#### Idea G — **Pre-supplied keywords / only_need_context API**

LR skips keyword LLM when hl/ll provided; can return context without answer LLM.  
EQ: expose agent-facing APIs for “retrieve only” and “keywords override” to cut double LLM tax in tool loops.

---

### P2 — Copy carefully / do not copy

| Idea | Verdict |
|------|---------|
| Sequential Mix arms | **Do not copy** — EQ parallelism is correct |
| Round-robin as Acc default | **Do not** — Acc peer RRF; optional LR-fusion mode only |
| Heuristic keywords as default | **Reject as default** after C1d; keep as escape hatch |
| Soft Mix / TOPIC Acc knobs | **STOP** ([043](./043-honesty-can-we-push.md)) |
| Surface-form synonym Acc fishing | **Deferred** product law, not Acc |

---

## 5. Recommended next packs (one confound)

| Pack | Confound | Success | Fail / stop |
|------|----------|---------|-------------|
| **`c1e`** | Fast KEYWORD LLM (env + Acc export) on C1b base | [062](./062-c1e-fast-keyword-llm.md) Law✓ / wall **REJECT** (ministral-3b not ultra-fast under Acc) | Acc promote without CI |
| **`c1f`** | Single-batch + batch embed cache (on c1e) | embed p50 ↓; no Acc claim | Extra RTT regressions |
| **`c1g`** | Stream TTFT metric + SUMMARY | TTFT published; UX claim separate from completion ratio | Confuse TTFT with Acc |
| **B10** | Naming filter Acc re-ingest | Acc gate or keep B5 | Soft Acc fishing |

```bash
# After c1e lands (sketch)
export BENCH001_EQ_WORKSPACE_ID=8e990410-43b5-44f4-9f56-87bd154570ce
# + EDGEQUAKE_KEYWORD_LLM_MODEL=<nano|local> via Acc whitelist
make bench001-c1e   # TBD
```

---

## 6. Mental model (one picture)

```text
                    ┌─────────────────────────────────────┐
   LightRAG law     │  KEYWORD fast · QUERY strong ·      │
                    │  1× embed batch · token budgets     │
                    └──────────────┬──────────────────────┘
                                   │ copy role economics
                                   ▼
                    ┌─────────────────────────────────────┐
   EdgeQuake keep   │  parallel arms · RRF Acc · intent   │
                    │  Fact protect · split peers         │
                    └──────────────┬──────────────────────┘
                                   │ measured ceiling
                                   ▼
         keyword≈1.8s + embed≈2.2s + generate≈2.4s  →  wall ~4× LR
                                   │
              ┌────────────────────┼────────────────────┐
              ▼                    ▼                    ▼
         Idea A               Idea B               Idea C
      fast KEYWORD         1-batch embed          TTFT / QUERY
```

---

## 7. Binding stop rules

- Acc Fact peer stays B5 + `a1fp` unless Acc gates clear.  
- Latency packs are **not** Acc Beat evidence.  
- One confound per pack; pin every env in SUMMARY.  
- Do not claim ≤1.5× until generate (+ KEYWORD role) is honest under the same pins.

**Product follow-through (065):** Smart/`mode=mix` arm set = LightRAG mix (always local∥global∥naive). Product `EDGEQUAKE_MIX_ARM_GATE` defaults **off**; Linked/`hybrid` keeps intent gating. See [065](./065-smart-lightrag-mix-arms.md).

---

## 8. Sources (law)

| Source | Use |
|--------|-----|
| `LightRAG/lightrag/operate.py` `extract_keywords_only`, `_perform_kg_search`, `kg_query` | Query path |
| `LightRAG/lightrag/llm_roles.py` | Role registry |
| `LightRAG/docs/RoleSpecificLLMConfiguration.md` | KEYWORD ultra-fast / QUERY strong |
| EQ `edgequake-core/src/llm_roles.rs` | Role exists; env pin missing |
| EQ archives `T013842Z` / `T014632Z` | Ceiling evidence |
