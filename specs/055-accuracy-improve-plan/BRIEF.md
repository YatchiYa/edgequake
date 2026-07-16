# Accuracy Plan — One Screen

**Path:** `specs/055-accuracy-improve-plan/` (working plan; not a second SPEC-055)  
**Baseline:** CORE @40 Acc **0.458** / F1 **0.356** · 40/40 · Chart/Table W1 PASS  
**July Acc/F1 reference:** Chart-8 Acc #2 **0.562 / 0.480** (different fixture)

---

## Decision

Optimize the pipeline in order: **W3/W4 → W2 → targeted W1**.  
Do **not** densify prompts or swap to a larger vision model first.

| Error mass (@40 zeros) | Count | Max Acc headroom |
|---|---:|---:|
| Answerable + page_hit@5 | 117 | +0.295 |
| Answerable + page miss@5 | 70 | +0.176 |
| Unanswerable + wrong answer | 26 | +0.065 |

---

## Targets

| Milestone | Acc | F1 | Key gate |
|---|---:|---:|---|
| M1 composition | ≥0.480 | ≥0.375 | no W1/retrieval regress |
| M2 retrieval | ≥0.500 | ≥0.400 | page_hit@5 ≥0.750 |
| M3 cross-page | ≥0.525 | ≥0.425 | cross-page Acc ≥0.320 |

M2 ≈ **17** additional full-credit equivalents of 397.

---

## Waves

0. **Firewall** — manifests, failure ledger, paired cluster bootstrap  
1. **Compose** — typed answer contract, deterministic %×N, list composer, calibrated refusal  
2. **Retrieve** — page candidates, larger pool + rerank, conditional decompose  
3. **Represent** — structured tables, figure packets, cross-page topology (fresh ingest)  
4. **Optional loop** — bounded expand/verify only if 1–3 plateau  

---

## Next experiment (G1)

Typed answer contract + normalizer · **query-only** · no ingest/retrieval change.

```text
Primary: CORE Acc paired Δ > 0
Slices: List +0.05 · Integer +0.03
Guards: UNA Acc ≥0.768 · page_hit unchanged · p95 +≤10%
Promote: ≥5 full-credit equivalents, no guard fail
```

Then G2 arithmetic as a **separate** causal run.

Full plan: [README.md](./README.md) · Tracker: [EXPERIMENT_LOG.md](./EXPERIMENT_LOG.md)
