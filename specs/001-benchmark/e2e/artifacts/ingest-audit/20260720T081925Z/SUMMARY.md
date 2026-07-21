# Ingest parity audit (028 B1)

**UTC:** 20260720T081927Z  
**EQ workspace:** `5daf07b4-6824-4548-8780-54b9bc93c70c`  
**LR stage:** `smoke`  
**EQ graph:** `eq_eq_default_graph`  

## Counts

| Side | Entities/nodes | Edges / rels | Linked chunks |
|------|----------------|--------------|---------------|
| LR | 3580 | 5325 | 199 |
| EQ | 0 | 0 | 0 |

## Identity parity (032 B3b)

- EQ entity vectors: **0**
- EQ AGE nodes (WS filter): **0**
- AGE/vectors ratio: **None** (target ≈ 1.0)
- B3b pass when age_over_vectors ≈ 1.0 (±0.10); pre-fix Acc WS often ~0.09 (global node_id collision).

## Entity name overlap (normalized)

- Jaccard: **0.0**
- EQ coverage of LR: **0.0**
- LR coverage of EQ: **None**
- EQ soft-overlap (substring ≥6): **None** (0/0)
- Only LR (sample): `17P_DELETION, 18_FLUORODEOXYGLUCOSE, 2022, 2D_DIGITAL_MAMMOGRAM, 2D_DIGITAL_MAMMOGRAPHY, 2D_DIGITAL_MAMMOGRAPHY_3D_MAMMOGRAM, 3D_MAMMOGRAM, 4KSCORE, 5_FLUOROURACIL, 5_FLUOROURACIL_5_FU, 5_FU, ABDOMEN`

## Linkage density

- LR mean chunks/entity: 2.204
- EQ mean chunks/entity: 0.0
- EQ zero-chunk entities: 0

## Re-ingest plan

- Forced new workspace + labeled ingest pins (never silent Acc pin change).
- Re-run Acc query-only (A0/A1) on new workspace after B2/B3.

Artifact: `/Users/raphaelmansuy/Github/03-working/edgequake/specs/001-benchmark/e2e/artifacts/ingest-audit/20260720T081925Z/audit_report.json`
