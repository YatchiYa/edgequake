# Ingest parity audit (028 B1)

**UTC:** 20260723T012740Z  
**EQ workspace:** `8e990410-43b5-44f4-9f56-87bd154570ce`  
**LR stage:** `smoke`  
**EQ graph:** `eq_eq_default_graph`  

## Counts

| Side | Entities/nodes | Edges / rels | Linked chunks |
|------|----------------|--------------|---------------|
| LR | 3580 | 5325 | 199 |
| EQ | 4543 | 8449 | 188 |

## Identity parity (032 B3b)

- EQ entity vectors: **4218**
- EQ AGE nodes (WS filter): **4543**
- AGE/vectors ratio: **1.0771** (target ≈ 1.0)
- B3b pass when age_over_vectors ≈ 1.0 (±0.10); pre-fix Acc WS often ~0.09 (global node_id collision).

## Entity name overlap (normalized)

- Jaccard: **0.4347**
- EQ coverage of LR: **0.724**
- LR coverage of EQ: **0.521**
- EQ soft-overlap (substring ≥6): **0.7465** (3371/4543)
- Only LR (sample): `2022, 2D_DIGITAL_MAMMOGRAPHY, 2D_DIGITAL_MAMMOGRAPHY_3D_MAMMOGRAM, 5_FLUOROURACIL, ABDOMINAL_CT, ABEMACICLIB, ABNORMAL_LYMPH_NODES, ABNORMAL_SKIN_GROWTHS, ABSOLUTE_NEUTROPHIL_COUNT, ACUTE_LYMPHOBLASTIC_LEUKEMIA, ACUTE_PROMYELOCYTIC_LEUKEMIA, ADOLESCENT_AND_YOUNG_ADULT`

## Linkage density

- LR mean chunks/entity: 2.204
- EQ mean chunks/entity: 2.373
- EQ zero-chunk entities: 0

## Relation linkage (049 B6)

- EQ mean chunks/edge: **1.0**
- LR mean chunks/relation: **1.17**
- EQ edges ≥2 chunks: **0** (rate **0.0**)
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

Artifact: `/Users/raphaelmansuy/Github/03-working/edgequake/specs/001-benchmark/e2e/artifacts/ingest-audit/20260723T012739Z/audit_report.json`
