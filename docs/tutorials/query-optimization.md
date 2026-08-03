---
title: 'Tutorial: Query Optimization'
---

> **Product: v0.23.0** · Contract: [`openapi.snapshot.json`](../../edgequake_webui/openapi/openapi.snapshot.json) · Spec ops: [Ingestion cancel & fairness](../ingestion-cancel-and-fairness.md)

# Tutorial: Query Optimization

> **Choosing and Tuning Query Modes for Best Results**

This tutorial teaches you how to select the right query mode for different question types and optimize retrieval quality.

**Time**: ~20 minutes  
**Level**: Intermediate  
**Prerequisites**: Completed [First RAG App](/docs/tutorials/first-rag-app/)

All query examples return **`QueryResponse`**: top-level `answer` + `sources` + `stats` (not `chunks` / `entities_used`). Use `X-Workspace-ID` for scoping.

> **Request fields that exist** (from `QueryRequest`): `query`, `mode`, `max_results`, `context_only`, `prompt_only`, `enable_rerank`, `rerank_top_k`, `rerank_model`, `document_filter`, `mix_weights`, `llm_provider`, `llm_model`, `system_prompt`, `include_references`, `include_subgraph`, `conversation_history`. Fields like `max_chunks`, `similarity_threshold`, `max_hops`, `max_communities`, or `temperature` are **not** per-query API fields — see [Tuning parameters](#tuning-parameters) for what to use instead.

---

## Query Mode Overview

EdgeQuake provides 6 query modes. **The production default is `mix`** — when `mode` is omitted the API falls back to `QueryMode::Mix` (weighted fusion of all three arms):

```
┌─────────────────────────────────────────────────────────────────┐
│                   QUERY MODE DECISION TREE                      │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  "What are the main themes?"  ──────────▶  GLOBAL               │
│  (overview; relationship-vector search)                        │
│                                                                 │
│  "Who is Sarah Chen?"  ─────────────────▶  LOCAL                │
│  (specific entity)                                              │
│                                                                 │
│  "How does X work?"  ───────────────────▶  HYBRID               │
│  (general questions; local+global+naive interleave)             │
│                                                                 │
│  "Find documents about..."  ────────────▶  NAIVE                │
│  (keyword/semantic search only)                                │
│                                                                 │
│  "Complex multi-part question"  ────────▶  MIX  (DEFAULT)       │
│  (weighted blend of all arms)                                   │
│                                                                 │
│  "Just chat, no retrieval"  ────────────▶  BYPASS               │
│  (direct LLM)                                                   │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## Mode 1: Naive (Vector Only)

**Best for**: Simple keyword lookups, document similarity

### How It Works

```
Query ──▶ [Embed] ──▶ [Vector Search] ──▶ Top-K Chunks ──▶ LLM ──▶ Answer
```

### Example

```bash
curl -X POST "http://localhost:8080/api/v1/query" \
  -H "Content-Type: application/json" \
  -H "X-Workspace-ID: $WORKSPACE_ID" \
  -d '{
    "query": "funding announcement",
    "mode": "naive"
  }'
```

### When to Use

| ✅ Good For           | ❌ Avoid For            |
| --------------------- | ---------------------- |
| Keyword search        | Multi-hop reasoning    |
| Finding similar docs  | Relationship questions |
| Simple factual lookup | Overview questions     |
| Fast responses        | Complex analysis       |

---

## Mode 2: Local (Entity-Focused)

**Best for**: Questions about specific entities and their relationships

### How It Works

```
Query ──▶ [Extract Entities] ──▶ [Graph Traversal] ──▶ Related Context ──▶ LLM ──▶ Answer
                                        │
                                        ▼
                              Entity descriptions
                              Related entities
                              Relationships
                              Source chunks
```

### Example

```bash
curl -X POST "http://localhost:8080/api/v1/query" \
  -H "Content-Type: application/json" \
  -H "X-Workspace-ID: $WORKSPACE_ID" \
  -d '{
    "query": "What is Sarah Chen'\''s background and role?",
    "mode": "local"
  }'
