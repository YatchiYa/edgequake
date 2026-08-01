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
| **W9** | Migration 141 CHECK (`deleting`/`delete_failed`) + `admit_documents_deleting` for single+batch (KV+SQL) |
| **W10** | List merge treats `deleting` as lifecycle inflight; status counts/filters honest |
| **W11** | FE delete pin/session SSOT; batch poll absence-proof; e2e + Playwright + CI |
| **W12** | Shell lifecycle pass-through; batch failed[{id,reason}]; admit SQL soft; FE badge/header/Retry honesty |
| **W13** | Cascade shared prune uses Replace write-mode (not eq_merge union); post-proof green on AGE |

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
- [x] Mid single/batch delete: `GET /documents` shows `deleting` (not Completed/Ready)
- [x] Batch admit sets KV+SQL `deleting` for all planned ids
- [x] FE feedback + table agree; no “Document removed” while row still listed
- [x] Migration 141 + delete e2e/Playwright wired in CI
- [x] Shell upsert preserves `deleting` / `delete_failed` (not cancelled/failed)
- [x] Batch result carries per-id failure reasons; FE displays them
- [x] Feedback header / badge / Retry Failed match lifecycle (not pipeline Failed)
- [x] Shared-entity cascade prune persists on AGE (Replace mode); post-proof passes
