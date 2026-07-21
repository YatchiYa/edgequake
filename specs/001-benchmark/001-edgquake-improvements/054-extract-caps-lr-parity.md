# 054 — Extract quantity caps = LightRAG (40 entities / 100 rows)

**Status:** Law shipped · Acc **REJECT** on B9 — keep B5+`a1fp` peer  
**Date:** 2026-07-21  
**Peer keep:** B5+`a1fp` [`T120315Z`](../e2e/artifacts/history/smoke-20260720T120315Z/) Acc **0.801**  
**Archive:** [`T011125Z`](../e2e/artifacts/history/smoke-20260721T011125Z/) Acc **0.745** on B9 `dcdffc3e-…`  
**Audit:** [`ingest-audit/20260721T010844Z`](../e2e/artifacts/ingest-audit/20260721T010844Z/) · eq_nodes **3950** (was B5 4543 / B8 4659; LR 3580)  
**Cross-ref:** [028](./028-first-principles-beat-roadmap.md) · [043](./043-honesty-can-we-push.md) · [053](./053-entity-types-lr-parity.md) · LightRAG `DEFAULT_MAX_EXTRACTION_ENTITIES=40` / `DEFAULT_MAX_EXTRACTION_RECORDS=100`

---

## 1. Assess vs LightRAG (no flaky heuristics)

| Gap | EQ | LR | Notes |
|-----|---:|---:|-------|
| Acc peer (B5) | **0.801** | 0.782 | CI includes 0 |
| Graph size (pre-054) | 4543–4659 | **3580** | EQ denser |
| Per-response extract caps | **none** | **40 ents / 100 rows** | **LAW GAP** |

**Law:** LightRAG prompts + early-stop at 40 entity / 100 total rows per response. EQ now mirrors that in prompts and post-parse truncate.

**Honesty:** Soft Mix Acc fishing exhausted; Beat CI on n=40 not near ([043](./043-honesty-can-we-push.md)).

---

## 2. One confound (shipped, always-on)

| Change | Location |
|--------|----------|
| Defaults 40 / 100 (+ env) | `prompts/extract_caps.rs` |
| Quantity limits in JSON + SOTA prompts | `json_prompts.rs` · `entity_extraction.rs` |
| Post-parse truncate | JSON + tuple parsers |
| Gleaning: drop “dates” focus | `json_prompts.rs` |
| B9 Acc | `make bench001-b9-reingest` |

---

## 3. Gates — results (B9 + `a1fp`)

| Gate | Threshold | Result |
|------|-----------|--------|
| Acc | ≥ **0.781** (prefer ≥ **0.801**) | **0.745** ✗ |
| Fact ER | ≥ **0.83** | **0.80** ✗ |
| ctx_rel | ≥ **0.50** | **0.506** ✓ |
| recall | ≥ LR−0.03 | **0.917** ✗ |
| STRUCT nodes | closer to LR / ≤ B5 | **3950** ✓ (Δ−593 vs B5) |
| STRUCT coverage | ≥ ~0.68–0.70 | **0.686** (soft OK with nodes↓) |

**Invalid prior attempt:** [`T190832Z`](../e2e/artifacts/history/smoke-20260720T190832Z/) — Mistral embed network fail mid-ingest (ctx=0); discarded.

**Verdict:** Law closed; graph denser→LR. Acc/Fact tax vs B5 peer. Keep code. **Do not** replace B5 Acc peer. Warm restored to `8e990410-…`.

---

## 4. Reproduce

```bash
make bench001-b9-reingest
# Acc peer keep:
export BENCH001_EQ_WORKSPACE_ID=8e990410-43b5-44f4-9f56-87bd154570ce
./tools/bench001/scripts/run_p_ladder_acc.sh a1fp
```
