# Cluster 04 — Pipeline reliability (chunk / LLM / tasks)

> **Sprint**: 2–3  
> **Laws**: LAW-2, LAW-3, LAW-5  
> **Defects**: Many FIXED (C-15/17/18/21–23, X-06/07/10/16/28/29, D-53); X-30 PARTIAL; C-16/X-08/X-13/X-14/X-18/X-19/X-31/D-51/D-52 CONFIRMED/PARTIAL

---

## WHY

Ingestion used to fail via wrong offsets, zero-attempt extract, N+1 fetches, partial KV, status drift, bare gleaning, substring retries, missing L2, weak FSM/checkpoints. Core reliability path is largely **FIXED**. Residual: atomic size guard (C-16), embed batch SSOT (X-08), page-marker SSOT (X-13), separator cascade (X-14), partial batch tolerance (X-18), backpressure (X-19), shutdown drain (X-31), cache never sets (D-52), ingest failure taxonomy still string-heavy (**X-30 PARTIAL**).

## ROOT CAUSE → STATUS

```
  C-15 offsets rebase          FIXED
  C-17 gleaning options        FIXED
  C-18 max_retries>=1          FIXED
  C-21 get_by_ids              FIXED
  C-22/C-23 KV + dedup         FIXED
  X-06 jitter + breaker type   FIXED
  X-07 typed retry_strategy    FIXED
  X-10 L2 normalize            FIXED
  X-16 fail-closed JSON        FIXED
  X-28/X-29 checkpoint + FSM   FIXED
  D-53 TokenEstimator          FIXED
  X-30 string taxonomy residual PARTIAL
  C-16 / X-08 / X-13 / D-52…   CONFIRMED backlog
```

## SOLUTION (DRY primitives)

Landed primitives: `rebase_offsets`, gleaning `CompletionOptions`, `.max(1)` attempts, `get_by_ids`, transactional KV, status SSOT, jittered embed retry, L2, full SHA-256 checkpoint, task FSM, `TokenEstimator`.

Still open: `AtomicSplitPolicy`, page-marker SSOT, separator prod default, embed batch SSOT, drain budget, cache set-or-remove, admission token bucket.

## E2E (FIXED evidence)

`e2e_page_aware_offsets_rebase`, `e2e_chunk_max_retries_zero_still_attempts_once_or_rejects`, `contract_gleaning_uses_completion_options`, `contract_no_substring_retry_matching`, `unit_retry_has_jitter`, `e2e_ollama_cosine_after_l2`, `e2e_kv_upsert_all_or_nothing`, `e2e_dedup_matches_completed_and_indexed`, `e2e_checkpoint_rejects_suffix_change`, `e2e_cancelled_cannot_mark_success`
