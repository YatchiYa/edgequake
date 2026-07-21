# Ablation — A1FPCE_p2b_rr_cer_fact_protect_topic_ce_v1

**Step:** a1fpce  
**Pins:** 039 a1fpce: A1 + FACT_PROTECT_BM25 + TOPIC_ENTITY_ADMIT + TOPIC_CE_PROTECT (Exploratory)  
**Workspace:** `2a7bcb2f-b156-4c49-9229-67f5bcde22a4`  
**Archive:** `smoke-20260720T111057Z`

## Gates

| Gate | Target | Result |
|------|--------|--------|
| Acc | ≥ 0.755 | **0.736 ✗** (a1fp peer 0.775) |
| Fact ER | ≥ 0.83 | **0.85 ✓** |
| Sum ER | ↑ vs 0.863 | **0.877 ✓** (slight) |
| Probe `Medical-0002d2de` C has `bone cancers` | yes | **✗** (0×; cervical/anal/AML) |
| ctx | ≥ 0.50 | **0.519 ✓** |

## Verdict

- [ ] Gate met
- [x] Gate missed (do not promote)

**First principles:** Admit + fuse metadata merge + CE id-protect still leave final C off-topic. Local/global arms logged `topic_chunks=16` for bone query; no fuse-force INFO → topic ids already in Mix pool; CE protect cannot put missing content into truncate/format budget. Next confound (one): **truncation / packing protect for `topic_admit_chunk_ids`**.

Keep Acc peer: **a1fp** (`T095809Z`). Leave `TOPIC_CE_PROTECT=0` / `TOPIC_ENTITY_ADMIT=0` off headline.