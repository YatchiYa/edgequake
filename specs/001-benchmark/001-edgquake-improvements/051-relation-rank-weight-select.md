# 051 — Local relation select: LightRAG `(rank, weight)` law

**Status:** Law shipped · Acc **REJECT** — keep B5+`a1fp` peer  
**Date:** 2026-07-20  
**Peer keep:** B5+`a1fp` [`T120315Z`](../e2e/artifacts/history/smoke-20260720T120315Z/) Acc **0.801**  
**Archive:** [`T154525Z`](../e2e/artifacts/history/smoke-20260720T154525Z/) Acc **0.761**  
**Cross-ref:** [028](./028-first-principles-beat-roadmap.md) · [050](./050-placeholder-vdb-parity.md) · LightRAG `operate.py` `_find_most_related_edges_from_entities`

---

## 1. Assess vs LightRAG (no flaky heuristics)

| Gap | EQ B5+`a1fp` | LR | Binding? |
|-----|-------------:|---:|----------|
| Acc | **0.801** | 0.782 | CI includes 0 |
| recall | 0.926 | 0.966 | yes (Parity) |
| Complex Acc | 0.813 | 0.863 | generation |
| AGE/VDB | closed (B7) | ≈1 | STRUCT✓ Acc REJECT |

**Rejected next ideas:**
- **DEGREE_RANK_LOCAL on entities** — not LR law. LR: *"Entities are sorted by cosine similarity"* (`operate.py` ~5207). Acc already uses `ENTITY_RANK=retrieval`.
- UNKNOWN demote / TOPIC_* / specificity / dual-list Acc headline.

**Open law (this step):** Local **relations** = all incident edges of retrieved entities, sorted by **`(edge_rank, weight)`** where `edge_rank = deg(src)+deg(tgt)`, before truncation. EQ default expands via PPR/BFS and caps by discovery order.

---

## 2. One confound (shipped)

| Change | Detail |
|--------|--------|
| `EDGEQUAKE_RELATION_SELECT=lightrag` | Seed-incident edges → sort `(deg_src+deg_tgt, weight)` → take `max_relationships` |
| Default | unchanged (PPR/BFS) |
| Code | `relation_select.rs` · wired in `graph_expand.rs` |
| Ladder | `a1fprw` on **B5** query-only |

---

## 3. Gates — results

| Gate | Threshold | Result |
|------|-----------|--------|
| Acc | ≥ **0.781** (peer ≥ **0.801**) | **0.761** ✗ REJECT |
| Fact Acc | (informational) | **0.666** (B5 was 0.765) |
| Complex Acc | (informational) | **0.824** (B5 was 0.813) — slight ↑ |
| ctx_rel | ≥ **0.50** | **0.525** ✓ |
| recall | ≥ LR−0.03 | 0.927 vs 0.964 ✗ |
| Δ Acc CI | Beat excludes 0 EQ | includes 0 (Δ −0.016) |

**Verdict:** Law is correct vs LR code; on Acc Mix+PPR peer it **replaces** HippoRAG edge selection and Acc-taxes Fact. Keep behind flag (`default`). Do **not** promote.

---

## 4. First-principles next

Remaining binding Acc/Parity gaps on B5 are **recall** and **naming/extract density** (`only_eq` / soft overlap), not relation sort.

Pick **one**:
1. **EXTRACT_DENSITY / naming overlap** (B1 leftover) — ingest law, not Acc fishing.
2. Stay on B5+`a1fp`; treat `RELATION_SELECT=lightrag` as labeled LR-parity pin for product Local mode, not Acc headline.

Do **not**: TOPIC_* Acc, specificity Acc, dual-list Acc headline, entity degree-rank Acc flip.
