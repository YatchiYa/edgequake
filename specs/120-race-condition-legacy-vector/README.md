# SPEC-120 — Race on `legacy_vector_id` Unique Index

> **Mission:** Concurrent same-workspace fleet mirror must never fail document ingest on `idx_*_embeddings_legacy_vector_id`. Absorb losing writers; keep provenance uniqueness.  
> **Trigger:** [GitHub #374](https://github.com/raphaelmansuy/edgequake/issues/374) — `duplicate key value violates unique constraint "idx_entity_embeddings_legacy_vector_id"`.

## Short verdict

| Layer | Finding |
|-------|---------|
| Symptom | Concurrent ingest → GraphMerge fail on legacy unique |
| Proximate | `upsert_batch` ON CONFLICT only covers `(model_id, fk)` |
| Nuance | Exact-name entity create is already UNIQUE+ON CONFLICT safe |
| Still open | Alias / dual-FK + same `legacy_vector_id` → unhandled 23505 |
| Fix | UPDATE stamp-once + INSERT `ON CONFLICT DO NOTHING` (targetless) |

```ascii
  Doc A / Doc B  (same workspace, same logical lid)
       │
       ▼
  mirror_legacy_batch → upsert_batch
       │
       ├─ PK conflict        → COALESCE stamp (ok today)
       └─ legacy unique      → 23505 → GraphMerge (BUG)
              │
              ▼ TARGET
         absorb loser; one lid owner; merge succeeds
```

## Document map

```ascii
 00-why
   → 01-first-principles (LAW-120-1..7)
   → 02-cross-ref-matrix
   → 03-code-as-is
   → 04-target-architecture
   → 05-lenses/ (PO, fullstack, DB, UX, front, marketing, system)
   → 06-ux-ui-spec
   → 07-implementation-plan
   → 08-test-protocol
   → 09-acceptance
   → 10-reproduction
   → 11-honest-assessment
   → 12-first-principles-gap-closability
   → 13-close-decision
```

## Status board

| ID | Item | Status |
|----|------|--------|
| D1 | Doc pack | Done |
| R1 | Local SQL reproduction of 23505 | Done |
| G1 | GitHub #374 investigation comment | Done |
| I1 | DRY absorb module (`fleet_legacy_absorb.rs`) | Done |
| I2 | `MirrorLegacyReport.absorbed_legacy_collisions` | Done |
| I3 | EntityNameIndex oldest-wins + `ORDER BY created_at, id` | Done |
| T1 | Contract dual-FK same-WS race | Done |
| T2 | Storage mirror e2e + merger entity/rel concurrent e2e | Done |
| A1 | Acceptance | Done |
| H1 | Honest assessment + FP close decision | Done |

## Related

- [Issue #374](https://github.com/raphaelmansuy/edgequake/issues/374)
- SPEC-111 migrations 143/144 (`legacy_vector_id` unique)
- SPEC-091 fleet mirror / typed embeddings
- SPEC-098 typed authority / fail-closed mirror
- SPEC-083 graph-identity (alias / normalize follow-up — non-goal v1)
- Issues #362 / #363 / #364 (SPEC-111 Cluster A — related subsystem, not duplicates)

## Non-goals (v1)

- Full normalized-name UNIQUE / alias merge rewrite (→ SPEC-083)
- Dropping `(workspace_id, legacy_vector_id)` uniqueness
- UI/frontend redesign
- Reducing ingestion concurrency as the primary “fix”
