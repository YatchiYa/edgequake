# 055 — Post Acc-ceiling program (First Principles)

**Status:** Active hub — Acc Beat fishing **STOP** · split peers are the publishable truth  
**Date:** 2026-07-21  
**Cross-ref:** [043 honesty](./043-honesty-can-we-push.md) · [028](./028-first-principles-beat-roadmap.md) · [019 business](../019-business-eq-vs-lightrag-and-rag.md) · [056 naming](./056-naming-identity-lr-parity.md) · [057 latency](./057-latency-horizon-c-baseline.md) · [013 latency](./013-lens-latency-ops.md)

---

## 1. First-principles decomposition

| Layer | Question | Law |
|-------|----------|-----|
| **L0 Acc** | Are answers as good as LR? | Generation over admitted context. On n=40 Acc fairness, EQ↔LR is a **statistical tie** (CI includes 0). Soft Mix / SELECT Acc knobs **exhausted**. |
| **L2 retrieval** | Did we admit the right evidence cleanly? | Membership + salience. Dual-list clears recall with **Acc tax** → separate peer. |
| **Ingest identity** | Same world model as LR? | Entity/relation naming + extract caps + types. Product fidelity — **not** Acc fishing. |
| **Latency** | Time-to-answer UX | Query ops cost. Orthogonal to Acc CI; SLO EQ/LR p50 ≤ 1.5×. |

**Ceiling rule:** Do not spend another confound on Soft Mix Acc. Spend on (A) honest packaging, (B) LR-shaped ingest law, (C) latency.

---

## 2. Split peers (publishable truth)

| Peer | Archive | Pins | Acc | ctx | recall | Use for |
|------|---------|------|----:|----:|-------:|---------|
| **Acc Fact** | [`T120315Z`](../e2e/artifacts/history/smoke-20260720T120315Z/) | B5 WS + `a1fp` (P2b + `rr_cer` + `FACT_PROTECT_BM25=1`, no dual-list) | **0.801** | 0.519 | 0.926 | Answer-quality / Fact claims |
| **L2 Parity** | [`T093152Z`](../e2e/artifacts/history/smoke-20260720T093152Z/) | `a1lrl2` (dual-list + LR VECTOR budget) | 0.718 | 0.525 | **0.933** | Evidence-coverage / L2 claims |

**WS Acc:** `8e990410-43b5-44f4-9f56-87bd154570ce` · machine index: [`peers.json`](../e2e/artifacts/peers.json)

**Forbidden claims:**
- “Beats LightRAG” (Beat gate unmet — CI includes 0)
- One workspace/pin pack that is both Acc headline and L2 Parity without labeling the Acc tax
- Acc without L2 as a publishable RAG claim (S4)

**Allowed claims:**
- Acc: statistical **tie** with LR under Acc peer pins; EQ Acc point ≥ LR on this slice
- L2: **Parity** under dual-list peer (recall gate); Acc point lower — disclose tax
- Product: EQ is a deployable GraphRAG stack; LR is often faster (~5× p50 on Acc peer)

---

## 3. Program tracks (ordered)

```text
  Track A — Ship honesty (055)     ← do first; no re-ingest
       │
       ▼
  Track B — Naming identity (056)  ← LR normalize / numeric drop; labeled B10
       │     Acc peer stays B5 on REJECT
       ▼
  Track C — Latency (Horizon C)    ← p50 ratio; not Acc promote
```

| Track | Work                                                                                                 | Status / success                             |
| -------| ------------------------------------------------------------------------------------------------------| ----------------------------------------------|
| **A** | Dual-peer docs + `peers.json` + business brief refresh                                               | **Done** — readers cannot confuse Acc vs L2  |
| **B** | [056](./056-naming-identity-lr-parity.md) short-numeric / dotted-numeric drop                         | **Law✓ code+tests**; B10 Acc deferred        |
| **C** | [057](./057-latency-horizon-c-baseline.md)–[060](./060-c1d-heuristic-keyword-latency.md) | CE✓ · keyword=0✓ · generate ceiling |
| **D** | [061](./061-lightrag-law-first-principles-eq.md)–[064](./064-product-ttft-cache-batch-embed.md) | KEYWORD env Law✓ · cold ≈1.01× · **064 product TTFT/cache/embed** |

---

## 4. Stop rules (binding)

- No Soft Mix / TOPIC_* / dual-list Acc headline fishing  
- No soft alias merge as Acc lever  
- No Beat claim from “near” on n=40  
- One confound per ingest experiment; keep B5 Acc peer on REJECT  

---

## 5. Remaining product/law gaps (after 052–054)

| Gap | Status | Acc? |
|-----|--------|------|
| Rel `source_chunk_ids` @ query | [052](./052-rel-chunk-ids-query-parity.md) Law✓ | REJECT keep B5 |
| Entity types = LR default | [053](./053-entity-types-lr-parity.md) Law✓ | REJECT keep B5 |
| Extract caps 40/100 | [054](./054-extract-caps-lr-parity.md) Law✓ | REJECT keep B5 |
| Short-numeric / dotted-numeric drop | [056](./056-naming-identity-lr-parity.md) Law✓ code+tests | B10 Acc deferred |
| Surface-form synonym merge | Open product law — **not** soft-match Acc fishing | Deferred |
| Query latency ≤1.5× | [059](./059-c1b-latency-ceiling-keyword-embed.md)/[060](./060-c1d-heuristic-keyword-latency.md) — gen ceiling | Not Acc promote |
| LR role economics → EQ | [061](./061-lightrag-law-first-principles-eq.md) · [062](./062-c1e-fast-keyword-llm.md) | A Law✓ (product) |
| Fair cold latency | [063](./063-why-lightrag-faster-cache-fairness.md) `c1cold` | **1.013× PASS** · same mistral-small |
| Product UX polish | [064](./064-product-ttft-cache-batch-embed.md) | TTFT metrics · opt-in answer cache · batch embed |

---

## 6. Reproduce peers

```bash
# Acc Fact peer (warm)
export BENCH001_EQ_WORKSPACE_ID=8e990410-43b5-44f4-9f56-87bd154570ce
./tools/bench001/scripts/run_p_ladder_acc.sh a1fp

# L2 Parity peer (query-only; same WS or documented pack)
./tools/bench001/scripts/run_p_ladder_acc.sh a1lrl2
```
