# 10 — Ops workaround

> **Cross-refs**: [Issue data](00-issue-data.md) · [GitHub reply](09-github-reply.md)

Until the binary with SPEC-140 is deployed:

## 1. List with an explicit page size

```bash
# Replace TENANT_ID. Max limit is 100.
curl -sS "$API/api/v1/tenants/$TENANT_ID/workspaces?limit=100" \
  -H "X-Tenant-ID: $TENANT_ID" | jq '{total, n:(.items|length), names:[.items[].name]}'

curl -sS "$API/api/v1/tenants?limit=100" | jq '{total, n:(.items|length)}'
```

If `total` equals `n` and both are 20 while SQL `COUNT(*)` is higher, you are
on a pre-140 API (lying total). Use `?offset=20&limit=100` to page blindly, or
upgrade.

## 2. Chip vs popover

The header chip is **current workspace only**. Open the selector popover.
Search box filters orgs **and** workspaces; clear it if only `g99-73` remains.

## 3. Same tenant vs three orgs

Run the SQL in [00-issue-data.md](00-issue-data.md). Three `tenant_id`s →
switch **Organization** first; the Workspaces group is per selected org.

## 4. Assignment

`memberships` do not change the list payload. Missing membership can 403
other routes when `EDGEQUAKE_STRICT_TENANT_BIND` is on; it does not subset
list items.
