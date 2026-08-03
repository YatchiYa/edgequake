# SPEC-104 — Production Data-Layer RCA + V23 Hardening

> **Source incident:** [00-issue-data.md](00-issue-data.md) (Steven JAMAN, image `0.22.0`).  
> **Harden / A+:** [14-harden-notes.md](14-harden-notes.md) · **Assessment v3:** [13-fix-assessment.md](13-fix-assessment.md).  
> **Verdict:** #1–#4 meet A+ (domain Conflict, safe SQL idents, dual INV-03, naming SSOT). #5 timeout under load remains capacity (SPEC-089), not a naming bug.

## Status board

| # | Crit | Issue | After A+ | Doc |
|---|------|-------|----------|-----|
| 1 | Critical | `workspaces.id` 42703 | **A+ Closed** | [03](03-issue-01-workspaces-pk.md) |
| 2 | High | `edgequake."Node"` 42P01 | **A+ Closed** (+ multi-graph GIN) | [04](04-issue-02-age-graph-name.md) |
| 3 | High | INV-03 without chunks | **A+ Closed** (chunks\|KV + safe idents) | [05](05-issue-03-inv03-chunk-drift.md) |
| 4 | Medium | `tenants_slug_key` 23505 | **A+ Closed** (atomic + service Conflict→409) | [06](06-issue-04-tenant-slug-race.md) |
| 5 | Medium | Node-counts 57014 | **Observable** (all-graph GIN); timeout OPS | [07](07-issue-05-node-counts-timeout.md) |

## Document map

```ascii
 00 → 01 laws → 02 matrix → 03..07 RCAs
              → 08 remediation → 09 edge cases → 10 e2e
              → 11 V22 repro → 12 release lessons
              → 13 assessment v3 → 14 harden + A+ notes
```

## Cross-spec anchors

| Spec | Relevance |
|------|-----------|
| [SPEC-021](../021-storage-study/) | StorageInspector origin |
| [SPEC-089](../089-health-check/) | Node-count bounds / timeout |
| [SPEC-091](../091-simplify-data-layer/) | Typed chunks / embeddings SSOT |
| [migrate-to-0.23](../../docs/operations/migrate-to-0.23.md) | Schema path unchanged (no new migs) |
