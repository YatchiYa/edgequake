# zz-raw — Intake (GitHub #381)

> Not the contract. Canonical analysis lives in `00-why.md` onward.

**Source:** https://github.com/raphaelmansuy/edgequake/issues/381  
**Related:** https://github.com/raphaelmansuy/edgequake/issues/377  
**Title:** `touch_document_status` fails with `documents_valid_status` CHECK violation during checkpoint-resume (downstream of #377)

## Summary (reporter)

Non-fatal WARN during P7e/CHECKPOINT-RESUME when `needs_reembed=true`. Status write violates `documents_valid_status`. Found on 0.24.2 → 0.24.4 upgrade verification. Both known occurrences followed a prior #377 entity-embedding collision + crash checkpoint + reprocess.

## Evidence (reporter logs)

```
P7e/CHECKPOINT-RESUME: Skipping LLM extraction — ... reuse=CrashCheckpoint needs_reembed=true
Re-generating embeddings (slim checkpoint or incomplete embed) ... embeddings_omitted=true
SPEC-047 P1: touch_document_status failed (non-fatal) ...
  documents_valid_status
```

## Reporter expected behavior

`touch_document_status` must only write CHECK-allowed values. Either add the missing status to the constraint, or fix the resume path to use a valid value.

## Maintainer note (post-code-law)

Widening CHECK is the wrong fix. Shell already maps `re_embedding` → `processing`. Touch bypassed that mapper. See [03-code-as-is.md](03-code-as-is.md).
