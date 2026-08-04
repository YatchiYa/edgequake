# 00 — Why SPEC-107

## Trigger

A partner shared Quantalogic production Postgres / EdgeQuake log errors and asked whether we see the same classes on our side, offering a joint deep-dive session.

## Why not only SPEC-104?

| Pack | Audience | Job |
|------|----------|-----|
| **SPEC-104** | Engineering | Reproduce, fix, A+ harden, ship ≥0.24.0 |
| **SPEC-107** | Partner + ops | Answer the thread, map symptoms → status, residual INV-03 ops, session agenda |

SPEC-104 already holds the incident dump ([00-issue-data](../104-fix-datalayer/00-issue-data.md)) and closed #1–#4. SPEC-107 is the **partner-incident lens** on the same four symptoms from the email (the email omits SPEC-104 #5 node-count timeout).

## Non-goals

- Re-implement or contradict SPEC-104 RCAs
- New SQL migrations
- Auto-requeue of INV-03 orphans (product policy / CAUTION)
- Deploying partner prod (their ops)
