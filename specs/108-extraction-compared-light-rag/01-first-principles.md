# 01 — First Principles (SPEC-108)

> Method: map partner vanity counts to axioms about chunking, extraction caps, and merge.  
> Algorithm depth: [SPEC-026](../026-egdequake-vs-lightrag/). Density product law: [SPEC-086](../086-improve-ingestion-ux/findings/F-extraction-quality-parity.md).

## Axioms

1. **A mention is not a node.** Summing entities across chunk LLM responses counts **mentions (M)**. Merge collapses same normalized name into **unique graph nodes (U)**. The document card stores M.
2. **Chunk count is the multiplier.** With per-response caps ≤40 entities, `M ≤ 40 × N_chunks` (and typically tracks N when the model fills the quota).
3. **Adaptive sizing is a product default, not LightRAG Acc fairness.** EQ adaptive ON: `<50KB→1200`, `50–100KB→800`, `>100KB→600`. LightRAG / Acc fair pin: fixed **1200/100**.
4. **Fair claims need matched confounds.** Chunk size, overlap, strategy, gleaning, extract caps, entity-type policy, and model must align before “EQ denser than LR” is a bug.
5. **Density beats vanity.** Absolute entity counts without chars/chunks mislead (SPEC-086 LAW-27).

## Laws

| Law | Statement | Partner symptom |
|-----|-----------|-----------------|
| **LAW-X1** Count SSOT | Never equate document `entity_count` (ProcessingStats sum) with unique AGE/LR graph nodes | 12 367 “entities” on the card |
| **LAW-X2** Geometry dominates M | `M ≈ f(N_chunks × yield)`; 600 vs 1200 roughly doubles N on large docs | “Chunking problem?” |
| **LAW-X3** Fair compare | Dual-SUT claims require matched size/overlap/gleaning/caps/types/model | Cross-SUT arguments |
| **LAW-X4** Density not vanity | Report entities/1k chars and U/chunk | Absolute 12k alone |
| **LAW-X5** Merge is the product | Unique graph after merge is “how many entities exist” | Relations ≈ entities (mention-space) |

## Pipeline identity

```ascii
  content
    → chunks(N)                    ← LAW-X2 (adaptive vs fixed)
    → LLM × N (≤40 ents, ≤100 rows)← caps LR-parity (054)
    → mentions M                   ← document card (LAW-X1)
    → merge / EntityId normalize
    → unique graph U               ← AGE / LR KV (LAW-X5)
```

## Root-cause classes (pre-rank)

```ascii
 ┌────────────────────┐     ┌─────────────────────┐     ┌──────────────────┐
 │ Metric illusion    │     │ Adaptive geometry   │     │ True over-extract│
 │ M ≫ U on card      │────▶│ N_B ≈ 1.5–2× N_A    │────▶│ fair pins EQ≫LR  │
 │ H1                 │     │ H2                  │     │ H4/H5            │
 └────────────────────┘     └─────────────────────┘     └──────────────────┘
```

## Back-of-envelope for 12 367

```text
M = 12367
min_N ≥ ceil(12367 / 40) ≈ 309 chunks
@ 600 tok/chunk  → large PDF under adaptive ON is plausible
@ 1200 tok/chunk → needs ~2× the text span or saturated yield
```

## DRY / SOLID

```ascii
 LAW-X1 ─▶ stats.rs sums extraction.entities.len() ─▶ status_updates entity_count
 LAW-X2 ─▶ adaptive_chunking.rs resolve_base_chunk_size_overlap
 LAW-X3 ─▶ fair_pins.py FAIR_CHUNK 1200/100 + Acc backend pin
 LAW-X5 ─▶ merger/entity.rs EntityId batch dedup → AGE upsert
```
