# 05 — Edge cases (SPEC-114)

| ID | Case | Expected | Mitigation / Test |
|----|------|----------|-------------------|
| EC-114-01 | Empty/absent `relation_types` | Free-form relations (no prompt section, no enforce) | G-114-03, G-114-09, G-114-15, G-114-16, G-114-17/19 soft |
| EC-114-02 | Strict on + unknown relation | Remap to `RELATED_TO` if in list else first type | G-114-03, G-114-15, G-114-16, G-114-17/19 soft closed-world |
| EC-114-03 | Strict off + unknown relation | Normalized pass-through | G-114-03, G-114-15, G-114-16, G-114-17/19 soft permissive |
| EC-114-04 | >50 relation types on write | Server caps at 50; UI blocks add | G-114-02, G-114-06 |
| EC-114-05 | Duplicate / mixed case / hyphen / slash | UPPER_SNAKE dedupe; slash normalized in pipeline | G-114-01 |
| EC-114-06 | Language change on entity preset | Entity list remaps; relations + custom entities stay | G-114-07 + spec096 |
| EC-114-07 | Preset switch with dirty custom lists | Replace both lists; set `kg_schema_preset`; confirm if needed | G-114-06 |
| EC-114-08 | Lists diverge from all presets | `kg_schema_preset=custom` or omit | G-114-04 |
| EC-114-09 | PUT schema without rebuild | Old AGE labels unchanged; review shows rebuild hint | G-114-07, G-114-18 |
| EC-114-10 | Create omits relation fields | Free-form default | G-114-04 |
| EC-114-11 | Color picker | Entity-only (SPEC-102); relations have no colors | G-114-10 |
| EC-114-12 | Strict key sparse encoding | `true` removes key; `false` stores false (mirror entity) | G-114-02 |
| EC-114-13 | Relation list non-empty + strict absent | Treat as strict=true | G-114-03 |
| EC-114-14 | Bulk edit relations replaces list | Same as entity bulk | G-114-06 |
| EC-114-15 | Observed graph labels vs config | Document stats ≠ workspace config; UI labels clearly | docs + card copy |
| EC-114-16 | Edge refs unknown entity/relation | Dropped on normalize when allow-lists present | G-114-11, G-114-15 |
| EC-114-17 | Duplicate triple | Dedupe on normalize | G-114-11 |
| EC-114-18 | Empty edges + non-empty relations | v1 label-only behavior (unconstrained endpoints) | G-114-12, G-114-15, G-114-16, G-114-17/19 soft |
| EC-114-19 | >100 edges | Cap at 100 | G-114-11 |
| EC-114-20 | Remove entity type chip | Drop edges using it as source/target | G-114-13, G-114-18 |
| EC-114-21 | Remove relation chip | Drop edges using that label | G-114-13, G-114-18 |
| EC-114-22 | Language remap entity types | Remap edge endpoints via catalog | G-114-13 + spec096 |
| EC-114-23 | Manual edge edit after preset | `kg_schema_preset=custom` | G-114-13 |

## ASCII — empty vs configured

```ascii
relation_types absent/[]          relation_types = [WORKS_AT, PART_OF]
        │                                    │
        ▼                                    ▼
  free-form LLM labels              prompt GUIDANCE/STRICT section
  no enforce_relation_type          enforce on parse/gleaning
```
