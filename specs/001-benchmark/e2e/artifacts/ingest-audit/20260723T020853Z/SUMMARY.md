# Ingest parity audit (028 B1)

**UTC:** 20260723T020854Z  
**EQ workspace:** `54806068-4a82-47b8-a7f9-aeb658f5eddc`  
**LR stage:** `smoke`  
**EQ graph:** `eq_eq_default_graph`  

## Counts

| Side | Entities/nodes | Edges / rels | Linked chunks |
|------|----------------|--------------|---------------|
| LR | 3580 | 5325 | 199 |
| EQ | 4005 | 3928 | 188 |

## Identity parity (032 B3b)

- EQ entity vectors: **4005**
- EQ AGE nodes (WS filter): **4005**
- AGE/vectors ratio: **1.0** (target ≈ 1.0)
- B3b pass when age_over_vectors ≈ 1.0 (±0.10); pre-fix Acc WS often ~0.09 (global node_id collision).

## Entity name overlap (normalized)

- Jaccard: **0.4543**
- EQ coverage of LR: **0.6951**
- LR coverage of EQ: **0.5674**
- EQ soft-overlap (substring ≥6): **0.7639** (3041/4005)
- Only LR (sample): `2022, 2D_DIGITAL_MAMMOGRAPHY_3D_MAMMOGRAM, ABDOMINAL_BLOATING, ABDOMINAL_CT, ABNORMAL_LYMPH_NODES, ABNORMAL_SKIN_GROWTHS, ABSOLUTE_NEUTROPHIL_COUNT_ANC, ADENOCARCINOMAS, ADOLESCENT_AND_YOUNG_ADULT, ADO_TRASTUZUMAB_EMTANSINE, ADRENOCORTICAL_ADENOMA, ADRENOCORTICAL_CANCER`

## Linkage density

- LR mean chunks/entity: 2.204
- EQ mean chunks/entity: 2.397
- EQ zero-chunk entities: 0

## Relation linkage (049 B6)

- EQ mean chunks/edge: **1.151**
- LR mean chunks/relation: **1.17**
- EQ edges ≥2 chunks: **446** (rate **0.1135**)
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

Artifact: `/Users/raphaelmansuy/Github/03-working/edgequake/specs/001-benchmark/e2e/artifacts/ingest-audit/20260723T020853Z/audit_report.json`