```

### When to Use

| ✅ Good For          | ❌ Avoid For         |
| -------------------- | ------------------- |
| "Who is X?"          | Overview questions  |
| "What does X do?"    | Theme analysis      |
| Entity relationships | When entity unknown |
| Biography questions  | General how-tos     |

---

## Mode 3: Global (Relationship-Centric)

**Best for**: Overview questions, theme analysis, corpus-wide insights

> **Not** Microsoft GraphRAG community-report search. EdgeQuake `global` runs a **high-level query embedding against relationship vectors**, then batch-fetches connected entities and their source chunks; when no relationship vectors match it falls back to **high-degree nodes** in the graph.

### How It Works

```
Query ──▶ [High-level keyword embedding] ──▶ [Vector ANN on relationship rows]
                                                      │
                                     ┌────────────────┴────────────────┐
                                     │ hits                            │ empty
                                     ▼                                 ▼
                             src/tgt entities               popular nodes by degree
                             + relationship text           (graph fallback)
                                     │                                 │
                                     └────────────┬────────────────────┘
                                                  ▼
                                    Batch node + degree fetch (no N+1)
                                                  │
                                                  ▼
                                    Collect linked chunk IDs → chunk re-rank
                                                  │
                                                  ▼
                                               LLM ──▶ Answer
```

### Example

```bash
curl -X POST "http://localhost:8080/api/v1/query" \
  -H "Content-Type: application/json" \
  -H "X-Workspace-ID: $WORKSPACE_ID" \
  -d '{
    "query": "What are the main themes and topics across all documents?",
    "mode": "global"
  }'
```

### When to Use

| ✅ Good For      | ❌ Avoid For          |
| ---------------- | --------------------- |
| "Main themes?"   | Specific entity facts |
| "Overview of..." | Detailed how-tos      |
| "Key topics?"    | Finding specific docs |
| Summary requests | Precise citations     |

---

## Mode 4: Hybrid (Local + Global + Naive)

**Best for**: General questions, balanced context needs

> Hybrid **interleaves** the Local, Global, **and** Naive arms round-robin. It is **not** the default — `mix` is. Use `hybrid` when you want a deterministic three-arm interleave without weight tuning.

### How It Works

```
                              ┌──▶ [Local arm] ──────┐
Query ──▶ [Interleave] ───────┼──▶ [Global arm] ─────┼──▶ [Combine] ──▶ LLM ──▶ Answer
                              └──▶ [Naive arm] ──────┘
```

### Example

```bash
curl -X POST "http://localhost:8080/api/v1/query" \
  -H "Content-Type: application/json" \
  -H "X-Workspace-ID: $WORKSPACE_ID" \
  -d '{
    "query": "How has TechCorp evolved since its founding?",
    "mode": "hybrid"
  }'
```

### When to Use

| ✅ Good For         | ❌ Avoid For           |
| ------------------- | ---------------------- |
| General questions   | When speed is critical |
| Unsure of best mode | Simple keyword search  |
| Deterministic 3-arm interleave | When you want weight tuning (`mix`) |
| Complex questions   |                        |

---

## Mode 5: Mix (Weighted Blend — DEFAULT)

**Best for**: Fine-tuned blending of retrieval strategies; this is the production default when `mode` is omitted

### How It Works

Mix runs the **Local**, **Global**, and **Naive** arms in parallel and blends their results by *weighted score* (min-max normalized per arm, then weighted sum). Weights are set **per request** via `mix_weights` and need not sum to 1.

```
                              ┌──▶ [Local] ──▶ local × wL ─┐
Query ──▶ [Parallel] ─────────┤                            ├──▶ [Rank] ──▶ LLM
                              ├──▶ [Global] ─▶ global × wG ─┤
                              └──▶ [Naive] ──▶ naive × wN ──┘
```

### Example

```bash
curl -X POST "http://localhost:8080/api/v1/query" \
  -H "Content-Type: application/json" \
  -H "X-Workspace-ID: $WORKSPACE_ID" \
  -d '{
    "query": "NeuralSearch capabilities and key people",
    "mode": "mix",
    "mix_weights": { "local": 1.0, "global": 0.5, "naive": 1.0 }
  }'
