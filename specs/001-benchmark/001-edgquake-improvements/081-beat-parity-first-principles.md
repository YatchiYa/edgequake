# 081 — Beat / Parity Path (First Principles)

**Status:** F0–F4 executed · F3/F4 **REJECT** · Phase G **blocked** · next Acc lever **[082](./082-gold-citation-compat.md)** · Acc `publish/latest` frozen (P0 mid)  
**Date:** 2026-07-23  
**Parent:** [080](./080-beat-lightrag-evidence-roadmap.md) (packing STOP D1–D3 · D4 B6 Fact ER label)  
**Successor:** [082 Gold / Citation Compat](./082-gold-citation-compat.md)  
**Keep query base:** E2 occ on B5 [`T133053Z`](../e2e/artifacts/history/medical-mid-20260722T133053Z/)  
**Fact ER label:** E2 on B6 ge2 [`T013716Z`](../e2e/artifacts/history/medical-mid-20260723T013716Z/)  
**Acc SSOT:** P0 mid [`T104918Z`](../e2e/artifacts/history/medical-mid-20260722T104918Z/) · warm WS `8e990410-…`  
**Prior:** [056](./056-naming-identity-lr-parity.md) · [052](./052-rel-chunk-ids-query-parity.md) · [055](./055-post-acc-ceiling-first-principles.md)

---

## 1. North star (binding)

**Beat** only on **medical-full n=2062** when all hold:

1. Δ Acc 95% CI excludes 0 **with EQ ahead**
2. `ctx_rel ≥ 0.50`
3. overall ER ≥ LR − 0.03 **and** Fact ER ≥ LR − 0.03
4. Only then replace Acc `publish/latest`

**Parity** (publishable peer): Acc CI includes 0 (or EQ ahead) ∧ ctx≥0.50 ∧ Fact ER≥LR−0.03 on medical-mid, then full-N must not reopen a large LR Acc CI. Mid E2 Acc tie alone is **not** Beat.

Split peers forever: Acc headline ≠ gap-close E2 ≠ B6 Fact-ER label ≠ latency.

---

## 2. Scorecard (post-080)

| Surface | n | EQ Acc | Acc Δ CI | EQ ctx | Fact ER |
|---------|---|--------|----------|--------|---------|
| Acc headline P0 mid | 200 | 0.706 | LR [−0.107, −0.033] | 0.396 | 0.790 |
| Gap-close E2-B5 (**keep**) | 200 | 0.765 | tie [−0.031, +0.040] | 0.491 | 0.917 |
| D4 E2-B6 ge2 (label) | 200 | 0.750 | tie [−0.062, +0.008] | 0.459 | **0.930** |
| F3 B10 naming (REJECT) | 200 | 0.742 | LR [−0.087, −0.015] | 0.489 | 0.923 |
| F4 groundedness (REJECT) | 200 | 0.733 | tie [−0.073, +0.004] | 0.484 | 0.907 |
| G1 gold/citation (REJECT · [082](./082-gold-citation-compat.md)) | 200 | 0.764 | tie [−0.057, +0.010] | **0.461** | 0.917 |
| E2 full | 2062 | 0.739 | LR [−0.069, −0.017] | 0.472 | 0.918 |

**Residual after packing STOP:** Fact ER closed on B6 but Acc/ctx tax; E2-B5 Acc-tied mid but ctx&lt;0.50 and Fact ER miss; full-N Acc CI LR-ahead. Many Fact LR-wins have adequate EQ context SNR → generation path; ingest identity (B10 naming) still unrun.

### Do not retry

NF · dense BM25=0 · post_truncate · D1 unify · D2 intent-w · D3 relsel · TOPIC_* / soft Mix · B7–B9 Acc promote · silent Acc B5 overwrite · cap-relation-chunks (anti-052 / anti-LR).

---

## 3. Program phases

| Phase | Work | Exit |
|-------|------|------|
| **F0** | This memo + peers honesty | Done when linked from 000-index |
| **F1** | Membership vs generation forensics on Fact LR-wins | **Done** — E2 mid/full + B6 mid all **100% generation** (`f1-*`) |
| **F2** | `pick_chunks_by_weight` uses `all_source_chunk_ids` (052 hygiene) | **Done** — unit test |
| **F3** | B10 naming reingest → E2 medical-mid on **new** WS | **REJECT** [`T021330Z`](../e2e/artifacts/history/medical-mid-20260723T021330Z/) — Acc CI LR-ahead; ctx 0.489; keep E2-B5 |
| **F4** | Generation groundedness on B5 under E2 | **REJECT** [`T022412Z`](../e2e/artifacts/history/medical-mid-20260723T022412Z/) Acc −3.2pp vs E2; retry now opt-in `EDGEQUAKE_ANSWER_GROUNDED_RETRY` |
| **G** | Promote Acc latest | **Blocked** — mid Beat/parity unmet (E2 still best Acc CI; ctx&lt;0.50; Fact ER gap) |

```bash
# F1
PYTHONPATH=tools/bench001 python3 tools/bench001/scripts/failure_slice_eq_lr.py \
  --archive specs/001-benchmark/e2e/artifacts/history/medical-mid-20260722T133053Z \
  --out specs/001-benchmark/e2e/artifacts/forensics/f1-e2-mid

# F3
make bench001-b10-reingest
# then E2 mid on B10 WS (script runs ladder)
```

---

## 4. Code map

| Concern | Files |
|---------|-------|
| Forensics | `tools/bench001/scripts/failure_slice_eq_lr.py` |
| Weight pick ge2 | `edgequake-query/src/kg_chunk_pick.rs` |
| Naming filters (056) | `edgequake-storage/src/entity_id.rs` |
| B10 reingest | `tools/bench001/scripts/run_b10_reingest_acc.sh` |
| Empty / groundedness | `edgequake-query/src/engine_impl/prompt.rs` |

---

## 5. Promote checklist (Phase G)

- [ ] Winner pack medical-mid: Acc CI ≥ tie + ctx≥0.50 + Fact ER≥LR−0.03
- [ ] Same pack medical-full: Beat CI preferred; parity if CI includes 0 and L2 gates hold
- [ ] Acc `publish/latest` → new archive; `peers.json` + [019](../019-business-eq-vs-lightrag-and-rag.md)
- [ ] Gap-close / B6 Fact-ER / latency peers stay labeled

Until then: **do not claim EQ beats LightRAG.**
