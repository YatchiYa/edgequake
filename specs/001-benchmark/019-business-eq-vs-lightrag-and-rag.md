# How EdgeQuake Compares to LightRAG and Other RAG

**Audience:** Business / product stakeholders  
<<<<<<< HEAD
**Date:** 2026-07-23 (refresh — Acc publish mid **statistical tie** · cold latency **1.02×** · product Equal **083** · Acc Beat / Acc Equal mid **STOP**)  
=======
**Date:** 2026-08-02 (refresh — Acc publish mid **point lead** CI excludes 0 · L2 incomplete · cold latency **1.02×** · product Equal **083** · Acc Beat / Acc Equal mid **STOP**)  
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
**Evidence base:** Fair head-to-head Acc on GraphRAG-Bench medical publish mid (n=200) · smoke n=40 peers for labeled Acc Fact / L2 packs · [peers.json](./e2e/artifacts/peers.json) · [Acc bench doc](../../docs/comparisons/eq-vs-lightrag-acc-bench.md) · [085 fairness Equal STOP](./001-edgquake-improvements/085-fairness-concurrency-equal-stop.md) · [083 product query API](./001-edgquake-improvements/083-lightrag-query-api-law.md) · [082 gold/citation](./001-edgquake-improvements/082-gold-citation-compat.md) · [081 Beat/Parity](./001-edgquake-improvements/081-beat-parity-first-principles.md) · [080 Beat roadmap](./001-edgquake-improvements/080-beat-lightrag-evidence-roadmap.md) · [055 First Principles hub](./001-edgquake-improvements/055-post-acc-ceiling-first-principles.md) · [063 why LR looked faster](./001-edgquake-improvements/063-why-lightrag-faster-cache-fairness.md) · [043 honesty](./001-edgquake-improvements/043-honesty-can-we-push.md)

---

## In one minute

| Question | Plain answer |
|----------|----------------|
<<<<<<< HEAD
| Does EdgeQuake beat LightRAG on answer quality today? | **No.** Publish Acc is a **statistical tie** (EQ 0.770 vs LR 0.779; CI includes 0). Do not claim “EdgeQuake beats LightRAG.” |
| Do we equal LightRAG anywhere? | **Yes — product query API** ([083](./001-edgquake-improvements/083-lightrag-query-api-law.md)): `hl_keywords`/`ll_keywords` skip keyword LLM + system/user generate. **Not** Acc mid Parity (ctx still &lt;0.50). |
| What is Acc CI “keep”? | Publish Acc mid is itself a **tie**. Labeled gap-close E2 occ mid Acc also ties (0.765, CI includes 0). Still **not** Parity. |
| What is the publish sample? | **n=200 medical-mid** (`make bench`). Smoke n=40 is a daily gate only — first principles forbid publishing smoke as the release score. |
| Is EdgeQuake a serious GraphRAG peer? | **Yes** — Acc-tied peer under fair Mistral pins; cold Mix latency ≈ LightRAG (**1.02×**). Product stack (Postgres, API, PDF) is a different job. |
| How do we report fairly? | **Split peers** — never one unlabeled “winner.” Product Equal ≠ Acc CI keep ≠ Acc headline ≠ L2 Parity ≠ warm-cache latency. |
| Where is LightRAG ahead? | **L2** on publish Acc (recall 0.950 vs 0.926; ctx 0.511 vs 0.408). **Fact Acc** on publish mid. **Scale** medical-full n=2062 historically: LR Acc CI excludes 0. Warm “~4× faster” was **LLM-cache-aided**. |
| Where is EdgeQuake ahead? | **Product/platform** (API, Postgres, multi-tenant) · **query API law** (083) · Summarize / Creative Acc on today’s publish mid · Fact Acc on labeled Acc Fact peer. |
| Are we “state of the art” vs all RAG? | **Peer on Acc with LightRAG; not retrieval SOTA.** HippoRAG2-class leads on clean+complete retrieval. |

