# Ablation — A1FPSCX_p2b_rr_cer_fact_protect_answer_specific_complex_v1

**Step:** a1fpscx  
**Pins:** 047 a1fpscx: A1 + FACT_PROTECT_BM25 + ANSWER_PROMPT=specific + SPECIFIC_TYPES=complex  
**Workspace:** `8e990410-43b5-44f4-9f56-87bd154570ce`  
**Archive:** `smoke-20260720T131406Z`

## Gates (047)

| Gate | Target | Result |
|------|--------|--------|
| Acc | ≥0.781 (prefer ≥0.801 peer) | **FAIL 0.764** |
| Fact ER | ≥0.83 | **PASS 0.85** |
| ctx_rel | ≥0.50 | **PASS 0.500** |
| Complex Δ vs LR | ≤0.03 | **FAIL −0.065** |
| PARP names | ≥1 drug | **PASS** (olaparib) |
| prompt gate | Fact≠specific / Complex=specific | **PASS** (live prompt_only) |

## Verdict

- [ ] Gate met
- [x] Gate missed (do not promote) — **REJECT**; keep B5+`a1fp` [`T120315Z`](../smoke-20260720T120315Z/) Acc 0.801
- Specificity Acc family **STOP** (046 + 047)
