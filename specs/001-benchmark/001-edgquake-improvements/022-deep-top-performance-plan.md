# 022 — Deep Top-Performance Plan (post T011703Z)

**Status:** Ladder complete (P0–P5) — **no Acc headline promote**; best peer = P2b  
**Date:** 2026-07-20  
**Trigger:** [`publish/latest/BUSINESS_REPORT.md`](../e2e/artifacts/publish/latest/BUSINESS_REPORT.md) — EQ Acc **0.699** vs LR **0.789** (CI excludes 0)  
**Cross-ref:** [021 F1–F4](./021-grounded-improvement-plan.md) · [018 E4 close](./018-e4-acc-tie-close.md) · [017 Beat LightRAG](./017-beat-lightrag.md) · [019 Business](../019-business-eq-vs-lightrag-and-rag.md)

---

## 1. One-screen

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│  T011703Z = Acc LOSS (−0.090) with BM25 + path=0.4 confound.                 │
│  Soft Mix Acc-win exhausted. Soft path alone on BM25 is FORBIDDEN.           │
│                                                                              │
│  P0 PATH=0 restore → P1 graph-walk compress → P2 LR packing →                │
│  P3 lexical/truncation → P4 Acc CI gate → P5 latency ≤1.5×                   │
│                                                                              │
│  “At the top” = Δ Acc CI excludes 0 (EQ) + ctx_rel ≥0.50 + recall ≥LR−0.03   │
└──────────────────────────────────────────────────────────────────────────────┘
```

---

## 2. Binding constraints (do not re-learn)

| Fact | Evidence |
|------|----------|
| Best Acc-tie = `T124903Z` (BM25, path **off**) | EQ 0.765 / LR 0.754 |
| Soft Mix knobs exhausted | E2–E3b ([018](./018-e4-acc-tie-close.md)) |
| High recall ≠ win | EQ recall 0.95 but ctx_rel 0.38 on T011703Z |
| Complex / Summarize drive Acc gap | −14pp / −12pp Acc by type |
| Latency dominated by embed+generate | stage p50 ~2388 + 2278 ms |

**Exhausted (do not re-run):** related_chunk↑, naive_weight×2, query_score entity rank alone, CE without protect Acc fishing, BM25+path.

---

## 3. Shipped code hooks

| Phase | Env / code | Files |
|-------|------------|-------|
| **P0** | `EDGEQUAKE_PATH_PRUNE=0` Acc default | `acc_env.py`, `start_acc_backend.py`, `fair_pins.py` |
| **P1** | `EDGEQUAKE_GRAPH_WALK_COMPRESS=1` | `graph_walk_compress.rs` → `query_pipeline.rs` |
| **P2** | `ENTITY_RANK=retrieval`, `MIX_FUSION=round_robin`, `CONTENT_HEADINGS=1`, path only with CE | `entity_rank.rs`, `context_format.rs`, ladder script |
| **P3** | `KEYWORD_LEXICAL_BOOST=1`, `POPULAR_NODE_FALLBACK=0`, `query_intent` on stats | `keyword_boost.rs`, `local.rs`/`global.rs`, `QueryStats` |
| **P4** | Decision profile `P4_acc_ci_decision_v1` | `run_p_ladder_acc.sh p4` |
| **P5** | `QUERY_ARM_CONCURRENCY=16` Acc default (24 for p5) | `start_acc_backend.py`, ladder p5 |

### Launch

```bash
cargo build --release --bin edgequake
make bench-warm                    # P0-class Acc (path off)
make bench001-p0                   # explicit P0 restore ladder
make bench001-p1a                  # gw compress BM25
make bench001-p1b                  # gw compress S1 (needs DASHSCOPE)
make bench001-p2a                  # round_robin fusion
make bench001-p2b                  # lr_pack on S1
make bench001-p3a / bench001-p3b
make bench001-p4                   # Acc CI decision (promote only if gates)
make bench001-p5                   # latency remeasure
# script: tools/bench001/scripts/run_p_ladder_acc.sh <step>
```

---

## 4. Gates

| Step | Success |
|------|---------|
| P0 | path pin **0**; EQ Acc within ±0.03 of T124903Z; CI includes 0 |
| P1 | Complex Acc Δ ≤0.05; ctx_rel ≥0.48; Fact drop ≤0.02 |
| P2 | Complex ΔF1 ≤0.03 **or** Complex Acc Δ ≤0.05 |
| P3 | Summarize evidence_recall ≥0.95 (or ≥LR−0.03) |
| P4 | Δ Acc CI excludes 0 **and** ctx_rel ≥0.50 → promote; else honest peer |
| P5 | EQ/LR p50 ≤1.5× at matched concurrency (or waiver with stages) |

---

## 5. P4 promotion rule (hard)

Promote knobs to Acc headline **only if all** hold on n=40 publishable smoke:

1. Δ Acc 95% bootstrap CI **excludes 0** in EQ’s favor  
2. EQ `context_relevancy` ≥ **0.50** (stable ≥2/3 runs)  
3. EQ evidence_recall ≥ LR − 0.03  
4. `ABLATION_NOTE.md` + scorecard pins record the package  

Until then: headline stays BM25 / `PATH_PRUNE=0` / `PROTECT_FIRST=0` / `PRUNE=0`.

---

## 6. Acc ladder ledger (warm `8b359190-…`)

| Step | Archive | EQ Acc | LR Acc | Notes |
|------|---------|--------|--------|-------|
| **P0** | `T013551Z` | **0.744** | 0.794 | Gate met: path=0, CI⊂0, Acc within ±0.03 of T124903Z |
| **P1a** | `T013827Z` | 0.721 | 0.784 | Miss: gw on BM25; ctx_rel 0.375; Complex Δ −0.226 |
| **P1b** | `T014115Z` | 0.739 | 0.776 | Partial: ctx_rel **0.494**; Complex still −0.132 |
| **P2a** | `T014452Z` | 0.723 | 0.791 | Miss: RR fusion Acc tax; Complex Δ −0.154; keep RRF |
| **P2b** | `T014814Z` | **0.752** | 0.780 | **Gate met**: Complex Acc Δ −0.023; ctx_rel **0.500**; Acc +P0 |
| **P3a** | `T015129Z` | 0.714 | 0.771 | Audit OK: Summarize recall **0.950**; `query_intent` 40/40 |
| **P3b** | `T015406Z` | 0.713 | 0.766 | Miss alone: Summarize recall 0.900 on BM25+lexical |
| **P4** | `T015647Z` | 0.677 | 0.788 | **No promote**: CI favors LR; recall miss; stack Acc-toxic |
| **P5** | `T015951Z` | 0.721 | 0.789 | Latency SLO miss (5.4×); waiver: embed+generate dominate |

**Best peer package (not headline):** P2b `T014814Z` — EQ Acc 0.752 / ctx_rel 0.500 / Complex Δ −0.023.

**Acc headline (unchanged):** BM25 / `PATH_PRUNE=0` / `PROTECT=0` / RRF — P0 restore class.

---

## 7. Non-goals

- Reopening soft Mix Acc-win fishing  
- Silent BM25 + path_prune (T011703Z failure mode)  
- Claiming SOTA / “beats LightRAG” without P4 gates  
- Changing LightRAG upstream  

