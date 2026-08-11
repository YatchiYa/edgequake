# 11 — Research Evidence (August 2026)

> Evidence pack for [`10-llm-power-first-principles.md`](10-llm-power-first-principles.md).  
> Every numeric claim cites a source. EdgeQuake anchors stay **geometry-matched** unless noted.

## Claim → evidence → EdgeQuake meaning

| ID | Claim | Evidence (Aug 2025 – Aug 2026) | EQ meaning |
|----|-------|--------------------------------|------------|
| C1 | Stronger **graph construction** LLM improves GraphRAG QA, especially multi-hop | [Han et al., arXiv:2502.11371](https://arxiv.org/abs/2502.11371) MultiHop-RAG, Llama-3.1-70B generator fixed; construction **GPT-4o-mini → GPT-4o**: Overall **71.17 → 75.08**; Comparison **60.16 → 66.59**; Temporal **49.06 → 58.49** | Upsize **extract** when multi-hop research is weak **after** Acc-fair pin — not to chase card \(M\) |
| C2 | RAG vs GraphRAG are complementary; GraphRAG helps reasoning-intensive queries | Same paper: RAG strong on single-hop / Null; GraphRAG better on multi-hop; hybrids help | Do not expect denser KG to win every query class |
| C3 | KG coverage limits QA — incomplete extract hurts | Same paper: ~**65.8%** HotPotQA / **65.5%** NQ answer entities appear in constructed KG | Low \(U\) can be under-extraction; diagnose with coverage, not vanity \(M\) alone |
| C4 | **Less is more** — denoising shrinks graph and improves GraphRAG QA | [DEG-RAG / Less is More, arXiv:2510.14271](https://arxiv.org/html/2510.14271): remove ~**40%** entities/relations → better QA across four GraphRAG variants; aggressive merge up to ~**70%** often not harmful if not over-merged | Bigger \(U\) from high-\(y\) models can be **noise**; merge / resolve matter (LAW-P4) |
| C5 | Extraction correctness rises with builder capability, but ceiling is imperfect | [CS-RAG / Mitigating KG Quality Issues, arXiv HTML 2603.14828](https://arxiv.org/html/2603.14828): correctness **positively correlated** with builder scale/capability; strongest builder still ~**68%** correct tuples on sampled gold supports | Even frontier extract leaves spurious noise + incomplete bridges |
| C6 | Two failure modes: **spurious noise** vs **incomplete information** | Same CS-RAG paper: over-generalized / mis-bound / flipped relations vs missing bridge edges / dropped qualifiers | High-\(y\) models can emit more spurious edges; weak models drop bridges |
| C7 | Unique volume ≠ informative retrieval; connectivity / ontology matter | [Wikontic, EACL 2026](https://aclanthology.org/2026.eacl-long.388.pdf); [SocraticKG, arXiv:2601.10003](https://arxiv.org/pdf/2601.10003) | Prefer Acc / partner QA + unique \(U\) + schema (SPEC-114) over raw mention sum |
| C8 | Local mid-2026: denser extract ≠ better answers; decoding discipline dominates “floor” myths | Graph Praxis mid-2026 local re-benchmark (Jul 2026): Llama 3.1 8B denser (**1172** ents / **696** rels, slow); Qwen 2.5 7B **smaller** graph (**330** ents) but **best answers**; Phi-4-mini unconstrained JSON collapse → constrained decoding restores validity | Do not equate parameter count or entity count with research quality; fix structured output first |
| C9 | LLM extract is the cost wall; classical/hybrid can approach LLM builders for some tasks | [E²GraphRAG, arXiv:2505.24226](https://arxiv.org/pdf/2505.24226) (~**10×** faster index vs GraphRAG; SpaCy entity graph); [Towards Practical GraphRAG, arXiv:2507.03226](https://arxiv.org/pdf/2507.03226) dependency parse ~**94%** of LLM builder QA (61.87 vs 65.83) | Stronger extract LLM is a **budget** choice; not the only path to usable graphs |
| C10 | Geometry (not model) explained EQ vs LightRAG density under **matched** brain | SPEC-115: Mistral Small + mistral-embed; fair \(U\) **375** vs LR **367** (~1.02×); product adaptive \(U\) **491** (~1.34× vs LR) | Pin Fixed 1200/100 before blaming / praising model power |

## EdgeQuake empirical anchors (matched model)

### SPEC-115 — live Mistral Small (paper gold MD)

| Arm | Geometry | N | M ents | U nodes | U edges |
|-----|----------|--:|-------:|--------:|--------:|
| LightRAG | F@1200/100 | 13 | ~425 | **367** | 318 |
| EQ fair | 1200/100 | 12 | 439 | **375** | 320 |
| EQ product | adaptive ~800 | 16 | **584** | **491** | 305 |

Source: [`../115-extraction-chunk-kg/measurements/SUMMARY.md`](../115-extraction-chunk-kg/measurements/SUMMARY.md).

**Read:** Under one model, \(M\) tracks \(N\) (~1.33×). That is **LAW-P1 geometry**, not a power upgrade.

### SPEC-108 — Acc fair unique (medical)

| Side | U nodes | Edges |
|------|--------:|------:|
| LightRAG | 3580 | 5325 |
| EdgeQuake | 3950 | 3927 |

Source: [`../108-extraction-compared-light-rag/measurements/SUMMARY.md`](../108-extraction-compared-light-rag/measurements/SUMMARY.md).

Partner “~12k entities” = **mentions \(M\)** × dense \(N\), not fair unique \(U\).

## Causal synthesis (ASCII)

```ascii
                 ┌─────────────────────┐
                 │ Extract LLM power   │
                 │  correctness ↑      │
                 │  y (yield) ↑/≠      │
                 │  spurious risk ↑    │
                 └──────────┬──────────┘
                            │
  ChunkingPolicy ──► N ─────┼──► M ──► merge(q) ──► U ──► multi-hop QA
                            │              │
                            │              └─► noise ──► QA ↓ (DEG-RAG)
                            │
                     Acc-fair Fixed keeps N matched
                     so ΔU / ΔQA can be attributed to model
```

## Bibliography (primary)

1. Han et al. — *RAG vs. GraphRAG: A Systematic Evaluation and Key Insights* — https://arxiv.org/abs/2502.11371 (v3)  
2. DEG-RAG — *Less is More: Denoising Knowledge Graphs For Retrieval Augmented Generation* — https://arxiv.org/html/2510.14271 / https://arxiv.org/abs/2510.14271  
3. CS-RAG — *Mitigating KG Quality Issues: A Robust Multi-Hop GraphRAG Retrieval Framework* — https://arxiv.org/html/2603.14828  
4. Wikontic — EACL 2026 long — https://aclanthology.org/2026.eacl-long.388.pdf  
5. SocraticKG — https://arxiv.org/pdf/2601.10003  
6. E²GraphRAG — https://arxiv.org/pdf/2505.24226  
7. Towards Practical GraphRAG — https://arxiv.org/pdf/2507.03226  
8. LightRAG (EMNLP Findings 2025) — dual-level retrieval; default chunk **1200**, gleaning 1 — https://aclanthology.org/2025.findings-emnlp.568.pdf  
9. Graph Praxis — *Local LLMs for Graph RAG Extraction: the mid-2026 re-benchmark* — https://medium.com/@shereshevsky/local-llms-for-graph-rag-extraction-the-mid-2026-re-benchmark-5f36b3d19383  
10. EdgeQuake — SPEC-108 / SPEC-115 measurement SUMMARYs (paths above)

## Optional follow-up protocol (not run in this doc pack)

Fair dual-arm **model** bake-off (geometry fixed Acc-fair):

```text
1. Workspace Fixed 1200/100 (SPEC-116 Acc-fair chip)
2. Same gold MD / Acc medical sample
3. Arm S: smaller extract (e.g. local 7–9B or gpt-5-nano)
4. Arm L: larger / stronger extract (cloud frontier)
5. Report: N (must match), M, U, Acc / multi-hop QA — never M alone
```

## Honesty rules

- Do not compare EQ card \(M\) to LightRAG unique \(U\) without saying so (SPEC-108).  
- Do not claim “stronger model ⇒ always more entities ⇒ always better research.”  
- Cite dates; mid-2026 local blogs are operational evidence, not peer-reviewed law.
