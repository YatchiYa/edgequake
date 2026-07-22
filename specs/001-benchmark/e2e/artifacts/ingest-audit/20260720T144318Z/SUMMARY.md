# Ingest parity audit (028 B1)

**UTC:** 20260720T144319Z  
**EQ workspace:** `dbaf36a1-6a59-4d3d-9438-8a84da92bdc9`  
**LR stage:** `smoke`  
**EQ graph:** `eq_eq_default_graph`  

## Counts

| Side | Entities/nodes | Edges / rels | Linked chunks |
|------|----------------|--------------|---------------|
| LR | 3580 | 5325 | 199 |
| EQ | 4465 | 8270 | 188 |

## Identity parity (032 B3b)

- EQ entity vectors: **4465**
- EQ AGE nodes (WS filter): **4465**
- AGE/vectors ratio: **1.0** (target ≈ 1.0)
- B3b pass when age_over_vectors ≈ 1.0 (±0.10); pre-fix Acc WS often ~0.09 (global node_id collision).

## Entity name overlap (normalized)

- Jaccard: **0.4459**
- EQ coverage of LR: **0.7292**
- LR coverage of EQ: **0.5344**
- EQ soft-overlap (substring ≥6): **0.7508** (3330/4465)
- Only LR (sample): `2022, 2D_DIGITAL_MAMMOGRAPHY, 2D_DIGITAL_MAMMOGRAPHY_3D_MAMMOGRAM, 3D_MAMMOGRAM, 5_FLUOROURACIL, ABDOMINAL_BLOATING, ABDOMINAL_CT, ABDOMINAL_ORGANS, ABNORMAL_LYMPH_NODES, ABNORMAL_SKIN_GROWTHS, ABSOLUTE_NEUTROPHIL_COUNT, ACUTE_LYMPHOBLASTIC_LEUKEMIA`

## Linkage density

- LR mean chunks/entity: 2.204
- EQ mean chunks/entity: 2.353
- EQ zero-chunk entities: 0

## Relation linkage (049 B6)

- EQ mean chunks/edge: **1.2**
- LR mean chunks/relation: **1.17**
- EQ edges ≥2 chunks: **1072** (rate **0.1296**)
- LR relations ≥2 chunks: **633** (rate **0.1189**)
- 049 B6: endpoint dedupe must union chunk ids (raise eq_edges_ge2_rate).

## Stub provenance (044 B5)

- Zero-chunk total: **0** (rate **0.0**)
- UNKNOWN empty stubs: **0**
- Named zero-chunk: **0**
- B5 pass when eq_zero_chunk_rate ≤ 0.01 after placeholder provenance inherit.

## Re-ingest plan

- Forced new workspace + labeled ingest pins (never silent Acc pin change).
- Re-run Acc query-only (A0/A1) on new workspace after B2/B3.

Artifact: `/Users/raphaelmansuy/Github/03-working/edgequake/specs/001-benchmark/e2e/artifacts/ingest-audit/20260720T144318Z/audit_report.json`
