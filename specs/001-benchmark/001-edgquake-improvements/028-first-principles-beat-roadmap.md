# 028 — First-Principles Beat Roadmap (EQ vs LightRAG)

**Status:** Active hub (post 024–027)  
**Date:** 2026-07-20  
**Warm / Acc peer:** `8e990410-43b5-44f4-9f56-87bd154570ce` (B5 placeholder provenance + a1fp [`T120315Z`](../e2e/artifacts/history/smoke-20260720T120315Z/))  
**Prior Acc peer (frozen):** `2a7bcb2f-…` / T095809Z 0.775 · **B2 Acc peak (frozen):** `e0270f5f-…` / T071732Z 0.785  
**Cross-ref:** [000 Index](./000-index.md) · [017](./017-beat-lightrag.md) · [020](./020-roadmap.md) · [027](./027-fact-bm25-intent-rerank.md) · [031 B3](./031-structure-aware-chunking.md) · LightRAG peer `/Users/raphaelmansuy/Github/03-working/LightRAG`

---

## 1. How far are we?

Under frozen Acc fairness pins (mistral-small + mistral-embed, top-k=30, chunk 1200/100):

| Dimension | **Acc Fact peer** B5+`a1fp` [`T120315Z`](../e2e/artifacts/history/smoke-20260720T120315Z/) | **L2 Parity** `a1lrl2` [`T093152Z`](../e2e/artifacts/history/smoke-20260720T093152Z/) |
|-----------|----------------------------------------------------------------------------------------|----------------------------------------------------------------------------------|
| Overall Acc | EQ **0.801** vs LR 0.782 (Δ **+0.019**, **CI includes 0**) | EQ 0.718 vs LR 0.781 (Δ −0.063, **CI includes 0**) |
| ctx_rel | **0.519** ≥0.50 | **0.525** ≥0.50 |
| evidence_recall | 0.926 (miss LR−0.03) | **0.933** ≥LR−0.03 |
| Fact ER | **0.85** | **0.85** |
| Dual-list | **off** | on (Acc tax) |

**Verdict:** Split peers — **044 B5+`a1fp`** for Acc/Fact (zero-chunk hygiene + Acc↑); **034 `a1lrl2`** for L2 Parity. Unified Beat/Parity still open (CI includes 0; recall gate). Soft Mix / TOPIC_* Acc fishing exhausted.

### Promote gates (frozen)

| Outcome | Gate |
|---------|------|
| **Beat** | Δ Acc CI excludes 0 **EQ** ∧ ctx≥0.50 ∧ recall≥LR−0.03 |
| **Parity** | CI includes 0 ∧ ctx≥0.50 ∧ recall≥LR−0.03 |

---

## 2. Can query-only beat LightRAG?

| Lever | Complex | Recall | Latency |
|-------|---------|--------|---------|
| Soft Mix knobs | Exhausted | Partial (T0d) | No |
| Query selection / context serialization | **Primary** | Secondary | Neutral/− if CE |
| Dual-list L2 Fact BM25 | No | Fact yes | Neutral |
| Answer / keyword **prompts** | Acc F1 | Indirect | Neutral |
| **Ingest** graph / chunk links | Indirect | Hard misses | Ingest wall |
| Concurrency / CE skip | No | No | **Yes** |

**First principles:** L0 Acc = generation over admitted context; L2 = membership + salience; latency = query ops. Query is necessary; ingest sets the ceiling; prompts convert context into Acc.

---

## 3. Horizons

```text
  Horizon A (query+prompt) → Parity gates
       │ miss Complex
       ▼
  Horizon B (ingest audit+re-ingest) → raise L2 ceiling
       │
       ▼
  Beat CI package (promote only if gates)

  Horizon C (latency ∥ product routing) — not Acc CI blocker
```

### Horizon A — Query + prompt (aim Parity)

