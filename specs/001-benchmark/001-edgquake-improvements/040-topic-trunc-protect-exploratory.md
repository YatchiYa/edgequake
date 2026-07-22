# 040 — Trunc/pack protect for topic-admitted chunks (Exploratory SELECT)

**Status:** CLOSED REJECT — Acc [`T111944Z`](../e2e/artifacts/history/smoke-20260720T111944Z/)  
**Cross-ref:** [039](./039-topic-ce-protect-exploratory.md) · [038](./038-topic-entity-admit-exploratory.md) · [037](./037-summarize-chunk-link-audit.md)  
**Warm WS:** `2a7bcb2f-b156-4c49-9229-67f5bcde22a4`

---

## 1. First principles

```text
q → admit → Mix fuse → CE (+ id protect) → truncate/pack → format → C
                                              ▲
                                              └── recall@k ≠ answer-in-context
                                                  (budgeted packing; arXiv 2607.00725)
```

**Confound tested:** `TOPIC_TRUNC_PROTECT=1` (+ max=4) on `a1fpce` — move in-list topic ids to front before greedy `truncate_chunks`.

---

## 2. Acc result (`T111944Z`)

| Check | Bar | Result |
|-------|-----|--------|
| Acc | ≥ 0.755 | **0.696 ✗** |
| Fact ER | ≥ 0.83 | **0.80 ✗** |
| Sum ER | ↑ vs 0.863 | **0.883 ✓** |
| Probe | `bone cancers` in C | **✗** |
| ctx | ≥ 0.50 | **0.456 ✗** |

Trunc prefer logged on other Exploratory queries; **binding bone query: no 040 log** → 0 topic ids present post-CE. Packing cannot help.

---

## 3. Verdict

**Do not promote.** Keep **a1fp**. Defaults stay off.

**Law update:** 038–040 protect ladder exhausted for Acc. Binding miss confirmed by [041](./041-topic-chunk-fidelity-audit.md) as **CE_GAP** — bodies contain `bone cancers`, Mix C does not. Next = materialize topic CONTENT chunks into Mix (not more reorder protect).

---

## 4. Reproduce

```bash
export BENCH001_EQ_WORKSPACE_ID=2a7bcb2f-b156-4c49-9229-67f5bcde22a4
./tools/bench001/scripts/run_p_ladder_acc.sh a1fptrunc
```
