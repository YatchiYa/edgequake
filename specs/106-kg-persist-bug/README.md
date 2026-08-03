# SPEC-106 — KG Persist graphid Operator Bug (#356)

> **Source:** [GitHub #356](https://github.com/raphaelmansuy/edgequake/issues/356)  
> **Broken through:** **v0.24.0**  
> **Shipped in:** **v0.24.1**  
> **Fix:** LAW-G1 `::text` joins in `pg_get_edges_for_nodes_batch`

## Status board

| Item | Status | Doc |
|------|--------|-----|
| RCA | Closed | [03](03-root-cause.md) |
| Code fix | Closed | [04](04-fix-plan.md) |
| Similar-site audit | Closed (one open site; now fixed) | [07](07-similar-issues.md) |
| E2E-106-01..03 | Closed + CI | [05](05-e2e-test-matrix.md) |
| Product cut | **v0.24.1** | CHANGELOG / GHCR |

## Document map

```ascii
 00-why / 00-issue-data → 01 laws → 02 matrix → 03 RCA
                       → 04 fix → 05 e2e → 06 edges → 07 similar
                       → measurements/
```

## Cross-spec anchors

| Spec / issue | Relevance |
|--------------|-----------|
| [#214](https://github.com/raphaelmansuy/edgequake/issues/214) | Same operator error; degrees path fixed; this batch path left behind |
| M072 | `(start_id::text)` / `(end_id::text)` indexes for LAW-G1 joins |
| [SPEC-104](../104-fix-datalayer/) | `graphid_ops` missing amplifies raw graphid compares |
| CHANGELOG 0.12.1 | Documented AGE has no `graphid =` |
