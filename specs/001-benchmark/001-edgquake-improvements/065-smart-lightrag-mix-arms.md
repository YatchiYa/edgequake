# 065 — Product Smart = LightRAG Mix Arms

**Status:** Implemented  
**Date:** 2026-07-21  
**Depends:** [061](./061-lightrag-law-first-principles-eq.md) · SPEC-047 / 020 B2  
**Law source:** LightRAG `mix` = merge local + global + naive (README)

## First principles

| Layer | Law | EdgeQuake |
|-------|-----|-----------|
| **Q0 Arm set** | LightRAG mix always runs three arms | Product Smart (`mode=mix`) must not intent-collapse to naive-only |
| **Q1 Cost routing** | Latency via parallel arms + role LLMs, not arm starvation | Linked (`hybrid`) keeps `intent_arm_mask_hybrid` |
| **Q2 Acc fairness** | Acc already pins `MIX_ARM_GATE=false` | Product default matches Acc/LR |
| **Q3 Chunk hydration** | Graph modes need page chunks | `append_score_ranked_chunks` SSOT `vector_type=chunk` |

## Changes

1. [`intent_arm_mask`](../../../../edgequake/crates/edgequake-query/src/mix_weights.rs) → always `(true, true, true)`.
2. `EDGEQUAKE_MIX_ARM_GATE` default **false** (empty env + Makefile product pin).
3. Chunk fetch filter SSOT in `chunk_retrieval.rs` (never reuse entity/relationship mf).
4. UI Smart tooltip states LightRAG mix / always three arms.

## Non-goals

- Do not copy LightRAG sequential arm awaits.
- Do not change Acc fusion (RRF) or Hybrid Linked masks.
- Do not Acc-fish Soft Mix.

## Verify

```bash
cargo test -p edgequake-query --lib mix_weights
cargo test -p edgequake-query --test e2e_spec047_arm_gate
cargo test -p edgequake-query --test contract_spec058_vector_type_sql
```
