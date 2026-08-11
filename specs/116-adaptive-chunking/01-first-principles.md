# 01 — First Principles

## Axioms

1. Chunk count \(N\) multiplies extract LLM calls and mention sum \(M\).
2. Adaptive shrink is a **policy choice**, not an invisible side effect of file bytes.
3. Fair dual-SUT claims require matched size/overlap (SPEC-108 LAW-X3, SPEC-115 LAW-C3).
4. Workspace settings that affect ingest must say **future-only** and point at rebuild.
5. UI must not re-implement threshold math — display resolved explainability only.

## Laws

| Law | Statement |
|-----|-----------|
| **LAW-116-1** | Geometry is a product policy |
| **LAW-116-2** | Precedence: document `ChunkOptions` > workspace > fleet env |
| **LAW-116-3** | Default = inherit (null metadata) |
| **LAW-116-4** | Future ingestions only |
| **LAW-116-5** | Count honesty (M ≠ U) — copy cross-ref SPEC-108 |
| **LAW-116-6** | One SSOT resolve in pipeline |
| **LAW-116-7** | Denser ≠ better research — \(N\) (geometry) × \(y\) (extract LLM) × merge; see [`10-llm-power-first-principles.md`](10-llm-power-first-principles.md) |

## Causal diagram

```ascii
  text_content.len()
         │
         ▼
  ┌──────────────────┐
  │ ChunkingPolicy   │  workspace (or Inherit→env)
  │ Inherit|Adaptive │
  │ |Fixed(size,ov)  │
  └────────┬─────────┘
           │ base (size, overlap)
           ▼
  ┌──────────────────┐
  │ ChunkOptions     │  document upload (optional)
  │ apply_to_config  │  WINS LAST
  └────────┬─────────┘
           ▼
        Chunker → N chunks → extract → M → merge → U
```

## LAW-116-7 (LLM × geometry)

```ascii
  N × y(model) → M → merge → U → research QA
  ↑                ↑
  SPEC-116         extract LLM power (orthogonal)
```

Denser \(M\)/\(U\) can come from adaptive geometry **or** a higher-yield model.  
Multi-hop research quality tracks **graph quality**, not raw size — evidence in [`10`](10-llm-power-first-principles.md) / [`11`](11-research-evidence-aug-2026.md).

Per-chunk extract budgets (\(K\)=40 ents / 100 rows) bound **yield per response** and interact with \(N\): when the model saturates \(K\), \(M \approx K \times N\) — see [`12`](12-extract-budget-first-principles.md) / [`13`](13-extract-budget-brainstorm.md).

## Acc-fair identity

```text
Fixed(1200, 100)  ≡  EDGEQUAKE_ADAPTIVE_CHUNKING=0
                     + EDGEQUAKE_CHUNK_SIZE=1200
                     + EDGEQUAKE_CHUNK_OVERLAP=100
```
