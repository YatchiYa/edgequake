# 12 — Per-Chunk Extract Budget (First Principles)

> **Question:** Is limiting entities / relations **per chunk (per LLM response)** a good idea at extraction time?  
> **Context:** EdgeQuake already ships LightRAG-parity caps **40 ents / 100 total rows** ([`extract_caps.rs`](../../edgequake/crates/edgequake-pipeline/src/prompts/extract_caps.rs), SPEC-001/054).  
> **Related:** geometry \(N\) ([SPEC-116](README.md)), LLM power \(y\) ([10](10-llm-power-first-principles.md)).

## Short verdict

**Yes — a per-response budget is a good idea**, primarily as a **reliability / cost / fairness control**, not as a “make the graph smarter” knob.

But: a **naive hard truncate** (keep first \(K\) entities in list order) is a **blunt instrument**. The *right* design is **soft prompt budget + hard safety net + recovery path (gleaning / continue) + geometry that keeps each chunk within the budget’s useful range**.

```ascii
  GOOD:   budget bounds output length, timeouts, junk fill, dual-SUT parity
  BAD:    treating “raise the cap” as the fix for “too few entities”
  WORSE:  tiny chunks × model that saturates the cap → vanity M explosion
```

## Decomposition (what a “budget” actually is)

| Layer | What it limits | EdgeQuake today |
|-------|----------------|-----------------|
| **Prompt soft cap** | Model’s intended output cardinality | `Quantity Limits (STRICT)` in extract prompts |
| **Hard post-parse truncate** | Parsed rows if model ignores prompt | `apply_extraction_caps` — first 40 ents, then rels, total ≤100 |
| **Output token / timeout** | Wall-clock & provider max_tokens | Ops / provider knobs (LightRAG docs stress this for local models) |
| **Chunk geometry \(N\)** | How many times the budget can fire | SPEC-116 ChunkingPolicy |
| **Gleaning** | Extra passes to recover missed items | Default max_gleaning=1 (LR) |

A budget is **per LLM response**, not a global “max entities in the workspace.” Global graph size is still \(\approx f(N, y_{\le K}, \text{merge})\).

## First principles (laws)

| Law | Statement |
|-----|-----------|
| **LAW-B1** | Unbounded list extraction is an **output-control** problem (timeouts, loops, token burn), not only a quality problem |
| **LAW-B2** | Soft instruction without hard truncate is insufficient — models over-extract; LightRAG and EQ both hard-cap after parse |
| **LAW-B3** | Hard truncate by **list order** encodes an arbitrary priority — mid/tail entities in dense chunks are at risk |
| **LAW-B4** | Budget \(K\) and chunk size \(S\) must be **co-designed**: if typical chunk content needs \(\gg K\) true entities, either raise \(K\), shrink \(S\), or add gleaning/continue |
| **LAW-B5** | When the model **saturates** \(K\) on most chunks, \(M \approx K \times N\) — adaptive \(\uparrow N\) looks like “over-extract” even with LR-parity caps (SPEC-108) |
| **LAW-B6** | Raising \(K\) without fixing merge/schema often **adds noise** (DEG-RAG: less can be more) |
| **LAW-B7** | Fair dual-SUT requires **matched** \(K\) (and \(N\), gleaning, model) — SPEC-108 LAW-X3 |
| **LAW-B8** | Prefer **importance / typed** selection under the budget over “fill to \(K\)” (prompt already says do not fill) |

## Causal diagram

```ascii
  chunk text (size S)
        │
        ▼
  Extract LLM  ──prompt soft K──► intended ≤K ents / ≤R rows
        │
        │  (may ignore / overshoot / loop)
        ▼
  Parse JSON/tuples
        │
        ▼
  Hard cap K/R ──truncate──► kept set (order-biased today)
        │
        ├──► gleaning / continue?  (recover misses)
        ▼
  Mentions M_chunk ≤ K
        │
        ▼
  Merge across N chunks → U
```

## Why budgets help (research + ops)

1. **Timeout / endless output** — LightRAG API docs: tables/citations can cause endless entity dumps; `max_tokens` + record caps prevent failed extracts ([LightRAG API Server](https://github.com/HKUDS/LightRAG/blob/main/docs/LightRAG-API-Server.md); PR [#2950](https://github.com/HKUDS/LightRAG/pull/2950)).  
2. **Structured-list degradation** — Practitioner guidance: unconstrained multi-entity lists burn token budget; boundary detection degrades as list length grows (batch input rather than unbounded lists).  
3. **Parity / Acc fairness** — EQ without caps was denser than LR (SPEC-054: nodes 4543–4659 → **3950** after 40/100). Caps close a confound.  
4. **Noise control (weak)** — Caps are a crude prior against “extract everything”; stronger denoising is merge / entity resolution ([DEG-RAG](https://arxiv.org/abs/2510.14271)).  
5. **Cost** — Output tokens dominate extract cost; \(K\) bounds worst-case \$/chunk.

## Why budgets hurt (if misdesigned)

1. **Order bias** — First-\(K\) truncate may drop the scientifically important entity mentioned late in the chunk.  
2. **Under-coverage on large dense chunks** — Fixed 1200 tokens of medical text can contain \(\gt 40\) true entities; hard \(K\) + no gleaning → incomplete bridges (CS-RAG “incomplete information”).  
3. **Saturation × adaptive geometry** — Model fills 40 on every small chunk → \(M\) scales with \(N\) (SPEC-115: \(M\) tracks \(N\) at ~1.33×). Cap does **not** prevent product densification vs LR.  
4. **False sense of quality** — “We capped at 40” ≠ “we extracted the right 40.”  
5. **Acc tax** — SPEC-054: closing the law reduced graph toward LR but Acc dropped vs prior peer (0.745 vs 0.801 keep). Caps are not free for research scores.

## Interaction with SPEC-116 / LLM power

```ascii
  Acc-fair Fixed 1200/100     +  K=40/100  → fair LR dual-SUT
  Adaptive ↑N                 +  K=40/100  → M↑ even with same K (LAW-B5)
  Stronger LLM (↑y)           +  K=40      → more often hits ceiling; quality of the 40 matters
  Raise K “to get more ents”  +  Adaptive  → cost↑ noise↑; usually wrong first move
```

**Product rule (aligned with [007 lens](05-lenses/007-llm-power-research.md)):**  
Pin Acc-fair geometry first → then judge coverage → only then consider \(K\), gleaning, or extract model — never raise \(K\) to chase card \(M\).

## See also

- [`13-extract-budget-brainstorm.md`](13-extract-budget-brainstorm.md) — options, decision matrix, implementation sketch  
- [`05-lenses/008-extract-budget.md`](05-lenses/008-extract-budget.md) — ops guidance  
- SPEC-001/054, [`extract_caps.rs`](../../edgequake/crates/edgequake-pipeline/src/prompts/extract_caps.rs)
