# 041 Topic chunk fidelity audit

**UTC:** 20260720T112429Z  
**Q:** `Medical-0002d2de` — How are bone cancers staged and what factors are considered in determining the stage?  
**WS:** `2a7bcb2f-b156-4c49-9229-67f5bcde22a4`  

## Verdict

**Law:** `NO_ENTITY`  
No exact-name entity from question bigrams in EQ graph.

## Observables

- Content bigrams: `bone cancers, cancers staged, staged stage`
- Entity norms: `BONE_CANCERS, CANCERS_STAGED, STAGED_STAGE`
- Mix chars/parts: 41061 / 6
- Mix bigram hits: `∅`
- RESOLVE_any=False CONTENT=False IN_MIX=False

## Entities

### `BONE_CANCERS` (EQ `None` · LR `None`)

- chunks EQ/LR: 0 / 0 · resolved 0/0 · content-hit 0

### `CANCERS_STAGED` (EQ `None` · LR `None`)

- chunks EQ/LR: 0 / 0 · resolved 0/0 · content-hit 0

### `STAGED_STAGE` (EQ `None` · LR `None`)

- chunks EQ/LR: 0 / 0 · resolved 0/0 · content-hit 0

## LR Mix (same Q)

- chars: 78664
- bigram hits: `bone cancers`

## Next (one confound)

- If `RESOLVE`: fix chunk id namespace between AGE `source_chunk_ids` and storage.
- If `CONTENT`: fix entity↔chunk provenance (wrong links) — not CE protect.
- If `CE_GAP`: one SELECT fix so CONTENT survivors enter Mix C.
- Forbidden: densify-all, stacking TOPIC_* protect without fidelity law.
