# Ablation — LR_RELSEL_FACT_L2_v1

**Step:** lr-relsel-fact-l2  
**Stage:** smoke  
**Pins:** 080 D3: E2 + `RELATION_SELECT=lightrag`; not Acc Beat  
**Archive:** `smoke-20260723T012653Z`

## Gates

| Gate | Result |
|------|--------|
| Pins | PASS — `relation_select=lightrag`, `fact_replace` |
| Acc vs E2 OCC smoke | **FAIL** — EQ 0.709 (−8.5pp vs 0.794); CI [−0.164, +0.031] |
| Fact ER | Flat — 0.95 (= E2 OCC smoke); no lift |
| ctx | 0.475 (<0.48 smoke preferred) |

## Verdict

- [x] **STOP** — do not run medical-mid / medical-full relsel
- Confirms hist. Acc REJECT for `RELATION_SELECT=lightrag`

**Next:** D4 ingest ceiling audit (labeled WS) if query packs remain exhausted; Acc latest stays P0 mid; gap-close keep remains E2 occ.
