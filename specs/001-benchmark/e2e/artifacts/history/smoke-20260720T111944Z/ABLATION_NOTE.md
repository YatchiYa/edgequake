# Ablation — A1FPTRUNC_p2b_rr_cer_fact_protect_topic_trunc_v1

**Step:** a1fptrunc  
**Pins:** 040 a1fptrunc: A1 + admit + CE protect + TOPIC_TRUNC_PROTECT (Exploratory pack)  
**Workspace:** `2a7bcb2f-b156-4c49-9229-67f5bcde22a4`  
**Archive:** `smoke-20260720T111944Z`

## Gates

| Gate | Target | Result |
|------|--------|--------|
| Acc | ≥ 0.755 | **0.696 ✗** (a1fp 0.775; a1fpce 0.736) |
| Fact ER | ≥ 0.83 | **0.80 ✗** |
| Sum ER | ↑ vs 0.863 | **0.883 ✓** |
| Probe `bone cancers` in C | yes | **✗** |
| ctx | ≥ 0.50 | **0.456 ✗** |

## Verdict

- [ ] Gate met
- [x] Gate missed (do not promote)

**First principles:** Trunc/pack prefer fired on other Exploratory queries (`topic_pack=1..4`) but **not** on binding `Medical-0002d2de` — zero topic ids in the post-CE chunk list at pack time. Packing cannot resurrect chunks CE never kept. Final C still cervical/anal (same ~41k blob).

**Keep:** `a1fp` Acc peer. Leave `TOPIC_*=0` off headline.

**Next (one confound):** Diagnose **topic chunk id/content fidelity** — do `topic_admit_chunk_ids` for `BONE_CANCER` actually contain “bone cancer” text, and why CE protect leaves 0 of them in pre-trunc Mix (id mismatch vs missing content). Not densify-all; not another uncapped protect.