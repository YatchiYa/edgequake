# 042 — Materialize topic CONTENT chunks into Mix (KV get-by-id)

**Status:** CLOSED REJECT — Acc [`T113404Z`](../e2e/artifacts/history/smoke-20260720T113404Z/) · Sum ER breakthrough  
**Cross-ref:** [041](./041-topic-chunk-fidelity-audit.md) · [040](./040-topic-trunc-protect-exploratory.md) · [038](./038-topic-entity-admit-exploratory.md)  
**Warm WS:** `2a7bcb2f-b156-4c49-9229-67f5bcde22a4`

---

## 1. First principles

```text
entity → source_chunk_ids → KV body (CONTENT ✓) → materialize → Mix/CE → C
                              ▲
                              └── 041 CE_GAP: CONTENT ∧ ¬IN_MIX
```

**Confound:** Before CE, KV `get_by_id` for capped `topic_admit_chunk_ids`; prepend; imply CE/trunc survival under `TOPIC_MATERIALIZE`.

---

## 2. Acc result (`T113404Z`)

| Check | Bar | Result |
|-------|-----|--------|
| Acc | ≥ 0.755 | **0.746 ✗** |
| Fact ER | ≥ 0.83 | **0.70 ✗** |
| Sum ER | ↑ vs 0.863 | **0.963 ✓** |
| Probe | `bone cancers` in C | **✗** (TNM yes; phrase no) |
| ctx | ≥ 0.50 | 0.506 ✓ |

Binding Q log: `042 topic_materialize … materialized=4 fetched=4 mix_len=10` then trunc prefer `topic_pack=4`.

---

## 3. Verdict

**Do not promote** (Acc/Fact). Keep **a1fp**.

**Law update:** Materialize fixes CE_GAP for Summarize ER. Remaining miss = **which** topic ids are injected (first-4 of 23 includes non-CONTENT / hub-adjacent chunks). Next = CONTENT-gated materialize (question bigram filter on KV body).

---

## 4. Reproduce

```bash
export BENCH001_EQ_WORKSPACE_ID=2a7bcb2f-b156-4c49-9229-67f5bcde22a4
./tools/bench001/scripts/run_p_ladder_acc.sh a1fpmat
```
