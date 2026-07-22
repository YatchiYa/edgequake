# Ablation — Q4_acc_ci_p2b_v1

**Step:** q4  
**Pins:** P2b alone (S1 CE+protect + retrieval + path0.4 + headings) — Q3 lr_budget rejected  
**Workspace:** `8b359190-0733-4949-994c-f39eca074d79`  
**Archive:** `smoke-20260720T024233Z`

## Results

| Metric | Value | Gate | Result |
|--------|-------|------|--------|
| EQ Acc | **0.756** | — | best Acc this program |
| LR Acc | 0.780 | — | — |
| Δ Acc 95% CI | [−0.124, +0.074] | excludes 0 EQ (beat) | **tie** (includes 0) |
| EQ ctx_rel | **0.506** | ≥0.50 | ✅ |
| evidence_recall EQ/LR | 0.914 / 0.987 | ≥LR−0.03 | **miss** (−0.073) |

## Verdict

- [ ] Beat → promote + “beats LightRAG”
- [ ] Parity → promote + peer claim
- [x] **No promote** — keep Acc headline P0 BM25 / `PATH_PRUNE=0` / `PROTECT=0`

**Peer pack (labeled only):** P2b remains the best Acc package (Acc 0.756, ctx_rel 0.506, Complex packing historically strong). Do not ship Fact VECTOR knobs as Acc default after Q1/Q2/Q3 results.
