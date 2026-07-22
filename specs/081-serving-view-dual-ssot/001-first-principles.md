# SPEC-081 — First principles (serving view ≠ RAG SSOT)

EdgeQuake has a **dual SSOT**: relational `documents`/`chunks` for ownership/lineage, and `eq_*_vectors` + KV + AGE for retrieval. Treating `chunks.embedding` alone as the corpus causes drift.

C5 narrows dual-SSOT for **admin/debug**:

1. `eq_serving_chunk_presence(workspace_id)` — chunks in the relational spine + whether `embedding_id` is set  
2. `eq_serving_vector_presence(workspace_id, vectors_regclass)` — optional LEFT JOIN to a namespace vectors table  

Use for retract audits and “allowed + embedded?” checks. Do **not** route ANN queries through these functions. Do **not** silently unify stores.
