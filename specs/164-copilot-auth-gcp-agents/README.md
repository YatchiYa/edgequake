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
