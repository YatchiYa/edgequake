# 036 — Phase B first attempt @ 5 docs — INVALID for CORE progression

**Date:** 2026-07-15  
**Status:** **VOID as Phase B CORE** — inherited `EDGEQUAKE_BENCH_FIXTURE=smoke_chart_doc_ids_v1.txt`  
**Artifact (archived):** `e2e/artifacts/core-checkpoints/at_5_docs_CHART8_FIXTURE_MISRUN/`  
**Workspace:** `1ec17559-…` (chart-8 5-doc prefix only)

---

## What happened

Shell env still had Acc-chain fixture override → `bench047 core --max-docs=5` ran **chart-8**, not `core_doc_ids_v1` (40).

Scores (chart-8 5-doc prefix only): Acc **0.4847** · F1 **0.3472** · Chart long **0.60 PASS**. Not a CORE milestone.

---

## Fix

- `run_phase_b_core.sh` now `unset EDGEQUAKE_BENCH_FIXTURE`
- Restored frozen `smoke_doc_ids_v1.txt` (freeze-core had rewritten it)
- Fresh Phase B CORE from max-docs=5 with true core fixture

Do **not** compare the archived at_5 misrun to Phase B ladder.
