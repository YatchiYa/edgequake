# zz-raw — Intake (GitHub #380)

> Not the contract. Canonical analysis lives in [`00-why.md`](00-why.md) onward.

**Source:** https://github.com/raphaelmansuy/edgequake/issues/380  
**Author:** @ravimohta  
**Opened:** 2026-08-14  
**Title:** SPEC-091 fleet mirror relationship resolution races against public.relationships writer — mirror queries ~1s before the edge exists

## Summary (reporter)

EdgeQuake 0.24.4 (upgraded from 0.24.2). Document KG persist fails deterministically with:

```text
SPEC-091: typed fleet mirror resolved 0/10 rows (relational entity/rel FK
miss or name mismatch — bare entities.name must match entity:NAME; ensure
PostgresEntitySink wrote the spine before fleet mirror; SPEC-098 misses:
[...])
```

Dominant failure mode in reporter environment: **199 of 9825** documents `Failed` (spans ≥2 days; predates 0.24.4 upgrade).

## Reporter root-cause claim

Race between relationship-vector mirror (`resolve_relationship_id_pool` → `mirror_legacy_batch`) and writer into `public.relationships` — “two independent writers with no ordering guarantee.”

Evidence cited:

1. Entities `MELISSA_BOTHA`, `FLAT_4` exist (same batch `created_at`).
2. Edge `MELISSA_BOTHA -[OWNER_OF]-> FLAT_4` exists in `public.relationships`.
3. Edge `created_at` ~1s after entities → inferred SELECT-before-INSERT.
4. `EDGEQUAKE_TASK_MAX_WORKERS=1` — still fails identically on retry.
5. RLS ruled out (`edgequake` has `rolbypassrls=t`).

## Reporter expected behavior

- Sequence / barrier so mirror runs after edge commit, or same transaction/UoW.
- Failing that: bounded retry/backoff inside mirror (not hard document failure).

## Environment (reporter)

- EdgeQuake 0.24.4
- Dedicated postgres; `rolbypassrls=t`
- High entity-density corpus (invoices / remittance; 50–350+ entities/doc)

## Maintainer note (post-code-law)

Typed path already sequences **RelGraph → RelVectors** with await barriers. The ~1s entity→rel `created_at` gap is **expected** (EntityVectors + AGE between sinks). Identical retries + leftover SQL spine contradict a pure visibility race. First-principles gap: **sink establishes relationship UUID then discards it; mirror re-resolves by name**. See [03-code-as-is.md](03-code-as-is.md) and [01-first-principles.md](01-first-principles.md).
