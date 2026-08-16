# 00 — Why SPEC-129

## Trigger

Intake: [`zz-raw.md`](zz-raw.md) — GitHub [#381](https://github.com/raphaelmansuy/edgequake/issues/381).
On slim-checkpoint resume (`P7e/CHECKPOINT-RESUME`, `needs_reembed=true`), SPEC-047 P1 logs:

```text
SPEC-047 P1: touch_document_status failed (non-fatal)
… violates check constraint "documents_valid_status"
```

Pipeline continues. Relational `documents.status` may stay on a prior terminal value (e.g. `failed`) while KV already shows `re_embedding`.

## Product WHY

```ascii
  Operator: “I reprocessed a failed doc — why is the list still Failed
             while logs say Re-generating embeddings?”
  Monitor:  “WARN every resume — is status dual-write broken?”
       │
       ▼
  Today (bug):
       KV  ← re_embedding          (honest stage, SPEC-057)
       SQL ← re_embedding raw      (CHECK rejects)
       List column / some readers stay on stale SQL status
              │
              ▼
  Blind spots:
       1. Touch path skipped shell normalizer (DRY break)
       2. Non-fatal WARN hides a real invariant violation
       3. UI freshness / retry eligibility may read SQL
```

## Five WHYs

1. **Why does the WARN fire?** Postgres rejects `UPDATE documents SET status = 're_embedding'`.
2. **Why is that string written?** Resume sets status `re_embedding` for honesty; P1 mirrors the same string.
3. **Why isn’t `re_embedding` in the CHECK?** Column allowlist is coarse (13 values after migration 141); rich stages live in KV.
4. **Why don’t shell upserts fail the same way?** They call `normalize_documents_column_status` (`re_embedding`→`processing`).
5. **Root cause:** Touch/sidecar writers duplicated a partial map (`completed`→`indexed`) and **bypassed** the shell normalizer SSOT.

## Job to be done

> When any pipeline stage updates document status, KV may use a rich stage slug, and `public.documents.status` always receives a CHECK-allowed value so the Documents list and SQL readers stay fresh without WARNs.

## Success criteria

1. `touch_document_status("re_embedding")` succeeds; column = `processing`.
2. Raw SQL `status = 're_embedding'` still fails CHECK (no widen).
3. KV still stores `re_embedding` on slim-resume (honesty preserved).
4. All relational status writers share one mapper (DRY).
5. e2e + unit gates prove the matrix in [10-edge-cases.md](10-edge-cases.md).

## Cross-refs

- Laws: [01-first-principles.md](01-first-principles.md)
- Code: [03-code-as-is.md](03-code-as-is.md)
- Plan: [07-implementation-plan.md](07-implementation-plan.md)
- Edges: [10-edge-cases.md](10-edge-cases.md)
