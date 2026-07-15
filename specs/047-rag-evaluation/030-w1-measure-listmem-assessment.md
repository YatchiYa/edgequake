# 030 — W1-measure-listmem assessment (re-audit Acc #2)

**Date:** 2026-07-15  
**Protocol:** `026-listmem-2026-07-15`  
**Workspace:** Acc #2 `215b293d-…` · artifact `smoke-chart8-026-fig-as-chart-20260715-1707`  
**Plan:** [029](./029-post-acc2-fp-plan-w1-measure.md)

---

## Verdict

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│  MEASURE FIX LANDED. Chart + Table long gates PASS on Acc #2 markdown.       │
│                                                                              │
│  Chart a_in_e_long: 0.214 FAIL  →  0.571 PASS  (n=14)                        │
│  Table a_in_e_long: 0.353 FAIL  →  0.588 PASS  (n=17)                        │
│  Aggregate a_in_e_long: 0.383 → 0.660                                        │
│                                                                              │
│  Acc/F1 UNCHANGED (0.562 / 0.480) — no new ingest/query.                     │
│  This is measurement honesty (list members + quote strip), not new pixels. │
│                                                                              │
│  Honest claim: prior gate was false-negative on on-page list/quoted facts. │
│  Do NOT claim “fig-as-chart fixed Chart representation” from this flip.     │
│  Remaining Chart long misses = TRUE scalar/list gaps → W1-dense-scalar.    │
└──────────────────────────────────────────────────────────────────────────────┘
```

---

## What changed in code

| Change | Law |
|---|---|
| List gold → all members must be in evidence text | MMLongBench List = per-element avg |
| Strip `"'` / curly quotes in normalize | gold `'"MMMU"'` ↔ page `MMMU` |
| Protocol version → `026-listmem-2026-07-15` | do not compare raw a_in_e to pre-listmem |

Tests: `tests/test_fidelity.py` — **12 passed**.

---

## Chart long inventory after re-audit

**Hits (8/14):** prior 3 whole-string hits + list-member flips (political domains, Indonesia ops ×2, 2311 error types) + quoted MMMU.

**True remaining misses (~6):** scalars / incomplete years — `541`, `128`, `4087`, `1251`, `198` (wrong page), pie years missing `1982` on gold page.

---

## Wave 1 storytelling (protocol)

| Requirement | Status |
|---|---|
| Chart `a_in_e_long` ≥ 0.50 | **PASS** (measure fix) |
| Chart exclusive Acc ↑ | Acc #2: 0.143→0.286 via **W4 extract**, not densify |
| Acc ↑ from representation | **No** — Acc flat vs coexist |

**Product claim:** measurement gate cleared on Acc #2 corpus. **Next product Acc:** densify true scalar Chart misses → Acc #3.

---

## Next

1. Rebuild Small with densify callout prompt seed (sample sizes / years in chart specialize).
2. Acc #3; compare remaining Chart long misses under `026-listmem`.
3. Keep listmem protocol for all future fidelity reports.
