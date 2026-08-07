# LENS — Product Owner (SPEC-112)

## Problem in product language

Partners run **EdgeQuake + QL (+ tools) on one PostgreSQL**. When EdgeQuake holds idle pool slots near the server ceiling, **other products cannot connect**. The recovery story “stop EdgeQuake” is a **reliability failure**, not an ops tip.

## “Done” means

| Outcome | Law |
|---------|-----|
| Co-tenants can connect while EdgeQuake is healthy | 112-2, 112-3 |
| Incidents attribute backends to EdgeQuake roles in minutes | 112-4 |
| Graceful stop frees DB slots promptly | 112-5 |
| We never ship “set max_connections=400” as the product fix | 112-6 |
| PPD can apply env sizing today from the runbook | 07-ops |

## Non-goals this pack

- Guaranteeing infinite horizontal scale without a pooler.
- Owning QL’s connection policy.
- UI redesigns unrelated to operator clarity.

## Acceptance narrative for partners

```text
  Before:  stop EdgeQuake to unblock QL
  After:   sized pools + named backends + close on drain
           + alert at 70% max_connections
           stop-EQ is emergency only
```

## Priority call

Ship **Wave A + B** before polish (C/D). Identity and budget stop the bleeding; timeouts and dashboards harden.
