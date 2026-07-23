# 081 / 080 failure slice

**Archive:** `/Users/raphaelmansuy/Github/03-working/edgequake/specs/001-benchmark/e2e/artifacts/history/medical-full-20260722T171906Z`
**Paired eval n:** 406
**Recommended next:** `F4_generation_after_F3`

## Scorecard snapshot

```json
{
  "eq_acc": 0.7390521215605286,
  "lr_acc": 0.784459102912658,
  "eq_ctx": 0.47167158103306706,
  "lr_ctx": 0.48563412570492925,
  "eq_er": 0.9453447023206517,
  "lr_er": 0.9469082267996299,
  "eq_fact_er": 0.9181350005918706,
  "lr_fact_er": 0.9375557878653854,
  "eq_empty_answer_rate": 0.00048496605237633366,
  "lr_empty_answer_rate": 0.0009699321047526673
}
```

## 081 F1 membership vs generation (Fact LR-wins)

```json
{
  "fact_lr_wins_n": 109,
  "membership_n": 0,
  "generation_n": 109,
  "membership_share": 0.0,
  "generation_share": 1.0,
  "note": "081 F1: membership = gold token coverage <0.15 in EQ Acc context; generation = coverage \u22650.15 but Acc still lags LR"
}
```

## Ranked failure modes

1. **R5_empty_answers** — EQ empty generated_answer with (likely) non-empty context
   `{"mode": "R5_empty_answers", "count": 1, "ids_sample": ["Medical-83d2a1a9"]}`

2. **Fact_Acc_LR_ahead** — Primary Acc type gap — often R6 list split / evidence miss
   `{"mode": "Fact_Acc_LR_ahead", "count": 115, "mean_delta": -0.0444}`

## Acc Δ by type (EQ − LR)

```json
{
  "Complex Reasoning": {
    "n": 40,
    "mean_acc_delta_eq_minus_lr": -0.0905,
    "eq_ahead": 15,
    "lr_ahead": 20,
    "near_tie": 5
  },
  "Contextual Summarize": {
    "n": 33,
    "mean_acc_delta_eq_minus_lr": 0.0487,
    "eq_ahead": 17,
    "lr_ahead": 11,
    "near_tie": 5
  },
  "Creative Generation": {
    "n": 20,
    "mean_acc_delta_eq_minus_lr": -0.0963,
    "eq_ahead": 4,
    "lr_ahead": 14,
    "near_tie": 2
  },
  "Fact Retrieval": {
    "n": 313,
    "mean_acc_delta_eq_minus_lr": -0.0444,
    "eq_ahead": 76,
    "lr_ahead": 115,
    "near_tie": 122
  }
}
```

## Empty answers

- EQ empty: 1
- LR empty: 2

## SNR proxy (gold/evidence tokens ∩ context)

```json
{
  "eq_mean_jaccard_gold_vs_context": 0.0099,
  "lr_mean_jaccard_gold_vs_context": 0.008,
  "eq_lt_lr_count": 239
}
```

Top Fact LR-wins written in `failure_slice.json` (40 rows).
