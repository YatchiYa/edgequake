# 086 — Beat LightRAG (evidence-first · Acc-law migrate + ingest)

**Status:** Phase B in progress · packing fishing still STOP · Acc Beat reopen via Acc-law migrate + ingest  
**Date:** 2026-08-02  
**Parent:** [085](./085-fairness-concurrency-equal-stop.md) · [081](./081-beat-parity-first-principles.md) · [080](./080-beat-lightrag-evidence-roadmap.md) · [077](./077-dense-arms-fact-l2.md)  
**Keep query base (pre-migrate):** E2-B5 / `LR_OCC_FACT_L2_v1` [`T133053Z`](../e2e/artifacts/history/medical-mid-20260722T133053Z/)  
**Prior Acc headline (frozen peer):** P0 RRF mid [`T104918Z`](../e2e/artifacts/history/medical-mid-20260722T104918Z/) + Aug pack [`T135513Z`](../e2e/artifacts/history/medical-mid-20260802T135513Z/)  
**Product Equal:** [083](./083-lightrag-query-api-law.md) (unchanged)  
**Fair latency:** [063](./063-why-lightrag-faster-cache-fairness.md) `c1cold` only

---

## 1. North star (binding)

**Beat** only when **all** hold on the **same** pack (fair Mix↔Mix, Mistral Small + `mistral-embed`, top-k 30, Acc LLM cache off both sides for quality honesty):

1. Δ Acc 95% CI excludes 0 with EQ ahead  
2. `ctx_rel ≥ 0.50`  
3. overall ER ≥ LR − 0.03 **and** Fact ER ≥ LR − 0.03  
4. Same three gates on **medical-full n≈2062**  
5. Only then replace Acc `publish/latest` + refresh comparison SSOT

**Parity (mid)** = gates 1–3 on medical-mid (CI may include 0 if Acc ≥ tie). Mid Acc point lead alone is **not** Beat.

---

## 2. Evidence register (why reopen)

Latest Acc mid under **old P0** (`medical-mid-20260802T135513Z`):

| Layer | EQ | LR | Gate |
|-------|----|----|------|
| Acc | 0.807 | 0.779 | CI includes 0 → not Beat |
| ctx_rel | 0.420 | 0.499 | &lt; 0.50 |
| ER overall | 0.909 | 0.958 | gap −0.049 |
| Fact ER | 0.850 | 0.970 | gap −0.120 |

By-type: Fact ctx ≈ LR; **Fact ER membership** fails; **Creative ctx 0.155 vs 0.330** dominates overall ctx.

Best Acc-safe L2 peer (do not reopen packing fishing): **E2-B5** — Acc CI tie, ctx **0.491**, Fact ER **0.917**. Dense clears dual L2 but **Acc CI REJECT** — forbidden.

GraphRAG-Bench (ICLR 2026) + 2026 RAG Triad: Acc = generation; L2 = Evidence Recall + Context Relevancy. Answer score ≠ retrieval score.

---

## 3. Hard reject list (carry forward)

NF `RR_ORDER=naive_first` · dense `BM25_RETRIEVAL=0` · `post_truncate` · D1 unify · D2 intent-w · D3 relsel · Soft Mix / `TOPIC_*` · B7–B9 Acc promote · `response_type` / chat-split / concurrency Acc fishing · warm Acc mid latency as engine win · Acc `publish/latest` without Beat gates · cap-relation-chunks · always-on groundedness · synonym Acc fishing · packing reopen beyond the single E2-occ Acc-law migrate below.

---

## 4. Chosen approach

```text
P0 Acc (RRF+PPR+degree+BM25 rerank)
  → Fact ER −12pp · Creative ctx noise
Adopt E2-occ as Acc law (one deliberate migrate)
  → residual ~+0.01 ctx · ~+0.02–0.03 Fact ER
Ingest / EXTRACT / lineage (non-packing)
  → medical-mid Parity → medical-full Beat → publish/latest
```

### Phase A — Acc law migrate (not fishing)

Make E2-occ Mix identity the **new Acc headline pins** (SSOT: `start_acc_backend.py` + `acc_env.PUBLICATION_ENV`).

| Knob | Old P0 | New Acc law (E2-occ) |
|------|--------|----------------------|
| `MIX_FUSION` / `HYBRID_FUSION` | `rrf` | `round_robin` |
| `BENCH001_ALLOW_ROUND_ROBIN` | unset | `1` |
| `BENCH001_EQ_ENABLE_RERANK` | `1` | `0` |
| `GRAPH_WALK` | `ppr` | `bfs` |
| `ENTITY_RANK` | `degree` | `retrieval` |
| `KG_CHUNK_PICK_LR_BUDGET` | `0` | `1` |
| `KG_CHUNK_OCCURRENCE_SORT` | `0` | `1` |
| `L2_BM25_UNION` / `MODE` | off / union | `1` / `fact_replace` |
| Keep | arm gate off · chunk 1200/100 · top-k 30 · `EDGEQUAKE_LLM_CACHE=0` · `BM25_RETRIEVAL=1` · `RR_ORDER=local_first` | |

**Profile id:** `ACC_E2OCC_086_v1`  
**Peer label (Phase A):** `ACC_E2OCC_086_v1` under `publish/peers/`  
**Publish latest:** **SKIP** until Beat (set `BENCH001_SKIP_PUBLISH_LATEST=1` unless `BENCH001_ALLOW_PUBLISH_LATEST=1`).

**Phase A success:** medical-mid matches E2-B5 band — ctx ≥ 0.48 · Fact ER ≥ 0.90 · Acc CI not LR-ahead.  
**Phase A fail:** Acc CI LR-ahead → **revert** Acc pins to prior P0 and STOP Beat reopen.

