# 01 — Session changes (2026-10-16)

Every fix and improvement made in this session, grouped by area. Each item names the root cause, the fix, and the test that guards it. Nothing here is committed yet: it is the working tree of branch `feat/copilot-edgequake`.

## A. Local setup and configuration

| Problem | Cause | Fix |
|---|---|---|
| Backend could not reach any working LLM | `OPENAI_API_KEY` and `GEMINI_API_KEY`/`GOOGLE_API_KEY` are rejected by their providers (401/400). Only `MISTRAL_API_KEY` is valid, and it lived only in `edgequake/.env`, which nothing loads outside Docker | Added `MISTRAL_API_KEY` to the root `.env` (the file `make` exports). Mistral workspaces now query and ingest. Default-tenant workspaces use OpenAI embeddings and need a valid OpenAI key |
| `make dev-auth` would refuse to start | Auth on + dev mode off requires `JWT_SECRET` ≥ 32 bytes and a bootstrap admin | Root `.env`: generated `JWT_SECRET`, `EDGEQUAKE_BOOTSTRAP_ADMIN_*`, `EDGEQUAKE_MASTER_API_KEY`, `ALLOW_REGISTRATION=false` |
| WebSocket handshake 403, "Connexion perdue" banner | With dev mode off, CORS/WebSocket origin check fails closed when `EDGEQUAKE_CORS_ORIGINS` is empty | `EDGEQUAKE_CORS_ORIGINS=http://localhost:3010,http://127.0.0.1:3010` |

Backups of the previous root `.env`: `.env.bak-before-auth`, `.env.bak-before-copilot` (git-ignored).

## B. Authentication and tenant isolation

| # | Problem | Root cause | Fix | Guard |
|---|---|---|---|---|
| B1 | After login every scoped request returned 403 *"Header tenant_id does not match authenticated token claims"*; UI stuck on "Selecting workspace…" | Password-login JWTs are pinned to the default tenant/workspace (SPEC-027 IMP-004) and nothing could re-bind them; the UI auto-selected another tenant | New `POST /api/v1/auth/switch-scope` (`handlers/auth/scope_switch.rs`) re-binds the **existing** session family after a server-side authorization check. Refresh keeps the scope and re-checks access each time (`session.rs::refresh_scope`). Web UI calls it before any scoped request (`lib/api/session-scope.ts`, wired in `apiClient`, `stream-client`, multipart upload) | `tests/e2e_switch_scope.rs`, `session-scope.test.ts` |
| B2 | Platform admin could list/manage every tenant but could not read their data | `bind_request` demanded a membership row; nothing in password mode ever creates one | `authorize_scope_in` (`services/request_authorization.rs`): platform admins may use any active workspace (entry is audited as `platform_admin_scope_entry`); everyone else needs a membership | e2e test above |
| B3 | Admin-owned API keys (MCP, agents) refused outside the admin's own membership | Key role is derived from its scopes (e.g. Readonly) | After a membership miss, the key **owner's** stored role is consulted; the key's own scope still limits writes (verified: read 200, write 403) | live check |
| B4 | Workspace deep links `/w/[slug]/…` rendered without login | Layout had no `AuthGuard` | Wrapped in `AuthGuard` | browser check |
| B5 | Readonly members could not switch workspace | `switch-scope` counted as a write | Added to `is_query` | — |

### Security review findings (independent reviewer) — all fixed

| Severity | Finding | Fix |
|---|---|---|
| High | `switch-scope` turned a stolen 15-min access token into a fresh 30-day refresh family | The refresh cookie (or body token) is now **required**, authorized first, then consumed and rotated **within the same family**. Logout and IdP back-channel logout still end it. A re-presented old token trips reuse detection |
| High | Markdown images (LLM answers, copilot, documents) could send the bearer token / exfiltrate data to any host | `isTrustedImageOrigin` (`AuthenticatedMarkdownImage.tsx`): only same-origin/API-origin/`data:`/`blob:` images auto-load; external images become a click-only link (http/https only). Also protects the Query page |
| Medium | SSO sessions lost their IdP binding and membership-derived role after a switch | SSO families keep provider/issuer/subject/sid; role recomputed via `revalidate_scope` |
| Medium | Copilot gate failed open on backend errors | Fails closed unless `/auth/me` returns 2xx |
| Medium | LLM cost amplification | Body ≤ 256 KB, messages/context clipped, ≤ 6 tool calls/step and 30/turn, 20 requests/user/minute, client disconnect aborts backend calls |
| Medium | Auto-inserted admin memberships outlived demotion | Removed: admin access is now decided per request from the live role; the two rows created earlier were deleted |
| Low | Approval card hid what would change | Card shows full new values, merge strategy and cascades |
| Low | `.`/`..` path segments in tool ids | Rejected |
| Low | Upstream LLM error bodies echoed to the browser | Logged server-side only; copilot requires explicit `COPILOT_LLM_*` (no silent fallback to another provider) |

