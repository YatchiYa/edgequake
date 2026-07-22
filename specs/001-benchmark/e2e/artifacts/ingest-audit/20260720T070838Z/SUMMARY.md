# Ingest parity audit (028 B1)

**UTC:** 20260720T070839Z  
**EQ workspace:** `e0270f5f-0b6c-4e90-882f-5f9b0eac8cff`  
**LR stage:** `smoke`  
**EQ graph:** `eq_eq_default_graph`  

## Counts

| Side | Entities/nodes | Edges / rels | Linked chunks |
|------|----------------|--------------|---------------|
| LR | 3580 | 5325 | 199 |
| EQ | 392 | 3069 | 125 |

## Entity name overlap (normalized)

- Jaccard: **0.0159**
- EQ coverage of LR: **0.0175**
- LR coverage of EQ: **0.1454**
- EQ soft-overlap (substring ≥6): **0.6403** (251/392)
- Only LR (sample): `17P_DELETION, 18_FLUORODEOXYGLUCOSE, 2022, 2D_DIGITAL_MAMMOGRAM, 2D_DIGITAL_MAMMOGRAPHY, 2D_DIGITAL_MAMMOGRAPHY_3D_MAMMOGRAM, 4KSCORE, 5_FLUOROURACIL, 5_FLUOROURACIL_5_FU, 5_FU, ABDOMEN, ABDOMINAL_AND_PELVIC_EXAM`

## Linkage density

- LR mean chunks/entity: 2.204
- EQ mean chunks/entity: 0.952
- EQ zero-chunk entities: 44

## Re-ingest plan

- Forced new workspace + labeled ingest pins (never silent Acc pin change).
- Re-run Acc query-only (A0/A1) on new workspace after B2/B3.

Artifact: `/Users/raphaelmansuy/Github/03-working/edgequake/specs/001-benchmark/e2e/artifacts/ingest-audit/20260720T070838Z/audit_report.json`
