---
title: 'EQ vs LightRAG — Acc Bench (SPEC-001)'
description: Measured GraphRAG-Bench Acc and fair cold latency for EdgeQuake vs LightRAG.
---

# EdgeQuake vs LightRAG — Acc Bench (SPEC-001)

> **Publish Acc SSOT** · GraphRAG-Bench medical-mid · `make bench` · 2026-08-12

Fair dual-SUT head-to-head: same corpus, questions, Mix mode, Mistral Small + `mistral-embed`, official `generation_eval`. **Not** UltraDomain win-rates · **not** paper Table-2 (GPT-4o-mini + BGE).

Protocol: [SPEC-001 index](../../specs/001-benchmark/000-index.md) · Business brief: [019](../../specs/001-benchmark/019-business-eq-vs-lightrag-and-rag.md)

---

## One-screen verdict

| Layer             | EdgeQuake | LightRAG  | Claim                                                                 |
| ----------------- | --------- | --------- | --------------------------------------------------------------------- |
| **Acc** (n=200)   | **0.783** | **0.774** | Statistical tie (Δ CI includes 0); **L2 incomplete** — not Acc Beat   |
| Evidence recall   | 0.924     | 0.952     | LightRAG                                                              |
| Context relevancy | 0.439     | 0.486     | LightRAG                                                              |
| Cold query p50    | 4447 ms   | 4359 ms   | **≈ tied (1.02×)** (peer `C1COLD_v1`, unchanged)                      |
| Warm Acc mid p50  | 3992 ms   | 1002 ms   | **Do not claim** — LR LLM cache                                       |

**Do not claim** “EdgeQuake beats LightRAG,” Acc mid Parity, or SOTA on GraphRAG-Bench. Acc Beat / Acc Equal mid remain **STOP** until [080 promote checklist](../../specs/001-benchmark/001-edgquake-improvements/080-phase-g-promote-checklist.md) is green (`ctx_rel ≥ 0.50`, ER gates, medical-full).

---

## Publish Acc (medical-mid n=200)

| Field | Value |
|-------|--------|
| Archive | [`medical-mid-20260812T004216Z`](../../specs/001-benchmark/e2e/artifacts/history/medical-mid-20260812T004216Z/) |
| Publish pack | [`publish/latest/`](../../specs/001-benchmark/e2e/artifacts/publish/latest/) |
| Profile | `ACC_E2OCC_086_v1_lrlike_arms_v2` |
| Fixture | `medical_publish_question_ids_v1` (50 questions / type) |
| Δ Acc 95% CI | **[-0.052, +0.118]** (bootstrap; CI includes 0 ⇒ tie) |
| Valid | `true` |
| `can_claim_beats_lightrag` | `false` (L2 gates unmet; Acc CI includes 0) |

### By question type (Acc)

| Type | EdgeQuake | LightRAG | Lead |
|------|-----------|----------|------|
| Fact Retrieval | 0.780 | 0.733 | EdgeQuake |
| Complex Reasoning | 0.756 | 0.729 | EdgeQuake |
| Contextual Summarize | 0.858 | 0.838 | EdgeQuake |
| Creative Generation | 0.736 | 0.794 | LightRAG |

### Ops (Acc pack)

- Empty-answer rates: EQ 0% · LR 0%
- Ingest wall: warm query-only (full-corpus workspace already present)
- Warm EQ/LR p50 ratio **~4.0×** — LightRAG keyword + answer **LLM cache** hits; not a fair engine claim ([063](../../specs/001-benchmark/001-edgquake-improvements/063-why-lightrag-faster-cache-fairness.md))

---

## Fair cold latency peer

| Field | Value |
|-------|--------|
| Peer | [`C1COLD_v1`](../../specs/001-benchmark/e2e/artifacts/publish/peers/C1COLD_v1/) |
| Archive | [`smoke-20260723T134452Z`](../../specs/001-benchmark/e2e/artifacts/history/smoke-20260723T134452Z/) |
| Pin | `BENCH001_LR_ENABLE_LLM_CACHE=0` · query-only · n=40 |
| EQ/LR p50 | 4447 / 4359 ms → **1.02×** (SLO ≤1.5× **PASS**) |

Prior Acc pack: [`medical-mid-20260802T135513Z`](../../specs/001-benchmark/e2e/artifacts/history/medical-mid-20260802T135513Z/) — EQ 0.807 / LR 0.779.

---

## Allowed / forbidden claims

| Allowed | Forbidden |
|---------|-----------|
| “Peer / statistical tie with LightRAG on Acc under fair Mistral pins (medical-mid n=200); L2 still trails LightRAG” | “Beats LightRAG” / Acc Beat / Acc Equal mid Parity |
| “Fair cold Mix latency ≈ LightRAG (~1.02×)” | Warm Acc mid “faster LightRAG” as product/engine win |
| “Product query API Equal LightRAG” ([083](../../specs/001-benchmark/001-edgquake-improvements/083-lightrag-query-api-law.md)) | Merging Acc, L2, latency, and product peers into one unlabeled winner |
| “Publish Acc on medical-mid n=200” | Publishing smoke n=40 as the release score |

Machine index: [`peers.json`](../../specs/001-benchmark/e2e/artifacts/peers.json).

---

## Reproduce

```bash
export MISTRAL_API_KEY=...
make bench001-doctor
make bench                    # medical-mid n=200 + publish/latest

# Fair cold latency (does not overwrite Acc publish/latest):
export BENCH001_EQ_WORKSPACE_ID=<warm-full-corpus-ws>
export BENCH001_SKIP_PUBLISH_LATEST=1
export BENCH001_PUBLISH_PEER=C1COLD_v1
make bench001-c1cold
```

Stakeholder pack after `make bench`:

- [BUSINESS_REPORT.md](../../specs/001-benchmark/e2e/artifacts/publish/latest/BUSINESS_REPORT.md)
- [EXEC_SUMMARY.txt](../../specs/001-benchmark/e2e/artifacts/publish/latest/EXEC_SUMMARY.txt)
- [SUMMARY.md](../../specs/001-benchmark/e2e/artifacts/publish/latest/SUMMARY.md)
