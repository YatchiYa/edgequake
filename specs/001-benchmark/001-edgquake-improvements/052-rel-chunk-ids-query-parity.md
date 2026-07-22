# 052 — Relation multi-chunk lineage at query (closes B6 end-to-end)

**Status:** Law shipped · Acc **REJECT** on B6 — keep B5+`a1fp` peer  
**Date:** 2026-07-20  
**Peer keep:** B5+`a1fp` [`T120315Z`](../e2e/artifacts/history/smoke-20260720T120315Z/) Acc **0.801**  
**Archive:** [`T155511Z`](../e2e/artifacts/history/smoke-20260720T155511Z/) Acc **0.759** on B6  
**Cross-ref:** [049](./049-rel-dedup-source-chunk-union.md) · [051](./051-relation-rank-weight-select.md) · LightRAG `_find_related_text_unit_from_relations`

---

## 1. Assess vs LightRAG (no flaky heuristics)

| Gap | EQ | LR | Notes |
|-----|---:|---:|-------|
| Acc peer (B5) | **0.801** | 0.782 | CI includes 0 |
| B6 Acc (pre-052) | 0.725 | — | STRUCT✓ ingest only |
| Relation → Mix chunks | was **1 id** | **all** `source_id` parts | **LAW GAP** |

**Law:** LightRAG splits relation `source_id` and admits every linked chunk. EQ merger (049) wrote plural `source_chunk_ids` on edges, but query only kept singular `source_chunk_id`.

---

## 2. One confound (shipped, always-on)

| Change | Location |
|--------|----------|
| Read `source_chunk_ids[]` (+ singular fallback) | `helpers.rs` |
| `RetrievedRelationship.source_chunk_ids` + `all_source_chunk_ids()` | `context.rs` |
| Union all rel chunk ids into KG collect | `kg_chunk_pick.rs` |
| Global arm metadata | `modes/global.rs` |

Test: `a1fp` query-only on **B6** WS `58ffe7da-…`.

---

## 3. Gates — results (B6 + 052)

| Gate | Threshold | Result |
|------|-----------|--------|
| Acc | ≥ **0.781** (peer ≥ **0.801**) | **0.759** ✗ (was 0.725 pre-052) |
| Fact ER | ≥ **0.83** | **0.80** ✗ |
| ctx_rel | ≥ **0.50** | **0.506** ✓ |
| Complex Acc | (info) | **0.852** (↑ vs B5 0.813) |

**Verdict:** Law closed end-to-end; Acc lift vs B6 baseline but still below promote. Keep code always-on. **Do not** replace B5 Acc peer.

---

## 4. First-principles next

B6 workspace still Acc-taxes Fact vs B5. Binding Beat/Parity leftovers on **B5 Acc peer**: recall 0.926 vs LR−0.03, CI includes 0. Query Acc knobs exhausted (STOP list). Next law-shaped ingest work: **naming / only_lr coverage** (897 LR-only names on B5 audit) — not Mix fishing.
