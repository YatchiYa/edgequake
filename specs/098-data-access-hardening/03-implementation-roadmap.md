# 03 — Implementation Roadmap (SPEC-098)

## Waves

| Wave | DoD |
|------|-----|
| **W0** | Spec folder complete; findings locked |
| **W1** | Saturated KEEP still ensures relational spine; AGE skip preserved; stats |
| **W2** | Relation-type SSOT; `MirrorLegacyReport`; typed fail on partial/invalid workspace |
| **W3** | Migration 139 marker + support reconcile; checksums.lock; PG16/17/18 portable |
| **W4** | contract + e2e wired; PG matrix smoke where available |
| **W5** | All gates green; checksum script green |
| **W6** | Migration 140 + single-arbiter reconcile every boot; relationship historical spine; trigger prefer-column |
| **W7** | Native upsert harden (`DISTINCT ON` + `eq_merge`); Cypher MERGE rel_type; relationship sink batch dedupe |
| **W8** | Cardinality / reconcile / perf / sink-dedupe e2e; CI wired; operator doc |

## Exit criteria

- [x] Spec docs under `specs/098-data-access-hardening/`
- [x] Saturated AGE entity without prior `entities` row completes typed persist (spine ensure + e2e)
- [x] Mixed-case relation type mirrors to `relationship_embeddings`
- [x] Typed mirror error lists sample miss ids when incomplete
- [x] Migration 139 applies on PG16/17/18 without version-branched DDL
- [x] Duplicate `(src,tgt,rel)` + multigraph edge batch never raises SQLSTATE 21000
- [x] Legacy EDGE UNIQUEs dropped on every boot when 3-col arbiter exists
- [x] Native DO UPDATE uses `eq_merge_graph_properties`
- [x] Relationship sink survives duplicate arbiter keys in one batch
- [x] Migration 140 + support reconcile + checksums.lock
