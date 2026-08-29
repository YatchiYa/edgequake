# 00 — Issue data

> **Cross-refs**: [WHY](00-why.md) · [GitHub reply](09-github-reply.md)

| Field | Value |
|-------|--------|
| GitHub | [#388](https://github.com/raphaelmansuy/edgequake/issues/388) |
| Title | Newly Created Workspaces Not Displayed in Frontend |
| Author | `@ankursingh-devops` |
| Opened | 2026-08-25 |
| Label | `bug` |
| Product version (field) | **0.24.2** |
| Deployment | Docker (containerized) |
| Database | PostgreSQL |
| API version (field) | v0.24.2 |
| HEAD when studied | **v0.26.3** (workspace/memberships DDL unchanged since 001/008) |

## Reported names

| Name | Visible in UI (report) | In DB | Active |
|------|-------------------------|-------|--------|
| g99-71 | No | Yes | true |
| g99-72 | No | Yes | true |
| g99-73 | Yes | Yes | true |

## Reporter steps

1. Create workspaces g99-71, g99-72, g99-73.
2. Assign them to users.
3. Open the frontend workspace list.
4. Observe only g99-73.

## Expected (reporter)

All **assigned and active** workspaces should display.

## What the code actually lists

```text
  GET /api/v1/tenants/{tenant_id}/workspaces
        │
        └─ SELECT * FROM workspaces WHERE tenant_id = $1
           ORDER BY created_at DESC
           then skip/take (default 20)
           NO join to memberships
           NO is_active filter
```

“Assigned” is `memberships`. It does not feed this endpoint.
There is **no** HTTP handler to assign a user to a workspace; assignment in
the field was SQL / out-of-band.

## Diagnostic SQL (operator)

See [10-ops-workaround.md](10-ops-workaround.md). Distinguishes:

- three rows, **same** `tenant_id` → Track A/B (pagination / chip)
- three rows, **three** `tenant_id`s → Track C (org vs workspace)
