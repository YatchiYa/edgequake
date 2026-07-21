# 041 Topic chunk fidelity audit

**UTC:** 20260720T112509Z  
**Q:** `Medical-0002d2de` — How are bone cancers staged and what factors are considered in determining the stage?  
**WS:** `2a7bcb2f-b156-4c49-9229-67f5bcde22a4`  

## Verdict

**Law:** `CE_GAP`  
Linked bodies contain question bigrams, but Mix C does not (SELECT after link — CE/fuse/trunc survivors).

## Observables

- Content bigrams: `bone cancers, cancers staged, staged stage`
- Entity norms: `BONE_CANCER, BONE_CANCERS, CANCERS_STAGED, STAGED_STAGE`
- Mix chars/parts: 41061 / 6
- Mix bigram hits: `∅`
- RESOLVE_any=True CONTENT=True IN_MIX=False

## Entities

### `BONE_CANCER` (EQ `BONE_CANCER` · LR `Bone cancer`)

- chunks EQ/LR: 5 / 6 · resolved 5/5 · content-hit 3
  - `019f7ea3-71dc-7bf2-85fa-cd86aa2dc14d-chunk-146` resolved=True hits=['bone cancers'] · "About bone cancer 5 What is bone cancer? 6 What is bone? 7 What's in this book? 7 What can you do to"
  - `019f7ea3-71dc-7bf2-85fa-cd86aa2dc14d-chunk-148` resolved=True hits=∅ · '. While these reports might be available to you through your patient portal or patient access system'
  - `019f7ea3-71dc-7bf2-85fa-cd86aa2dc14d-chunk-149` resolved=True hits=∅ · '. The goal is to look for gene mutations inherited from your birth parents called germline mutations'
  - `019f7ea3-71dc-7bf2-85fa-cd86aa2dc14d-chunk-150` resolved=True hits=['bone cancers'] · '. New bone formation, known as ossification, starts in the womb and ends during adolescence, between'
  - `019f7ea3-71dc-7bf2-85fa-cd86aa2dc14d-chunk-151` resolved=True hits=['bone cancers'] · '. Poorly differentiated (G3) means the cancer cells look very different compared to normal cells. GX'

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
