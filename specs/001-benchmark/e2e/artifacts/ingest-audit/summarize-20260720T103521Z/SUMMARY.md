# Summarize chunk-link audit (037 Horizon B) — First Principles

**UTC:** 20260720T103521Z  
**EQ workspace:** `2a7bcb2f-b156-4c49-9229-67f5bcde22a4`  
**LR stage:** `smoke`  
**EQ preds:** `specs/001-benchmark/e2e/artifacts/history/smoke-20260720T095809Z/predictions_eq.json`  
**LR preds:** `specs/001-benchmark/e2e/artifacts/history/smoke-20260720T095809Z/predictions_lr.json`  

## Laws (necessary conditions)

| Law | Meaning |
|-----|---------|
| LINK | Exact-name topic entity: LR has source chunks, EQ has none |
| SELECT | Topic entities linked both sides; EQ Mix hits fewer gold phrases |
| GEN_OR_EVAL | EQ Mix gold coverage not below LR |

No `+N` / `%` / domain-needle promote heuristics.

## Global hygiene (observables only)

- EQ entities: **4559** · mean chunks **2.228** · zero-chunk **344**
- LR entities: **3580** · mean chunks **2.204** · zero-chunk **0**
- EQ AGE nodes/edges: **4560** / **8247**

## Per Summarize question

| ID | Law | EQ parts | LR parts | EQ gold-phr | LR gold-phr | EQ on-gold parts | LR on-gold parts | LINK empty |
|----|-----|---------:|---------:|------------:|------------:|-----------------:|-----------------:|-----------:|
| `Medical-c2a36052` | GEN_OR_EVAL | 18 | 14 | 5/5 | 5/5 | 17/18 | 14/14 | 0 |
| `Medical-1991db28` | GEN_OR_EVAL | 22 | 16 | 6/6 | 6/6 | 18/22 | 16/16 | 0 |
| `Medical-e168b4d3` | GEN_OR_EVAL | 18 | 14 | 7/9 | 7/9 | 16/18 | 13/14 | 0 |
| `Medical-6809b810` | GEN_OR_EVAL | 18 | 14 | 7/8 | 7/8 | 18/18 | 14/14 | 0 |
| `Medical-00bf955d` | GEN_OR_EVAL | 18 | 16 | 9/10 | 9/10 | 17/18 | 13/16 | 0 |
| `Medical-0002d2de` | GEN_OR_EVAL | 6 | 14 | 10/11 | 10/11 | 6/6 | 14/14 | 0 |
| `Medical-b5a3c96e` | GEN_OR_EVAL | 19 | 12 | 10/10 | 10/10 | 15/19 | 9/12 | 0 |
| `Medical-296c7595` | GEN_OR_EVAL | 20 | 16 | 13/13 | 13/13 | 20/20 | 16/16 | 0 |
| `Medical-25f9adbb` | GEN_OR_EVAL | 19 | 12 | 15/15 | 15/15 | 18/19 | 12/12 | 0 |
| `Medical-8f9d5dde` | GEN_OR_EVAL | 17 | 12 | 21/24 | 21/24 | 17/17 | 12/12 | 0 |

## Binding question (lowest EQ gold phrase hits)

**Medical-c2a36052** — How do biomarkers influence treatment selection in colon cancer?

**Law:** `GEN_OR_EVAL`  
**Why:** EQ Mix gold phrase coverage is not below LR on this probe; if Summarize ER still lags, investigate generation / eval, not chunk-link density.

### Facts

```json
{
  "eq_mix_parts": 18,
  "lr_mix_parts": 14,
  "eq_gold_part_fraction": 0.9444,
  "lr_gold_part_fraction": 1.0,
  "eq_phrase_hit_fraction": 1.0,
  "lr_phrase_hit_fraction": 1.0,
  "n_exact_pairs_both_linked": 8,
  "n_exact_pairs_eq_empty_lr_linked": 0,
  "n_exact_pairs_lr_only": 0,
  "eq_empty_names": [],
  "both_linked_sample": [
    {
      "norm": "COLON_CANCER",
      "eq_n": 3,
      "lr_n": 5
    },
    {
      "norm": "BIOMARKERS",
      "eq_n": 6,
      "lr_n": 2
    },
    {
      "norm": "BIOMARKER",
      "eq_n": 3,
      "lr_n": 8
    },
    {
      "norm": "TREATMENT",
      "eq_n": 11,
      "lr_n": 5
    },
    {
      "norm": "COLON",
      "eq_n": 12,
      "lr_n": 9
    },
    {
      "norm": "CANCER",
      "eq_n": 46,
      "lr_n": 34
    },
    {
      "norm": "BIOMARKER_TESTING",
      "eq_n": 25,
      "lr_n": 29
    },
    {
      "norm": "PMMR_MSS_CANCERS",
      "eq_n": 1,
      "lr_n": 1
    }
  ]
}
```

