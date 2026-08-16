# 081 / 080 failure slice

**Archive:** `/Users/raphaelmansuy/Github/03-working/edgequake/specs/001-benchmark/e2e/artifacts/history/medical-mid-20260815T110218Z`
**Paired eval n:** 200
**Recommended next:** `F4_generation_after_F3`

## Scorecard snapshot

```json
{
  "eq_acc": 0.7915453483563613,
  "lr_acc": 0.7858737967830962,
  "eq_ctx": 0.47125,
  "lr_ctx": 0.51,
  "eq_er": 0.9315970953066541,
  "lr_er": 0.9486312792599557,
  "eq_fact_er": 0.8466666666666666,
  "lr_fact_er": 0.95,
  "eq_empty_answer_rate": 0.0,
  "lr_empty_answer_rate": 0.0
}
```

## 081 F1 membership vs generation (Fact LR-wins)

```json
{
  "fact_lr_wins_n": 10,
  "membership_n": 0,
  "generation_n": 10,
  "membership_share": 0.0,
  "generation_share": 1.0,
  "note": "081 F1: membership = gold token coverage <0.15 in EQ Acc context; generation = coverage \u22650.15 but Acc still lags LR"
}
```

## Ranked failure modes

1. **Fact_Acc_LR_ahead** — Primary Acc type gap — often R6 list split / evidence miss
   `{"mode": "Fact_Acc_LR_ahead", "count": 12, "mean_delta": 0.0234}`

2. **Fact_ER_gap** — Supports D1 R6 Acc/L2 unify before packing retries
   `{"mode": "Fact_ER_gap", "eq_fact_er": 0.8467, "lr_fact_er": 0.95}`

## Acc Δ by type (EQ − LR)

```json
{
  "Complex Reasoning": {
    "n": 50,
    "mean_acc_delta_eq_minus_lr": 0.0186,
    "eq_ahead": 25,
    "lr_ahead": 17,
    "near_tie": 8
  },
  "Contextual Summarize": {
    "n": 50,
    "mean_acc_delta_eq_minus_lr": -0.0293,
    "eq_ahead": 16,
    "lr_ahead": 28,
    "near_tie": 6
  },
  "Creative Generation": {
    "n": 50,
    "mean_acc_delta_eq_minus_lr": 0.01,
    "eq_ahead": 26,
    "lr_ahead": 16,
    "near_tie": 8
  },
  "Fact Retrieval": {
    "n": 50,
    "mean_acc_delta_eq_minus_lr": 0.0234,
    "eq_ahead": 13,
    "lr_ahead": 12,
    "near_tie": 25
  }
}
```

## Empty answers

- EQ empty: 0
- LR empty: 0

## SNR proxy (gold/evidence tokens ∩ context)

```json
{
  "eq_mean_jaccard_gold_vs_context": 0.0128,
  "lr_mean_jaccard_gold_vs_context": 0.0154,
  "eq_lt_lr_count": 189
}
```

Top Fact LR-wins written in `failure_slice.json` (10 rows).