| Step | Work | Env / hook |
|------|------|------------|
| **A0** | P2b stability (baseline) | `make bench001-a0` |
| **A1** | LR-like **relation-first** context (`rr_cer`) under P2b | `EDGEQUAKE_CONTEXT_FORMAT=rr_cer` · `make bench001-a1` |
| **A2** | Fact `query_intent` coverage (LLM prompt + optional bias) | `EDGEQUAKE_INTENT_FACTUAL_BIAS=1` · `make bench001-a2` |
| **A3** | Answer prompt closer to LR `rag_response` | `EDGEQUAKE_ANSWER_PROMPT=lightrag` · `make bench001-a3` |
| **A4** | Acc CI decision | `make bench001-a4` · promote only Beat/Parity |

**A1 success:** Complex Δ vs LR ≤ 0.05 ∧ Acc ≥ 0.736 ∧ ctx≥0.50.

### Horizon B — Ingest (separate workspace)

| Step | Work |
|------|------|
| **B1** | Paired EQ↔LR extract/`source_id` audit — [`tools/bench001/scripts/audit_eq_lr_ingest.py`](../../../tools/bench001/scripts/audit_eq_lr_ingest.py) · [029](./029-ingest-parity-audit.md) |
| **B2** | Gleaning / merge / section breadcrumbs — [030](./030-ingest-gleaning-parity.md) **done** (Acc↑, L2 miss) |
| **B3** | FAQ/structure induction + extract density — [031](./031-structure-aware-chunking.md) |
| **B5** | Relation-placeholder provenance inherit — [044](./044-horizon-b-placeholder-provenance.md) |
| **B6** | Relation dedupe source-chunk **union** — [049](./049-rel-dedup-source-chunk-union.md) |
| **B7** | Placeholder **entities_vdb** parity — [050](./050-placeholder-vdb-parity.md) |
| **B8** | Entity types = LR default (no DATE) — [053](./053-entity-types-lr-parity.md) |
| **B9** | Extract caps 40/100 = LR — [054](./054-extract-caps-lr-parity.md) |

### Horizon C — Latency (deferred from Acc CI)

| Step   | Work                                                      | Gate                                                  |
| --------| -----------------------------------------------------------| -------------------------------------------------------|
| **C1** | Arm concurrency · Fact CE-skip · keyword cache            | EQ/LR p50 ≤ **1.5×** — **not** an Acc promote blocker |
| **C2** | Product type-routing (arm gate on); Acc headline arms-off | Per-type Acc ≥ always-mix                             |

See [013 Latency](./013-lens-latency-ops.md) · §C below.

---

## 4. Stop rules

- No P4 stack · no BM25+path alone · no soft Mix Acc fishing  
- No silent Acc ingest pin changes during query ablations  
- Never claim “beats LightRAG” without Beat gates  
- Do not promote T0d FactReplace as Acc headline  

---

## 5. Ledger

