# 10 — LLM Power × Extraction × Graph (First Principles)

> **Scope:** Orthogonal to chunk **geometry** (SPEC-116 Acc-fair).  
> **Question:** How does extract-LLM *capability* change entity/relation yield, unique graph size, and research/QA performance?  
> **As of:** literature + EdgeQuake measurements through **August 2026**.

## WHY

Partners often read “more entities on the card” as “stronger model / better RAG.”  
That confounds three levers:

```ascii
  N  = chunk count          ← ChunkingPolicy / adaptive vs Fixed 1200/100
  y  = yield per chunk      ← extract LLM capability + prompt + caps
  q  = merge / resolve quality ← dedupe, schema, gleaning, entity resolution
```

Graph and research outcomes are **not** monotone in raw mention count \(M\).

## Decomposition (laws of composition)

```ascii
  text
    │
    ▼
  ChunkingPolicy ──► N chunks
    │
    ▼
  Extract LLM (y) ──► mentions M ≈ Σ min(cap, y_i)
    │
    ▼
  Merge / resolve (q) ──► unique U (nodes, edges)
    │
    ▼
  Retrieve + answer ──► research / multi-hop QA
         ▲
         │
    noise / dups from weak q or over-extraction
```

| Symbol | Controlled by | Spec anchor |
|--------|---------------|-------------|
| \(N\) | Workspace chunking / Acc-fair | SPEC-116, SPEC-115 |
| \(y\) | Extract model, temp, caps 40/100 | SPEC-108 LAW-X3 |
| \(M\) | Document card vanity sum | SPEC-108 H1 |
| \(U\) | Post-merge graph | Acc / SPEC-115 |
| QA | Retrieval + generator + graph quality | This pack |

## Power laws (LAW-P)

| Law | Statement |
|-----|-----------|
| **LAW-P1** | \(N\) is geometry; \(y\) is model — never attribute a partner density gap to one lever alone |
| **LAW-P2** | Stronger construction LLMs raise **multi-hop** QA when **graph quality** rises — not merely when \(\|E\|,\|R\|\) rise |
| **LAW-P3** | Raw \(M\)/\(U\) can grow from true capability **or** from over-extraction / weak merge |
| **LAW-P4** | Denoising / entity resolution often beats “extract more” for downstream QA |
| **LAW-P5** | Structured-output / decoding discipline ≠ parameter count (mid-2026 local extract) |
| **LAW-P6** | Fair dual-SUT still requires matched LLM + embed + geometry (SPEC-108 LAW-X3, SPEC-115 LAW-C5) |

## Interaction with SPEC-116 Acc-fair

```ascii
  Adaptive ON (smaller chunks → ↑N)
       ×
  High-y extract LLM
       =
  Larger M, often larger U
       +
  Higher risk of noise if merge/schema weak
       →
  “Looks denser than LightRAG” even when brain is matched

  Fixed 1200/100 (Acc fair)
       ×
  Same extract LLM
       =
  N matched to LightRAG paper pin
       →
  Fair density comparison; then judge y / QA separately
```

**Product rule:** Pin Acc-fair **first** when comparing to LightRAG or Acc.  
Upsize extract LLM **after** geometry is fair, if multi-hop research QA is still weak.

## What “LLM power” means here

Not only parameter count. Operationally:

1. **Extraction correctness** on supporting sentences (builder quality).  
2. **Schema / JSON reliability** (pipeline completes).  
3. **Relation precision** (less over-generalized / mis-bound edges).  
4. **Cost latency** (indexing wall-clock and \$).

A smaller model with constrained decoding may **complete** extract better than a larger unconstrained one (LAW-P5), while still under-yielding on complex multi-entity passages.

## Non-goals of this pack

- Changing fleet adaptive default or Acc publication pins  
- Claiming that denser \(U\) is always better research  
- New live dual-arm model bake-offs (protocol only — see `11`)

## See also

- [`11-research-evidence-aug-2026.md`](11-research-evidence-aug-2026.md) — citations and numbers  
- [`05-lenses/007-llm-power-research.md`](05-lenses/007-llm-power-research.md) — ops recommendations  
- [`01-first-principles.md`](01-first-principles.md) — LAW-116-1..7 (geometry SSOT)
