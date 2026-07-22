# Ingest parity audit (028 B1)

**UTC:** 20260721T010845Z  
**EQ workspace:** `dcdffc3e-19a3-44a3-81de-90817883ee80`  
**LR stage:** `smoke`  
**EQ graph:** `eq_eq_default_graph`  

## Counts

| Side | Entities/nodes | Edges / rels | Linked chunks |
|------|----------------|--------------|---------------|
| LR | 3580 | 5325 | 199 |
| EQ | 3950 | 3927 | 188 |

## Identity parity (032 B3b)

- EQ entity vectors: **3950**
- EQ AGE nodes (WS filter): **3950**
- AGE/vectors ratio: **1.0** (target ≈ 1.0)
- B3b pass when age_over_vectors ≈ 1.0 (±0.10); pre-fix Acc WS often ~0.09 (global node_id collision).

## Entity name overlap (normalized)

- Jaccard: **0.4504**
- EQ coverage of LR: **0.6858**
- LR coverage of EQ: **0.5675**
- EQ soft-overlap (substring ≥6): **0.7711** (3029/3950)
- Only LR (sample): `2022, 2D_DIGITAL_MAMMOGRAPHY_3D_MAMMOGRAM, 5_FLUOROURACIL, ABDOMINAL_CT, ABLATIVE_THERAPIES, ABNORMAL_LYMPH_NODES, ABSOLUTE_NEUTROPHIL_COUNT, ADOLESCENT_AND_YOUNG_ADULT, ADO_TRASTUZUMAB_EMTANSINE, ADRENAL_CORTEX, ADRENAL_MEDULLA, ADRENAL_TUMOR`

## Linkage density

- LR mean chunks/entity: 2.204
- EQ mean chunks/entity: 2.416
- EQ zero-chunk entities: 0

## Relation linkage (049 B6)

- EQ mean chunks/edge: **1.141**
- LR mean chunks/relation: **1.17**
- EQ edges ≥2 chunks: **432** (rate **0.11**)
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

Artifact: `/Users/raphaelmansuy/Github/03-working/edgequake/specs/001-benchmark/e2e/artifacts/ingest-audit/20260721T010844Z/audit_report.json`