| Step | Archive | EQ Acc | Complex Δ | ctx | recall | Notes |
|------|---------|--------|-----------|-----|--------|-------|
| A1 | [`T061345Z`](../e2e/artifacts/history/smoke-20260720T061345Z/) | 0.772 (Δ−0.014) | **−0.029** | 0.438 ✗ | 0.866 ✗ | Complex PASS; L2 FAIL |
| A2 | [`T062120Z`](../e2e/artifacts/history/smoke-20260720T062120Z/) | 0.714 (Δ−0.058) | −0.113 | 0.456 ✗ | 0.928 ✗ | Acc tax — no promote |
| A3 | [`T062428Z`](../e2e/artifacts/history/smoke-20260720T062428Z/) | 0.739 (Δ−0.035) | −0.090 | **0.519** ✓ | 0.921 ✗ | ctx OK; Acc < A1 |
| A4 | [`T062706Z`](../e2e/artifacts/history/smoke-20260720T062706Z/) | **0.767 (Δ+0.011)** | **−0.017** | 0.494 ✗ | 0.914 ✗ | A1 CI re-run; near Parity; **no promote** |
| B1 | [`ingest-audit/20260720T055323Z`](../e2e/artifacts/ingest-audit/20260720T055323Z/) | — | — | — | — | EQ 429 vs LR 3580; soft≈0.66 → naming gap |
| B2 ingest | [`audit/20260720T070838Z`](../e2e/artifacts/ingest-audit/20260720T070838Z/) | — | — | — | — | EQ 392 · soft 0.640 · zero-chunk 11% — density FAIL |
| B2+A1 | [`T071732Z`](../e2e/artifacts/history/smoke-20260720T071732Z/) | **0.785 (Δ+0.006)** | **−0.006** | 0.494 ✗ | 0.928 ✗ | Best Acc/Complex; L2 miss by hair; **no promote** |
| B2 A1 bad | [`T071121Z`](../e2e/artifacts/history/smoke-20260720T071121Z/) | — | — | — | — | Invalid: DB pool timeouts @ concurrency=8 |
| B3a+A1 | [`T074835Z`](../e2e/artifacts/history/smoke-20260720T074835Z/) | 0.663 (Δ−0.112) | −0.052 | 0.488 ✗ | 0.916 ✗ | FAQ induce Acc tax (396 chunks); **no promote** |
| B3b+033 | [`T090743Z`](../e2e/artifacts/history/smoke-20260720T090743Z/) | **0.773 (Δ+0.017)** | — | 0.481 ✗ | 0.914 ✗ | LR 6k/8k pack; identity PASS; **no promote** |
| a1l2 | [`T092505Z`](../e2e/artifacts/history/smoke-20260720T092505Z/) | 0.724 (Δ−0.035) | — | **0.506** ✓ | 0.915 ✗ | citation fix; Fact ER flat; Acc tax |
| a1lr | [`T092930Z`](../e2e/artifacts/history/smoke-20260720T092930Z/) | **0.758 (Δ+0.001)** | — | **0.506** ✓ | 0.928 ✗ | recall miss by 0.005 |
| **a1lrl2** | [`T093152Z`](../e2e/artifacts/history/smoke-20260720T093152Z/) | 0.718 (Δ−0.063) | — | **0.525** ✓ | **0.933** ✓ | **L2 Parity**; Acc point tax |
| **a1fp** | [`T095809Z`](../e2e/artifacts/history/smoke-20260720T095809Z/) | **0.775 (Δ−0.012)** | — | **0.500** ✓ | 0.926 ✗ | **Acc Fact peer** · Fact ER 0.85 · no dual-list |
| a1fplr | [`T100053Z`](../e2e/artifacts/history/smoke-20260720T100053Z/) | 0.738 (Δ−0.046) | — | **0.519** ✓ | 0.918 ✗ | stack Acc-toxic · reject |
| a1fpm0 | [`T100538Z`](../e2e/artifacts/history/smoke-20260720T100538Z/) | 0.753 | — | **0.525** ✓ | 0.914 ✗ | min_rerank0 · reject |
| a1fpcov | [`T101322Z`](../e2e/artifacts/history/smoke-20260720T101322Z/) | 0.748 | — | **0.519** ✓ | 0.916 ✗ | cov protect30 · reject |
| **B5+a1fp** | [`T120315Z`](../e2e/artifacts/history/smoke-20260720T120315Z/) | **0.801 (Δ+0.019)** | — | **0.519** ✓ | 0.926 ✗ | **PROMOTE peer** · zero-chunk 0% · CI includes 0 |
| a1fpcmat | [`T121724Z`](../e2e/artifacts/history/smoke-20260720T121724Z/) | 0.733 (Δ−0.052) | — | 0.513 ✓ | **0.933** ✗ | Sum ER 0.96 · probe✓ · Acc/Fact tax · **REJECT** |
| a1fpspec | [`T124735Z`](../e2e/artifacts/history/smoke-20260720T124735Z/) | 0.746 (Δ−0.024) | **−0.014** | 0.481 ✗ | 0.926 ✗ | Complex Δ✓ · Acc tax · **REJECT** |
| a1fpscx | [`T131406Z`](../e2e/artifacts/history/smoke-20260720T131406Z/) | 0.764 (Δ−0.011) | **−0.065** | **0.500** ✓ | 0.922 ✗ | gate✓ · Acc/Complex tax · **REJECT** |
| a1fpsumx | [`T132225Z`](../e2e/artifacts/history/smoke-20260720T132225Z/) | 0.749 (Δ−0.040) | — | **0.500** ✓ | 0.924 ✗ | Sum ER **0.963** · Fact ER 0.75 · **REJECT** |
| B6+a1fp | [`T140822Z`](../e2e/artifacts/history/smoke-20260720T140822Z/) | 0.725 (Δ−0.039) | — | **0.506** ✓ | 0.928 ✗ | ge2 **12.5%** STRUCT✓ · Acc **REJECT** |
| B7+a1fp | [`T144511Z`](../e2e/artifacts/history/smoke-20260720T144511Z/) | 0.676 (Δ−0.093) | — | **0.506** ✓ | 0.914 ✗ | age/vdb **1.0** STRUCT✓ · Acc **REJECT** |
| a1fprw | [`T154525Z`](../e2e/artifacts/history/smoke-20260720T154525Z/) | 0.761 (Δ−0.016) | — | **0.525** ✓ | 0.927 ✗ | REL_SELECT=lightrag · Fact↓ · **REJECT** |
| B6+052 | [`T155511Z`](../e2e/artifacts/history/smoke-20260720T155511Z/) | 0.759 (Δ−0.028) | — | **0.506** ✓ | 0.914 ✗ | rel chunks@query · Acc↑ vs 0.725 · **REJECT** |
| B8+a1fp | [`T161836Z`](../e2e/artifacts/history/smoke-20260720T161836Z/) | 0.748 (Δ−0.028) | — | 0.488 ✗ | 0.923 ✗ | entity types LR · cov 0.735 · Acc **REJECT** |
| B9+a1fp | [`T011125Z`](../e2e/artifacts/history/smoke-20260721T011125Z/) | 0.745 (Δ−0.045) | — | **0.506** ✓ | 0.917 ✗ | extract caps · nodes **3950** · Acc **REJECT** |

