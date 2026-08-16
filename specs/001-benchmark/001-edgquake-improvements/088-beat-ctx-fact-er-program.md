# 088 — Beat ctx_rel + Fact ER program (First Principles)

**Status:** Active · Acc Beat fishing STOP until Phase G green  
**Date:** 2026-08-16  
**Parent:** [080](./080-phase-g-promote-checklist.md) · [081](./081-beat-parity-first-principles.md) · [055](./055-post-acc-ceiling-first-principles.md)  
**Acc SSOT:** [`medical-mid-20260815T110218Z`](../e2e/artifacts/history/medical-mid-20260815T110218Z/) · `publish/latest`  
**Forensics:** [`f1-t110218z`](../e2e/artifacts/forensics/f1-t110218z/)

---

## 1. Beat gates (binding)

Same winner pack must clear:

1. Δ Acc 95% CI excludes 0 with EQ ahead  
2. `ctx_rel ≥ 0.50`  
3. overall ER ≥ LR−0.03 **and** Fact ER ≥ LR−0.03  
4. Same on medical-full n=2062  

Then: replace Acc `publish/latest` + `peers.json` + [019](../019-business-eq-vs-lightrag-and-rag.md).

---

## 2. Baseline (T110218Z)

| Metric | EQ | LR | Gate |
|--------|-----|-----|------|
| Acc | 0.792 | 0.786 | tie CI · FAIL Beat |
| ctx_rel | 0.471 | 0.510 | FAIL (&lt;0.50) |
| overall ER | 0.932 | 0.949 | PASS |
| Fact ER | 0.847 | 0.950 | FAIL (need ≥0.920) |

**W0 forensics:** Fact LR-wins **generation_share=1.0** (gold in EQ Acc context). Summarize Acc Δ mean −0.029 (LR ahead). Fact_ER_gap mode still open.

---

## 3. Program peers (one confound)

| Step | Peer | Pin | Exit |
|------|------|-----|------|
| W1 | `CTX_GWC_v1` | `EDGEQUAKE_GRAPH_WALK_COMPRESS=1` only | KEEP: ctx↑ / Acc CI≥tie / Fact ER≥base−0.01 |
| W2 | `CTX_COSINE_v1` | cosine prune KEEP=12 · no CE/path | same; one soften then STOP |
| W3 | `FACT_ER_L2_v1` | citation/membership honesty (query-only) | Fact ER ≥ LR−0.03 |
| W4 | Summarize policy | only if L2 near-green | Summarize Acc ≥ LR · ctx hold |
| G | Acc promote | mid+full same pack | all 080 boxes |

Warm latency peer `EQ_LLM_CACHE_WARM_v1` and cold `C1COLD_v1` stay labeled — **not** Acc Beat.

---

## 4. Stop / forbidden

NF · dense BM25=0 · post_truncate · D1 unify Acc · D2 intent-w Acc · D3 relsel · TOPIC_* / PASSAGE_PACK · F3 naming Acc · F4 always-on groundedness · G1 gold Acc · CE+path Acc stack · silent `publish/latest` overwrite · Soft Mix packing fishing.

---

## 5. Reproduce

```bash
# W0
PYTHONPATH=tools/bench001 python3 tools/bench001/scripts/failure_slice_eq_lr.py \
  --archive specs/001-benchmark/e2e/artifacts/history/medical-mid-20260815T110218Z \
  --out specs/001-benchmark/e2e/artifacts/forensics/f1-t110218z

# W1
export BENCH001_EQ_WORKSPACE_ID=23b09c73-aa3f-4497-8e11-c448ffad8c53
export EDGEQUAKE_GRAPH_WALK_COMPRESS=1
export BENCH001_QUERY_ONLY=1 BENCH001_SKIP_PUBLISH_LATEST=1
export BENCH001_PUBLISH_PEER=CTX_GWC_v1
make bench001-acc-backend && BENCH001_SKIP_BACKEND_RESTART=1 make bench001-medical-mid

# W3
export EDGEQUAKE_GRAPH_WALK_COMPRESS=0
export EDGEQUAKE_L2_FACT_BM25_POOL=acc
export BENCH001_PUBLISH_PEER=FACT_ER_L2_v1
make bench001-acc-backend && BENCH001_SKIP_BACKEND_RESTART=1 make bench001-medical-mid
```

## 6. Results log