**Bottom line:** Ship **product Equal LightRAG** on the query contract ([083](./001-edgquake-improvements/083-lightrag-query-api-law.md)). Acc mid Parity / Beat remain **STOP**. Publish Acc is a **tie** — report as peer, never Beat. Next investment: **UX (TTFT) / product caches** and L2 cleanliness — not Acc Beat fishing.
=======
| Does EdgeQuake beat LightRAG on answer quality today? | **No Acc Beat claim.** Publish Acc shows an **Acc point lead** (EQ 0.811 vs LR 0.778; CI excludes 0) but **L2 incomplete** (ctx 0.415 &lt; 0.50). Do not claim “EdgeQuake beats LightRAG.” |
| Do we equal LightRAG anywhere? | **Yes — product query API** ([083](./001-edgquake-improvements/083-lightrag-query-api-law.md)): `hl_keywords`/`ll_keywords` skip keyword LLM + system/user generate. **Not** Acc mid Parity (ctx still &lt;0.50). |
| What is Acc CI “keep”? | Publish Acc mid CI excludes 0 with EQ ahead on Acc — still **not** Parity / Beat without L2 gates ([080](./001-edgquake-improvements/080-phase-g-promote-checklist.md)). |
| What is the publish sample? | **n=200 medical-mid** (`make bench`). Smoke n=40 is a daily gate only — first principles forbid publishing smoke as the release score. |
| Is EdgeQuake a serious GraphRAG peer? | **Yes** — Acc-competitive under fair Mistral pins; cold Mix latency ≈ LightRAG (**1.02×**). Product stack (Postgres, API, PDF) is a different job. |
| How do we report fairly? | **Split peers** — never one unlabeled “winner.” Product Equal ≠ Acc CI keep ≠ Acc headline ≠ L2 Parity ≠ warm-cache latency. |
| Where is LightRAG ahead? | **L2** on publish Acc (recall 0.946 vs 0.904; ctx 0.498 vs 0.415). Warm “~6× faster” was **LLM-cache-aided**. |
| Where is EdgeQuake ahead? | **Product/platform** (API, Postgres, multi-tenant) · **query API law** (083) · Acc by-type on today’s publish mid (all four types point-lead). |
| Are we “state of the art” vs all RAG? | **Competitive on Acc with LightRAG; not retrieval SOTA.** HippoRAG2-class leads on clean+complete retrieval. |

**Bottom line:** Ship **product Equal LightRAG** on the query contract ([083](./001-edgquake-improvements/083-lightrag-query-api-law.md)). Acc mid Parity / Beat remain **STOP**. Publish Acc may show a point lead — report Acc honestly, never Beat without L2. Next investment: **UX (TTFT) / product caches** and L2 cleanliness — not Acc Beat fishing.
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042

### Split peers (required reading)

| Peer | What it optimizes | Headline numbers |
|------|-------------------|------------------|
<<<<<<< HEAD
| **Acc headline** (P0 mid) | Publish Acc SSOT | EQ Acc **0.770** · LR **0.779** · CI [−0.045, +0.026] · [T134124Z](./e2e/artifacts/history/medical-mid-20260723T134124Z/) |
=======
| **Acc headline** (P0 mid) | Publish Acc SSOT | EQ Acc **0.811** · LR **0.778** · CI [+0.001, +0.065] · [T132630Z](./e2e/artifacts/history/medical-mid-20260802T132630Z/) |
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
| **Cold latency** (`c1cold`) | Fair EQ/LR p50 | ratio **1.02×** · [T134452Z](./e2e/artifacts/history/smoke-20260723T134452Z/) · peer `C1COLD_v1` |
| **Product Equal** ([083](./001-edgquake-improvements/083-lightrag-query-api-law.md)) | Query API = LightRAG (hl/ll + chat roles) | keyword_time_ms **909→0** · peer `PRODUCT_QUERY_API_v1` |
| **Acc CI keep** (E2 occ mid) | Labeled Acc statistical tie | EQ Acc **0.765** · CI [−0.031, +0.040] · ctx 0.491 · [T133053Z](./e2e/artifacts/history/medical-mid-20260722T133053Z/) — **not** Parity |
| **Acc Fact** (`a1fp` / B5, n=40) | Answer quality + Fact | EQ Acc **0.801** · ctx 0.519 · [T120315Z](./e2e/artifacts/history/smoke-20260720T120315Z/) |
| **L2 Parity** (`a1lrl2`, n=40) | Evidence recall + clean context | EQ Acc 0.718 (tax) · recall **0.933** · [T093152Z](./e2e/artifacts/history/smoke-20260720T093152Z/) |

