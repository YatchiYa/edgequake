# 087 — First-principles Beat: packing law (not EXTRACT)

**Status:** H1 **PARTIAL REJECT** (Acc CI keep · L2 miss) · next P2 dual-list RR  
**Date:** 2026-08-02  
**Parent:** [086](./086-beat-lightrag-ingest-first.md) · [078](./078-eq-vs-lightrag-first-principles-next.md) · [077](./077-dense-arms-fact-l2.md) · LR `operate.py`  
**Acc base law:** E2-occ (086 Acc pins) — `round_robin` · rerank off · bfs · occ_sort · LR_BUDGET · Fact L2 `fact_replace`

---

## 1. First principles (binding)

| Law | Meaning |
|-----|---------|
| **L1** | GraphRAG-Bench L2 scores **chunk texts only** (Evidence Recall + Context Relevancy). Acc is generation. Acc ≠ L2 (RAG Triad). |
| **L2** | Beat needs **both**: gold membership in the top‑k chunk list (Fact ER) **and** precision of that list (ctx, esp. Creative). |
| **L3** | EQ already extracts a **denser** graph than LR → Beat is not “extract harder.” 086 B1/B2 (same-model + Medium EXTRACT) **failed** L2 gates. |
| **L4** | Same Mix *shape* (local+global+naive). The gap is **who wins seats in the final chunk list**. |
| **L5** | One confound · labeled peer · `SKIP_PUBLISH_LATEST` until Beat on medical-full. Acc CI LR-ahead → **REJECT** that packing migrate. |

### Structural EQ↔LR diffs that justify today’s gaps

| Diff | LightRAG | EQ Acc (086) | L2 effect |
|------|----------|--------------|-----------|
| Chunk RR order | **C → E → R** (naive first) | **local → global → naive** | Fact ER membership + Creative ctx |
| In-arm BM25 | off (vector arms) | **on** | noise → ctx; some Fact recall |
| Dual-list | single list | Fact `fact_replace` only | Fact L2 patch; Creative unprotected |
| Graph density | ~3.6k nodes | ~4–4.7k | more E/R candidates under graph-first RR |

Evidence (086 Phase A mid): Fact ER 0.87 vs 0.98; Creative ctx 0.195 vs 0.330; Acc Fact LR-wins **100% generation** (gold often already in context — Acc path ≠ L2 path).

---

## 2. Why 086 ingest-first closed cold

```text
EXTRACT≠QUERY / force-ingest  →  denser graph, same packing law
                              →  Fact ER stuck ~0.88 · ctx ~0.43
                              →  STOP spending on Medium EXTRACT
```

Ingest remains useful only if forensics show **membership miss** (missing gold chunk ids). Current Acc Fact LR-win slices are generation-side.

---

## 3. Beat program (ordered)

### Phase P0 — Freeze EXTRACT fishing

- Clear Acc `EDGEQUAKE_EXTRACT_LLM_*` (empty = workspace QUERY model).
- Keep E2-occ Acc law as base.
- Do not reopen Soft Mix / TOPIC / dense+Acc promote / D1–D3 without new evidence.

### Phase P1 — Acc-law packing remasure (not blind fishing)

Historic **NF** `RR_ORDER=naive_first` REJECT was under pre-086 Acc base ([078](./078-eq-vs-lightrag-first-principles-next.md) CI LR-ahead).  
**Hypothesis H1:** Under **today’s E2-occ Acc law**, LR chunk RR order (`naive_first`) is the single Acc-law migrate that closes L2 without Acc CI collapse — same *class* of decision as 086 adopting E2-occ.

| Pin | Base (086) | H1 |
|-----|------------|-----|
| `EDGEQUAKE_RR_ORDER` | `local_first` | **`naive_first`** |
| All other Acc pins | E2-occ | unchanged |
| EXTRACT | off | off |
| WS | warm full corpus (prefer small-EXTRACT B1 `a6682988-…`) | query-only |

