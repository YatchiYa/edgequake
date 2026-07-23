# 074 — Why EdgeQuake Lags LightRAG (Medical-Mid First Principles)

**Status:** Analysis SSOT · LR-identity gate run · **not** Acc Beat fishing  
**Date:** 2026-07-22  
**Publish truth:** [`../e2e/artifacts/publish/latest/`](../e2e/artifacts/publish/latest/) · archive `medical-mid-20260722T104918Z`  
**LR-identity gate:** [`../e2e/artifacts/history/smoke-20260722T122410Z/`](../e2e/artifacts/history/smoke-20260722T122410Z/) (`LR_IDENTITY_v1`)  
**Law source:** `/Users/raphaelmansuy/Github/03-working/LightRAG` (`lightrag/operate.py`)  
**Cross-ref:** [010](./010-lens-retrieval-noise.md) · [017](./017-beat-lightrag.md) · [018](./018-e4-acc-tie-close.md) · [055](./055-post-acc-ceiling-first-principles.md) · [061](./061-lightrag-law-first-principles-eq.md) · [029](./029-ingest-parity-audit.md)

---

## 0. One-screen verdict

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│  medical-mid n=200 (fair Mix↔Mix, Mistral Small + mistral-embed):            │
│    Acc  EQ 0.706  ·  LR 0.774  ·  Δ −0.068  ·  CI [−0.107, −0.033]           │
│    L2   recall 0.887 vs 0.951 (−0.064)  ·  ctx_rel 0.396 vs 0.491 (−0.095)  │
│                                                                              │
│  Binding failure = noisier / less complete context (L2) → Acc lag.           │
│  NOT missing Mix arms · NOT wrong chunk/embed under fair pins.               │
│  Soft Mix Acc knobs exhausted (018/055 STOP). Do not claim EQ beats LR.      │
└──────────────────────────────────────────────────────────────────────────────┘
```

**First principle:** Acc follows context. Fix packing / admission / provenance before fishing generation knobs.

---

## 1. Causal model

```text
Question
  → keywords (hl/ll)
  → Mix arms (local ∥ global ∥ naive)     # both SUTs always-on
  → Fuse                                  # EQ RRF vs LR round-robin
  → KG→chunk pick                         # EQ PPR vs LR VECTOR×budget
  → Post-fuse rank                        # EQ BM25 (no protect) vs LR rerank off
  → Pack Entities→Relations→Chunks        # EQ degree hubs vs LR VDB order
  → Answer LLM
       ├→ L2 (evidence_recall, ctx_rel)
       └→ Acc (0.75·F1 + 0.25·cos)
