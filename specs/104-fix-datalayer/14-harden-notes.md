# 14 — Harden Notes (SPEC-104 Waves A–F)

> Follow-on to the first remediation. Closes residuals called out in [13-fix-assessment.md](13-fix-assessment.md) v1.  
> Code landed with this harden pass; assessment v2 lives in the same `13` doc (rewritten).

## Wave map

```ascii
 A naming SSOT ──▶ B era-safe INV ──▶ C tenant 409 ──▶ D multi-GIN
        │                │                 │               │
        └────────────────┴─────────────────┴───────────────┴──▶ E e2e ──▶ F assess
```

| Wave | Change | Primary files |
|------|--------|---------------|
| A | `PostgresConfig::{age_graph_name,bare_kv_table,bare_vectors_table}`; graph + inspector call helpers only | `config.rs`, `graph/mod.rs`, `storage_inspector.rs` |
| B | INV-03 dual `chunks`\|KV; INV-01 `chunk_embeddings` then legacy, never silent | `storage_inspector.rs` |
| C | Atomic `ON CONFLICT DO UPDATE … RETURNING`; HTTP 200 same-name / 409 different-name | `tenant_ops.rs`, `tenants.rs` |
| D | GIN check every `eq_%_graph` in `ag_catalog` | `storage_inspector.rs` |
| E | E2E-104-06..10 (+ updated 01–05 contracts) | `contract_spec104_datalayer.rs` |
| F | Honest re-assessment | `13`, `09`, `10`, README, issue docs |

## DRY / SOLID after harden

- **DRY:** one naming formula in storage; inspector does not re-`format!` relation names.
- **SRP:** INV-01/03 own era detection; **domain** owns slug identity Conflict; API maps errors only.
- **DIP:** inspector depends on `PostgresConfig` helpers, not string literals.
- **OCP:** new graphs picked up via `ag_catalog` without code change for GIN.

## Still not claimed cured

- Capacity **57014** with GIN present (SPEC-089).
- Full per-workspace **INV-C** (cost-bounded to default graph).
- PG19 `ON CONFLICT DO SELECT` (nice-to-have on PG18+).

## A+ pass (first principles)

| Change | Why |
|--------|-----|
| `Error::Conflict` in core + in-memory/PG `create_tenant` | Natural-key identity belongs in the domain service (SRP), not only HTTP |
| `CoreError::Conflict → ApiError::Conflict` | Single mapping site (DRY); handlers use `?` |
| `require_safe_sql_ident` before INV-01/03 `format!` | Data-eng: never interpolate unchecked identifiers |
| INV-01 prefer `document_id` column | Prefer typed FK join over `split_part` heuristics |

Assessment: [13-fix-assessment.md](13-fix-assessment.md) **v3**.
