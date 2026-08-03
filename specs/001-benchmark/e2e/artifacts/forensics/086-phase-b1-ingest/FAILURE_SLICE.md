# 081 / 080 failure slice

**Archive:** `/Users/raphaelmansuy/Github/03-working/edgequake/specs/001-benchmark/e2e/artifacts/history/medical-mid-20260802T143215Z`
**Paired eval n:** 200
**Recommended next:** `F4_generation_after_F3`

## Scorecard snapshot

```json
{
  "eq_acc": 0.7920196670800517,
  "lr_acc": 0.7834308761087777,
  "eq_ctx": 0.42500000000000004,
  "lr_ctx": 0.49624999999999997,
  "eq_er": 0.9169011017577193,
  "lr_er": 0.9560115448277213,
  "eq_fact_er": 0.88,
  "lr_fact_er": 0.95,
  "eq_empty_answer_rate": 0.0,
  "lr_empty_answer_rate": 0.0
}
```

## 081 F1 membership vs generation (Fact LR-wins)

```json
{
  "fact_lr_wins_n": 17,
  "membership_n": 0,
  "generation_n": 17,
  "membership_share": 0.0,
  "generation_share": 1.0,
  "note": "081 F1: membership = gold token coverage <0.15 in EQ Acc context; generation = coverage \u22650.15 but Acc still lags LR"
}
```

## Ranked failure modes

1. **Fact_Acc_LR_ahead** — Primary Acc type gap — often R6 list split / evidence miss
   `{"mode": "Fact_Acc_LR_ahead", "count": 21, "mean_delta": 0.0193}`

2. **Fact_ER_gap** — Supports D1 R6 Acc/L2 unify before packing retries
   `{"mode": "Fact_ER_gap", "eq_fact_er": 0.88, "lr_fact_er": 0.95}`

## Acc Δ by type (EQ − LR)

```json
{
  "Complex Reasoning": {
    "n": 50,
    "mean_acc_delta_eq_minus_lr": 0.0166,
    "eq_ahead": 22,
    "lr_ahead": 16,
    "near_tie": 12
  },
  "Contextual Summarize": {
    "n": 50,
    "mean_acc_delta_eq_minus_lr": -0.0235,
    "eq_ahead": 17,
    "lr_ahead": 23,
    "near_tie": 10
  },
  "Creative Generation": {
    "n": 50,
    "mean_acc_delta_eq_minus_lr": 0.0219,
    "eq_ahead": 28,
    "lr_ahead": 16,
    "near_tie": 6
  },
  "Fact Retrieval": {
    "n": 50,
    "mean_acc_delta_eq_minus_lr": 0.0193,
    "eq_ahead": 10,
    "lr_ahead": 21,
    "near_tie": 19
  }
}
```

## Empty answers

- EQ empty: 0
- LR empty: 0

## SNR proxy (gold/evidence tokens ∩ context)

```json
{
  "eq_mean_jaccard_gold_vs_context": 0.0124,
  "lr_mean_jaccard_gold_vs_context": 0.0154,
  "eq_lt_lr_count": 190
}
```

Top Fact LR-wins written in `failure_slice.json` (17 rows).
