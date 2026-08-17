---
title: 'EQ vs LightRAG — Acc Bench (SPEC-001)'
description: Measured GraphRAG-Bench Acc and fair cold latency for EdgeQuake vs LightRAG.
---

# EdgeQuake vs LightRAG — Acc Bench (SPEC-001)

<<<<<<< HEAD
> **Publish Acc SSOT** · GraphRAG-Bench medical-mid · `make bench` · 2026-07-23
=======
> **Publish Acc SSOT** · GraphRAG-Bench medical-mid · `make bench` · 2026-08-02
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042

Fair dual-SUT head-to-head: same corpus, questions, Mix mode, Mistral Small + `mistral-embed`, official `generation_eval`. **Not** UltraDomain win-rates · **not** paper Table-2 (GPT-4o-mini + BGE).

Protocol: [SPEC-001 index](../../specs/001-benchmark/000-index.md) · Business brief: [019](../../specs/001-benchmark/019-business-eq-vs-lightrag-and-rag.md)

---

## One-screen verdict

<<<<<<< HEAD
| Layer             | EdgeQuake | LightRAG  | Claim                                 |
| -------------------| -----------| -----------| ---------------------------------------|
| **Acc** (n=200)   | **0.770** | **0.779** | **Statistical tie** (Δ CI includes 0) |
| Evidence recall   | 0.926     | 0.950     | LightRAG                              |
| Context relevancy | 0.408     | 0.511     | LightRAG                              |
| Cold query p50    | 4447 ms   | 4359 ms   | **≈ tied (1.02×)**                    |
| Warm Acc mid p50  | 4089 ms   | 1016 ms   | **Do not claim** — LR LLM cache       |

**Do not claim** “EdgeQuake beats LightRAG,” Acc mid Parity, or SOTA on GraphRAG-Bench. Acc Beat / Acc Equal mid remain **STOP**.
=======
| Layer             | EdgeQuake | LightRAG  | Claim                                                                 |
| ----------------- | --------- | --------- | --------------------------------------------------------------------- |
| **Acc** (n=200)   | **0.807** | **0.779** | Statistical tie (Δ CI includes 0); **L2 incomplete** — not Acc Beat   |
| Evidence recall   | 0.909     | 0.958     | LightRAG                                                              |
| Context relevancy | 0.420     | 0.499     | LightRAG                                                              |
| Cold query p50    | 4447 ms   | 4359 ms   | **≈ tied (1.02×)** (peer `C1COLD_v1`, unchanged)                      |
| Warm Acc mid p50  | 4650 ms   | 647 ms    | **Do not claim** — LR LLM cache                                       |

**Do not claim** “EdgeQuake beats LightRAG,” Acc mid Parity, or SOTA on GraphRAG-Bench. Acc Beat / Acc Equal mid remain **STOP** until [080 promote checklist](../../specs/001-benchmark/001-edgquake-improvements/080-phase-g-promote-checklist.md) is green (`ctx_rel ≥ 0.50`, ER gates, medical-full).
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042

---

## Publish Acc (medical-mid n=200)

| Field | Value |
|-------|--------|
<<<<<<< HEAD
| Archive | [`medical-mid-20260723T134124Z`](../../specs/001-benchmark/e2e/artifacts/history/medical-mid-20260723T134124Z/) |
| Publish pack | [`publish/latest/`](../../specs/001-benchmark/e2e/artifacts/publish/latest/) |
| Profile | `P0_mistral_small_mix_chunk1200_v1_lrlike_arms_v2` |
| Fixture | `medical_publish_question_ids_v1` (50 questions / type) |
| Δ Acc 95% CI | **[-0.045, +0.026]** (n=200, bootstrap) |
| Valid | `true` |
=======
| Archive | [`medical-mid-20260802T135513Z`](../../specs/001-benchmark/e2e/artifacts/history/medical-mid-20260802T135513Z/) |
| Publish pack | [`publish/latest/`](../../specs/001-benchmark/e2e/artifacts/publish/latest/) |
| Profile | `P0_mistral_small_mix_chunk1200_v1_lrlike_arms_v2` |
| Fixture | `medical_publish_question_ids_v1` (50 questions / type) |
| Δ Acc 95% CI | **[-0.005, +0.059]** (n=200, bootstrap) |
| Valid | `true` |
| `can_claim_beats_lightrag` | `false` (L2 gates unmet; Acc CI includes 0) |
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042

