# 080 — Beat LightRAG (evidence roadmap)

**Status:** D0–D4 executed · packing STOP · Acc Beat fishing STOP · continue in **[088](./088-beat-ctx-fact-er-program.md)**  
**Date:** 2026-07-23 · **refresh 2026-08-16**  
**Acc SSOT now:** E2-occ 086 mid [`T110218Z`](../e2e/artifacts/history/medical-mid-20260815T110218Z/) · `publish/latest` (EQ Acc **0.792** / LR **0.786** tie) · warm WS `23b09c73-…`  
**Acc-law full:** [`T012004Z`](../e2e/artifacts/history/medical-full-20260816T012004Z/) · peer `ACC_E2OCC_086_MEDICAL_FULL_v1` (0.786/0.786 point tie; ctx 0.427 — **not Beat** · chunk **1200/100**)  
**Keep query base (Jul-22 CI):** E2 occ on B5 [`T133053Z`](../e2e/artifacts/history/medical-mid-20260722T133053Z/)  
**Fact ER label:** E2 on B6 ge2 [`T013716Z`](../e2e/artifacts/history/medical-mid-20260723T013716Z/) (not gap-close keep)  
**Historical Acc SSOT (Jul-22 rows below):** P0 mid [`T104918Z`](../e2e/artifacts/history/medical-mid-20260722T104918Z/)  
**Prior:** [079](./079-medical-full-scale-compare.md) · [078](./078-eq-vs-lightrag-first-principles-next.md) · [055](./055-post-acc-ceiling-first-principles.md)  
**Next:** [088](./088-beat-ctx-fact-er-program.md) — query-only L2 exhausted; Acc ingest **chunk 1200/100**; Phase G still blocked on ctx

---

## 1. North star (binding)

**Publish Beat** only when **all** hold on **medical-full n=2062** (fair Mix↔Mix, Mistral Small + mistral-embed, top-k=30, LR `enable_rerank=false`, EQ parallel Mix kept):

1. Δ Acc 95% CI excludes 0 **with EQ ahead**
2. `ctx_rel ≥ 0.50`
3. overall ER ≥ LR − 0.03 **and** Fact ER ≥ LR − 0.03
4. Only then replace Acc `publish/latest`

Split peers forever: Acc headline ≠ gap-close E2 ≠ Acc Fact smoke ≠ L2 Parity ≠ warm latency.

Scope: GraphRAG-Bench medical Acc + L2 + fair-cold latency + product TTFT/UX. Not HippoRAG2 / UltraDomain.

---

## 2. Scorecard (leverage guide)

| Surface | n | EQ Acc | LR Acc | Acc Δ CI | EQ ctx | Fact ER |
|---------|---|--------|--------|----------|--------|---------|
| Acc headline 086 mid (**SSOT now**) | 200 | 0.792 | 0.786 | tie [−0.022, +0.034] | 0.471 | 0.847 |
| Acc-law 086 full | 2062 | 0.786 | 0.786 | tie [−0.160, +0.047] | 0.427 | 0.914 |
| Acc headline P0 mid (Jul-22) | 200 | 0.706 | 0.774 | LR [−0.107, −0.033] | 0.396 | 0.790 |
| Gap-close E2 occ mid | 200 | 0.765 | 0.760 | tie [−0.031, +0.040] | 0.491 | 0.917 |
| D1 unify mid (**REJECT**) | 200 | 0.734 | 0.787 | LR [−0.084, −0.022] | 0.503 | 0.903 |
| D2 intent-w mid (**REJECT**) | 200 | 0.718 | 0.764 | LR [−0.082, −0.014] | 0.477 | 0.913 |
| D4 E2 on B6 ge2 (**label**) | 200 | 0.750 | 0.775 | tie [−0.062, +0.008] | 0.459 | **0.930** |
| E2 full | 2062 | 0.739 | 0.784 | LR [−0.069, −0.017] | 0.472 | 0.918 |
| P0 full | 2062 | 0.724 | 0.784 | LR [−0.107, −0.042] | 0.394 | 0.905 |

**Law:** Acc follows admitted context. Jul-22 mid E2 Acc tie did **not** hold at full-N; Acc-law 086 full (chunk **1200/100**) **closed** that Acc scale gap (point tie) but ctx remains the Beat blocker. Residual = ctx&lt;0.50 (full 0.427).

### Do not retry

