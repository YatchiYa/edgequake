# Ablation — 081 F4 generation groundedness (E2 on B5)

**Step:** lr-occ-fact-l2 (E2) + always-on F4 (verbatim prompt + low-coverage retry)  
**Stage:** medical-mid  
**Workspace:** `8e990410-43b5-44f4-9f56-87bd154570ce` (B5 Acc warm)  
**Archive:** `medical-mid-20260723T022412Z`  
**Smoke:** `smoke-20260723T022038Z` EQ Acc 0.789 tie (CI includes 0)  
**Baseline keep:** E2-B5 `medical-mid-20260722T133053Z`

## Gates vs E2-B5

| Gate | E2-B5 | F4 | Result |
|------|-------|----|--------|
| Acc CI | tie [−0.031, +0.040] · EQ 0.765 | tie [−0.073, +0.004] · EQ **0.733** | **REGRESS** (Acc −3.2pp) |
| ctx_rel | 0.491 | **0.484** | FAIL (<0.50) |
| Fact ER | 0.917 | **0.907** | FAIL (<LR−0.03≈0.913) |

## Verdict

- [ ] Gate met
- [x] Gate missed — **REJECT** (stop; do not promote; do not medical-full)

Post-gate: F4 retry made **opt-in** via `EDGEQUAKE_ANSWER_GROUNDED_RETRY=1` (default off). Verbatim sentence remains in `grounding_instructions()`. Acc warm + `publish/latest` unchanged. Phase G **blocked**.
