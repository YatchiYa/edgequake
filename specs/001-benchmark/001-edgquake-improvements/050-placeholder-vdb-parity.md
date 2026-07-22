# 050 — Horizon B7: Placeholder entity VDB parity

**Status:** STRUCT✓ · Acc **REJECT** — keep B5+`a1fp` peer  
**Date:** 2026-07-20  
**Peer keep:** B5+`a1fp` [`T120315Z`](../e2e/artifacts/history/smoke-20260720T120315Z/) Acc **0.801**  
**B7 WS:** `dbaf36a1-6a59-4d3d-9438-8a84da92bdc9` · archive [`T144511Z`](../e2e/artifacts/history/smoke-20260720T144511Z/)  
**Cross-ref:** [028](./028-first-principles-beat-roadmap.md) · [044](./044-horizon-b-placeholder-provenance.md) · [049](./049-rel-dedup-source-chunk-union.md) · LightRAG `operate.py` ~1916

---

## 1. Assess vs LightRAG (no flaky heuristics)

| Gap | EQ (B5) | LR | Law |
|-----|--------:|---:|-----|
| Acc peer | **0.801** | 0.782 | CI includes 0 |
| Multi-chunk edges | **12.5%** (B6) | 11.9% | closed |
| AGE / entity VDB | was **~1.08** | ≈1.0 | **PLACEHOLDER_VDB_PARITY** |

**Forbidden:** question_type / intent / TOPIC_* / bigrams / ANSWER_PROMPT Acc / soft Mix.

**Law:** every relation-endpoint AGE placeholder must get an `entities_vdb` row. LightRAG upserts UNKNOWN nodes with content `{name}\n{relation_description}` into `entities_vdb`. EQ previously wrote AGE only (B5/B6).

---

## 2. One confound (shipped)

| Change | Location |
|--------|----------|
| Seed placeholder description from longest incident relation desc | `merger/relationship.rs` |
| Embed + upsert entity vectors for new placeholders via `TextEmbedder` | same + `KnowledgeGraphMerger::with_text_embedder` |
| Wire `IngestionPersistConfig.text_embedder` → merger | `ingestion_persister.rs` |
| Audit gate `age_over_vectors ∈ [0.98, 1.02]` | `run_b7_reingest_acc.sh` · `make bench001-b7-reingest` |

Pins: md + glean=1 · chunk 1200/100 · query **`a1fp`** · **new WS**.

---

## 3. Gates — results

| Gate | Threshold | Result |
|------|-----------|--------|
| `age_over_vectors` | ∈ **[0.98, 1.02]** | **1.0** ✓ (4465 AGE = 4465 entity vectors) |
| zero-chunk rate | ≤ **0.01** | **0.0** ✓ |
| Acc | ≥ **0.781** (promote peer only if ≥ **0.801**) | **0.676** ✗ REJECT |
| Fact ER / ctx | ≥ **0.83** / ≥ **0.50** | Fact ER **0.80** ✗ · ctx **0.506** ✓ |
| recall | ≥ LR−0.03 | 0.914 vs LR 0.991 ✗ |

**Verdict:** Structural law **closed**. Acc tax (Fact Acc 0.518 vs B5 0.765) — same pattern as B6. **Keep code** for identity/Local parity; **do not** replace Acc peer.

---

## 4. First-principles next (no flaky heuristics)

B7 made UNKNOWN endpoints Local-retrievable. Acc drop (Fact Acc 0.52) is consistent with **~2k EQ-only names** now competing in `entities_vdb` (audit only_eq≈2065) — vector pollution, not a missing law.

LR Local ranks by **node degree**, not by demoting UNKNOWN (Neo4j/LightRAG retrieval write-up; `entities_vdb.query` then degree sort). So UNKNOWN-demote is **not** LR law.

Next law-shaped candidates (pick **one**):

1. **DEGREE_RANK_LOCAL** — if EQ Local truncates by vector score only, align to LR degree-after-retrieve (structural salience).
2. **EXTRACT_DENSITY / naming overlap** — close only_eq gap vs LR (B1 leftover) so VDB isn't dominated by EQ-unique stubs.
3. Stay on B5 Acc peer; use B7 WS only for identity audits.

Do **not**: TOPIC_* Acc, specificity Acc, dual-list as Acc headline, FAQ induce, UNKNOWN demote without LR evidence.
