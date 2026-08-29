# 03 — Root cause (code is law)

> **Cross-refs**: [Laws](01-first-principles.md) · [Fix](05-fix-plan.md)

## Track I — Injections

`GET /workspaces/{id}/injections` defaults `limit=50` and returns honest
`total` / `has_more`. `listInjections` sends no query. The knowledge grid
renders `data.items` only. The 51st name never appears.

## Track Q — Admin quotas

`AdminQuotaSection` calls `apiClient("/tenants?limit=100")` and uses
`items`. `getTenants()` already exhausts. Duplicate client = silent 101st org.

## Track C — Conversations

`useConversations` sends `cursor` and stops when `!next_cursor`.
`ConversationServiceImpl::list_conversations` and in-memory both pass
`offset=0` and set `next_cursor: None`. `has_more = total > items.len()` is
true on a full last page when `total > limit` even if no next page — but
here `next_cursor` is always `None`, so `getNextPageParam` is always
undefined. History stuck at 20.

## Track D — Documents

`useDocumentsInventory` hardcodes `currentPage: 1`. Title search filters
that page in memory. API already has `document_pattern`.
`PaginationControls` exists and is unused.

## Track P — cancelPipeline

`getTasksList` default page size 20. Cancel loops that page only.

## Track M — MCP / SDK

REST is honest after SPEC-140. TypeScript SDK `list()` / `listWorkspaces()`
unwrap `.items` with no `limit`/`offset`. MCP `workspace_list` inherits the
silent 20.
