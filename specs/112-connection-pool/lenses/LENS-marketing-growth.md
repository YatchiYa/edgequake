# LENS — Marketing / Growth (SPEC-112)

## Credibility rule

Do **not** market “production-ready multi-tenant PostgreSQL” while the recovery story is “restart EdgeQuake so others can connect.”

Falsifiable claim we *can* make after Waves A–B:

> EdgeQuake publishes a connection budget formula and labels every backend `edgequake:<role>` so shared-database fleets can size and diagnose pools.

## What not to claim

| Hype | Why reject |
|------|------------|
| “Unlimited connections / elastic DB” | PostgreSQL has a hard ceiling |
| “We fixed the 400 max_connections bug” | 400 was an ops band-aid, not a product feature |
| “Zero idle connections” | Pools intentionally keep idle capacity |
| Peak-incident numbers from the PPD CSV | CSV is not a saturation capture ([BRUTAL-HONESTY](../measurements/BRUTAL-HONESTY.md)) |

## Growth / partner enablement

- Publish the sizing table from [07-ops-runbook.md](../07-ops-runbook.md) in ops docs.
- Offer a “shared PostgreSQL checklist” for PPD-like fleets (EQ + QL).
- Case study angle after fix: **co-tenant safety** as a differentiator vs naive large pools.

## One-liner (post-fix)

```text
  Named pools. Budget math. Close on drain.
  Good co-tenant on shared PostgreSQL.
```
