# 08 — E2E Test Matrix

> Tests must **fail** if a hallucinated page appears in href/link text or click
> lands on the wrong page.

| ID | Layer | Assert |
|----|-------|--------|
| U-142-01 | Rust unit | `[1]` + catalog page 4 → name + `p.4`, href `page=4`; `[99]` stripped; `page 999` not in href |
| U-142-02 | Rust unit | Fenced code `[1]` not rewritten; gold path unchanged |
| HTTP-142-01 | API mock LLM | `POST /query` answer href `page=4` matches fixture; `sources[].page_start==4` |
| HTTP-142-02 | Stream | Done verified answer ≡ sync rewrite |
| HTTP-142-03 | Chat persist | Reload conversation keeps same href |
| PW-142-01 | Playwright mock | Click inline link → `pdf-page-indicator` `data-page="4"` + chunk selected; not page 1 |
| PW-142-02 | Mapper | UUID chunk id + `document_id` → `/documents/{doc}` |
| PW-142-03 | Non-PDF | Name in link; no `?page=` |
| MCP-142 | MCP tool | Same markdown href as HTTP |
| Acc | Existing gold | Still no `[N]` / rewriter skipped |

## Harness rules

- Scripted / mock LLM only for page assertions — **no live model in CI**.
- Fixture PDF or mocked viewer with known `page_start=4`.
- Pattern: SPEC-135 mocked routes + `AppState::test_state()` where applicable.

## Cross-refs

- Plan: [07-implementation-plan.md](07-implementation-plan.md)
- Edge cases: [09-edge-cases.md](09-edge-cases.md)
