# 06 — Edge cases

| EC | Issue | Case | Expected after fix |
|----|-------|------|--------------------|
| EC1 | #364 | Covered chunks, legacy rows still present | `retirable()==true`; drop enabled in dry-run |
| EC2 | #364 | Uncovered chunk remains | `retirable()==false`; 126 SQL aborts |
| EC3 | #364 | Regenerated embeddings, coverage 100% | Coverage-ready; copy-equality verify optional/off |
| EC4 | #364 | Shared table chunk+entity | 126 deletes chunks, keeps entity rows; fleet_retirable still false until iw2+131 |
| EC5 | #363 | Display name vs normalized key | Join hits after normalize |
| EC6 | #363 | True graph divergence (different entities) | Unresolved counted; job not GREEN |
| EC7 | #363 | Missing workspace_id in legacy metadata | Skip/fail visibly (today silent continue) |
| EC8 | #362 | Invalid UUID substring | Prefer safe cast (`uuid` cast error → treat as non-match) without seq-scan blowup |
| EC9 | #362 | 125 already applied | Cast fix only affects advisor; no re-run of 125 |
| EC10 | #362 | 125 pending | Patch SQL + checksums.lock before apply |
| EC11 | #366 | Wipe mid-upload | Admission blocks new ingest; post-wipe list empty |
| EC12 | #366 | Dual workspace | Wipe scoped; other workspace untouched |
| EC14 | #366 | Authoritative empty + raw KV residue | List stays empty (no suffix resurrect) |
| EC15 | #366 | Post-125 KV table absent | Wipe residual purge no-op; list still empty |
| EC16 | #366 | `EDGEQUAKE_KV_FAMILY_WSDOC=kv` soak | Legacy index path may suffix-fallback when index empty (rollback only) |
| EC13 | #361 | Local Ollama + many PDFs | Concurrency≈1 expected; not a regression |

## Anti-patterns to reject

- Pre-delete legacy rows in backfill solely to satisfy emptiness gate.
- Raising `statement_timeout` as the #362 “fix”.
- Treating `processed_count == estimated_total` as coverage proof.
- Closing #360/#366 as “user error” or “fixed by #309” without LAW-111-9 + dual-surface e2e.
- Treating authoritative empty membership as “try KV next”.
