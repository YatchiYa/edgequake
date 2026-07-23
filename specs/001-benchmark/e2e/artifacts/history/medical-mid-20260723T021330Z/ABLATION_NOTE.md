# Ablation — 081 F3 B10 naming identity (E2 occ on new WS)

**Step:** lr-occ-fact-l2 (E2) after B10 force-ingest  
**Stage:** medical-mid  
**Pins:** 056 naming filters (short pure-numeric / short dotted-numeric) + md + glean; STRUCTURE_INDUCE=0; SKIP_PUBLISH_LATEST; peer `LR_OCC_FACT_L2_B10_v1`  
**Workspace:** `54806068-4a82-47b8-a7f9-aeb658f5eddc` (new; Acc warm left on B5 `8e990410-…`)  
**Archive:** `medical-mid-20260723T021330Z`  
**Baseline keep:** E2-B5 `medical-mid-20260722T133053Z`

## Gates vs E2-B5

| Gate | Target | B10 | Result |
|------|--------|-----|--------|
| Acc CI | not clearly LR-ahead | EQ 0.742 / LR 0.792 · CI **[-0.087, -0.015]** | **FAIL** |
| ctx_rel | ≥0.50 or ≥E2+0.02 (0.511) | **0.489** | **FAIL** |
| Fact ER | ≥LR−0.03 (0.913) or ≥E2+0.02 (0.937) | **0.923** (≥LR−0.03) | PASS (alone) |

## Verdict

- [ ] Gate met
- [x] Gate missed (do not promote) — **REJECT**

Naming reingest does not recover Acc/ctx vs E2-B5. Acc CI clearly LR-ahead. Keep E2-B5 as gap-close peer. Acc `publish/latest` unchanged. Next: **F4** generation groundedness (F1 was 100% generation on Fact LR-wins).
