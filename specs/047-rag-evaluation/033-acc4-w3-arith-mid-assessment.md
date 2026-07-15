# Acc #4 mid-assessment (year-span measure + smoke in flight)

**Date:** 2026-07-15  
**Build under Acc #4 smoke:** `20260715.121257`  
**Tag:** `chart8-026-w3-arith-20260715-2012`  
**Stack:** Acc #2 fig-as-chart − densify Acc #3 fluff + **W3-arith** + **year-span**  
**Honesty:** Acc↑ from W3-arith = Gen composition, **not** a W1 Chart representation claim.

---

## Early signal (measure-only on Acc #2 workspace)

Re-ran fidelity against Acc #2 MD (`215b293d-…`) with year-span expand in `normalize_for_containment`:

| Metric | Acc #2 listmem | + year-span |
|---|---:|---:|
| Chart `a_in_e_long` | 0.571 (8/14) | **0.643 (9/14)** |
| afe620 years `['1981','1982','2001','2002']` | miss | **HIT** (page has `1981-82` / `2001-02`) |

**Remaining Chart long misses (unchanged):** `541`, `128`, `4087`, `1251` (derived %×N), `198` (wrong page / W2).

This confirms 032 FP: year-span is a measure/Pass-A fix; derived counts need **W3-arith** for Acc, not denser pixels.

---

## Smoke progress

- Backend: mistral-small + mistral-embed · healthy  
- Ingest: doc 1/8 political completed; 2311.16502 in progress (large)  
- Full Acc #4 scorecard: pending (Cursor-managed bg)

---

## Compare target (Acc #2)

Acc **0.562** · F1 **0.480** · ChartEx **0.286** · Chart long **0.571** (listmem; **0.643** with year-span)