A shared lock (`serializeAuth`, `client-context.ts`) prevents a token refresh and a scope switch from presenting the same refresh token concurrently (which would trigger reuse-detection logout).

## C. API defects found by the live endpoint sweep

The sweep exercised all **231** endpoints (229 OpenAPI operations + Swagger + OpenAPI JSON) three times, only in throwaway workspaces.

| Run | PASS | Expected 4xx | FAIL | Blocked by LLM keys | Skipped |
|---|---|---|---|---|---|
| 1 | 159 | 26 | 21 | 14 | 11 |
| 2 | 185 | 27 | 10 | 2 | 7 |
| 3 | **191** | 26 | **5** | 2 | 7 |

| ID | Defect | Root cause → fix |
|---|---|---|
| F7 **critical** | `admin/entities/reconcile` planned to rename ~1 523 healthy nodes (data corruption) | Normalized the whole `{uuid}::NAME` id → only the NAME part is canonicalized; execute recomputes the token and refuses non-canonical plans (`edgequake-storage/src/entity_reconcile.rs`) |
| SEC | `POST /pipeline/cancel` cancelled every tenant's tasks; `recover-stuck` swept the whole instance | Scoped to the caller's tenant/workspace (`services/task_cancel.rs`, `orphan_staging_recovery.rs`) |
| F1 | metrics-history/snapshot → 500 | Table created in `ag_catalog` by migration 016 → new idempotent migration **170** moves it to `public` |
| F2 | `/documents/{id}/lineage` (+export) → 404 | Typed artifact read got no store → pass the request-scoped sidecar store |
| F3 | lineage/provenance 404 for plain entity names | Use `EntityId::exact_lookup_candidates` |
| F4 | Injections created but never readable | Typed writes got no DB pool → pass the pool everywhere |
| F8 | Cancel on a finished task → 200 | 409 for Failed/Indexed |
| F9 | Reprocess/recover/rebuild returned ids that 404 | Point `Location`/`job_id` at a real task; 200 without Location when nothing queued; v2 no-op jobs resolvable |
| F10 | Delete missing tenant → 204 | 404 |
| F11 | Cost estimate with unknown model → 200 | 400 |
| F12 | `start_node`, `graph/nodes/{id}`, `degrees/batch` need full ids | Resolve plain names |
| F13 | PDF status `document_id: null` | Fallback to the admitted KV shell |
| N1 | Lineage CSV had empty content/tokens/entity_count | Read chunk text from the chunk store; count `entity_ids` |
| N2 | Injection processing hit RLS error | Injection rows now carry `tenant_id` |
| N3 | Deleted injection resurrected by its running task | Delete cancels the task; status writes are update-only |
| N4 | Orphan graph nodes survived workspace delete | `Replace` writes are now update-only (never create); workspace clear also matches the `{workspace}::` prefix. Two orphans removed from the dev DB |
| R1 | (found in run 3) admin keys refused | see B3 |

Still open: `PATCH /costs/budget` is a stub (F5) and `POST /conversations/import` is not implemented (F6) — features, not bugs. `/api/generate` and `/api/chat` (Ollama compat) use the server-default OpenAI provider.

Full per-endpoint evidence: the sweep reports kept outside the repo (scratchpad `endpoint-sweep/report-run{1,2,3}.md`).

## D. New features

- **Copilot** in the web UI — see [02-features.md](02-features.md#copilot).
- **Google ADK agents** in `agents/` — see [02-features.md](02-features.md#adk-agents).
- **One-command GCP deploy** (`deploy/gcp/up.sh` / `down.sh`) — see [03-deployment.md](03-deployment.md).

## E. Tests added or changed

Backend: `e2e_switch_scope.rs`, `e2e_endpoint_sweep_relational_pg.rs`, `e2e_f9_resolvable_job_ids.rs`, `e2e_n3_injection_delete_race.rs`, `e2e_tenant_delete_not_found.rs`, `edgequake-storage/tests/e2e_n4_replace_update_only.rs`, unit tests in reconcile/lookup/graph/lineage/injection modules; the OpenAPI parity count and the `auth_memory_store` caller allowlist were updated for the new route.

Web UI: `session-scope.test.ts` (scope switch + `serializeAuth` ordering), `copilot/__tests__/tools.test.ts`, `trusted-image-origin.dom.test.ts`; `stream-client-spec149.test.ts` mock updated; locale parity kept (en/fr/zh).

Agents: unit tests (graph rendering, config, MCP write-tool rejection, workspace pin) and live smoke tests.

Results at the end of the session: OpenAPI contract 121/121, auth unit tests 27/27, every new e2e test green. Failures that remain in older suites (`e2e_data_model`, `e2e_reindexing`, `e2e_spec088_improvements`, `e2e_spec149_projection_replay` in parallel mode, 40 Vitest files that import `bun:test`, the module-size guard) fail identically on an untouched checkout of HEAD.