Machine index: [`e2e/artifacts/peers.json`](./e2e/artifacts/peers.json).
---

## What we compared (fair rules)

We asked the same medical questions of:

1. **EdgeQuake** (Mix mode — combines graph + vector retrieval)  
2. **LightRAG** (Mix mode — the peer open-source GraphRAG)

Same documents, same question set, same language model family (Mistral Small), same embedding model, same scoring method. That is the only comparison we treat as publishable for “EQ vs LightRAG.”

We did **not** claim wins on UltraDomain leaderboards, paper Table-2 with different models, or unbenchmarked demos.

---

## Scorecard in business language

Think of three layers:

```text
  Question
     →  Retriever finds passages / graph facts   (retrieval quality)
     →  Model writes an answer                   (generation)
     →  We score the answer vs gold              (Acc = answer quality)
```

| Layer | What it means for the business | EdgeQuake vs LightRAG (fair Acc) |
|-------|--------------------------------|----------------------------------|
<<<<<<< HEAD
| **Answer quality (Acc)** | “Are answers roughly as good?” | **Statistical tie** on publish mid (0.770 vs 0.779) |
| **Evidence coverage** | “Did we find the right source material?” | LightRAG ahead on publish (0.950 vs 0.926); labeled L2 Parity peer ~93% EQ |
| **Context cleanliness** | “Is the prompt full of noise that confuses the model?” | LightRAG ahead on publish (0.511 vs 0.408); do not claim product L2 parity |
| **Speed** | “How long until the user sees an answer?” | **Fair cold Mix ≈ tied** (1.02×, [T134452Z](./e2e/artifacts/history/smoke-20260723T134452Z/)). Warm Acc LR ~1s was answer/keyword **cache**. |
=======
| **Answer quality (Acc)** | “Are answers roughly as good?” | Acc **point lead** on publish mid (0.811 vs 0.778; CI excludes 0) — **not** Beat without L2 |
| **Evidence coverage** | “Did we find the right source material?” | LightRAG ahead on publish (0.946 vs 0.904) |
| **Context cleanliness** | “Is the prompt full of noise that confuses the model?” | LightRAG ahead on publish (0.498 vs 0.415); do not claim product L2 parity |
| **Speed** | “How long until the user sees an answer?” | **Fair cold Mix ≈ tied** (1.02×, [T134452Z](./e2e/artifacts/history/smoke-20260723T134452Z/)). Warm Acc LR sub-second was answer/keyword **cache**. |
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
| **Product / ops** | API, database, tenancy, PDF pipeline, UI | **EdgeQuake’s strength** as a deployable stack — different job from a research library |

### By question type (where each system feels stronger)

<<<<<<< HEAD
Publish medical-mid n=200 ([T134124Z](./e2e/artifacts/history/medical-mid-20260723T134124Z/)): Acc overall **tied**. LightRAG leads **Fact**; EdgeQuake leads **Summarize** / **Creative**; Complex is a dead heat.

| User need | Who leads on publish Acc | Takeaway |
|-----------|--------------------------|----------|
| Simple fact lookup | **LightRAG** | Close provenance / VECTOR chunk pick |
| Multi-hop reasoning | **Tie** | Keep Mix arms + packing honest |
| Long summarization | **EdgeQuake** | Coverage strength on this pin |
| Creative / open-ended | **EdgeQuake** | Small edge; do not overclaim |
=======
Publish medical-mid n=200 ([T132630Z](./e2e/artifacts/history/medical-mid-20260802T132630Z/)): Acc overall **point lead** for EdgeQuake on all four types; LightRAG still leads **L2**.

