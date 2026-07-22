# SPEC-082 — First principles (push tests, honest floors)

Mid-scale (SPEC-079) showed A6 soft-green @100k and B2 soft-fail. DiskANN opt-in is already Supported @150k; SPEC-072 spot noted 250k green @q_list=800.

**Push laws:**

1. Larger N tests do not auto-raise floors.  
2. Floor raise requires full-gate: recall@20 ∧ latency ∧ concurrent (DiskANN dedicated) or filtered Wave-2 full-gate.  
3. Tips (A6 labels) stay opt-in OFF even if recall looks good.  
4. Silent flip forbidden.

| Arm | N | Pass meaning | Outcome (2026-07-18) |
|-----|---|--------------|----------------------|
| A6 Filtered-DiskANN labels | 150k, 250k | Soft recall vs Wave-2 ≥0.90 — tip archive | 0.95 @150k; **0.05 @250k** — tip stays OFF |
| Wave-2 filtered | 150k | Soft product floors; hang cliff hard — default floor stay unless concurrent full green | Single spot green — **Wave-2 floor stays 100k** |
| DiskANN dedicated | **250k primary** | Full-gate → may raise opt-in `highest_green_N` to 250k | **Promoted** — list≥800 + HQ build |
