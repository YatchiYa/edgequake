# Ablation — 033 Mix packing LR token caps (post-032)

**Archive:** `smoke-20260720T090743Z`  
**WS:** `2a7bcb2f-b156-4c49-9229-67f5bcde22a4` (B3b identity-correct)  
**Query:** A1 `rr_cer` · concurrency≤4 · **no FAQ**  
**Packing:** `EDGEQUAKE_MAX_ENTITY_TOKENS=6000` · `MAX_RELATION_TOKENS=8000` (LightRAG constants.py)

## First principles

Legacy EQ 10k/10k entity/relation tax overfilled Mix after 032 admitted ~4k WS AGE nodes.
Aligning caps to LR 6k/8k restores chunk remainder without soft Mix heuristics.

## Results

| Metric | EQ | LR | Gate |
|--------|-----:|-----:|------|
| Acc | **0.7735** | 0.7570 | Δ+0.016 · CI includes 0 (tie) |
| evidence_recall | 0.914 | 0.965 | miss (LR−0.03) |
| context_relevancy | 0.481 | 0.538 | miss (≥0.50) |

vs prior B3b A1+labelFTS (T085257Z Acc 0.749): **+0.024 Acc**.  
vs B2 Acc candidate (T071732Z Acc 0.785): −0.012 Acc; B3b is identity-correct.

## Promote

**No Beat / no Parity** — ctx and recall gates fail. Warm → B3b forensics + packing stack.

## Next

Principled L2 (recall/ctx) under full WS graph — still no FAQ / soft Mix Acc fishing.
