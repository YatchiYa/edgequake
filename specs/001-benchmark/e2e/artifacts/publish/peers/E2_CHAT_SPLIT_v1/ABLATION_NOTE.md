# Ablation — E2_CHAT_SPLIT_v1 (084)

**Step:** e2-chat-split  
**Stage:** medical-mid  
**Archive:** `medical-mid-20260723T041648Z`  
**Pins:** E2 occ + 083 `chat(system,user)` generate (COMPLETE_BLOB off)  
**Workspace:** `8e990410-43b5-44f4-9f56-87bd154570ce`  
**Acc `publish/latest`:** skipped (P0 mid SSOT unchanged)

## Results vs E2-B5 keep (`T133053Z`)

| Metric | E2-B5 | E2_CHAT_SPLIT | Gate |
|--------|-------|---------------|------|
| EQ Acc | 0.765 | **0.792** | ≥ E2 − 0.01 → **PASS** |
| Acc Δ CI | [−0.031, +0.040] | **[−0.016, +0.048]** | not LR-ahead → **PASS** |
| EQ ctx_rel | 0.491 | **0.473** | ≥0.50 Parity / ≥E2−0.01 keep → **FAIL** |
| Fact ER | 0.917 / LR 0.953 | **0.950 / LR 0.963** | ≥ LR−0.03 → **PASS** |

## Verdict

- [x] Gate missed for **Parity** (ctx 0.473 < 0.50 and < E2−0.01)
- Acc CI improved (tie, EQ point-ahead) — labeled keep candidate for Acc CI only
- Do **not** Acc promote · do **not** open packing · **no medical-full** (H4 skipped)

**Claim:** generation role-split helps Acc without Beat; Equal LightRAG mid Parity still unfinished (ctx).
