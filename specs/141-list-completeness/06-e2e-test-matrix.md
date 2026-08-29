# 06 — E2E test matrix

> **Cross-refs**: [Fix](05-fix-plan.md)

Tests must fail if the first page is still treated as “the list”.

| ID | Layer | Assert |
|----|--------|--------|
| HTTP-141-01 | 25 workspaces | `?limit=10&offset=20` nonempty; `total` unchanged |
| HTTP-141-02 | 51 injections | default GET `items.len()==50`, `total>=51`; `?limit=200` all names |
| HTTP-141-03 | 25 conversations | page 1 `len==20`, `has_more`, `next_cursor` set; second GET with cursor returns the rest |
| Playwright-inj | knowledge grid | 51st name visible |
| Playwright-docs | documents | `total>page_size`; next page enabled; a name only on page 2 reachable |
| Playwright-history | chat history | 21st conversation after scroll/load-more |
| Admin quotas | 101st tenant | quota row present (or skip if not admin / skipUnlessLiveStack) |
| MCP | `workspace_list` | 21 uniquely created names in tool JSON |
| Vitest | `fetchAllPagesByIndex`; injection exhaust | unit |

Wire live-stack skip like SPEC-140. HTTP conversations/injections use
`AppState::test_state()`.