### W0 — forensics `f1-t110218z` (2026-08-15)

- Fact LR-wins **generation_share=1.0**
- Summarize Acc Δ mean −0.029
- Fact ER gap mode open (0.847 / 0.950)

### W1 — `CTX_GWC_v1` archive `medical-mid-20260815T133601Z` — **KEEP (ctx)**

| Metric | Baseline T110218Z | GWC | Gate |
|--------|-------------------|-----|------|
| Acc | 0.792 / 0.786 tie | 0.763 / 0.792 tie CI | Acc CI ≥tie KEEP; point tax |
| ctx_rel | 0.471 | **0.501** | **≥0.50 KEEP** |
| overall ER | 0.932 | 0.942 | PASS |
| Fact ER | 0.847 | 0.876 | ≥base−0.01 KEEP; still &lt;0.920 Beat |

Pin: `EDGEQUAKE_GRAPH_WALK_COMPRESS=1` only. Soft Mix off.

### W2 — SKIP

ctx_rel gate met by W1; do not Acc-fish cosine on this cycle.

### W3 — `FACT_ER_L2_v1` archive `medical-mid-20260815T135129Z` — **KEEP (Fact ER progress)**

| Metric | Baseline T110218Z | FACT_ER | Gate |
|--------|-------------------|---------|------|
| Acc | 0.792 / 0.786 tie | 0.775 / 0.773 tie CI | Acc CI ≥tie KEEP |
| ctx_rel | 0.471 | 0.471 | flat · FAIL Beat |
| overall ER | 0.932 | 0.949 | PASS |
| Fact ER | 0.847 | **0.896** vs LR 0.930 | ≥base−0.01 KEEP; Beat ≥0.900 miss −0.004 |

Pin: `EDGEQUAKE_L2_FACT_BM25_POOL=acc` only. Soft Mix off. Mechanism: Fact L2 BM25 over Acc-admitted chunks (judge sources window honesty).

### Mid candidate — `MID_GWC_FACT_v1` archive `medical-mid-20260815T135805Z` — **REJECT (composition)**

| Metric | GWC alone | FACT alone | Combined | Beat Mid |
|--------|-----------|------------|----------|----------|
| Acc | 0.763/0.792 tie | 0.775/0.773 tie | 0.776/0.772 tie | need EQ-ahead CI |
| ctx_rel | **0.501** | 0.471 | **0.473** | ≥0.50 FAIL |
| Fact ER | 0.876 | **0.896** | **0.908** | ≥ LR−0.03 (0.920) FAIL |

Pins: `GRAPH_WALK_COMPRESS=1` + `L2_FACT_BM25_POOL=acc`. Soft Mix off.

### Mid candidate — `MID_GWC_FACT_PRE_v1` archive `medical-mid-20260815T141834Z` — **REJECT**

Coupled lever: GWC + `L2_FACT_BM25_POOL=acc` + `POOL_PRE_COMPRESS=1` (citation pool = pre-compress snapshot).

| Metric | EQ | LR | Beat Mid |
|--------|-----|-----|----------|
| Acc | 0.768 | 0.782 | CI includes 0, LR point ahead FAIL |
| ctx_rel | 0.485 | 0.491 | <0.50 FAIL |
| Fact ER | 0.905 | 0.940 | <0.910 FAIL |

**First-principles read (from traces):** Fact LR-wins have gold in EQ ctx (membership OK) but EQ ctx precision ≈0.004 vs LR 0.005 — the generation miss is an **SNR** problem, not membership. GWC raises SNR (ctx); Acc-honest BM25 raises Fact ER; they do not compose additively.

### Mid candidate — `MID_GWC_P3_FACT_v1` archive `medical-mid-20260815T142806Z` — **REJECT (hard compress)**

GWC + `L2_FACT_BM25_POOL=acc` + `NAIVE_PROTECT=3`.

| Metric | EQ | LR | Beat Mid |
|--------|-----|-----|----------|
| Acc | 0.794 | 0.786 | tie CI |
| ctx_rel | 0.478 | 0.494 | <0.50 FAIL |
| Fact ER | 0.894 | 0.950 | <0.920 FAIL |

