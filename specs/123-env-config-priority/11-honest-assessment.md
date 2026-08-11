# 11 — Honest Assessment (SPEC-123) — re-assess after residual close

> Updated after closing sync-tenant + FE LLM/embedding Resolves-to gaps.

## Confidence legend

| Level | Meaning |
|-------|---------|
| **High** | Code path + tests green this session |
| **Medium** | Implemented; residual dual-path or thin UI surface |
| **Low** | Documented / partner not yet confirmed |

---

## What closed (this pass)

| Gap | Fix | Evidence |
|-----|-----|----------|
| Sync query/chat path dropped tenant | `resolve_llm_provider_with_workspace(..., tenant)` + async `resolve_llm_provider_for_workspace` loads tenant; all query/chat/MCP/context callers await it | `providers/resolver.rs`, `query_context.rs`, handlers |
| Case 2 ignored SSOT when no query role | Falls through to `resolve_llm_choice` with tenant | same |
| FE LLM/embedding cards dishonest | `WorkspaceModelConfigGrid` uses `effective*FromWorkspace` + Resolves-to badges | `workspace-model-config-grid.tsx` |
| Provenance e2e only checked vision | GET asserts tenant source for LLM + embedding after clear | `e2e_spec123_parser_priority` |
| Sync tenant unit missing | `test_resolve_tenant_via_for_workspace_when_no_ws_override` | resolver integration tests |

## Scorecard (re-assessed)

| Domain | Law compliance | Confidence |
|--------|----------------|------------|
| PDF parser | Strong | High |
| Vision LLM (upload + GET + tenant update + VLM try_*) | Strong | High |
| LLM (async + sync query/chat/MCP) | Strong | High |
| Embedding (pipeline + resolver + GET) | Strong | High |
| FE honesty (PDF + vision + LLM + embedding cards) | Strong | High |
| Spec / first principles | Strong | High |
| Acceptance hygiene | Honest (PDF Done / Models Done / Partner Open) | High |

**Overall:** Models slice is **Done** for the cascade contract (resolve = run, SSOT, tenant wired, GET provenance, FE Resolves-to). Partner operator repro remains Open. Painted concrete DTO fields remain for backward compat, mitigated by `resolved_*` + `*_resolution_source`.

## Residual (honest, smaller)

1. **Painted `llm_*` / `embedding_*` on GET** for backward compat — UI must prefer `resolved_*` (now wired). Ideal later: stop paint entirely.
2. **Partner operator repro** from `10-reproduction.md` still Open.
3. **`config_resolution` explainability** (server settings card) remains a parallel env/server_config story — orthogonal to runtime LAW-123-2.
4. Acc **Extract/Keyword** stay env-first (intentional exception, not a gap).

## E2E status (verified this session)

```
cargo test -p edgequake-api --test e2e_spec123_parser_priority
→ 8 passed (PDF + models + GET tenant LLM/embedding provenance)

cargo test -p edgequake-api --lib providers::resolver::tests
→ 10 passed (incl. tenant via for_workspace)

cargo test -p edgequake-api --test e2e_spec026_llm_roles
→ 3 passed

bun test src/lib/config/__tests__/resolve-model-choice.test.ts
→ 8 passed (incl. effective*FromWorkspace)
```

## Cross-refs

- Laws: [01-first-principles.md](01-first-principles.md)
- Acceptance: [09-acceptance.md](09-acceptance.md)
- Tests: [08-test-protocol.md](08-test-protocol.md)
