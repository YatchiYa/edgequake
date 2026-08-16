# Lens 003 — Database Expert

## Stake

SPEC-131 is an **LLM transport / parameter policy** change. There is **no DDL**, no new tables, no CHECK widen, no migration.

## What touches storage (indirect only)

```ascii
  LLM 400 → document terminal Failed
       │
       ▼
  KV metadata: failure_class, recommended_action, error message
       │
       ▼
  public.documents.status → failed / partial_failure (existing allowlist)
```

After fix, fewer false `failed` rows from temperature 400s. Classifier change updates **KV string** `failure_class`, not a SQL enum.

## Non-goals

| Item | Why out |
|------|---------|
| New migration | No schema |
| Index on failure_class | Still KV / JSON metadata unless already relational |
| Persist ApiFormat per workspace | v1 is **process env** fleet policy (server-level) |
| Store Responses `response.id` | `store:false`; no retention of Mantle conversation ids |

## Observability note

If Prometheus already labels `edgequake_ingestion_failures_total{failure_class=…}`, add `llm_unsupported_param` to dashboards when classifier ships (SPEC-045 SRE lineage). No DB change required.

## Privacy / retention (Bedrock)

LAW-131-7 (`store:false`) prevents Amazon Bedrock from retaining Responses payloads for 30 days. That is a **compliance** concern adjacent to data layer, not a Postgres concern — call it out in ops docs.

## Cross-refs

- Laws: [../01-first-principles.md](../01-first-principles.md)
- Failure taxonomy: [../../045-fix-ingestion-errors/](../../045-fix-ingestion-errors/)
- Touch/status CHECK (unrelated): [../../129-touchd_document_faill/](../../129-touchd_document_faill/)
