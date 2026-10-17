# SPEC-164 — Copilot, authenticated multi-workspace, ADK agents, one-command GCP

Work of the 2026-10-16 session on branch `feat/copilot-edgequake` (uncommitted working tree at the time of writing).

| File | Content |
|---|---|
| [01-session-changes.md](01-session-changes.md) | Every fix and improvement: configuration, authentication, security review, the 231-endpoint sweep and its defects, tests |
| [02-features.md](02-features.md) | What the new features do: copilot, workspace switching, ADK agents, GCP deploy |
| [03-deployment.md](03-deployment.md) | What is deployed, where, how it fits together, how to operate, update and destroy it |
| [../165-agents-platform](../165-agents-platform/README.md) | The ChatGPT-style agents platform (`/agents`): contract, design, status |

User-facing chapters: [tutorial/14-copilot.md](../../tutorial/14-copilot.md), [tutorial/15-adk-agents.md](../../tutorial/15-adk-agents.md), [tutorial/16-deploy-on-gcp.md](../../tutorial/16-deploy-on-gcp.md).

## In one paragraph

Authentication was switched on and made usable across tenants: JWTs pinned to one workspace are re-bound server-side by `POST /auth/switch-scope`, refresh keeps the scope, and an independent security review's findings were fixed. A live sweep of all 231 endpoints found 22 defects (one critical data-corruption bug in entity reconcile, one cross-tenant cancel); all but two unimplemented features are fixed with tests. A copilot was added to the web UI: page-aware, streaming, tool-using, and able to merge/split/regroup graph entities only after the user approves a change card. Google ADK agents (support, explorer, coordinator, investigator) talk to EdgeQuake over MCP and render graph artifacts. A ChatGPT-style agents platform at `/agents` (SPEC-165) adds per-user conversations and an agent builder. Plain `terraform apply` in `deploy/gcp/terraform` builds the fork's images, stands the whole stack up on a GCP VM with HTTPS and waits until it is healthy; `terraform destroy` removes everything.

## Rebased on v0.35.6 (2026-10-17)

The work was first done on an upstream snapshot older than the fork's `edgequake-main`; committing that tree on top of `edgequake-main` silently reverted four upstream commits (math spans, PR #496, and the 0.35.4 AGE-drop deploy step) and left the fork 110 commits behind release **v0.35.6**. The session's own changes were extracted as a pure diff from their real base and replayed on the `v0.35.6` tag, in branch `feat/agents-x-copilot-v0.35.6` of the fork (`YatchiYa/edgequake`). Conflict decisions:

- **Migrations**: upstream owns 170–207 (plain-SQL graph replacing Apache AGE). The fork's `workspace_metrics_history` fix moved from 170 to **208** (idempotent; still needed: v0.35.6 queries the table in `public` and does not move the `ag_catalog` copy). `deploy.sh` deletes the old `170 | workspace metrics history public` ledger row so upstream's 170 applies on volumes deployed before the rebase.
- **Fixes upstream had also made** (injection `tenant_id`, lineage cache sidecar store): upstream's version kept.
- **N3 (injection delete race)**: re-applied on upstream's files; workers without a request pool fall back to upstream's sidecar pool.
- **`edgequake/models.toml`**: upstream's file kept (the working tree carried an older copy; provider choice comes from `.env` / Secret Manager).
- **GCP**: the Postgres image stays on the AGE-capable `0.34.0` tag; upstream's "drop AGE with the 0.35.4 image" step is now keyed on the Postgres tag (our app images use content tags) and pulls its 0.35.4 API image from GHCR.

Checks on the rebased branch: `cargo check --workspace --tests`; `edgequake-api` lib 1514 tests, `contract_spec150_train_head` 40, and the session's e2e tests (switch-scope, tenant members, N3, F9, tenant delete) against a throwaway Postgres; web UI typecheck, locale parity and tests (2 failures that also fail on v0.35.6: module-size guard, status badge); agents 48 tests.

**Workspace isolation on v0.35.6.** Upstream re-adds every user to the Default Workspace each time the user record is saved (`sync_default_membership_to_postgres`, SPEC-027 phase 34) and re-activates that membership if an admin removed it, so alice and bob could open the Default Workspace (the live isolation check caught it: 16/18). New `EDGEQUAKE_DEFAULT_WORKSPACE_AUTO_MEMBERSHIP` (default `true` = upstream behaviour; the GCP compose sets `false`) skips that membership; platform admins keep access through admin virtual access. The two memberships already created on GCP were removed through the members API.

**VM boot disk.** Each deploy pulled new images and kept the old ones; on the 20 GB boot disk the v0.35.6 deploy filled it and the agents container crash-looped (matplotlib could not create a temp dir → 502 on `/api/agents`). `deploy.sh` now runs `docker image prune -af` after `DEPLOY_OK`.

## Rebased on edgequake-main v0.36.0 (2026-10-22)

After "Sync fork", the PR `feat/agents-x-copilot-v0.35.6 → edgequake-main` showed 42 conflicts: upstream re-landed 0.35.x on `edgequake-main` as new commits instead of merging the `v0.35.6` tag, so the same changes appeared twice. The fork commit was replayed on `origin/edgequake-main` (v0.36.0) in `feat/agents-x-copilot-v0.36.0`; 13 real conflicts:

- **Migrations**: upstream now owns 208–214; the fork's metrics-history migration is **215**. `deploy.sh` forgets ledger rows 170 *and* 208 named "workspace metrics history public".
- **Members**: upstream SPEC-169 owns `/tenants/{id}/members` (organization members, invitations, access links). The fork's per-workspace membership API moved to `/tenants/{id}/workspace-members` (same handlers, web UI card and e2e test).
- **Default membership**: upstream made it insert-only (no re-adding removed users); the fork's `EDGEQUAKE_DEFAULT_WORKSPACE_AUTO_MEMBERSHIP=false` still skips the first default membership.
- **Admin scope bypass**: upstream added one too; the fork's `authorize_scope_in` (also used by `switch-scope`'s refresh path and admin-owned API keys) is kept and now grants `Owner`, like upstream.
- **Sidebar**: upstream moved navigation to `lib/keyboard/destinations.ts`; Agents added there (chord `g i`).
