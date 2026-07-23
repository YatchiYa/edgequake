# Ablation — 082 G1 gold/citation compat (E2 on B5)

**Step:** lr-occ-fact-l2 (E2) + Acc gold-compat generate (omit `[N]` mandates + strip artifacts)  
**Stage:** medical-mid  
**Workspace:** `8e990410-43b5-44f4-9f56-87bd154570ce` (B5 Acc warm)  
**Archive:** `medical-mid-20260723T024205Z`  
**Smoke:** `smoke-20260723T023833Z` EQ Acc 0.827 tie (CI includes 0)  
**Baseline keep:** E2-B5 `medical-mid-20260722T133053Z`

## Gates vs E2-B5

| Gate | E2-B5 | G1 | Result |
|------|-------|----|--------|
| Acc CI | tie [−0.031, +0.040] · EQ 0.765 | tie [−0.057, +0.010] · EQ **0.764** | PASS (not LR-ahead; Acc ≥ E2−0.01) |
| ctx_rel | 0.491 | **0.461** | **FAIL** (&lt; E2−0.01) |
| Fact ER | 0.917 | **0.917** | PASS |

## Verdict

- [ ] Gate met
- [x] Gate missed — **REJECT** (L2 ctx tax)

Acc non-regression alone is not enough. Not Parity (ctx&lt;0.50; Fact ER 0.917 &lt; LR−0.03≈0.940). No medical-full. No Acc promote. Gold-compat code **kept** (law under Acc gold pin). **H1 honesty freeze:** mid Parity unfinished; Acc Beat fishing STOP.
