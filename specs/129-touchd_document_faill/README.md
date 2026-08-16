# SPEC-129 — Touch document status CHECK violation (#381)

> **Mission:** Make every relational dual-write of `public.documents.status` go through one CHECK-safe mapper so SPEC-047 P1 `touch_document_status` never violates `documents_valid_status` — especially on slim-checkpoint resume (`re_embedding`).
>
> **Trigger:** [GitHub #381](https://github.com/raphaelmansuy/edgequake/issues/381) (downstream of crash-checkpoint retries often created by #377).

## Short verdict

| Layer | Finding |
|-------|---------|
| Gap | `touch_document_status` only maps `completed`→`indexed`; passes KV stages like `re_embedding` raw into the column |
| Shell | `normalize_documents_column_status` already maps `re_embedding`→`processing` for shell upserts |
| CHECK (141) | 13 allowlisted values — **no** `re_embedding` |
| Fix | Single `relational_documents_status_for_write` (normalize + `completed`→`indexed`); wire all touch/stats/sidecar writers |
| Non-fix | Do **not** widen CHECK; do **not** change KV `re_embedding` honesty; #377 collision is out of scope |

```ascii
  KV / UI stage                    public.documents.status
  -----------------                -----------------------
  re_embedding ──touch (bug)──X──► REJECT documents_valid_status
  re_embedding ──shell / fix───►   processing (allowed)
  completed    ──touch─────────►   indexed (allowed)
```

## Document map

```ascii
 00-why
  → 01-first-principles (LAW-129-*)
  → 02-cross-ref-matrix
  → 03-code-as-is
  → 04-target-architecture
  → 05-lenses/ (PO, fullstack, DB, UX, front)
  → 06-ux-ui-spec
  → 07-implementation-plan
  → 08-test-protocol
  → 09-acceptance
  → 10-edge-cases
  → 11-honest-assessment
  → zz-raw.md (intake, not the contract)
```

## Status board

| ID | Item | Status |
|----|------|--------|
| D0 | Intake `zz-raw.md` / #381 | Done |
| D1 | Doc pack (this folder) | Done |
| I0 | `relational_documents_status_for_write` SSOT | Done |
| I1 | Wire postgres/memory touch + sidecar + stats | Done |
| I2 | Wire stage_mirror + finalize + ensure_document_record defense | Done |
| T1 | Unit + `e2e_spec129_touch_status_check` | Done |
| C1 | GitHub #381 root-cause comment | Done |

## Related

- [#381](https://github.com/raphaelmansuy/edgequake/issues/381) — this bug
- [#377](https://github.com/raphaelmansuy/edgequake/issues/377) — upstream collision that often leaves crash checkpoints (trigger ≠ root cause)
- [SPEC-047](../047-rag-evaluation/) — P1 early relational touch; P5 slim checkpoint + re-embed
- [SPEC-057](../057-pipeline-reliability/) — stage honesty (`re_embedding`)
- [SPEC-098](../098-data-access-hardening/) — migration 141 CHECK; LAW-098-11 lifecycle passthrough
- Shell mapper: `edgequake-storage/.../document_shell.rs`

## Non-goals

- Fixing `idx_entity_embeddings_legacy_vector_id` / #377
- Adding UI stage slugs to `documents_valid_status`
- Making touch failures fatal (keep best-effort; make them **succeed**)
- Changing Documents list UX vocabulary beyond column freshness