NF `naive_first` · dense `BM25_RETRIEVAL=0` · R3 `post_truncate` · Soft Mix / TOPIC_* Acc fishing · warm LR “4–5×” as engine win (063 cold ≈1.01×) · **D1 `L2_BM25_MODE=unified` on Acc** (ctx↑ but Acc/Fact ER↓ vs E2 — keep `fact_replace`) · **D2 `MIX_INTENT_WEIGHTS=1` on Acc** (Acc CI LR-ahead mid) · **D3 `RELATION_SELECT=lightrag`** (smoke Acc↓).

---

## 3. Program phases

| Phase | Work | Exit |
|-------|------|------|
| **D0** | Failure forensics on E2 mid/full | **Done** → `e2e/artifacts/forensics/d0-e2-{mid,full}/` · next **D1** |
| **D1** | R6 Acc/L2 list unify (`L2_BM25_MODE=unified`) | **REJECT** mid [`T011525Z`](../e2e/artifacts/history/medical-mid-20260723T011525Z/) — Acc CI LR-ahead; Fact ER 0.903&lt;LR−0.03; ctx 0.503 only win. No medical-full unify. Keep E2 `fact_replace`. |
| **D2** | Type-aware Mix weights (`MIX_INTENT_WEIGHTS=1`) on E2 base (no unify) | **REJECT** mid [`T012436Z`](../e2e/artifacts/history/medical-mid-20260723T012436Z/) — Acc CI LR-ahead; ctx 0.477; Fact ER ≈flat. Keep E2. |
| **D3** | `RELATION_SELECT=lightrag` last-resort | **STOP** smoke [`T012653Z`](../e2e/artifacts/history/smoke-20260723T012653Z/) — Acc −8.5pp vs E2 OCC smoke; Fact ER flat. No mid/full. See [080-d3-d4-deferred](./080-d3-d4-deferred.md) |
| **D4** | Ingest ge2 / source_id ceiling (labeled WS) | **Done (label)** — B5 Acc WS ge2=0%; B6 WS ge2=12.5% ([audit](../e2e/artifacts/ingest-audit/20260723T013324Z/)). E2 mid on B6 [`T013716Z`](../e2e/artifacts/history/medical-mid-20260723T013716Z/): Fact ER **0.930** (≥LR−0.03) but Acc/ctx below E2-B5 keep. Warm restored to B5; labeled peers no longer hijack warm. |
| **D5** | Empty-answer reliability (R5) | Code: retry + extractive fallback |
| **L** | TTFT + opt-in answer cache + c1cold ≤1.5× | **Done** in [064](./064-product-ttft-cache-batch-embed.md) |
| **G** | Promote Acc latest | **Blocked** — Acc-law full **ran** (`T012004Z`) but Beat gates unmet (ctx 0.427) |

```bash
# D0
PYTHONPATH=tools/bench001 python3 tools/bench001/scripts/failure_slice_eq_lr.py \
  --eq-archive specs/001-benchmark/e2e/artifacts/history/medical-full-20260722T171906Z \
  --lr-peer-same \
  --out specs/001-benchmark/e2e/artifacts/forensics/d0-e2-full

# D1 ladder (E2 base + unify)
make bench001-lr-unify-fact-l2
make bench001-medical-mid-lr-unify-fact-l2
# medical-full only if mid KEEP
make bench001-medical-full-lr-unify-fact-l2
```

---

## 4. Code map

| Concern | Files |
|---------|-------|
| L2 / Acc list (R6) | `edgequake-query/src/l2_bm25_union.rs`, `query_pipeline.rs` |
| Type Mix weights | `mix_weights.rs`, `keywords/llm_extractor.rs` |
| Relation select | relation-select / local arm (051) |
| Empty answers | generate / `prompt.rs` fail path |
| Answer cache / TTFT | `cache.rs`, query engine `with_answer_cache_from_env` |
| Ladder | `tools/bench001/scripts/run_p_ladder_acc.sh`, `Makefile` |

---

## 5. Promote checklist (Phase G)

- [ ] Winner pack medical-mid: Beat CI + ctx≥0.50 + ER gates  
- [x] Acc-law 086 medical-full n=2062 **ran** ([`T012004Z`](../e2e/artifacts/history/medical-full-20260816T012004Z/), chunk **1200/100**) — Acc point tie; ctx FAIL — **not** Beat  
- [ ] Same pack medical-full n=2062: Beat CI + ctx + ER  
- [ ] Acc `publish/latest` → new archive; `peers.json` + [019](../019-business-eq-vs-lightrag-and-rag.md)  
- [x] Gap-close / latency / Acc-law full peers stay labeled  

Until then: **do not claim EQ beats LightRAG.**
