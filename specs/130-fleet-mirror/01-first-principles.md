# 01 — First Principles (LAW-130)

## Domain

Typed KG persist uses **two identity planes** for relationships:

```ascii
  Plane A — Relational spine (public.relationships)
            UUID primary key; endpoints = entity UUIDs;
            arbiter (tenant, workspace, source_id, target_id, relation_type)

  Plane B — Legacy vector id (fleet mirror input)
            String "SRC->TGT:TYPE" (bare names + uppercase type)
```

Plane B is a **display/transport key**. Plane A is the **FK identity**. Mirror must never invent Plane A from Plane B when the sink already produced Plane A in the same session.

## Laws

| ID | Law | Rationale |
|----|-----|-----------|
| LAW-130-1 | **Identity produced once** — `PostgresEntitySink` (or equivalent) is the SSOT that creates/returns `relationships.id` for in-session rows | First principles / SOLID-SRP |
| LAW-130-2 | **In-session mirror consumes UUIDs** — RelVectors fleet write uses the sink map; no name re-resolve for those rows | Eliminate divergent SSOT |
| LAW-130-3 | **Typed order RelGraph → RelVectors** — await barrier already required; keep as invariant | SPEC-091 / SPEC-098 |
| LAW-130-4 | **Timing is not the root** — entity↔rel `created_at` skew is expected; identical retries + non-compensated SQL spine falsify pure SELECT-before-INSERT as the dominant mode | Honest diagnosis |
| LAW-130-5 | **Offline resolve stays name-based** — migration/coverage/backfill may still call `resolve_relationship_id_pool` | Do not break tools |
| LAW-130-6 | **Relation type uppercase SSOT** — vector id, sink, and any residual resolve share `normalize_relation_type_str` | LAW-098-3 |
| LAW-130-7 | **Fail-closed stays** — typed `resolved < eligible` still fails; hints must name relationship identity correctly | LAW-098-4 + operator trust |
| LAW-130-8 | **No sleep-as-fix** — bounded retry in resolve is optional hygiene, not the primary fix | Avoid papering over identity bugs |
| LAW-130-9 | **Compensation honesty** — merge compensation does not delete SQL spine; leftover edges after fail are expected | Explains “edge exists after fail” |
| LAW-130-10 | **Unfakable proof** — order contract + duplicate-name miss→fixed e2e + happy-path UUID mirror | Honest acceptance |

## Projection (identity flow)

```ascii
  ExtractedRelationship
       │
       ├─► RelGraph: AGE edges + sink INSERT … RETURNING id
       │              legacy_key "SRC->TGT:TYPE" → Uuid   (LAW-130-1)
       │
       └─► RelVectors: mirror(legacy_key, embedding) via Uuid map
                      (LAW-130-2); fail-closed if map miss (LAW-130-7)
```

## Cross-refs

- Why: [00-why.md](00-why.md)
- Architecture: [04-target-architecture.md](04-target-architecture.md)
- Parents: [../098-data-access-hardening/](../098-data-access-hardening/), [../091-simplify-data-layer/](../091-simplify-data-layer/)