### By question type (Acc)

| Type | EdgeQuake | LightRAG | Lead |
|------|-----------|----------|------|
<<<<<<< HEAD
| Fact Retrieval | 0.692 | 0.762 | LightRAG |
| Complex Reasoning | 0.769 | 0.770 | Tie |
| Contextual Summarize | 0.845 | 0.812 | EdgeQuake |
| Creative Generation | 0.774 | 0.772 | EdgeQuake |

### Ops (Acc pack)

- Empty-answer rates: EQ 0.5% · LR 0%
- Ingest wall: ~598 s (full medical corpus, chunk 1200/100)
- Warm EQ/LR p50 ratio **4.03×** — LightRAG keyword + answer **LLM cache** hits; not a fair engine claim ([063](../../specs/001-benchmark/001-edgquake-improvements/063-why-lightrag-faster-cache-fairness.md))
=======
| Fact Retrieval | 0.789 | 0.749 | EdgeQuake |
| Complex Reasoning | 0.793 | 0.777 | EdgeQuake |
| Contextual Summarize | 0.833 | 0.830 | EdgeQuake |
| Creative Generation | 0.812 | 0.761 | EdgeQuake |

### Ops (Acc pack)

- Empty-answer rates: EQ 0% · LR 0%
- Ingest wall: warm query-only (full-corpus workspace already present)
- Warm EQ/LR p50 ratio **7.19×** — LightRAG keyword + answer **LLM cache** hits; not a fair engine claim ([063](../../specs/001-benchmark/001-edgquake-improvements/063-why-lightrag-faster-cache-fairness.md))
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042

---

## Fair cold latency peer

| Field | Value |
|-------|--------|
| Peer | [`C1COLD_v1`](../../specs/001-benchmark/e2e/artifacts/publish/peers/C1COLD_v1/) |
| Archive | [`smoke-20260723T134452Z`](../../specs/001-benchmark/e2e/artifacts/history/smoke-20260723T134452Z/) |
| Pin | `BENCH001_LR_ENABLE_LLM_CACHE=0` · query-only · n=40 |
| EQ/LR p50 | 4447 / 4359 ms → **1.02×** (SLO ≤1.5× **PASS**) |

<<<<<<< HEAD
Smoke Acc gate (not release): [`smoke-20260723T132527Z`](../../specs/001-benchmark/e2e/artifacts/history/smoke-20260723T132527Z/) — EQ 0.759 / LR 0.797 · CI includes 0.
=======
Prior Acc pack (point lead, CI excluded 0): [`medical-mid-20260802T132630Z`](../../specs/001-benchmark/e2e/artifacts/history/medical-mid-20260802T132630Z/) — EQ 0.811 / LR 0.778.
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042

---

## Allowed / forbidden claims

| Allowed | Forbidden |
|---------|-----------|
<<<<<<< HEAD
| “Peer / statistical tie with LightRAG on Acc under fair Mistral pins (medical-mid n=200)” | “Beats LightRAG” / Acc Beat / Acc Equal mid Parity |
| “Fair cold Mix latency ≈ LightRAG (~1.02×)” | Warm Acc mid “4× faster LightRAG” as product/engine win |
=======
| “Peer / statistical tie with LightRAG on Acc under fair Mistral pins (medical-mid n=200); L2 still trails LightRAG” | “Beats LightRAG” / Acc Beat / Acc Equal mid Parity |
| “Fair cold Mix latency ≈ LightRAG (~1.02×)” | Warm Acc mid “7× faster LightRAG” as product/engine win |
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
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