### Exact-name pairs (question-derived)

| norm | EQ name | EQ n | LR name | LR n | gap |
|------|---------|-----:|---------|-----:|-----|
| `COLON_CANCER` | COLON_CANCER | 3 | Colon cancer | 5 | BOTH_LINKED |
| `BIOMARKERS` | BIOMARKERS | 6 | Biomarkers | 2 | BOTH_LINKED |
| `BIOMARKER` | BIOMARKER | 3 | Biomarker | 8 | BOTH_LINKED |
| `TREATMENT` | TREATMENT | 11 | Treatment | 5 | BOTH_LINKED |
| `COLON` | COLON | 12 | Colon | 9 | BOTH_LINKED |
| `CANCER` | CANCER | 46 | Cancer | 34 | BOTH_LINKED |
| `BIOMARKER_TESTING` | BIOMARKER_TESTING | 25 | Biomarker testing | 29 | BOTH_LINKED |
| `PMMR_MSS_CANCERS` | PMMR/MSS_CANCERS | 1 | pMMR/MSS Cancers | 1 | BOTH_LINKED |

### EQ Mix part heads

1. Colon cancer basics 5 6 6 The colon Polyps Key points Colon cancer is common and treatable. Many cancers that start in the colon can be cured, especially when f
2. Rectal cancer basics 5 The rectum 6 Polyps 6 Key points Rectal cancer is common and treatable. Many cancers that start in the rectum can be cured, especially wh
3. . FDG-PET/CT can be done at the same time as a CT used for diagnosis. Ultrasound Ultrasound (US) uses high-energy sound waves to form pictures of the inside of 
4. . Levels that are too high or too low may be a sign that an organ isn’t working well. Abnormal levels may also be caused by the spread of cancer or by other dis
5. . CEA blood test Carcinoembryonic antigen (CEA) is a protein found in blood. The level of CEA is often higher than normal in people with colon cancer, especiall
6. . Surveillance When there are no signs of cancer after treatment, expect to see your oncologist on a regular basis for physical and pelvic exams. First 2 years:
7. . The stages are explained below. Stage 0 There are abnormal cells on the innermost layer of the colon wall. These abnormal cells may become cancer and spread i
8. . Physical exam Your hematologist/oncologist will perform a physical exam of your body. This exam will include: Checking your vital signs—blood pressure, heart 

### LR Mix part heads

1. I was dismissed for more than a year for so many different reasons. My journey taught me this: never give up, explore every reasonable option, and prioritize yo
2. information on cancer. Take our survey to let us know what we got right and what we could do better. NCCN.org/patients/feedback 3 Nasopharyngeal cancer staging 
3. into cancer include hyperplastic and inflammatory polyps. While most polyps do not become cancer, almost all colon cancers start in a polyp. Removing polyps can
4. N Guidelines for Patients Immunotherapy Side Effects: Immune Checkpoint Inhibitors at NCCN.org/ patientguidelines and on the NCCN Patient Guides for Cancer app.
5. ps are non-cancerous growths that form on the inner lining of the colon and rectum. The most common type is called an adenoma. While it may take many years, ade
6. will give you information on: How to prepare for surgery What to expect during and after surgery Recovery Possible short- and long-term side effects of colectom
7. ative medicine. Don’t be shy. Be your own advocate—or ask someone close to be one for you.” Multidisciplinary care means that a number of doctors, specialists, 
8. cancer A polyp is an overgrowth of cells on the inner lining of the colon wall. The most common type is called an adenoma. While it may take many years, adenoma

## Next confound (from law)

- See law why; do not invent a density threshold.
