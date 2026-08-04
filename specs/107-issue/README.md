# SPEC-107 — Partner Prod Error Report (First-Principles Analysis)

> **Source:** Partner thread (Quantalogic prod logs, image `0.22.0`) — [00-issue-data.md](00-issue-data.md)  
> **Engineering SSOT:** [SPEC-104](../104-fix-datalayer/) (RCA + code fixes)  
> **Shipped:** naming / idempotency / INV-03 dual-read in **≥ v0.24.0** (pin **v0.24.1**)  
> **This pack:** partner-facing answer, residual INV-03 ops, e2e absence proofs

## Partner question

> Est-ce que ce sont des erreurs que tu constates toi aussi de ton côté ?

**Oui.** Reproduced against `ghcr.io/raphaelmansuy/edgequake:0.22.0` (same SQLSTATE classes, same hourly cadence). Three of four are **monitor/write bugs** closed in SPEC-104; INV-03 is a **true integrity alarm** that still needs ops cleanup after upgrade.

## Status board

| ID | Crit | Symptom | Law | Code (≥0.24.0) | Residual | Doc |
|----|------|---------|-----|----------------|----------|-----|
| E1 | Critical | `workspaces.id` 42703 (~2300/24h) | LAW-I1 | **Closed** | Upgrade | [03](03-root-cause.md) |
| E2 | High | `edgequake."Node"` 42P01 (24/24h) | LAW-I1 | **Closed** | Upgrade | [03](03-root-cause.md) |
| E3 | High | INV-03 CRITICAL (20 sample) | LAW-I2 | Monitor **Closed** | **Ops: requeue/delete orphans** | [04](04-residual-ops.md) |
| E4 | Medium | `tenants_slug_key` 23505 (6/24h) | LAW-I3 | **Closed** | Upgrade | [03](03-root-cause.md) |

## Document map

```ascii
 00-why / 00-issue-data → 01 laws → 02 matrix → 03 RCA
                       → 04 residual ops → 05 e2e → 06 partner reply
                       → 07 residual risks → 08 R2 node-count 57014
                       → measurements/
```

## Cross-spec anchors

| Spec / doc | Relevance |
|------------|-----------|
| [SPEC-104](../104-fix-datalayer/) | Full RCA + A+ harden; do not fork |
| [SPEC-021](../021-storage-study/) | StorageInspector / INV-03 origin |
| [SPEC-089](../089-health-check/) | Node-count bounds / LAW-H* |
| [SPEC-106](../106-kg-persist-bug/) | Unrelated KG persist `graphid` (v0.24.1) |
| [migrate-to-0.23](../../docs/operations/migrate-to-0.23.md) | Schema path for upgrade |
| [CHANGELOG.md](../../CHANGELOG.md) (0.24.0 / 0.24.1) | SPEC-104 / SPEC-106 ship notes |
| [07-residual-risks.md](07-residual-risks.md) | Post-assessment open risks |
| [08-r2-node-count-57014.md](08-r2-node-count-57014.md) | R2 57014 first principles + INV-C batch |

## DRY rule

Deep remediation text lives in SPEC-104. SPEC-107 **cross-refs** and adds partner reply + INV-03 ops + E2E-107 absence gates. If SPEC-104 and SPEC-107 disagree, **SPEC-104 wins** for engineering truth.
