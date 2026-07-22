# Ingest parity audit (028 B1)

**UTC:** 20260720T084155Z  
**EQ workspace:** `2a7bcb2f-b156-4c49-9229-67f5bcde22a4`  
**LR stage:** `smoke`  
**EQ graph:** `eq_eq_default_graph`  

## Counts

| Side | Entities/nodes | Edges / rels | Linked chunks |
|------|----------------|--------------|---------------|
| LR | 3580 | 5325 | 199 |
| EQ | 4560 | 8247 | 188 |

## Identity parity (032 B3b)

- EQ entity vectors: **4215**
- EQ AGE nodes (WS filter): **4560**
- AGE/vectors ratio: **1.0819** (target ≈ 1.0)
- B3b pass when age_over_vectors ≈ 1.0 (±0.10); pre-fix Acc WS often ~0.09 (global node_id collision).

## Entity name overlap (normalized)

- Jaccard: **0.4409**
- EQ coverage of LR: **0.7329**
- LR coverage of EQ: **0.5252**
- EQ soft-overlap (substring ≥6): **0.7466** (3386/4560)
- Only LR (sample): `2022, 2D_DIGITAL_MAMMOGRAPHY_3D_MAMMOGRAM, 5_FLUOROURACIL, ABDOMINAL_CT, ABNORMAL_BLEEDING, ABSOLUTE_NEUTROPHIL_COUNT, ACUTE_LYMPHOBLASTIC_LEUKEMIA, ACUTE_PROMYELOCYTIC_LEUKEMIA, ADOLESCENT_AND_YOUNG_ADULT, ADO_TRASTUZUMAB_EMTANSINE, ADRENOCORTICAL_ADENOMA, ADRENOCORTICAL_CANCER`

## Linkage density

- LR mean chunks/entity: 2.204
- EQ mean chunks/entity: 2.228
- EQ zero-chunk entities: 345

## Re-ingest plan

- Forced new workspace + labeled ingest pins (never silent Acc pin change).
- Re-run Acc query-only (A0/A1) on new workspace after B2/B3.

Artifact: `/Users/raphaelmansuy/Github/03-working/edgequake/specs/001-benchmark/e2e/artifacts/ingest-audit/20260720T084152Z/audit_report.json`
