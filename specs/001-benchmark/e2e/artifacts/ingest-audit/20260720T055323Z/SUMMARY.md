# Ingest parity audit (028 B1)

**UTC:** 20260720T055324Z  
**EQ workspace:** `8b359190-0733-4949-994c-f39eca074d79`  
**LR stage:** `smoke`  
**EQ graph:** `eq_eq_default_graph`  

## Counts

| Side | Entities/nodes | Edges / rels | Linked chunks |
|------|----------------|--------------|---------------|
| LR | 3580 | 5325 | 199 |
| EQ | 429 | 4087 | 128 |

## Entity name overlap (normalized)

- Jaccard: **0.0091**
- EQ coverage of LR: **0.0102**
- LR coverage of EQ: **0.0769**
- Only LR (sample): `17P_DELETION, 18_FLUORODEOXYGLUCOSE, 2022, 2D_DIGITAL_MAMMOGRAPHY, 2D_DIGITAL_MAMMOGRAPHY_3D_MAMMOGRAM, 3D_MAMMOGRAM, 4KSCORE, 5_FLUOROURACIL, 5_FLUOROURACIL_5_FU, 5_FU, ABDOMEN, ABDOMINAL_AND_PELVIC_EXAM`

## Linkage density

- LR mean chunks/entity: 2.204
- EQ mean chunks/entity: 0.881
- EQ zero-chunk entities: 62

## Re-ingest plan

- Forced new workspace + labeled ingest pins (never silent Acc pin change).
- Re-run Acc query-only (A0/A1) on new workspace after B2/B3.

Artifact: `/Users/raphaelmansuy/Github/03-working/edgequake/specs/001-benchmark/e2e/artifacts/ingest-audit/20260720T055323Z/audit_report.json`
