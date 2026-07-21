# 032 — Workspace-scoped graph identity (Horizon B3b)

**Status:** Fix shipped · Acc re-ingest gated (disk + settle)  
**Cross-ref:** [031](./031-structure-aware-chunking.md) · [029](./029-ingest-parity-audit.md) · [028](./028-first-principles-beat-roadmap.md)

**Invalid Acc attempt:** [`smoke-20260720T081853Z`](../e2e/artifacts/history/smoke-20260720T081853Z/) — ENOSPC during relationship merge → saga wiped 4228 scoped nodes; Acc raced rollback (`empty_context_rate`). Not an identity-heuristic failure. Warm restored to B2 `e0270f5f-…`.

---

## 1. First principles (no flaky heuristics)

Acc medical extract is **not** density-starved:

| Store (B2 WS `e0270f5f-…`) | Count |
|----------------------------|------:|
| Entity vectors | **4221** |
| `chunk_entity_links` distinct | **4191** |
| AGE nodes with `workspace_id=B2` | **392** |
| LightRAG unique entities | 3580 |

Every B2 vector entity **exists** in AGE — but **~90% live under other workspaces’ `workspace_id`**. Cause: shared AGE graph + **global** `node_id = EntityId` (bare `JOHN_DOE`) + `UNIQUE(eq_node_id)`. First writer owns the vertex; later Acc ingests merge into foreign nodes and Mix query filters them out.

This is **identity isolation**, not FAQ regex / chunk heuristics. B3a Acc tax stands closed. Do not stack structure-induce for Acc.

---

## 2. Fix

When `workspace_id` is set on the merger / query:

- Graph node id = `{workspace_id}::{NORMALIZED_NAME}`
- `label` / vector id stay bare normalized name (`entity:NAME`) for display + vector search
- Query maps vector hits → scoped graph ids before `get_nodes_batch` / edge expand

No env knob (fail-closed multi-tenant identity). Labeled Acc re-ingest required (new WS).

---

## 3. Success gates (same Acc promote)

After force-ingest + A1 (`rr_cer`, concurrency≤4):

- AGE nodes with WS filter ≈ entity vector count (±10%)
- Zero-chunk (UNKNOWN orphans) ≤ 5% of WS nodes
- ctx≥0.50 ∧ recall≥LR−0.03 ∧ Acc ≥ B2 A1−0.01 (0.775)

---

## 4. Non-goals

- FAQ / structure-induce Acc fishing  
- Soft Mix query knobs  
- Full per-workspace AGE graph namespace migration (future; this is attribute-scoped id parity)

---

## 5. Acc results (2026-07-20)

| Run | Acc | recall | ctx | age/vec | Note |
|-----|----:|-------:|----:|--------:|------|
| T081853Z | 0.658 | — | — | 0 | ENOSPC saga wipe — invalid |
| T084149Z | 0.734 | **0.960** | 0.394 | **1.08** | Identity gate PASS |
| T085257Z A1+labelFTS | 0.749 | 0.941 | **0.519** | 1.08 | Tie vs LR; Acc < B2 0.785 |
| T090743Z A1+033 pack | **0.773** | 0.914 | 0.481 | LR 6k/8k caps; Acc↑; L2 miss |

**Warm:** B3b `2a7bcb2f-…` + 033 packing ([T090743Z](../e2e/artifacts/history/smoke-20260720T090743Z/)). B2 Acc peak frozen at T071732Z (0.785). No Beat/Parity yet.

FAQ / structure-induce: still closed for Acc.
