# 01 — First Principles (Aug 2026 AI Engineering)

> Method: reduce “too many entities on PDF” to physics of chunking × extract caps × merge.  
> Grounded in GraphRAG practice (2025–2026): extraction quality > vanity density; chunk size is a **yield multiplier**, not a free quality knob.

## Axioms

1. **Tokens in → LLM calls out.** Extract cost and mention count scale with chunk count \(N\).
2. **Per-chunk caps bound mentions.** LightRAG / EQ caps: ≤40 entities, ≤100 records per response (`DEFAULT_MAX_EXTRACTION_*`). So \(M \le 40N\) (and typically tracks \(N\) when the model saturates).
3. **Merge defines the knowledge graph.** Mentions across chunks collapse to unique nodes \(U\) by normalized entity id. Product truth for “how many entities exist” is \(U\), not pre-merge \(M\).
4. **Chunk size trades context vs fan-out.** Larger chunks: fewer LLM calls, better intra-chunk relations, fewer duplicate aliases. Smaller chunks: more calls, higher mention M, noisier graph hop retrieval if precision &lt; ~80% (industry GraphRAG hybrid guidance, 2026).
5. **Fair dual-SUT requires matched confounds.** Size, overlap, strategy, gleaning, caps, entity-type policy, **and model** must match before claiming over-extract.
6. **PDF bytes ≠ text bytes.** Adaptive thresholds keyed on `document_size_bytes` of **extracted text** in EQ ingest (not raw PDF file size). A 1.1 MB PDF that yields 61 KB markdown pins **800**, not **600**.

## Laws (SPEC-115)

| Law | Statement |
|-----|-----------|
| **LAW-C1** Geometry first | Before blaming prompts, measure \(N\) under product vs fair pins on the same text. |
| **LAW-C2** Multiplier | \(\Delta N\) from adaptive shrink ≈ proportional \(\Delta M\) when yield/chunk is stable. |
| **LAW-C3** Fair pin | Acc / LR-matched: `EDGEQUAKE_ADAPTIVE_CHUNKING=0`, size 1200, overlap 100. |
| **LAW-C4** Count honesty | Report columns separately: \(N\), \(M\) (mentions), \(U\) (unique), density per 1k chars. |
| **LAW-C5** Same brain | Dual-SUT extract claims require identical LLM+embed (here: Mistral Small + mistral-embed). |
| **LAW-C6** Strategy confound | EQ auto-`Pdf` page-aware chunking ≠ LightRAG default **F** even at equal `chunk_token_size`. |

## Causal diagram

```ascii
                 PDF bytes (~1.1MB, 16pp)
                          |
              +-----------+-----------+
              |                       |
         parse/OCR               gold MD twin
         (path D)                (~61KB text)
              |                       |
              v                       v
        text_content.len() -----> adaptive ON?
              |                  /          \
              |            yes /            \ no (fair)
              |               v              v
              |         size=800           size=1200
              |         ov~66              ov=100
              |               \              /
              |                \            /
              |                 v          v
              |                  chunks N
              |                 /    |    \
              |           N_prod   N_fair  N_LR(F)
              |           (~20)    (~13)   (~13)
              |               \    |    /
              |                v   v   v
              |            LLM extract (≤40 ents/chunk)
              |                |         |
              |                v         v
              |           mentions M   unique U (after merge)
              |
              +-- EQ document card shows M (SPEC-108 LAW-X1)
              +-- LightRAG graph/KV stores U
```

## Aug 2026 engineering notes

- **Overlap:** recent systematic QA chunking work finds overlap often adds cost without retrieval gain; LightRAG still defaults overlap **100** — fair compares keep it for parity, not because overlap is “SOTA required.”
- **GraphRAG hybrid:** relation precision below ~80% makes denser graphs *hurt* retrieval (false edges). Prefer denser graphs only when extract quality is high.
- **Context cliff:** generator quality can fall past ~2.5k context tokens; that is a *query* concern. Indexing chunks of 600–1200 remain in the classic LightRAG paper regime (paper used **1200** throughout).
- **Paper itself:** LightRAG arXiv HTML notes chunk size **1200** and gleaning **1** in experiments — product adaptive shrink is an EdgeQuake deviation, not paper fidelity.

## Back-of-envelope (this paper)

```text
doc_tokens ≈ 14156 (tiktoken on gold MD)
N_F(1200/100) ≈ 13     ← LightRAG default F (measured)
N_adapt(800/66) ≈ 20   ← EQ adaptive for 50–100KB text (measured)
N_adapt(600/50) ≈ 26   ← only if size keyed on >100KB (PDF bytes mistake)

If yield ≈ 25 ents/chunk:
  M_LR  ≈ 13*25 = 325
  M_EQ  ≈ 20*25 = 500   (~1.54×)
After merge, U << M; compare U under LAW-C4.
```