```

| Gap ID | Divergence | L2 symptom | Acc symptom |
|--------|------------|------------|-------------|
| **R1** | Noise packing (degree + BM25 + RRF tails) | ctx_rel −0.095 | overall Acc lag |
| **R2** | Evidence miss (PPR vs VECTOR; ingest density) | recall −0.064 | Fact Acc −0.107 |
| **R3** | BM25 post-fuse without protect | Complex/Sum ctx_rel | Complex/Creative Acc |
| **R4** | RRF vs round-robin fusion | order / unique-tail burial | secondary Acc |
| **R5** | Empty answers 3.5% vs 0% | — | residual Acc drag |

---

## 2. LightRAG law vs EdgeQuake Acc publish pins

| Law | LightRAG (code) | EQ Acc headline (`start_acc_backend.py`) | Winner for Acc/L2 cleanliness |
|-----|-----------------|------------------------------------------|-------------------------------|
| Mix merge | **round-robin** `_merge_all_chunks` | **RRF** `EDGEQUAKE_MIX_FUSION=rrf` | LR identity |
| Post-fuse rerank | fair dual-SUT **`enable_rerank=False`** | **BM25 always on**, protect=0 | LR identity |
| KG→chunk | **VECTOR** + `related×N/2` | **PPR** walk; `KG_CHUNK_PICK_LR_BUDGET=0` | LR identity |
| Entity prompt order | VDB / retrieval order | **`ENTITY_RANK=degree`** | LR identity |
| Lexical in Mix arms | dense-only | BM25/FTS inside arms | LR identity |
| Token pack | 6k / 8k / 30k | matched (033) | tie |
| Chunk / overlap | 1200 / 100 | matched | tie |
| Mix arms | always 3 | always 3 (065) | tie |
| Arm parallelism | sequential | **parallel** | **EQ keep** |

### Code cites

| Concern | EdgeQuake | LightRAG |
|---------|-----------|----------|
| Mix fuse | `edgequake-query/.../modes/mix.rs`, `fusion.rs` | `operate.py` `_merge_all_chunks` |
| Round-robin | `hybrid_merge.rs` | same |
| KG→chunk | `kg_chunk_pick.rs`, `graph_ppr.rs` | `pick_by_vector_similarity` (~5352+) |
| Entity order | `entity_rank.rs` | VDB order |
| Acc pins | `tools/bench001/scripts/start_acc_backend.py` | harness `lr_query_param_overrides.enable_rerank=False` |

---

## 3. Medical-mid metrics → R1–R5

| Type / layer | EQ | LR | Δ | Maps to |
|--------------|-----|-----|---|---------|
| Acc overall | 0.706 | 0.774 | −0.068 (CI excludes 0) | R1–R5 composite |
| Fact Acc | 0.673 | 0.780 | −0.107 | **R2** primary |
| Complex Acc | 0.708 | 0.754 | −0.046 | R1/R3 |
| Summarize Acc | 0.779 | 0.824 | −0.045 | R1/R3 |
| Creative Acc | 0.665 | 0.737 | −0.072 | R1/R3 |
| evidence_recall | 0.887 | 0.951 | −0.064 | **R2** |
| context_relevancy | 0.396 | 0.491 | −0.095 | **R1** binding |
| empty answers | 3.5% | 0% | — | **R5** |
| query p50 | 5962 ms | 5288 ms | 1.13× | latency OK (≠ Acc cause) |

---

## 4. Explicitly NOT the cause

- Missing Mix arms (product Smart Mix = always-on after 065)
- Wrong chunk size / embed dims under fair pins
- Sequential LR Mix (EQ parallel is strictly better — do **not** copy)
- Soft Mix Acc fishing (path, topic, entity_rank ablations) — 018/055 STOP
- Claiming smoke n=40 as publish truth (forbidden; medical-mid is publish Acc)

---

## 5. Remediation ladder (identity first)

**Rule:** One labeled pack. No “beats LightRAG” copy unless Δ Acc CI excludes 0 **and** L2 gates clear. Split Acc Fact / L2 Parity peers stay separate.

| Step | Pack | Pins | Success |
|------|------|------|---------|
| **L0** | `LR_IDENTITY_v1` | `MIX_FUSION=round_robin` · `enable_rerank=false` · `GRAPH_WALK=bfs` · `KG_CHUNK_PICK=VECTOR` · `KG_CHUNK_PICK_LR_BUDGET=1` · `ENTITY_RANK=retrieval` | ctx_rel Δ vs LR shrinks on smoke gate |
| **L1** | `LR_PACK_BM25_v1` ([075](./075-lr-pack-bm25-close-gap.md)) | L0 packing **+ BM25 on** | Fact ER✓ · ctx_rel tax (R3) |
| **L1.5** | `LR_IDENTITY_FACT_L2_v1` ([075](./075-lr-pack-bm25-close-gap.md)) | L0 + `L2_BM25 fact_replace` | **Best smoke:** Acc tie · ER≈LR · Fact ER 1.0 · ctx≥0.48 |
| **L1.5 mid** | medical-mid peer ([076](./076-mix-law-remaining-after-l15.md)) | same pins · n=200 · `publish/peers/` | Defend Acc CI + L2 at release scale |
| L1b | Ingest / B6 ge2 | Only if medical-mid under L1.5 still misses Fact ER | separate WS · not Acc silent |
| L2 | Labeled CE+protect | Acc Fact / L2 Parity peers only | never unlabeled headline |

### L0 gate result (`smoke-20260722T122410Z`)

| Metric | Acc headline Δ (medical-mid) | LR-identity Δ (smoke) | Verdict |
|--------|------------------------------|------------------------|---------|
| ctx_rel | −0.095 | **−0.013** | **PASS** — packing/fuse/rerank confounds bind L2 noise |
| evidence_recall | −0.064 | −0.062 | **MISS** — R2 remains (ingest / VECTOR density) |
| Acc | −0.068 (CI≠0) | −0.048 (CI includes 0) | Smoke tie only — **not** Acc Beat · publish stays medical-mid |

### Launch

```bash
export BENCH001_EQ_WORKSPACE_ID=<warm-full-corpus-ws>   # or resolve-warm-workspace
make bench001-lr-identity    # smoke n=40 query-only · profile LR_IDENTITY_v1
```

Artifacts: `specs/001-benchmark/e2e/artifacts/history/smoke-<utc>/` + `ABLATION_NOTE.md`.  
Does **not** overwrite `publish/latest` (`BENCH001_SKIP_PUBLISH_LATEST=1`).

---

## 6. Allowed / forbidden claims

| Allowed | Forbidden |
|---------|-----------|
| “LR Acc ahead on medical-mid under fair pins (CI excludes 0)” | “EQ beats LightRAG” |
| “EQ lags primarily on context cleanliness / Fact evidence” | Soft Mix Acc-win without CI+L2 |
| “LR-identity pack tests fusion/rerank/chunk-pick confounds” | Promoting LR-identity as product Acc default without gates |
| “EQ parallel Mix arms are better — keep” | Copying LR sequential arms for Acc |

---

## 7. Reading order

1. This memo (why we lag)  
2. [055 Post Acc-ceiling](./055-post-acc-ceiling-first-principles.md) — split peers  
3. [061 LR-as-law](./061-lightrag-law-first-principles-eq.md) — what to copy vs not  
4. [010 Retrieval noise](./010-lens-retrieval-noise.md) — L2 binding  
5. [019 Business brief](../019-business-eq-vs-lightrag-and-rag.md) — stakeholder language  
