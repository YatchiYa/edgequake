# Ablation — A1FPMAT_p2b_rr_cer_fact_protect_topic_mat_v1

**Step:** a1fpmat  
**Pins:** 042 a1fpmat: A1 + TOPIC_ENTITY_ADMIT + TOPIC_MATERIALIZE (KV into Mix before CE)  
**Workspace:** `2a7bcb2f-b156-4c49-9229-67f5bcde22a4`  
**Archive:** `smoke-20260720T113404Z`

## Gates

| Gate | Target | Result |
|------|--------|--------|
| Acc | ≥ 0.755 | **0.746 ✗** (a1fp 0.775) |
| Fact ER | ≥ 0.83 | **0.70 ✗** |
| Sum ER | ↑ vs 0.863 | **0.963 ✓** (+10pp) |
| Probe `bone cancers` in C | yes | **✗** (TNM present; phrase absent) |
| ctx | ≥ 0.50 | **0.506 ✓** |

## Verdict

- [ ] Gate met
- [x] Gate missed (do not promote)

**First principles:** Materialize closed CE_GAP for Summarize ER (0.86→0.96) — KV bodies enter Mix (`materialized=4` on binding Q). Acc/Fact tax; probe phrase still missing (first-4 topic ids include non-CONTENT / off-neighborhood chunks).

**Keep:** `a1fp` Acc peer. Leave `TOPIC_MATERIALIZE=0` off headline.

**Next (one confound):** Filter materialize to KV bodies that contain a question content bigram (CONTENT-gated inject), not densify-all.