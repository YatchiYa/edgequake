# 081 / 080 failure slice

**Archive:** `/Users/raphaelmansuy/Github/03-working/edgequake/specs/001-benchmark/e2e/artifacts/history/medical-mid-20260802T141536Z`
**Paired eval n:** 199
**Recommended next:** `F4_generation_after_F3`

## Scorecard snapshot

```json
{
  "eq_acc": 0.7856817014567021,
  "lr_acc": 0.7812871077413082,
  "eq_ctx": 0.43125,
  "lr_ctx": 0.495,
  "eq_er": 0.9236729430700019,
  "lr_er": 0.9584725491012256,
  "eq_fact_er": 0.87,
  "lr_fact_er": 0.98,
  "eq_empty_answer_rate": 0.0,
  "lr_empty_answer_rate": 0.0
}
```

## 081 F1 membership vs generation (Fact LR-wins)

```json
{
  "fact_lr_wins_n": 14,
  "membership_n": 0,
  "generation_n": 14,
  "membership_share": 0.0,
  "generation_share": 1.0,
  "note": "081 F1: membership = gold token coverage <0.15 in EQ Acc context; generation = coverage \u22650.15 but Acc still lags LR"
}
```

## Ranked failure modes

1. **Fact_Acc_LR_ahead** — Primary Acc type gap — often R6 list split / evidence miss
   `{"mode": "Fact_Acc_LR_ahead", "count": 17, "mean_delta": -0.0163}`

2. **Fact_ER_gap** — Supports D1 R6 Acc/L2 unify before packing retries
   `{"mode": "Fact_ER_gap", "eq_fact_er": 0.87, "lr_fact_er": 0.98}`

## Acc Δ by type (EQ − LR)

```json
{
  "Complex Reasoning": {
    "n": 50,
    "mean_acc_delta_eq_minus_lr": -0.0043,
    "eq_ahead": 17,
    "lr_ahead": 21,
    "near_tie": 12
  },
  "Contextual Summarize": {
    "n": 49,
    "mean_acc_delta_eq_minus_lr": 0.0119,
    "eq_ahead": 26,
    "lr_ahead": 19,
    "near_tie": 4
  },
  "Creative Generation": {
    "n": 50,
    "mean_acc_delta_eq_minus_lr": 0.0261,
    "eq_ahead": 25,
    "lr_ahead": 16,
    "near_tie": 9
  },
  "Fact Retrieval": {
    "n": 50,
    "mean_acc_delta_eq_minus_lr": -0.0163,
    "eq_ahead": 13,
    "lr_ahead": 17,
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
  "eq_mean_jaccard_gold_vs_context": 0.0126,
  "lr_mean_jaccard_gold_vs_context": 0.0154,
  "eq_lt_lr_count": 188
}
```

Top Fact LR-wins written in `failure_slice.json` (14 rows).