```

### Weight Presets (via `mix_weights`)

| Use Case       | local | global | naive |
| -------------- | ----- | ------ | ----- |
| Factual lookup | 0.5   | 0.0    | 1.0   |
| Relationship Q | 1.0   | 0.5    | 0.5   |
| Overview Q     | 0.5   | 1.0    | 0.5   |
| Balanced       | 1.0   | 1.0    | 1.0   |

Fleet defaults: `EDGEQUAKE_MIX_LOCAL_WEIGHT`, `EDGEQUAKE_MIX_GLOBAL_WEIGHT`, `EDGEQUAKE_MIX_NAIVE_WEIGHT`. Fusion is round-robin by default; `EDGEQUAKE_MIX_FUSION=rrf` is an ablation option.

---

## Mode 6: Bypass (Direct LLM)

**Best for**: When retrieval isn't needed

### How It Works

```
Query ──▶ [Direct LLM Call] ──▶ Answer
           (no retrieval)
```

### Example

```bash
curl -X POST "http://localhost:8080/api/v1/query" \
  -H "Content-Type: application/json" \
  -H "X-Workspace-ID: $WORKSPACE_ID" \
  -d '{
    "query": "What is the capital of France?",
    "mode": "bypass"
  }'
```

### When to Use

| ✅ Good For       | ❌ Avoid For       |
| ----------------- | ------------------ |
| General knowledge | Document questions |
| Code generation   | Anything in corpus |
| Format conversion | Fact-checking      |
| Math/logic        | Citations needed   |

---

## Choosing the Right Mode

### Decision Flowchart

```
                           Question Type?
                               │
          ┌────────────────────┼────────────────────┐
          │                    │                    │
    About specific        General/mixed       Overview/themes
       entity?               question?            wanted?
          │                    │                    │
          ▼                    ▼                    ▼
        LOCAL               MIX (default)        GLOBAL
          │                    │                    │
          │                    │                    │
     Need tuning?         Need interleave?     Need more?
          │                    │                    │
          ▼                    ▼                    ▼
         MIX                 HYBRID                MIX
```

### Quick Reference

| Question Pattern         | Best Mode     |
| ------------------------ | ------------- |
| "Who is X?"              | local         |
| "What is X?"             | hybrid        |
| "How does X work?"       | hybrid        |
| "Main themes?"           | global        |
| "Overview of..."         | global        |
| "Find docs about..."     | naive         |
| "Compare X and Y"        | mix           |
| "X's relationship to Y?" | local         |
| Omit `mode` entirely     | mix (default) |

---

## Performance Comparison

### Latency by Mode

| Mode   | Avg Latency | Notes                |
| ------ | ----------- | -------------------- |
| naive  | ~200ms      | Fastest, vector only |
| local  | ~300ms      | Graph traversal      |
| global | ~400ms      | Relationship vectors |
| hybrid | ~500ms      | 3-arm interleave     |
| mix    | ~500ms      | Weighted blend       |
| bypass | ~100ms      | No retrieval         |

### Quality by Question Type

| Question Type | Naive    | Local    | Global   | Hybrid   |
| ------------- | -------- | -------- | -------- | -------- |
| Entity facts  | ⭐⭐     | ⭐⭐⭐⭐ | ⭐⭐     | ⭐⭐⭐   |
| Relationships | ⭐       | ⭐⭐⭐⭐ | ⭐⭐     | ⭐⭐⭐   |
| Overview      | ⭐       | ⭐⭐     | ⭐⭐⭐⭐ | ⭐⭐⭐   |
| Similarity    | ⭐⭐⭐⭐ | ⭐⭐     | ⭐       | ⭐⭐⭐   |
| Complex       | ⭐       | ⭐⭐⭐   | ⭐⭐⭐   | ⭐⭐⭐⭐ |

---

## Tuning Parameters

Only these are per-request query fields:

| Field | Default | Effect |
| ----- | ------- | ------ |
| `max_results` | 20 (engine `max_chunks`) | Max chunks retrieved (the per-query knob) |
| `enable_rerank` | `true` | Apply reranking to improve relevance |
| `rerank_top_k` | `null` (model default) | Number of top chunks after reranking |
| `rerank_model` | provider default | Rerank model id (e.g. `cohere-rerank-v3`) |
| `document_filter` | `null` | Restrict RAG context by date / id / pattern |
| `mix_weights` | engine/env defaults | `{local, global, naive}` arm weights for `mix` |
| `context_only` | `false` | Return retrieved context only, no LLM answer |
| `prompt_only` | `false` | Return the formatted prompt for debugging |
| `include_subgraph` | `true` | Include matched graph (entities + relationships) |
| `include_references` | `false` | Add detailed reference metadata to sources |

Example — cap chunks and scope to a date range:

```json
{
  "query": "Detailed analysis of TechCorp",
  "mode": "hybrid",
  "max_results": 10,
  "document_filter": { "date_from": "2024-01-01", "date_to": "2024-12-31" }
}
```

### Engine-level knobs (not per-query)

- `EDGEQUAKE_MIN_ENTITY_SCORE` — entity similarity floor (default `0.1`); lower for rare entities.
- `EDGEQUAKE_LLM_MAX_TOKENS` — HTTP safety-layer response cap (default `16384`).
- `EDGEQUAKE_MIX_{LOCAL,GLOBAL,NAIVE}_WEIGHT`, `EDGEQUAKE_MIX_FUSION` — fleet mix defaults.
- **Temperature is chat-only** (default `0.7` on `/api/v1/chat/completions`); query requests have no temperature field.

---

## A/B Testing Modes

Compare modes programmatically:

```python
import requests