**Horizon A close:** keep A1 (`rr_cer`). **L2 Parity:** [034](./034-l2-dual-list-under-full-ws-graph.md) `a1lrl2`. **Acc Fact peer:** [044](./044-horizon-b-placeholder-provenance.md) B5+`a1fp` Acc **0.801**. **B6–B9 / 051–054** ship structural laws (keep code); Acc peer unchanged. **SELECT/TOPIC Acc STOP**. Acc Beat fishing **STOP** — [055](./055-post-acc-ceiling-first-principles.md): split peers · [056 naming](./056-naming-identity-lr-parity.md) · latency [058](./058-c1a-fact-ce-skip-latency.md)–[060](./060-c1d-heuristic-keyword-latency.md) (generate ceiling; keyword=0 no wall win). Peers: [`peers.json`](../e2e/artifacts/peers.json).

---

## 6. Code map

| Feature | Module / env |
|---------|----------------|
| A1 rr_cer | `context_format.rs` · `EDGEQUAKE_CONTEXT_FORMAT=rr_cer` |
| A2 factual bias | `keywords/llm_extractor.rs` · `EDGEQUAKE_INTENT_FACTUAL_BIAS` |
| A3 LR answer | `engine_impl/prompt.rs` · `EDGEQUAKE_ANSWER_PROMPT=lightrag` |
| Acc ladder | `tools/bench001/scripts/run_p_ladder_acc.sh` `a0`…`a4` |
| B1 audit | `tools/bench001/scripts/audit_eq_lr_ingest.py` · [029](./029-ingest-parity-audit.md) |
| B2 markdown+glean | `document_admission.rs` · `bench001/client.py` · [030](./030-ingest-gleaning-parity.md) · `make bench001-b2-reingest` |
| B3a FAQ induce | [031](./031-structure-aware-chunking.md) · Acc tax — closed |
| B3b WS graph id | [032](./032-workspace-graph-identity.md) · identity PASS |
| B4 Mix pack | [033](./033-denser-graph-mix-packing.md) · LR 6k/8k · T090743Z Acc 0.773 tie — L2 miss |
| L2 Parity | [034](./034-l2-dual-list-under-full-ws-graph.md) · `a1lrl2` · citation fix + LR VECTOR budget + Mix∪CE |
| Acc Fact peer | [035](./035-fact-ce-bm25-protect.md) · `a1fp` · `FACT_PROTECT_BM25=1` · no dual-list |
| Recall w/o dual-list | [036](./036-a1fp-recall-without-dual-list.md) · closed FAIL · Mix ceiling |
| Summarize chunk-link audit | [037](./037-summarize-chunk-link-audit.md) · SELECT · `audit_summarize_chunk_links.py` |
| Topic-entity admit | [038](./038-topic-entity-admit-exploratory.md) · `EDGEQUAKE_TOPIC_ENTITY_ADMIT` · `a1fpsel` REJECT |
| Topic CE/fuse protect | [039](./039-topic-ce-protect-exploratory.md) · `EDGEQUAKE_TOPIC_CE_PROTECT` · `a1fpce` REJECT |
| Topic trunc/pack protect | [040](./040-topic-trunc-protect-exploratory.md) · `EDGEQUAKE_TOPIC_TRUNC_PROTECT` · `a1fptrunc` REJECT |
| Topic chunk fidelity | [041](./041-topic-chunk-fidelity-audit.md) · law **CE_GAP** · `audit_topic_chunk_fidelity.py` |
| Topic chunk materialize | [042](./042-topic-chunk-materialize.md) · `EDGEQUAKE_TOPIC_MATERIALIZE` · `a1fpmat` REJECT |
| B5 placeholder provenance | [044](./044-horizon-b-placeholder-provenance.md) · relation stub `source_chunk_ids` · `bench001-b5-reingest` |
| CONTENT-gated materialize | [045](./045-content-gated-materialize.md) · `TOPIC_MATERIALIZE_CONTENT` · `a1fpcmat` REJECT |
| Answer specificity | [046](./046-answer-specificity-prompt.md) · `ANSWER_PROMPT=specific` · `a1fpspec` REJECT |
| Type-scoped specificity | [047](./047-type-scoped-specificity.md) · `ANSWER_SPECIFIC_TYPES=complex` · `a1fpscx` REJECT |
| Summarize-only materialize | [048](./048-summarize-only-materialize.md) · `MATERIALIZE_TYPES=summarize` · `a1fpsumx` REJECT |
| Rel dedupe chunk union | [049](./049-rel-dedup-source-chunk-union.md) · `merger/relationship.rs` · `make bench001-b6-reingest` · STRUCT✓ Acc REJECT |
| Placeholder VDB parity | [050](./050-placeholder-vdb-parity.md) · `with_text_embedder` · `make bench001-b7-reingest` · STRUCT✓ Acc REJECT |
| Entity types LR parity | [053](./053-entity-types-lr-parity.md) · `default_entity_types` · `make bench001-b8-reingest` |
| Relation rank+weight | [051](./051-relation-rank-weight-select.md) · `EDGEQUAKE_RELATION_SELECT=lightrag` · `a1fprw` |
| Rel chunk ids @ query | [052](./052-rel-chunk-ids-query-parity.md) · plural `source_chunk_ids` → Mix · a1fp on B6 |
| Extract caps LR parity | [054](./054-extract-caps-lr-parity.md) · `extract_caps.rs` · `make bench001-b9-reingest` |

---

## 7. A3 answer-prompt diff (EQ default vs LR `rag_response`)

| Aspect | EQ default | LR `rag_response` / EQ `ANSWER_PROMPT=lightrag` |
|--------|------------|--------------------------------------------------|
| Grounding | Partial OK + grounded arithmetic | Strict “do not guess”; insufficient → say so |
| Structure cues | Entities / Relations / Chunks | Knowledge Graph Data + Document Chunks |
| Citations | Page grounding block | Explicit `### References` (≤5) |
| Conversation | Optional history section | Explicit history-aware intent |

Ablation: `make bench001-a3` (single confound on A2 pack).