**Lever grid (Fact ctx / ctx_rel / Fact ER):** baseline 157k/0.471/0.847 · GWC 155k/**0.501**/0.876 · pool=acc 115k/0.471/0.896 · GWC+acc 115k/0.473/0.908 · +pre 156k/0.485/0.905 · +p3 115k/0.478/0.894. Fact ER ceiling ≈0.91; ctx gate met only by GWC alone. protect floor is not the SNR knob (k=30 budget dominates).

### Mid candidate — `MID_BUDGET2_v1` archive `medical-mid-20260815T143947Z` — **REJECT (budget axis)**

`EDGEQUAKE_RELATED_CHUNK_NUMBER=2` (chunk-admission budget). Result: Acc 0.782/0.764 tie · ctx 0.478 · Fact ER 0.883 · Fact ctx 152k (unchanged) · EQ p50 4180 ms (↓ from 5715).

**Exhaustion proof:** three orthogonal query-only levers — GWC compress, Acc-honest BM25 pool, chunk-admission budget — all hit the same wall. The prompt blob is bounded by the k=30 token budget (`min_chunk_budget_ratio=0.4`), not by compress floor or admission count. Cutting it further drops evidence recall below the ER gate. **Ingest kept at LR parity** (chunk 1200/100, adaptive off, extract 40/100+fifo); all probes query-only on frozen ws `23b09c73`.

**Stop:** No query-only, LR-parity-ingest lever clears all 080 boxes on medical-mid. W4 SKIP. Phase G STOP — no medical-full Acc Beat scale, no `publish/latest` promote. Acc SSOT stays `T110218Z`.

---

## 7. First-principles ingest analysis (2026-08-15, user request)

**Ingest is NOT the blob source — but it sets SNR.** Code + data read:

- **Blob is query-time-assembled**: `max_chunks` k=30 (`fair_pins.py:22-25`) × full ~1200-word chunks under `balance_context` 30k-token budget with `min_chunk_budget_ratio=0.40` (Fact intent raises to 0.55) — `truncation.rs:145-171,301-325`. Nothing at ingest caps chunk count/size beyond the 1200 target.
- **Token-counting asymmetry (the real lever)**: EQ `SimpleTokenizer` counts **max(chars/4, words)** (`tokenizer.rs:49-55`); chunker counts **words** for Latin (`recursive.rs:42`). Medical prose ≈ 5 chars/word, so EQ *under-counts* ~20-25% → packs ~157k chars where LR's tiktoken holds ~84k. EQ prompts are **~1.87× LR** on every type (157k vs 84k Fact, confirmed in `eval_*.raw.json`).
- **Benchmark physics (GraphRAG-Bench, ICLR'26)**: LightRAG-style ~100k prompts score CR 41-45; compact ~7k prompts (HippoRAG2) score CR 80-88. **CR rewards compactness; ER rewards coverage.** EQ at 157k is past the dilution regime.

**Why query-only levers all failed:** they re-rank/re-select within the same k=30 × 1200-word admission. The SNR ceiling is fixed by **what a chunk is (1200 words) and how many (30)**.

**The actual ingest-level levers (2026 evidence) stay labeled research — Acc ingest pin is chunk 1200/100:**
1. Chunk granularity (512) — withdrawn. Acc law remains LightRAG **CHUNK_SIZE 1200 / overlap 100**.
2. Contextual Retrieval (Anthropic) — ingest-time preamble; not Acc-headline.
3. ER gap is extraction-quality (TACL'26) — entity resolution/dedup + gleaning before chunk tuning.

**Constraint:** Acc SSOT + Acc-law full both use frozen LR-parity ingest (chunk **1200/100**, adaptive off, extract 40/100+fifo) on ws `23b09c73`.

### Scale — `ACC_E2OCC_086_MEDICAL_FULL_v1` archive `medical-full-20260816T012004Z` (n=2062)

Best known Acc pack (E2-occ 086, **chunk 1200/100**, GWC off, pool=mix, query-only on `23b09c73`). **Not Beat; not `publish/latest`.**

| Metric | EQ | LR | vs Beat |
|--------|-----|-----|---------|
| Acc | **0.786** | **0.786** | point tie · CI includes 0 (paired n=16 underpowered) |
| ctx_rel | 0.427 | 0.485 | FAIL (&lt;0.50) |
| overall ER | 0.927 | 0.947 | PASS |
| Fact ER | 0.914 | 0.945 | at LR−0.03 |

Vs P0 full (`T204100Z`): Acc 0.724→0.786 (closed 6pp scale gap). ctx still the Beat blocker.