WORKSPACE_ID = "ws_abc123"
QUERY = "What are TechCorp's main products and leadership?"

modes = ["naive", "local", "global", "hybrid", "mix"]
results = {}

for mode in modes:
    resp = requests.post(
        "http://localhost:8080/api/v1/query",
        headers={"X-Workspace-ID": WORKSPACE_ID},
        json={"query": QUERY, "mode": mode}
    )
    body = resp.json()
    results[mode] = {
        "answer_len": len(body.get("answer", "")),
        "sources": len(body.get("sources", [])),
        "total_ms": body.get("stats", {}).get("total_time_ms"),
    }

for mode, data in results.items():
    print(f"\n=== {mode.upper()} ===")
    print(f"Sources: {data['sources']}, total_ms: {data['total_ms']}")
```

---

## Common Issues

### Too Few Results

**Symptoms**: Empty or very short answers.

**Solutions**:

1. Increase `max_results` (per-query chunk cap; default 20)
2. Lower `EDGEQUAKE_MIN_ENTITY_SCORE` for rare entities
3. Try `hybrid` or `mix` instead of `naive`

### Irrelevant Results

**Symptoms**: Answer doesn't match question.

**Solutions**:

1. Enable/raise reranking (`enable_rerank: true`, tune `rerank_top_k`)
2. Use a more specific mode (`local` for entity questions)
3. Check if documents cover the topic

### Slow Queries

**Symptoms**: Latency > 2 seconds.

**Solutions**:

1. Reduce `max_results` (fewer chunks = faster)
2. Use `naive` mode for simple questions
3. Check LLM provider latency (`stats.retrieval_time_ms` vs `generation_time_ms`)

---

## What You Learned

✅ All 6 query modes and their strengths  
✅ `mix` is the production default; `hybrid` is the 3-arm interleave  
✅ `global` is relationship-vector search (not community reports)  
✅ Real per-query tuning fields (`max_results`, `mix_weights`, rerank, filters)  
✅ A/B testing approaches  
✅ Common issues and solutions

---

## Next Steps

| Tutorial                                  | Description                 |
| ----------------------------------------- | --------------------------- |
| [Multi-Tenant Setup](/docs/tutorials/multi-tenant/)     | Building a SaaS application |
| [Custom Entity Types](/docs/concepts/entity-extraction/) | Domain-specific extraction  |
| [API Integration](/docs/integrations/custom-clients/)     | Building on EdgeQuake       |

---

## See Also

- [Query Modes Deep-Dive](/docs/deep-dives/query-modes/) - Detailed algorithm explanation
- [REST API](/docs/api-reference/rest-api/) - Query endpoint reference
- [Hybrid Retrieval](/docs/concepts/hybrid-retrieval/) - Conceptual overview
