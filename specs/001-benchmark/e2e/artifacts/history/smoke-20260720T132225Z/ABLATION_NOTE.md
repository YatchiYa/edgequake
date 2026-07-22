# Ablation — A1FPSUMX_p2b_rr_cer_fact_protect_topic_mat_content_summarize_v1

**Step:** a1fpsumx  
**Pins:** 048 a1fpsumx: A1 + admit + CONTENT mat + MATERIALIZE_TYPES=summarize  
**Workspace:** `8e990410-43b5-44f4-9f56-87bd154570ce`  
**Archive:** `smoke-20260720T132225Z`

## Gates (048)

| Gate | Target | Result |
|------|--------|--------|
| Acc | ≥0.781 | **FAIL 0.749** |
| Fact ER | ≥0.83 | **FAIL 0.75** |
| ctx_rel | ≥0.50 | **PASS 0.50** |
| Sum ER | ≥0.95 | **PASS 0.963** |

## Verdict

- [ ] Gate met
- [x] Gate missed (do not promote) — **REJECT**; Sum ER win does not clear Fact/Acc tax from admit  
- TOPIC Acc fishing **STOP**; keep B5+`a1fp` Acc **0.801**
