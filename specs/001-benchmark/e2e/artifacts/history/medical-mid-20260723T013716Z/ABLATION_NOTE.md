# Ablation — LR_OCC_FACT_L2_B6_v1

**Step:** lr-occ-fact-l2 on B6 WS (049 ge2)  
**Stage:** medical-mid  
**Workspace:** `58ffe7da-d181-4a31-8941-9621b051a678` (ge2 rate 0.1247)  
**Peer:** `LR_OCC_FACT_L2_B6_v1` · Acc `publish/latest` skipped  
**Baseline keep:** E2 occ on B5 [`T133053Z`](../medical-mid-20260722T133053Z/)  
**Memo:** [080](../../../../001-edgquake-improvements/080-beat-lightrag-evidence-roadmap.md) · [049](../../../../001-edgquake-improvements/049-rel-dedup-source-chunk-union.md)

## Gates vs E2-on-B5

| Gate | Target | Result |
|------|--------|--------|
| ge2 structural | ≥0.05 | PASS (audit 0.1247 on this WS) |
| Acc CI not worse | tie-ish / near E2 | SOFT — CI [−0.062,+0.008] includes 0; Acc 0.750 (−1.5pp vs E2 0.765) |
| ctx ≥0.50 or ≥E2+0.02 | | **FAIL** — 0.459 (E2 0.491) |
| Fact ER ≥LR−0.03 or ≥E2+0.02 | | **PASS** — EQ 0.930 / LR 0.950 (≥0.920); +1.3pp vs E2 |

## Verdict

- [ ] REPLACE E2-B5 gap-close keep
- [x] **LABEL only** — Fact ER lift from ge2; Acc/ctx do not beat E2-B5
- Acc warm restored to B5 `8e990410-…` (run briefly hijacked warm pointer — harness fixed)

**Next:** No Acc packing; optional B7 placeholder VDB if AGE−vectors gap matters. Acc Beat still blocked.