**Gates (medical-mid):**

| Check | Pass |
|-------|------|
| Acc CI | not LR-ahead (ci_high ≥ 0 or ci_low ≥ −ε tie band) |
| ctx_rel | ≥ 0.50 **or** ≥ base+0.05 toward 0.50 |
| Fact ER | ≥ LR−0.03 **or** ≥ base+0.03 toward LR−0.03 |

**Stop:** Acc CI LR-ahead → REJECT H1; do **not** make `naive_first` Acc default; proceed P2.

### Phase P2 — If H1 Acc CI fails but L2 lifts: dual-list RR (new code)

Keep Acc prompt on `local_first`; build **L2 citation list** with LR C→E→R order (extend dual-list beyond Fact BM25).  
One confound. Success = L2 gates without Acc CI tax. Fail → freeze packing; Acc-only grounded retry is not Beat.

### Phase P3 — Precision without NF (only if H1 Acc-toxic and P2 unavailable)

Intent-gated BM25-in-arm off for Exploratory/Creative (ctx) while Fact keeps BM25 — product-shaped, Acc CI kill switch.

### Phase P4 — Promote

Same as 086 Phase C: mid Parity → medical-full Beat → `ALLOW_PUBLISH_LATEST` → comparison SSOT.

### Phase P5 — Orthogonal

Product Equal [083] · SPEC-103 cache ON · fair latency `c1cold` · opt-in grounded retry for Acc CI only after L2 green.

---

## 4. Success metrics

| Checkpoint | Acc CI | ctx_rel | Fact ER |
|------------|--------|---------|---------|
| H1 mid | not LR-ahead | ≥0.50 or +0.05 vs 086-A | ≥LR−0.03 or +0.03 vs 086-A |
| Parity mid | not LR-ahead (Acc ≥ tie) | ≥0.50 | ≥LR−0.03 |
| Beat mid/full | EQ-ahead excludes 0 | ≥0.50 | ≥LR−0.03 |

086-A baseline: ctx **0.431** · Fact ER **0.87** · Acc CI tie.

---

## 5. Runbook (H1)

```bash
export MISTRAL_API_KEY=...
unset EDGEQUAKE_EXTRACT_LLM_MODEL EDGEQUAKE_EXTRACT_LLM_PROVIDER
export EDGEQUAKE_RR_ORDER=naive_first
export BENCH001_ALLOW_ROUND_ROBIN=1
export BENCH001_SKIP_PUBLISH_LATEST=1
export BENCH001_PUBLISH_PEER=ACC_E2OCC_087_H1_NAIVE_FIRST_v1
export BENCH001_EQ_WORKSPACE_ID=<warm-full-corpus-ws>   # prefer B1 a6682988-…
export BENCH001_QUERY_ONLY=1
# restart Acc so RR_ORDER applies (do not SKIP_BACKEND_RESTART)
python3 tools/bench001/scripts/start_acc_backend.py --port 8090
make bench001-medical-mid
```

---

## 6. Status log

| Step | Status | Notes |
|------|--------|-------|
| Plan memo | Done | this file |
| H1 naive_first mid | **PARTIAL REJECT** | peer `ACC_E2OCC_087_H1_NAIVE_FIRST_v1` · `medical-mid-20260802T151348Z` · WS `a6682988-…` · Acc 0.808/0.780 CI **[−0.004,+0.058] keep** (historic NF Acc-toxic **not** reproduced under E2-occ) · ctx **0.429** (no lift) · Fact ER **0.89**/0.953 (+0.02 vs 086-A, still &lt; LR−0.03) · Creative ctx **0.195** unchanged. Do **not** make `naive_first` Acc default |
| P2 dual-list RR | Next | Acc CI was not toxic → optional Acc-law still weak for L2; prefer L2-only C→E→R citation rebuild without moving Acc prompt |
| Publish latest | Blocked | Beat gates |
