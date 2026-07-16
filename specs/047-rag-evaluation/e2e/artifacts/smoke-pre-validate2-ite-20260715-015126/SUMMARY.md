# SUPERSEDED — failed validate-2 attempt (do not use)

> **This archive is NOT the successful `ite` smoke.**  
> Read instead:
> - Live: [`../smoke/SUMMARY.md`](../smoke/SUMMARY.md)
> - Snapshot: [`../smoke-validate2-ite-20260715-015708-complete/SUMMARY.md`](../smoke-validate2-ite-20260715-015708-complete/SUMMARY.md)

## What happened here
- Stage: smoke (2 docs intended)
- valid: `False` (`PARTIAL_INGEST`)
- Acc / F1: **0** (n_scored=0)
- Ingest coverage: **0.00** — both docs marked failed after PDF conversion

**Root cause:** Postgres connection-pool timeouts + 500s on `/api/v1/documents/pdf/{id}` while polling, after PDFs had already reached `completed`. Not an `ite` config miss (doctor had PASS with `process_options=ite`).

The retry cleared stuck tasks, lowered vision concurrency, hardened client 5xx retries, and completed successfully with Acc≈0.54 / valid=true.
