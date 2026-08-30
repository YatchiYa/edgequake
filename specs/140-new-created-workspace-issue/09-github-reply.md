# 09 — GitHub reply (#388)

> Post this body with `gh issue comment 388`. Operator copy; code is law.

---

## Diagnosis (SPEC-140)

Thank you for the DB verification — that rules out “the insert failed.”

The catalog is **not** membership-scoped. `GET /api/v1/tenants/{tenant_id}/workspaces` is:

```text
  SELECT … FROM workspaces
   WHERE tenant_id = $1
   ORDER BY created_at DESC
```

No join to `memberships`. Create-workspace does not insert a membership row.
“Assigned and active” is correct ops language; the UI list never used assignment.

On 0.24.2 **and** current HEAD the list handler then does:

- default **`limit=20`**
- `total = len(page)` (so `total` is never > 20)
- the WebUI **does not send `limit`** and **does not request the next page**

So any tenant with more than 20 workspaces silently drops older rows. The
header chip also shows **only the selected** workspace; auto-select is newest
first (`created_at DESC`) — after creating `g99-73` the chip is expected to
read `g99-73` even when 71/72 are in the popover.

If `g99-71` / `g99-72` / `g99-73` are **three organizations** (one workspace
each), the Workspaces group shows **one** row for the selected org. Check with
the SQL below.

### Workaround (until upgrade)

```http
GET /api/v1/tenants/{tenant_id}/workspaces?limit=100
GET /api/v1/tenants?limit=100
```

Open the header **popover** (not only the chip). Confirm you are on the
tenant that owns the three rows.

### Diagnostic SQL

```sql
SELECT workspace_id, tenant_id, name, slug, is_active, created_at
FROM workspaces
WHERE name IN ('g99-71','g99-72','g99-73')
   OR slug IN ('g99-71','g99-72','g99-73')
ORDER BY created_at;

SELECT tenant_id, name, slug
FROM tenants
WHERE name IN ('g99-71','g99-72','g99-73')
   OR slug IN ('g99-71','g99-72','g99-73');

-- Compare to the UI call:
-- GET /api/v1/tenants/{selected_tenant_id}/workspaces
SELECT COUNT(*) FROM workspaces WHERE tenant_id = '<selected_tenant_id>';
```

- **Same `tenant_id`, COUNT > 20:** Track A (silent cap) — `?limit=100` should
  show 71 and 72 if they are among the newest 100.
- **Three `tenant_id`s:** Track C — switch Organization in the popover.
- **COUNT ≤ 20 and same tenant but popover still missing names:** tell us the
  JSON `items` length from that GET.

### Fix in progress

SPEC-140: honest `total` = `COUNT(*)`, SQL pagination, WebUI exhausts pages,
popover lists every workspace of the selected tenant, cmdk keyed by UUID.
We are **not** changing the list to membership-only (that would hide
unassigned workspaces).

Spec: `specs/140-new-created-workspace-issue/`
