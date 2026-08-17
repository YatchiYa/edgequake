# 05 — Edge Cases

| EC | Case | Handling |
|----|------|----------|
| EC-01 | Mid-upgrade KV present | Dual INV/FTS; legacy_tables allowed |
| EC-02 | Pending 125 with rows | 142 not reached until confirm-drop train advances |
| EC-03 | Empty leftover tables post-drop | 142 DROP IF EXISTS |
| EC-04 | Unknown env | TypedEmbeddings |
| EC-05 | Fresh DB zero vectors | Refuse legacy_tables (census 0) |
