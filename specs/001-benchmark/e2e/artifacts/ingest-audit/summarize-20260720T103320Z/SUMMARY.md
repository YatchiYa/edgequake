# Summarize chunk-link audit (037 Horizon B)

**UTC:** 20260720T103321Z  
**EQ workspace:** `2a7bcb2f-b156-4c49-9229-67f5bcde22a4`  
**LR stage:** `smoke`  
**Predictions:** `specs/001-benchmark/e2e/artifacts/history/smoke-20260720T095809Z/predictions_eq.json`  

## Global (warm WS)

- EQ AGE nodes: **4560** · edges **8247**
- LR entities: **3580**

## Per Summarize question (sorted by EQ context parts ↑)

| ID | EQ parts | EQ∪chunks | LR∪chunks | Δ∪ | EQ mean | LR mean | Intent |
|----|---------:|----------:|----------:|---:|--------:|--------:|--------|
| `Medical-0002d2de` | 6 | 188 | 198 | -10 | 2.333 | 2.264 | exploratory |
| `Medical-8f9d5dde` | 17 | 187 | 195 | -8 | 2.395 | 2.289 | exploratory |
| `Medical-6809b810` | 18 | 186 | 193 | -7 | 1.984 | 1.972 | exploratory |
| `Medical-c2a36052` | 18 | 187 | 193 | -6 | 2.29 | 2.162 | relational |
| `Medical-e168b4d3` | 18 | 111 | 111 | +0 | 1.74 | 1.725 | factual |
| `Medical-00bf955d` | 18 | 157 | 119 | +38 | 2.179 | 2.189 | exploratory |
| `Medical-b5a3c96e` | 19 | 147 | 152 | -5 | 2.563 | 2.636 | comparative |
| `Medical-25f9adbb` | 19 | 164 | 164 | +0 | 3.07 | 2.894 | exploratory |
| `Medical-296c7595` | 20 | 180 | 190 | -10 | 2.161 | 2.154 | exploratory |
| `Medical-1991db28` | 22 | 155 | 114 | +41 | 2.291 | 2.344 | exploratory |

## Binding miss (fewest EQ context parts)

**Medical-0002d2de** — How are bone cancers staged and what factors are considered in determining the stage?

- EQ context parts/chars: **6** / 41061
- EQ matched entities: **732** · union chunks **188** · mean **2.333** · zero-chunk **80**
- LR matched entities: **667** · union chunks **198** · mean **2.264** · zero-chunk **0**
- EQ top entities: `[{'name': 'LYMPH_NODES', 'n_chunks': 89}, {'name': 'NCCN_PATIENT_GUIDES_FOR_CANCER_APP', 'n_chunks': 49}, {'name': 'CANCER', 'n_chunks': 46}, {'name': 'TUMOR', 'n_chunks': 41}, {'name': 'SYSTEMIC_THERAPY', 'n_chunks': 32}, {'name': 'CANCER_CELLS', 'n_chunks': 30}, {'name': 'BONE_MARROW', 'n_chunks': 28}, {'name': 'BONE', 'n_chunks': 17}]`
- LR top entities: `[{'name': 'Tumor', 'n_chunks': 52}, {'name': 'Lymph nodes', 'n_chunks': 51}, {'name': 'Cancer', 'n_chunks': 34}, {'name': 'NCCN Patient Guides for Cancer app', 'n_chunks': 33}, {'name': 'Bone marrow', 'n_chunks': 28}, {'name': 'Systemic therapy', 'n_chunks': 28}, {'name': 'Metastasis', 'n_chunks': 23}, {'name': 'Lymph node', 'n_chunks': 18}]`
- Sample tokens: `['BONE', 'CANCER', 'CANCERS', 'COMBINING', 'CONSIDERS', 'DETERMINED', 'FACTORS', 'FINAL', 'GRADE', 'INVOLVEMENT', 'LOCATION', 'LYMPH', 'METASTASIS', 'NODE', 'SCORES', 'SIZE', 'STAGE', 'STAGED', 'SYSTEM', 'TNM', 'TUMOR', 'USING']`

## Decision

- **EQ query-matched entity→chunk pool ≪ LR** → Horizon B fix: denser `source_chunk_ids` on extract/merge for Summarize entities (or broader entity hit for staging/bone topics). Re-ingest when disk ≥15 Gi.
