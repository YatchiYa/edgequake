# Ablation — LR_POSTTRUNC_FACT_L2_v1

**Step:** lr-posttrunc-fact-l2  
**Stage:** medical-mid  
**Pins:** 078 R3 — E2 + `EDGEQUAKE_KG_CHUNK_PICK_TIMING=post_truncate`  
**Workspace:** `8e990410-43b5-44f4-9f56-87bd154570ce`  
**Archive:** `medical-mid-20260722T141105Z`

## Gates vs E2 keep

| Gate | Target | Result |
|------|--------|--------|
| Acc CI | not worse than E2 (ci_low ≥ −0.031 or includes 0) | **FAIL** [−0.076, −0.001] LR |
| Fact ER | ≥ LR−0.03 or ≥ E2+0.02 | **PASS** 0.930 ≥ LR−0.03 (0.910); < E2+0.02 |
| ctx_rel | ≥0.50 or ≥0.511 | **MISS** 0.484 |
| Acc `publish/latest` | untouched | **PASS** P0 |

## Verdict

- [x] Gate missed — **REJECT R3** Acc CI worse than E2 keep; keep E2; do not stack R4