```bash
export MISTRAL_API_KEY=...
export BENCH001_EQ_WORKSPACE_ID=<warm-full-corpus-ws>   # or resolve-warm-workspace
make bench001-086-phase-a
```

### Phase B — Close residual L2 via ingest/graph

One confound each; forensics first (Fact LR-wins → membership vs generation):

1. **EXTRACT≠QUERY ingest quality** ([061 Idea F](./061-lightrag-law-first-principles-eq.md)) — schema-strong EXTRACT; QUERY Acc pin; new WS force-ingest.  
2. **Source-chunk lineage completeness** — repair only if membership forensics show missing gold chunk ids.  
3. **Naming / entity identity fidelity** (B10 law) — promote only if mid gates improve without Acc CI tax.  
4. **Opt-in grounded retry** — Acc Acc only if CI improves; never for ctx.

**Phase B success:** ctx ≥ 0.50 · Fact ER ≥ LR−0.03 · Acc CI not LR-ahead.  
**Phase B fail (three ingest confounds):** freeze Beat again; do not invent Mix knobs.

### Phase C — Promote ladder

1. medical-mid under Acc law + best ingest WS → Parity gates  
2. If Parity green: medical-full n≈2062 → Beat gates  
3. `BENCH001_ALLOW_PUBLISH_LATEST=1 make bench` (or medical-full promote) → replace `publish/latest`  
4. Update [080c checklist](./080-phase-g-promote-checklist.md) · [019](../019-business-eq-vs-lightrag-and-rag.md) · [eq-vs-lightrag-acc-bench](../../../docs/comparisons/eq-vs-lightrag-acc-bench.md) · `peers.json`  
5. Keep old P0 and E2 archives as labeled peers forever

### Phase D — Orthogonal (not Beat)

Product Equal [083](./083-lightrag-query-api-law.md) · SPEC-103 product cache ON · fair latency = `c1cold` only.

---

## 5. Stop rules (086)

- One confound per experiment; label every pin in scorecard  
- No packing reopen beyond the single E2-occ Acc-law migrate  
- No `publish/latest` replace until Beat gates on medical-full (require `BENCH001_ALLOW_PUBLISH_LATEST=1`)  
- If Phase A Acc CI goes LR-ahead → revert Acc pins to prior P0 and STOP Beat reopen  
- If Phase B three ingest confounds fail Fact ER ≥ LR−0.03 → freeze Beat again

---

## 6. Success metrics

| Checkpoint | Acc CI | ctx_rel | Fact ER |
|------------|--------|---------|---------|
| A: Acc-law migrate mid | not LR-ahead | ≥ 0.48 | ≥ 0.90 |
| B: ingest close mid | not LR-ahead | ≥ 0.50 | ≥ LR−0.03 |
| C: Beat mid | EQ-ahead excludes 0 | ≥ 0.50 | ≥ LR−0.03 |
| C: Beat full | EQ-ahead excludes 0 | ≥ 0.50 | ≥ LR−0.03 |

---

## 7. Code / ops map

| Surface | Path |
|---------|------|
| Acc backend pins | `tools/bench001/scripts/start_acc_backend.py` |
| Publication env force | `tools/bench001/bench001/acc_env.py` `PUBLICATION_ENV` |
| Phase A make target | `make bench001-086-phase-a` |
| Ladder peer (historical) | `make bench001-medical-mid-lr-occ-fact-l2` |
| Forensics | `tools/bench001/scripts/failure_slice_eq_lr.py` |

---

## 8. Program status log

| Step | Status | Notes |
|------|--------|-------|
| A0 memo + index | Done | this file · `000-index` |
| A1 Acc pin SSOT E2-occ | Done | start_acc_backend + PUBLICATION_ENV |
| A2 labeled mid `SKIP_PUBLISH_LATEST` | Done | peer `ACC_E2OCC_086_v1` · archive `medical-mid-20260802T141536Z` |
| A2 gate verdict | **Band FAIL** · Acc CI keep | Acc 0.786/0.781 CI [−0.028,+0.035] **not LR-ahead** · ctx **0.431** (&lt;0.48) · Fact ER **0.87** (&lt;0.90). Warm WS = `e3216f05-…` (bench001-smoke); historical B5 WS gone. **No Acc pin revert** (CI keep). |
| A2 forensics | Done | [`086-phase-a-e2occ`](../e2e/artifacts/forensics/086-phase-a-e2occ/FAILURE_SLICE.md) — Fact LR-wins **100% generation** (membership_share=0); Fact_ER_gap still open (0.87 vs 0.98) |
| B1 force-ingest same EXTRACT=QUERY | **FAIL gates** | peer `ACC_E2OCC_086_B1_INGEST_v1` · `medical-mid-20260802T143215Z` · WS `a6682988-…` · Acc 0.792/0.783 CI keep · ctx **0.425** · Fact ER **0.88**/0.95 (need ≥0.92). Forensics: Acc Fact LR-wins **100% generation**; Fact_ER_gap open |
| B2 EXTRACT≠QUERY medium | **FAIL gates** | peer `ACC_E2OCC_086_B2_EXTRACT_MEDIUM_v1` · `medical-mid-20260802T150441Z` · Acc 0.793/0.769 CI keep · ctx **0.427** · Fact ER **0.88**/0.96. Medium EXTRACT does not close L2; denser graph only |
| C medical-full + publish | Blocked | Parity gates unmet — do not ALLOW_PUBLISH_LATEST |