| User need | Who leads on publish Acc | Takeaway |
|-----------|--------------------------|----------|
| Simple fact lookup | **EdgeQuake** | Acc lead; L2 provenance still LR-favored |
| Multi-hop reasoning | **EdgeQuake** | Keep Mix arms + packing honest |
| Long summarization | **EdgeQuake** | Coverage strength on this pin |
| Creative / open-ended | **EdgeQuake** | Small edge; do not overclaim Beat |
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042

---

## How this sits vs “other RAG” (July 2026 landscape)

Industry research on [GraphRAG-Bench](https://github.com/GraphRAG-Bench/GraphRAG-Benchmark) (ICLR 2026) is clear: **classic vector RAG is often enough for simple facts**; graphs help most when answers need **multi-hop reasoning or synthesis across documents**. EdgeQuake and LightRAG are both GraphRAG-family systems in that debate.

RAG is not one product. Rough map for executives:

| Family | What it is | Where EdgeQuake sits |
|--------|------------|----------------------|
| **Classic vector RAG** | Embed chunks → top-k → LLM | EdgeQuake includes this (naive arm) but adds a knowledge graph |
| **GraphRAG (general)** | Entities/relations + text to help multi-hop | EdgeQuake and LightRAG are both in this family |
| **LightRAG** | Popular dual-level GraphRAG; strong UX/dev adoption | **Peer on Acc** under our fair test; cold latency ≈ EQ when caches match |
| **Microsoft GraphRAG / RAPTOR / Fast-GraphRAG** | Other graph or hierarchy approaches on the same research suite | Directional peers in the GraphRAG-Bench conversation — we did not re-run their paper pins head-to-head here |
| **HippoRAG2-class** | High evidence recall **and** high context relevancy with compact prompts | **Aspirational retrieval SOTA** on this task family — the quality bar for “less noise, still complete” |

**Important honesty note:** Absolute Acc numbers from the academic paper (GPT-4o-mini + BGE) are **not** directly comparable to our Mistral Acc runs. Use relative lessons (“who is cleaner / faster / better at multi-hop”), not raw score copy-paste.

---

## What we improved (and what we did not claim)

We ran a disciplined improvement program (noise control, reranking, path pruning, Mix weights). Results in plain English:

| Outcome | Meaning |
|---------|---------|
| **L2 / context quality improved** under a labeled advanced profile | We can pack less noise while keeping answer quality near baseline |
| **Publish Acc is a statistical tie** vs LightRAG | Do **not** promote to Beat; CI still includes 0 |
| **Headline product defaults unchanged** | Default Acc path stays conservative; advanced retrieval stays opt-in / labeled |

Allowed external language (recommended):

- “EdgeQuake equals LightRAG on the **product query API** (hl/ll keyword override + system/user generate) — [083](./001-edgquake-improvements/083-lightrag-query-api-law.md).”  
- “Under fair Acc mid pins, answer Acc is a **statistical tie** with LightRAG — not Beat, not Acc Equal mid Parity.”  
- “Evidence-coverage parity is a separate labeled pack (L2 Parity) with an Acc trade-off — we disclose both.”  
- “Peer GraphRAG system with a full production stack (Postgres, API, document pipeline).”  
- “Fair cold Mix latency is ≈ LightRAG (≤1.5× PASS); warm Acc LR looks faster only with LLM cache hits.”

Avoid:

- “Beats LightRAG” / “wins Acc” / “Equal on Acc mid Parity” (ctx≥0.50 unmet; Acc Equal STOP)  
- Merging Product Equal, Acc CI keep, Acc Fact, and L2 Parity into one unlabeled claim  
- “#1 on GraphRAG-Bench” without matching the paper’s model and full evaluation protocol  

---

## When to choose EdgeQuake vs LightRAG vs simpler RAG

| If you need… | Prefer |
|--------------|--------|
| Production RAG with tenancy, Postgres, PDF→Markdown, REST/UI | **EdgeQuake** |
| Fastest Mix-style GraphRAG experiment / library-first workflow | **LightRAG** (or EQ if you already standardize on our stack) |
| Lowest cost / latency for pure fact lookup | Often **lean vector RAG** (or EQ with fact-oriented routing later) |
| Best published multi-hop + high relevancy on GraphRAG-Bench-class tasks | Study **HippoRAG2-class** designs; EQ roadmap points there as research, not current Acc default |

---

## Risks and next investments (business priority)

**Current program (post Acc-ceiling):** **[055 First Principles hub](./001-edgquake-improvements/055-post-acc-ceiling-first-principles.md)** — Acc Beat fishing STOP.

1. **Ship split peers honestly** — Acc Fact vs L2 Parity ([`peers.json`](./e2e/artifacts/peers.json)); never one unlabeled winner.  
2. **Naming / ingest fidelity** — [056](./001-edgquake-improvements/056-naming-identity-lr-parity.md) short-numeric law (Acc re-ingest deferred).  
3. **Latency honesty** — [063](./001-edgquake-improvements/063-why-lightrag-faster-cache-fairness.md) cold ≈1.0×; product polish [064](./001-edgquake-improvements/064-product-ttft-cache-batch-embed.md) (TTFT + opt-in caches) — not Acc Beat.  
4. **Do not claim Beat** — CI still includes 0 on n=40; Soft Mix Acc knobs exhausted.  

Historical ladder (E0–E4 / P0–P5): [022](./001-edgquake-improvements/022-deep-top-performance-plan.md) · [028](./001-edgquake-improvements/028-first-principles-beat-roadmap.md).

---

## Run the benchmark (publish pack)

```bash
make bench          # medical-mid Acc n=200 + business report
make bench-warm     # query-only (auto latest warm EQ workspace)
```

Measured Acc scorecard: [docs/comparisons/eq-vs-lightrag-acc-bench.md](../../docs/comparisons/eq-vs-lightrag-acc-bench.md)

Latest stakeholder pack (regenerated each run):

- [BUSINESS_REPORT.md](./e2e/artifacts/publish/latest/BUSINESS_REPORT.md)
- [EXEC_SUMMARY.txt](./e2e/artifacts/publish/latest/EXEC_SUMMARY.txt)

## Pointers for deeper reading

| Doc | For whom |
|-----|----------|
| This page | Business / GTM / product (static brief) |
| [publish/latest/BUSINESS_REPORT.md](./e2e/artifacts/publish/latest/BUSINESS_REPORT.md) | Latest run — regenerate with `make bench` |
| [011 Publication Acc Report](./011-publication-acc-report.md) | Technical leaders / reviewers |
| [018 E4 Acc-tie close](./001-edgquake-improvements/018-e4-acc-tie-close.md) | Exact claim language and CI ledger |
| [022 Deep top-performance](./001-edgquake-improvements/022-deep-top-performance-plan.md) | Engineering Acc recovery program P0–P5 |
| [017 Beat LightRAG](./001-edgquake-improvements/017-beat-lightrag.md) | Engineering architecture differences |
| [020 Roadmap](./001-edgquake-improvements/020-roadmap.md) | Phases and Acc ladder |
| [021 Grounded plan](./001-edgquake-improvements/021-grounded-improvement-plan.md) | F1–F4 cleanliness · multi-hop · latency |

---

## Glossary (30 seconds)

| Term | Meaning |
|------|---------|
| **RAG** | Retrieval-Augmented Generation — look up text, then ask an LLM |
| **GraphRAG** | RAG that also uses a knowledge graph (entities & relations) |
| **Acc** | Our answer-quality score (blend of factual match + semantic similarity) |
| **Statistical tie** | Score difference is too small to trust as a real win given sample size |
| **Context relevancy** | How on-topic the retrieved text is (low noise) |
| **Evidence recall** | Whether gold supporting text was retrieved at all |
