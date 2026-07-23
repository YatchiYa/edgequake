# Ablation — LR_INTENT_W_FACT_L2_v1

**Step:** lr-intent-w-fact-l2  
**Stage:** medical-mid  
**Pins:** 080 D2: E2 + `MIX_INTENT_WEIGHTS=1` + `fact_replace`; not Acc Beat  
**Archive:** `medical-mid-20260723T012436Z`  
**Peer:** `LR_INTENT_W_FACT_L2_v1` (Acc latest skipped)  
**Baseline keep:** E2 occ [`medical-mid-20260722T133053Z`](../medical-mid-20260722T133053Z/)

## Gates vs E2 keep

| Gate | Target | Result |
|------|--------|--------|
| Honesty | Acc latest frozen | PASS |
| Acc CI not worse | not clearly LR-ahead | **FAIL** — CI [−0.082, −0.014]; EQ 0.718 / LR 0.764 (E2 tie 0.765/0.760) |
| ctx_rel | ≥0.50 or ≥E2+0.02 | **FAIL** — 0.477 (E2 0.491) |
| Fact ER | ≥LR−0.03 or ≥E2+0.02 | BORDERLINE — EQ 0.913 / LR 0.943 (=LR−0.03); E2 0.917 |

## Verdict

- [ ] KEEP
- [x] **REJECT** — Acc CI LR-ahead (−4.7pp vs E2); ctx down; Fact ER flat. Keep E2 occ packing. No medical-full D2.

**Next:** D3 `RELATION_SELECT=lightrag` is last-resort smoke only (hist. Acc REJECT). Prefer D4 ingest audit / empty-answer product path over more Acc packing. Do not promote Acc latest.
