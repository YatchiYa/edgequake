# Ingest parity audit (028 B1)

**UTC:** 20260720T074558Z  
**EQ workspace:** `951bb6fa-0fa0-4986-8c9b-527e616f613a`  
**LR stage:** `smoke`  
**EQ graph:** `eq_eq_default_graph`  

## Counts

| Side | Entities/nodes | Edges / rels | Linked chunks |
|------|----------------|--------------|---------------|
| LR | 3580 | 5325 | 199 |
| EQ | 533 | 4705 | 222 |

## Entity name overlap (normalized)

- Jaccard: **0.0139**
- EQ coverage of LR: **0.016**
- LR coverage of EQ: **0.0976**
- EQ soft-overlap (substring ≥6): **0.636** (339/533)
- Only LR (sample): `17P_DELETION, 18_FLUORODEOXYGLUCOSE, 2022, 2D_DIGITAL_MAMMOGRAM, 2D_DIGITAL_MAMMOGRAPHY, 2D_DIGITAL_MAMMOGRAPHY_3D_MAMMOGRAM, 3D_MAMMOGRAM, 4KSCORE, 5_FLUOROURACIL, 5_FLUOROURACIL_5_FU, 5_FU, ABDOMEN`

## Linkage density

- LR mean chunks/entity: 2.204
- EQ mean chunks/entity: 0.946
- EQ zero-chunk entities: 68

## Re-ingest plan

- Forced new workspace + labeled ingest pins (never silent Acc pin change).
- Re-run Acc query-only (A0/A1) on new workspace after B2/B3.

Artifact: `/Users/raphaelmansuy/Github/03-working/edgequake/specs/001-benchmark/e2e/artifacts/ingest-audit/20260720T074557Z/audit_report.json`
