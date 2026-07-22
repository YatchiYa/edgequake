# 044 — Horizon B5: Relation-placeholder provenance (First Principles)

**Status:** **PROMOTE** Acc peer candidate (hygiene + Acc) · Beat not claimed  
**Date:** 2026-07-20  
**Archive:** [`T120315Z`](../e2e/artifacts/history/smoke-20260720T120315Z/) · ingest audit [`20260720T120040Z`](../e2e/artifacts/ingest-audit/20260720T120040Z/)  
**WS:** `8e990410-43b5-44f4-9f56-87bd154570ce`  
**Prior Acc peer (frozen):** `2a7bcb2f-b156-4c49-9229-67f5bcde22a4` [`T095809Z`](../e2e/artifacts/history/smoke-20260720T095809Z/)  
**Cross-ref:** [028](./028-first-principles-beat-roadmap.md) · [029](./029-ingest-parity-audit.md) · [037](./037-summarize-chunk-link-audit.md) · [043](./043-honesty-can-we-push.md)

---

## 1. Law (ingest)

**STUB_PROVENANCE** — warm Acc WS had **345** zero-chunk nodes; **100%** were `entity_type=UNKNOWN`, empty description, **no** `source_chunk_ids`. Root cause: `placeholder_node_properties` for missing relation endpoints omitted lineage. LightRAG/GraphRAG attach source evidence on every retrieval node.

Binding Summarize miss (`Medical-0002d2de`) remains **SELECT** — B5 does not claim Mix phrase fix.

---

## 2. One confound

| Change | Location |
|--------|----------|
| New placeholders inherit union of relation `source_chunk_id`s | `merger/relationship.rs` |
| Existing zero-chunk stubs enriched when later relations arrive | same |
| Audit stub split + gate `eq_zero_chunk_rate ≤ 0.01` | `audit_eq_lr_ingest.py` · `run_b5_reingest_acc.sh` |

Pins: md + glean=1 · no FAQ · chunk 1200/100 · query **`a1fp`**.

---

## 3. Results

| Gate | Result |
|------|--------|
| Zero-chunk rate ≤ 1% | **PASS 0.0** (345 → 0) |
| age_over_vectors ∈ [0.90, 1.20] | **PASS 1.077** |
| Acc ≥ 0.755 | **PASS 0.801** (peer 0.775) |
| Fact ER ≥ 0.83 | **PASS 0.85** |
| ctx ≥ 0.50 | **PASS 0.519** |
| Beat (CI excludes 0) | **FAIL** CI [-0.064, +0.100] |

---

## 4. Reproduce

```bash
cargo test -p edgequake-pipeline placeholder_ --lib
unset BENCH001_EQ_WORKSPACE_ID
export BENCH001_ACC_QUERY_CONCURRENCY=4
make bench001-b5-reingest
# or query-only on B5 WS:
export BENCH001_EQ_WORKSPACE_ID=8e990410-43b5-44f4-9f56-87bd154570ce
./tools/bench001/scripts/run_p_ladder_acc.sh a1fp
```
