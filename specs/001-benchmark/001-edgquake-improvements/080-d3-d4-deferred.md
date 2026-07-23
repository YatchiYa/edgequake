# 080 — D3 / D4 deferred gates

**Status:** D3 STOP · D4 label (Fact ER on B6; Acc keep stays E2-B5)  
**Date:** 2026-07-23  
**Parent:** [080](./080-beat-lightrag-evidence-roadmap.md)

---

## D3 — `RELATION_SELECT=lightrag` (last-resort)

- Ladder: `make bench001-lr-relsel-fact-l2` (smoke hard-gate first).
- History: [051](./051-relation-rank-weight-select.md) Acc REJECT — do not stack with NF/dense/post_truncate.
- **2026-07-23 smoke STOP:** [`smoke-20260723T012653Z`](../e2e/artifacts/history/smoke-20260723T012653Z/) — Acc 0.709 (−8.5pp vs E2 OCC smoke); Fact ER flat 0.95; **no medical-mid**.
- Do not retry Acc packing with relsel.

---

## D4 — Ingest ge2 / source_id ceiling

- Reuse B1 audit: `make bench001-b1-audit` · [029](./029-ingest-parity-audit.md).
- Multi-chunk ge2 / source union: [049](./049-rel-dedup-source-chunk-union.md) (**STRUCT✓** already shipped).
- **2026-07-23:**
  - B5 Acc WS `8e990410-…` ge2 **0%** ([audit](../e2e/artifacts/ingest-audit/20260723T012739Z/)) — pre-B6 ingest
  - B6 WS `58ffe7da-…` ge2 **12.5%** ([audit](../e2e/artifacts/ingest-audit/20260723T013324Z/)) — STRUCT pass
  - E2 packing mid on B6 [`T013716Z`](../e2e/artifacts/history/medical-mid-20260723T013716Z/): Fact ER **0.930** (≥LR−0.03, +1.3pp vs E2-B5) · Acc 0.750 (−1.5pp) · ctx 0.459 — **label only**, do not replace E2-B5 gap-close keep
- Acc warm restored to B5; harness: labeled peers (`SKIP_PUBLISH_LATEST` / `PUBLISH_PEER`) must not overwrite global warm.
- Acc `publish/latest` frozen. No silent Acc WS overwrite / reingest onto B5.

```bash
# Audit B5 (Acc) vs B6 (ge2)
make bench001-b1-audit
BENCH001_EQ_WORKSPACE_ID=58ffe7da-d181-4a31-8941-9621b051a678 make bench001-b1-audit
# E2 mid on B6 (labeled; skip Acc latest)
BENCH001_EQ_WORKSPACE_ID=58ffe7da-d181-4a31-8941-9621b051a678 \
  BENCH001_LADDER_STAGE=medical-mid BENCH001_SKIP_PUBLISH_LATEST=1 \
  BENCH001_PUBLISH_PEER=LR_OCC_FACT_L2_B6_v1 \
  ./tools/bench001/scripts/run_p_ladder_acc.sh lr-occ-fact-l2
```
