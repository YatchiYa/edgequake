# 08 — Similar issues

> **Cross-refs**: [RCA](03-root-cause.md)

| Item | Relation |
|------|----------|
| [SPEC-101 EC-101-15](../101-wizard-mode-tenant-workspace/05-edge-cases.md) | Workspace missing after **tenant switch** — reload that tenant’s list, do not keep previous org’s rows |
| [SPEC-101 LAW-101-11](../101-wizard-mode-tenant-workspace/) | Chip is one pair; not a bug that chip shows only g99-73 |
| Admin quota UI | Already calls `/tenants?limit=100` — proves the default-20 trap was known for orgs |
| [SPEC-104 issue-01](../104-fix-datalayer/03-issue-01-workspaces-pk.md) | Inspector used `workspaces.id`; catalog PK is `workspace_id`. List DTO already maps `id`. Not the list miss |
| [#316](https://github.com/raphaelmansuy/edgequake/issues/316) | Ingest fairness across workspaces — not catalog listing |
| `e2e_workspace_include_stats` | Same GET path; does not assert `total` vs COUNT |

No in-repo mention of #388 before this pack.
