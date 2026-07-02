# SPEC-040 — GitHub Issues Cross-Reference (Issues #250–#253, #259, #262)

**Spec:** `040-edgequake-issues`  
**Date:** 2026-07-02  
**Status:** `CLOSED` — implemented in **v0.13.2**  
**Method:** Code is law — every claim maps to a file, migration, or E2E proof path

---

## Scope

| Issue | Title | Severity | Fix state (v0.13.2) |
| ----- | ----- | -------- | ------------------- |
| [#262](https://github.com/raphaelmansuy/edgequake/issues/262) | Graph stream / workspace stats 15s timeout | P0 perf | **Closed** — M078 + `graph_lifecycle.rs` + concurrent ops script |
| [#259](https://github.com/raphaelmansuy/edgequake/issues/259) | FK error on `messages.conversation_id` | P1 data | **Closed** — guard + UI workspace reset + `CONVERSATION_GONE` |
| [#253](https://github.com/raphaelmansuy/edgequake/issues/253) | Duplicate upload ghost hash loop | P1 UX | **Closed** — `workspace_content_hash_dedup.rs` |
| [#251](https://github.com/raphaelmansuy/edgequake/issues/251) | `models.toml` runtime override ignored | P2 ops | **Closed** — runtime-first loader |
| [#250](https://github.com/raphaelmansuy/edgequake/issues/250) | UI footer version ≠ API version | P3 trust | **Closed** — Docker `NEXT_PUBLIC_APP_VERSION` + release gate |

---

## Release

See [010-release-runbook.md](./010-release-runbook.md) for tag `v0.13.2` cut procedure.

---

## Documents

| File | Lens |
| ---- | ---- |
| [001-five-whys.md](./001-five-whys.md) | 5 WHY |
| [002-first-principles.md](./002-first-principles.md) | First principles |
| [003-code-is-law.md](./003-code-is-law.md) | Code is law |
| [004-product-owner-lens.md](./004-product-owner-lens.md) | Product owner |
| [005-fullstack-developer-lens.md](./005-fullstack-developer-lens.md) | Full stack |
| [006-postgres-age-pgvector-lens.md](./006-postgres-age-pgvector-lens.md) | AGE / pgvector |
| [007-on-complexity-lens.md](./007-on-complexity-lens.md) | O(N) expert |
| [008-implementation-plan.md](./008-implementation-plan.md) | Battle-tested fix plan |
| [009-cross-reference-matrix.md](./009-cross-reference-matrix.md) | Cross-ref matrix |
| [010-release-runbook.md](./010-release-runbook.md) | Release cycle |

---

## E2E proof commands

```bash
cargo test -p edgequake-storage --features postgres --test graph_sota_tests
cargo test -p edgequake-api --features postgres --test workspace_document_scope
PLAYWRIGHT_BASE_URL=http://localhost:3000 pnpm exec playwright test \
  e2e/spec040-workspace-switch-conversation.spec.ts \
  e2e/stale-conversation-recovery.spec.ts
./scripts/release_gates.sh
```
