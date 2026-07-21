# Summarize chunk-link audit (037 Horizon B) — First Principles

**UTC:** 20260720T103653Z  
**EQ workspace:** `2a7bcb2f-b156-4c49-9229-67f5bcde22a4`  
**LR stage:** `smoke`  
**EQ preds:** `specs/001-benchmark/e2e/artifacts/history/smoke-20260720T095809Z/predictions_eq.json`  
**LR preds:** `specs/001-benchmark/e2e/artifacts/history/smoke-20260720T095809Z/predictions_lr.json`  

## Laws (necessary conditions)

| Law | Meaning |
|-----|---------|
| LINK | Exact-name topic entity: LR has source chunks, EQ has none |
| SELECT | Topic entities linked both sides; EQ Mix hits fewer question content bigrams |
| GEN_OR_EVAL | EQ Mix question-bigram hits not below LR |

Probes: question content bigrams (verbatim) + exact-name entity pairs.  
Forbidden: `+N` / `%` / domain-needle / token-overlap promote heuristics.  
Note: GraphRAG-Bench evidence strings are paraphrases — not used as corpus probes.

## Global hygiene (observables only)

- EQ entities: **4559** · mean chunks **2.228** · zero-chunk **344**
- LR entities: **3580** · mean chunks **2.204** · zero-chunk **0**
- EQ AGE nodes/edges: **4560** / **8247**

## Per Summarize question

| ID | Law | EQ parts | LR parts | EQ q-bigrams | LR q-bigrams | LINK empty |
|----|-----|---------:|---------:|-------------:|-------------:|-----------:|
| `Medical-0002d2de` | SELECT | 6 | 14 | 0/3 | 1/3 | 0 |
| `Medical-1991db28` | SELECT | 22 | 16 | 0/4 | 1/4 | 0 |
| `Medical-8f9d5dde` | GEN_OR_EVAL | 17 | 12 | 1/7 | 1/7 | 0 |
| `Medical-00bf955d` | SELECT | 18 | 16 | 1/6 | 2/6 | 0 |
| `Medical-e168b4d3` | GEN_OR_EVAL | 18 | 14 | 1/6 | 1/6 | 0 |
| `Medical-c2a36052` | GEN_OR_EVAL | 18 | 14 | 1/5 | 1/5 | 0 |
| `Medical-25f9adbb` | GEN_OR_EVAL | 19 | 12 | 1/5 | 1/5 | 0 |
| `Medical-296c7595` | GEN_OR_EVAL | 20 | 16 | 1/5 | 1/5 | 0 |
| `Medical-6809b810` | GEN_OR_EVAL | 18 | 14 | 2/9 | 2/9 | 0 |
| `Medical-b5a3c96e` | GEN_OR_EVAL | 19 | 12 | 3/9 | 3/9 | 0 |

## Binding question (fewest EQ question-bigram hits)

**Medical-0002d2de** — How are bone cancers staged and what factors are considered in determining the stage?

**Law:** `SELECT`  
**Why:** Topic entities are linked on both sides (not LINK starvation), yet EQ Mix contains fewer of the question's own content bigrams than LR — Mix admission selected the wrong neighborhood.

**Question bigrams:** `['bone cancers', 'cancers staged', 'staged stage']`

### Facts

```json
{
  "eq_mix_parts": 6,
  "lr_mix_parts": 14,
  "eq_question_bigrams_hit": 0,
  "lr_question_bigrams_hit": 1,
  "eq_missed_bigrams": [
    "bone cancers",
    "cancers staged",
    "staged stage"
  ],
  "lr_missed_bigrams": [
    "cancers staged",
    "staged stage"
  ],
  "n_exact_pairs_both_linked": 4,
  "n_exact_pairs_eq_empty_lr_linked": 0,
  "n_exact_pairs_lr_only": 0,
  "eq_empty_names": [],
  "both_linked_sample": [
    {
      "norm": "BONE_CANCER",
      "eq_n": 5,
      "lr_n": 6
    },
    {
      "norm": "BONE",
      "eq_n": 17,
      "lr_n": 17
    },
    {
      "norm": "CANCER",
      "eq_n": 46,
      "lr_n": 34
    },
    {
      "norm": "STAGE",
      "eq_n": 2,
      "lr_n": 3
    }
  ]
}
```

### Exact-name pairs (question-derived)

| norm | EQ name | EQ n | LR name | LR n | gap |
|------|---------|-----:|---------|-----:|-----|
| `BONE_CANCER` | BONE_CANCER | 5 | Bone cancer | 6 | BOTH_LINKED |
| `BONE` | BONE | 17 | Bone | 17 | BOTH_LINKED |
| `CANCERS` | CANCERS | 3 | — | — | EQ_ONLY |
| `CANCER` | CANCER | 46 | Cancer | 34 | BOTH_LINKED |
| `STAGE` | STAGE | 2 | Stage | 3 | BOTH_LINKED |

### EQ Mix part heads

1. . Options for non–fertility-sparing treatment are provided next according to stage. EBRT or chemoradiation may be needed after surgery. Stage 1A1 Stage 1A1 canc
2. 7 The anus 8 Risk factors 9 Diagnosis and treatment planning 13 Staging 14 Fertility and family planning 15 Key points Anal cancer basics The anus Anal cancer i
3. . Key points Cervical cancer is most often diagnosed by cervical biopsy. Samples of cervical tissue are removed and tested for dysplasia and cancer. A cone biop
4. . In general, to be diagnosed with AML, 20 percent or more myeloblasts must be present in the bone marrow or blood. This means that at least 1 out of every 5 ce
5. . In general, to be diagnosed with AML, 20 percent or more myeloblasts must be present in the bone marrow or blood. This means that at least 1 out of every 5 ce
6. . This tube will allow your kidneys to drain. Your urine will now exit the body through a small opening called a stoma. A small disposable bag attached to the o

### LR Mix part heads

1. physis (growth plate) and diaphysis. Diaphysis – the middle region of the bone. Physis – the growth plate, which is made of cartilage. After skeletal maturity, 
2. , and produce hormones. Bone is light, yet strong and can regrow. What's in this book? This book is organized into the following chapters: Chapter 2: Testing fo
3. ostic stage also includes the assumption that you are treated with the standard-of-care approaches. Prognostic stages are divided into clinical and pathologic. 
4. This is called a somatic mutation or somatic change. MSI-H/dMMR mutation Microsatellites are short, repeated strings of DNA. When errors or defects occur, they 
5. It might include cancer in the base of the skull or carotid artery. Metastatic – This is cancer that has spread to other parts of the body, including lung and d
6. If the cancer doesn't take up iodine, targeted therapy may be an option. But, if the metastatic tumors are growing slowly (or not at all) and aren't causing sym
7. treatment begins. Online portals are a great way to access your test results. Please discuss your results with your health care provider A medical history and p
8. , siblings, and children. Next, talk to half-siblings, aunts and uncles, nieces and nephews, grandparents, and grandchildren. Write down what you learn about yo

## Next confound (from law)

- Do **not** densify-all links. Next confound: Mix entity ranking / related-chunk pick so linked question-topic entities admit into C instead of generic neighbors (off-topic Mix).
