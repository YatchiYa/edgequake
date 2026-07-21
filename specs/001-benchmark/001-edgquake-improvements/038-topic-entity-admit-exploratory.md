# 038 — Mix topic-entity admission (Exploratory SELECT)

**Status:** Closed — **no promote** (`a1fpsel` v1+v2 Acc tax; binding probe still ✗)  
**Cross-ref:** [037](./037-summarize-chunk-link-audit.md) · [036](./036-a1fp-recall-without-dual-list.md) · [035](./035-fact-ce-bm25-protect.md)  
**Warm WS:** `2a7bcb2f-b156-4c49-9229-67f5bcde22a4`  
**Law:** SELECT (037) — topic entity linked; Mix admits wrong neighborhood

---

## 1. First principles

```text
q → entity VDB hit → source_chunk_ids → VECTOR take → Mix C → Summarize ER
         ▲
         └── topic entity may exist in AGE but lose VDB top-k to hub entities
```

LightRAG Mix expands chunks from **retrieved** entities/relations via `source_id` + `RELATED_CHUNK_NUMBER` + VECTOR. EQ mirrors that. Binding miss (`Medical-0002d2de`): `BONE_CANCER` has links (5≈6) but Mix has **0** × question bigram `bone cancers`.

**Confound (one pin):** Before KG chunk collect, **admit** graph entities whose exact normalized name matches a **question content bigram** (or multi-token LL keyword), Exploratory intent only. Pin their `source_chunk_ids` into the VECTOR shortlist.

**Not this confound:** densify-all ingest, dual-list, LR VECTOR budget, bare unigram hubs (`CANCER`, `STAGE`).

---

## 2. Pin

```bash
EDGEQUAKE_TOPIC_ENTITY_ADMIT=1   # default 0
# Exploratory-only inside engine (keywords.query_intent)
```

Ladder: `a1fpsel` = a1fp + `TOPIC_ENTITY_ADMIT=1`

---

## 3. Insertion

| Step | Where |
|------|--------|
| Candidate norms | query content bigrams + multi-token `low_level` keywords → `EntityId` (+ singular) |
| Graph lookup | `get_nodes_batch` with `{ws}::NAME` |
| Admit | prepend to `context.entities` if linked (`source_chunk_ids` non-empty) |
| Pin | `append_score_ranked_chunks`: prefer admitted chunk ids in VECTOR take |
| Gate | `QueryIntent::Exploratory` only |

Files: `topic_entity_admit.rs` · `local.rs` / `global.rs` · `chunk_retrieval.rs`

---

## 4. Gates (reject / promote)

| Check | Bar |
|-------|-----|
| Acc | ≥ a1fp − 0.02 (0.755) |
| Fact ER | ≥ a1fp − 0.02 (0.83) |
| Sum ER | ↑ vs 0.863 |
| recall | prefer ≥ 0.935 (LR−0.03) |
| Probe | `Medical-0002d2de` Mix contains `bone cancers` |
| Forbidden | dual-list, LR-budget, densify-all |

**Promote Parity** only if Acc point holds (CI includes 0 or Acc≥a1fp−0.02), ctx≥0.50, recall≥LR−0.03, no dual-list.

---

## 5. Results

| Run | Acc | ctx | recall | Fact ER | Sum ER | `bone cancers` in Mix | Note |
|-----|----:|----:|-------:|--------:|-------:|:---------------------:|------|
| a1fp T095809Z | **0.775** | 0.500 | 0.926 | 0.85 | 0.86 | ✗ | Acc peer |
| a1fpsel T105410Z | 0.746 ✗ | 0.481 ✗ | 0.928 | 0.85 | 0.86 | ✗ | v1 reject — PPR skipped pin |
| a1fpsel T110102Z | 0.719 ✗ | 0.525 | 0.911 ✗ | 0.80 ✗ | 0.85 | ✗ | v2 reject — pool +8 ids; CE/post still drops |

**v1 hole:** pin gated off under Acc `graph_walk=Ppr`.  
**v2:** union into PPR pool (`total_chunk_ids` 30→38 on binding Q) + pin after fetch. Admit still fires (3 ents / 16 chunks). Final Mix still **0× `bone cancers`** — same off-topic 6-part blob.

**Law update:** SELECT has a second choke after KG collect — **CE / protect / fusion** can discard admitted topic chunks before C is serialized. Entity admit alone does not move Summarize ER under a1fp CE pins.  
**Keep:** `a1fp` Acc peer. **Do not** leave `TOPIC_ENTITY_ADMIT=1` on as Acc headline.  
**Next:** Done as [039](./039-topic-ce-protect-exploratory.md) (`a1fpce` REJECT) — CE protect ≠ C; next choke = truncate/packing.

---

## 6. Reproduce

```bash
export BENCH001_EQ_WORKSPACE_ID=2a7bcb2f-b156-4c49-9229-67f5bcde22a4
export BENCH001_ACC_QUERY_CONCURRENCY=4
./tools/bench001/scripts/run_p_ladder_acc.sh a1fpsel
```
