# 001-edgquake-improvements — Multi-Lens EdgeQuake Improvement Pack

**Status:** Acc-win E0–E4 **closed** · **034 L2 Parity** `[T093152Z](../e2e/artifacts/history/smoke-20260720T093152Z/)` · **044 B5 Acc peer** `[T120315Z](../e2e/artifacts/history/smoke-20260720T120315Z/)` Acc **0.801** · prior 035 peer frozen · Beat not met  
**Date:** 2026-07-20  
**Baseline / S1:** `[T124903Z](../e2e/artifacts/history/smoke-20260719T124903Z/)` · `[T151125Z](../e2e/artifacts/history/smoke-20260719T151125Z/)`  
**Contaminated publish:** `[T011703Z](../e2e/artifacts/history/smoke-20260720T011703Z/)` (BM25+path=0.4 Acc loss — fixed by 022 P0)  
**E4 close:** [018 E4 Acc-tie close](./018-e4-acc-tie-close.md) · ladder [017](./017-beat-lightrag.md)  
**Cross-ref:** [028](./028-first-principles-beat-roadmap.md) · [035 Fact CE∩BM25](./035-fact-ce-bm25-protect.md) · [034 L2 dual-list](./034-l2-dual-list-under-full-ws-graph.md) · [033](./033-denser-graph-mix-packing.md) · [032](./032-workspace-graph-identity.md) · [001 First Principles (eval)](../001-first-principles.md) · [005 Mode Map & Pins](../005-mode-map-and-pins.md)

---



