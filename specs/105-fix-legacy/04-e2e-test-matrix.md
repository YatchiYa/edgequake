# 04 — E2E Test Matrix

| ID | Assert | Location |
|----|--------|----------|
| E2E-105-01 | Unknown VECTOR_BACKEND → Typed | storage `vector_backend` tests |
| E2E-105-02 | Refuse legacy when census 0; allow when census>0 | cutover_flag_guard + contract_spec105 |
| E2E-105-03 | INV-03 dual iff KV table present | contract_spec104 + spec105 |
| E2E-105-04 | 142 SQL: abort on rows / drop empty | migration + PG contract |
| E2E-105-05 | Health chunk_text_ssot=relational | e2e_spec024 fix |
| E2E-105-07 | Defer 142 while residue; ≤0.22 soft-exit | migration_bootstrap + live migrate |
| E2E-105-08 | ≤0.22 soak adjacency | `make spec091-upgrade-soak` |
