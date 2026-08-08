# LENS — Database Expert (SPEC-113)

## Is this a database problem?

**No** as root cause. #369 is an LLM provider wire-protocol defect. PostgreSQL / AGE / pools are not on the failure path for the reporter’s alias proof.

## Why this lens still matters

| Concern | Note |
|---------|------|
| Pipeline durability | Failed extract/query may leave documents `Failed` in KV — ops sees “DB/pipeline red” when LLM is red |
| Idempotent retry | After fix, reprocess should succeed without schema migration |
| No migration | SPEC-113 must not invent DDL; capability cache is **in-process** (memory), not PG |
| Observability tables | Optional future: persist last think decision in audit — **out of scope** v1 |

## Guidance

```text
  Do NOT store "is_thinking_model" heuristics in Postgres.
  Do NOT add a models_capabilities table for Ollama in Wave A.
  Capability TTL cache lives beside the HTTP client.
```

If product later wants fleet-wide model cards, reuse discovery snapshots — still sourced from Ollama API, not name regexes.

## Interaction with SPEC-112

Pool starvation and think injection can co-occur in an incident window but are **independent** failure modes. Diagnose with `/api/show` + `/health`, not `pg_stat_activity`, for this issue class.
