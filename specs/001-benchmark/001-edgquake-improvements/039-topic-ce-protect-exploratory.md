# 039 — CE/fuse protect for topic-admitted chunks (Exploratory SELECT)

**Status:** CLOSED REJECT — Acc [`T111057Z`](../e2e/artifacts/history/smoke-20260720T111057Z/)  
**Cross-ref:** [038](./038-topic-entity-admit-exploratory.md) · [037](./037-summarize-chunk-link-audit.md) · [035](./035-fact-ce-bm25-protect.md)  
**Warm WS:** `2a7bcb2f-b156-4c49-9229-67f5bcde22a4`

---

## 1. First principles

```text
q → admit topic entities → arm chunks → Mix fuse → CE (+ min_rerank) → truncate → C
                                                      ▲
                                                      └── 038: pool gains topic ids;
                                                          final C still 0× question bigrams
```

**Law (038):** CE must not discard topic-admitted chunks that are already in the first-stage Mix. Protect is for set membership; CE still orders.

**Industry note:** Hybrid RRF + CE is recall→precision; CE cannot invent docs missing from the shortlist, and packing after CE can still drop survivors ([hybrid + CE staging](https://www.digitalapplied.com/blog/hybrid-search-bm25-vector-reranking-reference-2026)).

**Confound tested:** `TOPIC_CE_PROTECT=1` + `TOPIC_ENTITY_ADMIT=1` — Exploratory only:
1. Propagate `topic_admit_*` through Mix fuse (metadata was dropped).
2. Force-include those ids into fused Mix[:top_k] when present in arm lookups.
3. After CE, `blend_protect_ids` re-inserts missing topic ids from pre-CE Mix.

Forbidden: densify-all, dual-list, LR-budget, raising global `protect_first`.

---

## 2. Pins

```bash
EDGEQUAKE_TOPIC_ENTITY_ADMIT=1
EDGEQUAKE_TOPIC_CE_PROTECT=1   # default 0
```

Ladder: `a1fpce` = a1fp + both.

---

## 3. Acc result (`T111057Z`)

| Check | Bar | Result |
|-------|-----|--------|
| Acc | ≥ 0.755 | **0.736 ✗** (a1fp 0.775) |
| Fact ER | ≥ 0.83 | 0.85 ✓ |
| Sum ER | ↑ vs 0.863 | 0.877 ✓ |
| Sum Acc | — | **0.706** (a1fp **0.858** — Acc tax) |
| Probe | `bone cancers` in C | **✗** |
| ctx | ≥ 0.50 | 0.519 ✓ |

Bone query logs: admit `topic_chunks=16`; no fuse-force INFO (ids already in Mix). Final C still cervical/anal/AML — **truncate/format packing** is the next choke after CE set membership.

---

## 4. Verdict

**Do not promote.** Keep **a1fp** Acc peer. Code stays behind defaults (`TOPIC_*=0`).

**Next:** Done as [040](./040-topic-trunc-protect-exploratory.md) (`a1fptrunc` REJECT) — pack prefer ≠ C when topic ids absent post-CE; next = chunk id/content fidelity.

---

## 5. Reproduce

```bash
export BENCH001_EQ_WORKSPACE_ID=2a7bcb2f-b156-4c49-9229-67f5bcde22a4
export BENCH001_ACC_QUERY_CONCURRENCY=4
./tools/bench001/scripts/run_p_ladder_acc.sh a1fpce
```
