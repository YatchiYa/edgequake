# SPEC-103 — Edge Cases

| ID | Case | Behavior |
|----|------|----------|
| EC-01 | Vision / images present | Skip answer cache (existing 064) |
| EC-02 | Empty context | Skip answer cache — no poison empties |
| EC-03 | Stream mid-flight | Do not cache partial; write after complete |
| EC-04 | Cache store error | Warn + recompute (LAW-C5) |
| EC-05 | Model change | Different hash → miss |
| EC-06 | Acc / publication | `EDGEQUAKE_LLM_CACHE=0` on Acc backend |
| EC-07 | Namespace isolation | Different storage namespace → miss |
| EC-08 | Graph ingest after warm answer | Context in prompt → new key → miss (EQ safer than LR) |