## One-screen north star

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│  Acc-win CLOSED: EQ vs LR = persistent STATISTICAL TIE (all Δ Acc CIs        │
│  include 0). Soft Mix knobs exhausted. Headline stays BM25/PRUNE=0.          │
│                                                                              │
│  Acc Beat fishing STOP. Split peers = publishable truth (peers.json).         │
│  Acc B5+a1fp 0.801 · latency C1b/C1d: generate ceiling; keyword=0 no wall win │
│  Latency truth: warm LR cache → false 4×; cold c1cold ≈ 1.01× PASS (063)     │
│  Product polish 064: TTFT + opt-in answer cache + batch embed (not Acc Beat) │
│  Detail: 028 · 043 · 055–064                                                 │
└──────────────────────────────────────────────────────────────────────────────┘
```

---



## Stop rules (hard)


| Rule   | Meaning                                                                                                                                                                                                                                                                  |
| ------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| **S1** | Do not stack Acc ablations until **EQ ctx_rel ≥ 0.50** on n=40 smoke under publication fairness pins, with Acc drop ≤ 0.02 and recall drop ≤ 0.03 — **or** Δ Acc 95% CI excludes 0. **Cleared** by labeled package `T151125Z` (see [020 §1b](./020-roadmap.md)).         |
| **S2** | Acc fairness pins stay frozen for headline runs: `MIX_ARM_GATE=false`, `RELATED_CHUNK_NUMBER=5`, `MIX_FUSION=rrf`, chunk 1200/100 adaptive off, mistral-small + mistral-embed, top-k 30. CE/protect stay **labeled** — Phase 2 Acc tie + L2 variance → **no promotion**. |
| **S3** | One confound per experiment. Label every pin change in `scorecard.pins` / SUMMARY / `ABLATION_NOTE.md`.                                                                                                                                                                  |
| **S4** | Acc without L2 is not a publishable RAG claim (eval P12).                                                                                                                                                                                                                |


---



## Winning Phase 1 pins (labeled — not headline default)

```bash
EDGEQUAKE_MIX_RELEVANCY_PRUNE=0
EDGEQUAKE_RERANKER=cross_encoder
EDGEQUAKE_RERANKER_PROVIDER=aliyun
EDGEQUAKE_RERANKER_MODEL=qwen3-rerank          # DashScope intl
EDGEQUAKE_PATH_PRUNE=0
EDGEQUAKE_RERANK_PROTECT_FIRST=12              # CE order; Mix top-12 guaranteed in set
BENCH001_EQ_RERANK_TOP_K=30
# + DASHSCOPE_API_KEY; rebuild: cargo build --release --bin edgequake
```

Reproduce: warm query-only Acc against workspace from baseline ingest, or force-ingest under the same fairness pins.

---



## Reading order

1. **[019 Business brief](../019-business-eq-vs-lightrag-and-rag.md)** — EQ vs LightRAG & other RAG (non-technical)
2. [001 First Principles](./001-first-principles.md) — what to optimize vs ignore
3. **[018 E4 Acc-tie close](./018-e4-acc-tie-close.md)** — Acc-win honesty verdict (persistent tie)
4. **[028 First-principles beat roadmap](./028-first-principles-beat-roadmap.md)** — gap + Horizons A/B/C
5. **[027 Fact→BM25 intent](./027-fact-bm25-intent-rerank.md)** — Fact lexical under P2b (no promote)
6. **[026 L2 Mix∪CE under P2b](./026-l2-sources-union-under-p2b.md)** — dual-list (no promote; Fact flat)
7. **[025 Recall under P2b](./025-recall-parity-under-p2b.md)** — CE protect↑ / min_rerank / chunk floor (no promote)
8. **[024 Acc parity / beat](./024-acc-parity-beat-plan.md)** — Q0–Q4 close (no promote; peer = P2b)
9. **[022 Deep top-performance plan](./022-deep-top-performance-plan.md)** — P0–P5 hard paths after T011703Z
10. **[021 Grounded plan F1–F4](./021-grounded-improvement-plan.md)** — truncation · path pack · latency · passage
11. **[017 Beat LightRAG](./017-beat-lightrag.md)** — EQ↔LR architecture diff + Acc-win ladder E0–E4
12. Lenses by priority (P0 → P3, then product/eval):
  - [010 Retrieval noise](./010-lens-retrieval-noise.md) **P0** — S1 cleared  
  - [011 Evidence coverage](./011-lens-evidence-coverage.md) **P1**  
  - [012 Multi-hop / graph](./012-lens-multihop-graph.md) **P2** — Complex packing (E2)  
  - [013 Latency / ops](./013-lens-latency-ops.md) **P3** — L1/L2 shipped; F3/P5 next  
  - [014 Generation & routing](./014-lens-generation-routing.md)  
  - [015 Ingest & chunking](./015-lens-ingest-chunking.md)  
  - [016 Eval & fairness](./016-lens-eval-fairness.md)
13. [020 Roadmap](./020-roadmap.md) — phases, ablation ladder · **028 supersedes next Acc steps**

---



## Document map


| #   | Doc                                                                       | Lens / purpose                                              |
| --- | ------------------------------------------------------------------------- | ----------------------------------------------------------- |
| 000 | This index                                                                | Hub, north star, stop rules, S1 package                     |
| 001 | [First Principles](./001-first-principles.md)                             | Decomposition laws + baseline vs S1 snapshot                |
| 010 | [Retrieval noise](./010-lens-retrieval-noise.md)                          | P0 Context Relevancy · Phase 1 results                      |
| 011 | [Evidence coverage](./011-lens-evidence-coverage.md)                      | P1 Evidence Recall                                          |
| 012 | [Multi-hop / graph](./012-lens-multihop-graph.md)                         | P2 Reasoning selection                                      |
| 013 | [Latency / ops](./013-lens-latency-ops.md)                                | P3 Query/ingest SLO                                         |
| 014 | [Generation & routing](./014-lens-generation-routing.md)                  | Acc by type + product router                                |
| 015 | [Ingest & chunking](./015-lens-ingest-chunking.md)                        | Contextual embed / adaptive                                 |
| 016 | [Eval & fairness](./016-lens-eval-fairness.md)                            | Ablation discipline                                         |
| 017 | [Beat LightRAG](./017-beat-lightrag.md)                                   | EQ↔LR diff · Acc-win ladder E0–E4                           |
| 018 | [E4 Acc-tie close](./018-e4-acc-tie-close.md)                             | Persistent Acc tie · publish language · deferred hard paths |
| 020 | [Roadmap](./020-roadmap.md)                                               | Phases, gates, code hooks, Acc ladder                       |
| 021 | [Grounded plan F1–F4](./021-grounded-improvement-plan.md)                 | Truncation · path pack · latency · passage pack             |
| 022 | [Deep top-performance](./022-deep-top-performance-plan.md)                | P0 PATH=0 · P1 gw-compress · P2 LR pack · P3–P5             |
| 023 | [P4 Acc CI decision gate](./023-p4-acc-ci-decision-gate.md)               | Promote-only-if CI + ctx_rel ≥0.50                          |
| 024 | [Acc parity / beat](./024-acc-parity-beat-plan.md)                        | Q0 P2b×3 · Q1/Q2 Fact VECTOR · Q3/Q4 promote                |
| 025 | [Recall under P2b](./025-recall-parity-under-p2b.md)                      | CE protect↑ · min_rerank0 · chunk floor (no promote)        |
| 026 | [L2 Mix∪CE under P2b](./026-l2-sources-union-under-p2b.md)                | Dual-list sources · S0 no promote                           |
| 027 | [Fact→BM25 intent](./027-fact-bm25-intent-rerank.md)                      | Fact lexical under P2b · T0/T1                              |
| 028 | [First-principles beat roadmap](./028-first-principles-beat-roadmap.md)   | Gap analysis · Horizons A/B/C · A0–A4 ladder                |
| 029 | [Ingest parity audit](./029-ingest-parity-audit.md)                       | B1 EQ↔LR extract / source_id                                |
| 030 | [Ingest gleaning parity](./030-ingest-gleaning-parity.md)                 | B2 markdown + glean — Acc↑ / L2 miss                        |
| 031 | [Structure-aware chunking](./031-structure-aware-chunking.md)             | B3 FAQ induction + extract density                          |
| 032 | [Workspace graph identity](./032-workspace-graph-identity.md)             | B3b `{ws}::NAME` AGE node_id                                |
| 033 | [Denser-graph Mix packing](./033-denser-graph-mix-packing.md)             | LR token caps 6k/8k under full WS graph                     |
| 034 | [L2 dual-list under full WS](./034-l2-dual-list-under-full-ws-graph.md)   | Citation budget fix + a1l2 Mix∪CE                           |
| 035 | [Fact CE∩BM25 protect](./035-fact-ce-bm25-protect.md)                     | Fact BM25 first-stage for CE protect (no dual-list)         |
| 036 | [a1fp recall without dual-list](./036-a1fp-recall-without-dual-list.md)   | min_rerank=0 / cov protect — closed FAIL                    |
| 037 | [Summarize chunk-link audit](./037-summarize-chunk-link-audit.md)         | Horizon B FP audit · law SELECT                             |
| 038 | [Topic-entity admit Exploratory](./038-topic-entity-admit-exploratory.md) | SELECT confound · `a1fpsel` REJECT                          |
| 039 | [Topic CE/fuse protect](./039-topic-ce-protect-exploratory.md)            | CE id-protect · `a1fpce` REJECT                             |
| 040 | [Topic trunc/pack protect](./040-topic-trunc-protect-exploratory.md)      | Pack prefer · `a1fptrunc` REJECT                            |
| 041 | [Topic chunk fidelity audit](./041-topic-chunk-fidelity-audit.md)         | CE_GAP · CONTENT∉Mix C                                      |
| 042 | [Topic chunk materialize](./042-topic-chunk-materialize.md)               | KV into Mix · `a1fpmat` REJECT                              |
| 043 | [Honesty: can we push?](./043-honesty-can-we-push.md)                     | SELECT ladder stop · one CONTENT-gated leftover             |
| 044 | [B5 placeholder provenance](./044-horizon-b-placeholder-provenance.md)    | Relation stub `source_chunk_ids` inherit                    |
| 045 | [CONTENT-gated materialize](./045-content-gated-materialize.md)           | Sum ER SELECT · `a1fpcmat` REJECT                           |
| 046 | [Answer specificity prompt](./046-answer-specificity-prompt.md)           | Complex Acc gen · `a1fpspec` REJECT                         |
| 047 | [Type-scoped specificity](./047-type-scoped-specificity.md)               | Complex-only · `a1fpscx` REJECT · specificity STOP          |
| 048 | [Summarize-only materialize](./048-summarize-only-materialize.md)         | Sum ER✓ · Fact admit tax · `a1fpsumx` REJECT                |
| 049 | [Rel dedupe source-chunk union](./049-rel-dedup-source-chunk-union.md)   | B6 STRUCT✓ ge2↑ · Acc REJECT · keep B5 peer                 |
| 050 | [Placeholder VDB parity](./050-placeholder-vdb-parity.md)                 | B7 STRUCT✓ age/vdb=1.0 · Acc REJECT · keep B5 peer          |
| 051 | [Relation rank+weight select](./051-relation-rank-weight-select.md)       | Law✓ · Acc REJECT 0.761 · keep B5 peer · flag default off   |
| 052 | [Rel chunk ids query parity](./052-rel-chunk-ids-query-parity.md)         | Law✓ · B6 Acc 0.759 REJECT · keep B5 peer                   |
| 053 | [Entity types LR parity](./053-entity-types-lr-parity.md)                 | Law✓ · B8 Acc 0.748 REJECT · keep B5 peer                   |
| 054 | [Extract caps LR parity](./054-extract-caps-lr-parity.md)                 | Law✓ · B9 Acc 0.745 REJECT · nodes↓3950 · keep B5 peer      |
| 055 | [Post Acc-ceiling FP hub](./055-post-acc-ceiling-first-principles.md)     | Split peers · naming · latency program                      |
| 056 | [Naming identity LR](./056-naming-identity-lr-parity.md)                  | Short-numeric filter · tests✓ · B10 Acc deferred            |
| 057 | [Latency Horizon C baseline](./057-latency-horizon-c-baseline.md)         | Acc ~5.1× · L2 ~4.9× · C1 recipe · not Acc promote          |
| 058 | [C1a Fact CE-skip latency](./058-c1a-fact-ce-skip-latency.md)             | Measured T012849Z · Fact rerank 9ms · 4.35× · not Acc       |
| 059 | [C1b latency ceiling](./059-c1b-latency-ceiling-keyword-embed.md)         | T013842Z · keyword 1782 · gen 2421 · 3.91× · SLO ceiling    |
| 060 | [C1d heuristic KEYWORD](./060-c1d-heuristic-keyword-latency.md)           | T014632Z · keyword 0 · wall flat · prefer fast KEYWORD LLM  |
| 061 | [LR-as-law FP ideas](./061-lightrag-law-first-principles-eq.md)           | KEYWORD≠QUERY · 1-batch embed · TTFT · packs c1e/f/g        |
| 062 | [C1e fast KEYWORD LLM](./062-c1e-fast-keyword-llm.md)                    | T020802Z · env Law✓ · wall REJECT (vs warm LR)              |
| 063 | [Why LR faster / cache fairness](./063-why-lightrag-faster-cache-fairness.md) | T022103Z cold ≈1.01× · warm 4× was LLM cache            |
| 064 | [Product TTFT / cache / batch embed](./064-product-ttft-cache-batch-embed.md) | Stream TTFT · opt-in answer cache · batch embed cache   |
| 065 | [Vision fail-loud + size timeout](./065-vision-fail-loud-deterministic-timeout.md) | Explicit Vision fail-closed · page-scaled budget · no flake |


---



## Lens template (every lens file)

1. **Observation** — numbers from baseline / latest archive
2. **First-principles diagnosis** — which law is violated
3. **July 2026 practice** — GraphRAG-Bench / HippoRAG2 / PathRAG / hybrid+rerank
4. **EQ insertion points** — concrete files/functions
5. **Experiments** — one confound + success criteria (+ status when known)
6. **Non-goals** — what not to touch under Acc pins

---



## Launch & watch

```bash
make bench001-full          # publication Acc under forced fairness pins
make bench001-watch STAGE=smoke
# Labeled S1 package: export CE/protect env above, restart Acc backend, then bench001-full
```

Artifacts land under `specs/001-benchmark/e2e/artifacts/` and archive to `history/`.