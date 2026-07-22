# 021 — Grounded Plan: Cleanliness, Multi-hop/Summarize, Latency

**Status:** Active post–E4 program (F1–F4) · superseded for Acc-loss recovery by **[022](./022-deep-top-performance-plan.md)**  
**Date:** 2026-07-20  
**Cross-ref:** [022 Deep top plan](./022-deep-top-performance-plan.md) · [018 E4 close](./018-e4-acc-tie-close.md) · [020 Roadmap](./020-roadmap.md) · [019 Business brief](../019-business-eq-vs-lightrag-and-rag.md) · [013 Latency](./013-lens-latency-ops.md)

---

## 1. One-screen

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│  Soft Mix Acc-win exhausted (E4 tie). Next: hard paths grounded in code +    │
│  GraphRAG-Bench / HippoRAG2 / LightRAG research.                             │
│                                                                              │
│  F1 Truncation Summarize floor → F2 Path-serialized packing →                │
│  F3 Latency ops (arms already parallel) → F4 labeled HippoRAG2 packing       │
│                                                                              │
│  Acc headline stays BM25 / PRUNE=0 / PATH_PRUNE=0 / PROTECT_FIRST=0.         │
│  Soft path only with CE+protect (never BM25+path — T011703Z confound).       │
└──────────────────────────────────────────────────────────────────────────────┘
```

---

## 2. Reality check (do not re-learn)

| Fact | Source |
|------|--------|
| Acc vs LR = statistical **tie**; soft Mix knobs exhausted | [018](./018-e4-acc-tie-close.md) |
| Best labeled cleanliness = S1 CE+protect (`T151125Z`) | [020](./020-roadmap.md) |
| E2 entity reorder / E3 related_chunk=8 / E3b naive×2 **failed** | `T153959Z` / `T154427Z` / `T155350Z` |
| Mix arms **parallel**; keyword cache **shipped** | `mix.rs` + `arm_timed.rs` + `CachedKeywordExtractor` |
| Bipartite PPR walk default | `EDGEQUAKE_GRAPH_WALK=ppr` |
| Lens 013 “serial arms” is **stale** | Corrected in [013](./013-lens-latency-ops.md) |

**Research anchors:** GraphRAG-Bench (ICLR 2026) · HippoRAG2 (arXiv:2502.14802) · LightRAG Mix parallel KG+vector + local rerank preference.

---

## 3. Tracks

### F1 — Truncation / Summarize budget

| Item | Detail |
|------|--------|
| Hook | `truncation.rs` `truncation_config_for_intent` — Exploratory/Relational/Comparative get chunk floor ≥0.60 + tighter E/R caps |
| Intent source | **LLM** `query_intent` from keyword extractor (stashed on context metadata); heuristic only as fallback |
| Env | `EDGEQUAKE_MIN_CHUNK_BUDGET_RATIO` still global override; intent raises floor when higher |
| Acc base | S1 CE+protect |
| Gate | Summarize evidence_recall ≥ 0.95 (or ≥ LR−0.03); Acc drop ≤0.02; ctx_rel ≥0.50 discovery |

### F2 — Path-serialized multi-hop packing

| Item | Detail |
|------|--------|
| Hook | `context_format.rs` + env `EDGEQUAKE_CONTEXT_FORMAT=path` (default `flat`) |
| Profile | `path_pack_v1`: format=path + soft `PATH_PRUNE=0.4` + S1 CE+protect |
| Gate | Complex ΔF1 vs LR ≤ 0.03; Summarize within F1 budgets |

### F3 — Latency

| Item | Detail |
|------|--------|
| Done | Parallel Mix arms; keyword cache |
| Next | Stage timing in Acc predictions/SUMMARY; fair concurrency remeasure; prefill from F1/F2; CE product vs BM25 |
| SLO | EQ p50 ≤ 1.5× LR matched concurrency **or** waiver with stage breakdown |
| Env | `EDGEQUAKE_QUERY_ARM_CONCURRENCY` ≥ 3× `BENCH001_QUERY_CONCURRENCY` for fair Acc |

### F4 — Labeled HippoRAG2 packing (research)

| Item | Detail |
|------|--------|
| Hook | `EDGEQUAKE_PASSAGE_PACK=1` prefers passage-centric prompt (chunks first / compact graph); tune `EDGEQUAKE_PPR_*` |
| Rule | Labeled profile only — never silent Acc default |
| Target | High recall + ctx_rel with **lower** prompt tokens than S1 Mix dump |

---

## 4. Experiment order

1. F1a — Summarize intent truncation only (S1 base)  
2. F1b — Confirm if green  
3. F2a — Path format + soft path  
4. F3a — Stage timing + concurrency remeasure  
5. F3b — Prefill/compact from winning F1/F2 → latency SLO  
6. F4 — Passage packing research run  

Each: `ABLATION_NOTE.md` + scorecard pins + one confound.

### Launch

```bash
# Headline Acc (BM25 / publication pins) + business publish pack
make bench
# → specs/001-benchmark/e2e/artifacts/publish/latest/BUSINESS_REPORT.md

# F1–F4 labeled Acc ladder (warm query-only; S1 CE base)
export BENCH001_EQ_WORKSPACE_ID=<warm-full-corpus-ws>
export DASHSCOPE_API_KEY=...
cargo build --release --bin edgequake
make bench001-f1a    # or bench001-f2a / f3a / f4a
# script: tools/bench001/scripts/run_f_ladder_acc.sh f1a
```

---

## 5. Code / env cheat sheet

| Knob | Default | Role |
|------|---------|------|
| `EDGEQUAKE_MIN_CHUNK_BUDGET_RATIO` | 0.40 | Global chunk floor |
| Intent Exploratory/Relational/Comparative | floor max(base, **0.60**) | F1 Summarize-like |
| `EDGEQUAKE_CONTEXT_FORMAT` | `flat` | `path` = path-serialized blocks |
| `EDGEQUAKE_PATH_PRUNE` | off when unset for Acc S1 | Soft 0.4 with `path_pack_v1` |
| `EDGEQUAKE_PASSAGE_PACK` | `0` | F4 chunks-first compact graph |
| `EDGEQUAKE_QUERY_ARM_CONCURRENCY` | 4 | Raise for Acc fairness |

Frozen Acc fairness: `MIX_ARM_GATE=false`, `RELATED_CHUNK=5`, `MIX_FUSION=rrf`, chunk 1200/100, mistral-small + mistral-embed, top-k 30.

---

## 6. Definition of done

- [x] This doc + 013 L1 corrected + 020/000/019 linked  
- [ ] F1 Summarize gate met or falsified  
- [ ] F2 Complex gate met or falsified  
- [ ] F3 p50 SLO or waiver with stages  
- [ ] Acc headline defaults unchanged unless CI excludes 0  
- [x] Business brief next investments → F1–F4  
