# 07 — Implementation plan

Single policy module: `edgequake-storage/src/adapters/postgres/fleet_legacy_absorb.rs` (no second absorb path).

1. Stamp UPDATE: `NOT EXISTS` other row in the **batch workspace** with the same lid (unnest `workspace_id`).
2. Catch SQLSTATE 23505 when constraint/message contains `legacy_vector_id`; treat as absorb (`Ok(0)`).
3. Count those skips in `absorbed_legacy_collisions` (INSERT-miss **and** NULL-lid PK stamp-skip). Entity / relationship / report share `EmbeddingFamily` SQL.

Out of scope: SPEC-083 alias merge, HTTP worker soak, SPEC-129, #383 compensate DELETE, embedding dimension mismatch.
