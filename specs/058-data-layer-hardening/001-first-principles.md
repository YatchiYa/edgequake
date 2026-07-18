# SPEC-058 — First principles

1. **Same Postgres ≠ one transaction.** KV, pgvector, and AGE commits are independent. Integrity requires explicit saga rules: what compensate may delete, and retract on cancel.

2. **Compensate must not delete shared updates.** Entity embeddings are keyed by normalized name. Rolling back an update for doc B must not wipe doc A's embedding.

3. **Retrieval must respect retract.** Cancelled documents must leave the ANN/graph retrieval set (`retract_document_indexes`).

4. **Generated columns only work if they see the SSOT.** Chunk text lives in KV (`content_ref`). Writable `content_tsv` populated at upsert + `NULLIF` empty for legacy rows.

5. **Isolation belongs in SQL on the table that stores the property.** EDGE expand must filter `tenant_id` / `workspace_id` in the incident-edge query.

6. **Never silent DROP on config change.** Dimension mismatch fails closed unless the operator opts in.

7. **HNSW defaults follow pgvector 0.8.** `ef_construction` default 64; filtered ANN already uses `iterative_scan` when supported. halfvec remains opt-in (M080).
