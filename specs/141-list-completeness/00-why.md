# 00 — Why

> **Cross-refs**: [Laws](01-first-principles.md) · [RCA](03-root-cause.md)

SPEC-140 proved that a catalog with `total = items.len()` after a silent
`limit` is data loss. That pack fixed tenant/workspace **selectors**.

The same class of bug remains wherever the UI looks like “the list of X”
but only the first page is consumed:

- Knowledge grid maps `items` from a default-50 API.
- Admin quotas fetch `?limit=100` and ignore `total`.
- Chat history infinite-query waits on `next_cursor`; the service hardcodes
  `offset=0` and `next_cursor: None`.
- Documents table can say “100 of 240” but never opens page 2.
- Cancel-all processing tasks sees 20 rows.
- MCP `workspace_list` unwraps SDK `.items` (REST default 20).

This pack audits first-party lists against three contracts and fixes the
**silent** ones. Labeled top-K (graph viz, typeahead, dashboard recent) stays.
