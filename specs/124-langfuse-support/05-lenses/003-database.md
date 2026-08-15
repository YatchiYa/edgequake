# Lens 003 — Database Expert

## Verdict

**No new tables / migrations in v1.**

Langfuse credentials and export are process env → memory → OTLP. Persisting secrets in Postgres would violate LAW-124-2 and create rotation/ACL complexity without product need.

## What we explicitly do not store

| Data | Storage |
|------|---------|
| Public/secret keys | Env / secret manager only |
| Trace payloads | Langfuse (external) / optional local Jaeger |
| UI preferences for Langfuse | None (status is live from env) |

## Future (out of scope)

If multi-tenant BYO Langfuse appears, design encrypted tenant vault + migration then — not now.

## Interactions with existing DB

None. AGE/pgvector/KV unchanged. Do not attach Langfuse config to workspace metadata (would collide with SPEC-123 honesty).

## Cross-refs

- Laws: [../01-first-principles.md](../01-first-principles.md)
- Architecture: [../04-target-architecture.md](../04-target-architecture.md)
