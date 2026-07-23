---
title: 'EQ vs LightRAG — Acc Bench (SPEC-001)'
description: Measured GraphRAG-Bench Acc and fair cold latency for EdgeQuake vs LightRAG.
---

# EdgeQuake vs LightRAG — Acc Bench (SPEC-001)

> **Publish Acc SSOT** · GraphRAG-Bench medical-mid · `make bench` · 2026-07-23

Fair dual-SUT head-to-head: same corpus, questions, Mix mode, Mistral Small + `mistral-embed`, official `generation_eval`. **Not** UltraDomain win-rates · **not** paper Table-2 (GPT-4o-mini + BGE).

Protocol: [SPEC-001 index](../../specs/001-benchmark/000-index.md) · Business brief: [019](../../specs/001-benchmark/019-business-eq-vs-lightrag-and-rag.md)

---

## One-screen verdict

| Layer             | EdgeQuake | LightRAG  | Claim                                 |
| -------------------| -----------| -----------| ---------------------------------------|
| **Acc** (n=200)   | **0.770** | **0.779** | **Statistical tie** (Δ CI includes 0) |
| Evidence recall   | 0.926     | 0.950     | LightRAG                              |
| Context relevancy | 0.408     | 0.511     | LightRAG                              |
| Cold query p50    | 4447 ms   | 4359 ms   | **≈ tied (1.02×)**                    |
| Warm Acc mid p50  | 4089 ms   | 1016 ms   | **Do not claim** — LR LLM cache       |

**Do not claim** “EdgeQuake beats LightRAG,” Acc mid Parity, or SOTA on GraphRAG-Bench. Acc Beat / Acc Equal mid remain **STOP**.

---

## Publish Acc (medical-mid n=200)

| Field | Value |
|-------|--------|
| Archive | [`medical-mid-20260723T134124Z`](../../specs/001-benchmark/e2e/artifacts/history/medical-mid-20260723T134124Z/) |
| Publish pack | [`publish/latest/`](../../specs/001-benchmark/e2e/artifacts/publish/latest/) |
| Profile | `P0_mistral_small_mix_chunk1200_v1_lrlike_arms_v2` |
| Fixture | `medical_publish_question_ids_v1` (50 questions / type) |
| Δ Acc 95% CI | **[-0.045, +0.026]** (n=200, bootstrap) |
| Valid | `true` |

### By question type (Acc)

| Type | EdgeQuake | LightRAG | Lead |
|------|-----------|----------|------|
| Fact Retrieval | 0.692 | 0.762 | LightRAG |
| Complex Reasoning | 0.769 | 0.770 | Tie |
| Contextual Summarize | 0.845 | 0.812 | EdgeQuake |
| Creative Generation | 0.774 | 0.772 | EdgeQuake |

### Ops (Acc pack)

- Empty-answer rates: EQ 0.5% · LR 0%
- Ingest wall: ~598 s (full medical corpus, chunk 1200/100)
- Warm EQ/LR p50 ratio **4.03×** — LightRAG keyword + answer **LLM cache** hits; not a fair engine claim ([063](../../specs/001-benchmark/001-edgquake-improvements/063-why-lightrag-faster-cache-fairness.md))

---

## Fair cold latency peer

| Field | Value |
|-------|--------|
| Peer | [`C1COLD_v1`](../../specs/001-benchmark/e2e/artifacts/publish/peers/C1COLD_v1/) |
| Archive | [`smoke-20260723T134452Z`](../../specs/001-benchmark/e2e/artifacts/history/smoke-20260723T134452Z/) |
| Pin | `BENCH001_LR_ENABLE_LLM_CACHE=0` · query-only · n=40 |
| EQ/LR p50 | 4447 / 4359 ms → **1.02×** (SLO ≤1.5× **PASS**) |

Smoke Acc gate (not release): [`smoke-20260723T132527Z`](../../specs/001-benchmark/e2e/artifacts/history/smoke-20260723T132527Z/) — EQ 0.759 / LR 0.797 · CI includes 0.

---

## Allowed / forbidden claims

| Allowed | Forbidden |
|---------|-----------|
| “Peer / statistical tie with LightRAG on Acc under fair Mistral pins (medical-mid n=200)” | “Beats LightRAG” / Acc Beat / Acc Equal mid Parity |
| “Fair cold Mix latency ≈ LightRAG (~1.02×)” | Warm Acc mid “4× faster LightRAG” as product/engine win |
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
