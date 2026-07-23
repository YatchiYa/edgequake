# 081 / 080 failure slice

**Archive:** `/Users/raphaelmansuy/Github/03-working/edgequake/specs/001-benchmark/e2e/artifacts/history/medical-mid-20260723T013716Z`
**Paired eval n:** 200
**Recommended next:** `F4_generation_after_F3`

## Scorecard snapshot

```json
{
  "eq_acc": 0.7495132732626798,
  "lr_acc": 0.7753971148169302,
  "eq_ctx": 0.45875,
  "lr_ctx": 0.495,
  "eq_er": 0.9439204178174767,
  "lr_er": 0.952426317554994,
  "eq_fact_er": 0.93,
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
   `{"mode": "Fact_Acc_LR_ahead", "count": 19, "mean_delta": -0.0155}`

## Acc Δ by type (EQ − LR)

```json
{
  "Complex Reasoning": {
    "n": 50,
    "mean_acc_delta_eq_minus_lr": -0.0016,
    "eq_ahead": 22,
    "lr_ahead": 22,
    "near_tie": 6
  },
  "Contextual Summarize": {
    "n": 50,
    "mean_acc_delta_eq_minus_lr": -0.0271,
    "eq_ahead": 23,
    "lr_ahead": 24,
    "near_tie": 3
  },
  "Creative Generation": {
    "n": 50,
    "mean_acc_delta_eq_minus_lr": -0.0593,
    "eq_ahead": 15,
    "lr_ahead": 31,
    "near_tie": 4
  },
  "Fact Retrieval": {
    "n": 50,
    "mean_acc_delta_eq_minus_lr": -0.0155,
    "eq_ahead": 11,
    "lr_ahead": 19,
    "near_tie": 20
  }
}
```

## Empty answers

- EQ empty: 0
- LR empty: 0

## SNR proxy (gold/evidence tokens ∩ context)

```json
{
  "eq_mean_jaccard_gold_vs_context": 0.0162,
  "lr_mean_jaccard_gold_vs_context": 0.0154,
  "eq_lt_lr_count": 120
}
```

Top Fact LR-wins written in `failure_slice.json` (17 rows).
