# 05 — Edge Cases (SPEC-098)

| ID | Case | Handling |
|----|------|----------|
| EC-098-01 | Non-UUID `workspace_id` in vector metadata | Typed hard fail with invalid-workspace evidence |
| EC-098-02 | Empty vector batch | No-op; no error |
| EC-098-03 | All entities saturated KEEP | Spine ensure for all; AGE skip; fleet resolves |
| EC-098-04 | Mixed new + saturated | New → graph+sink; saturated → sink only; fleet resolves both |
| EC-098-05 | RLS / missing tenant GUC | Sink fail-closed already; mirror uses same pool — surface sink error first |
| EC-098-06 | Legacy scoped `entities.name` (`{uuid}::NAME`) | Existing tolerant OR lookup retained |
| EC-098-07 | Partial FK miss in batch | Fail closed (`resolved < eligible`) with sample miss ids |
| EC-098-08 | PG16 / PG17 / PG18 | Unified SQL; `NULLS NOT DISTINCT` + `ON CONFLICT`; capability probe only |
| EC-098-09 | Duplicate `(src,tgt,rel)` in one edge batch | Rust LWW + SQL `DISTINCT ON`; no SQLSTATE 21000 |
| EC-098-10 | Multigraph `KNOWS` + `WORKS_WITH` same endpoints | 3-col arbiter; both persist |
| EC-098-11 | Mixed-case `knows` / `KNOWS` in one batch | `normalize_relation_type_str` collapses before upsert |
| EC-098-12 | Legacy `idx_edge_source_target_unique` still present | Boot/mig 140 drops when 3-col exists |
| EC-098-13 | Duplicate relationship sink rows | Sink-side dedupe before UNNEST ON CONFLICT |
| EC-098-14 | Native writes off (Cypher) | MERGE includes `relation_type` (D-30) |
| EC-098-15 | Concurrent edge upserts same key | `eq_merge_graph_properties` unions `source_ids` |
