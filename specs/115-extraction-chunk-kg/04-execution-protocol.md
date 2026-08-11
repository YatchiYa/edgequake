# 04 — Execution Protocol (Dual-SUT, Mistral Small)

> Binding sample: LightRAG paper PDF + gold MD twin.  
> Binding LLM: **mistral-small-latest** + **mistral-embed** (1024-d), base `https://api.mistral.ai/v1`.

## Hypotheses under test

| ID | Hypothesis | Falsifier |
|----|------------|-----------|
| H-C1 | Product adaptive → N ≈ 1.5× fair/LR on gold MD | Geometry N_B/N_A ≈ 1 |
| H-C2 | M tracks N under same model | Live M_B/M_A ≈ N_B/N_A |
| H-C3 | Fair EQ U ≈ LR U (same model, same text) | \|U_A−U_C\|/U_C &gt; 0.5 with Jaccard ≪ 0.4 |
| H-C4 | Pdf strategy changes N vs F at same size | Arm D N differs at matched size |
| H-C5 | True over-extract under fair pins | H-C3 fails after matched geometry |

## Arms

| Arm | SUT | Chunk pins | Strategy | LLM |
|-----|-----|------------|----------|-----|
| **A** | EdgeQuake fair | `ADAPTIVE=0`, 1200/100 | Recursive (or Fixed F for pure size parity) | mistral-small-latest |
| **B** | EdgeQuake product | Adaptive ON → 800/~66 on gold | Product default (Pdf if PDF source) | same |
| **C** | LightRAG | CHUNK_SIZE=1200, OVERLAP=100 | **F** (default) | same |
| **D** | Geometry-only | Matched sizes | EQ Pdf vs LR F | none |

## Confound control

```ascii
  MUST MATCH across live A/B/C extract claims
  ┌─────────────────────────────────────────┐
  │  text bytes (gold MD)                   │
  │  LLM model + temperature policy         │
  │  embedding model + dim                  │
  │  gleaning max = 1                       │
  │  extract caps 40 / 100                  │
  │  extraction language = English          │
  └─────────────────────────────────────────┘
  MAY DIFFER (document as confound)
  ┌─────────────────────────────────────────┐
  │  chunk_token_size / overlap (A vs B)    │
  │  strategy Pdf vs F (Arm D / PDF upload) │
  │  storage backend (memory vs PG vs nano) │
  └─────────────────────────────────────────┘
```

## Metrics schema

| Column | Definition |
|--------|------------|
| `arm` | A / B / C / D |
| `sample_id` | `S1-pdf` \| `S1-md` |
| `chars` / `doc_tokens` | Input size |
| `chunk_size_pin` / `overlap_pin` | Effective targets |
| `chunk_count` N | Chunks produced |
| `mention_entities` M | Pre-merge sum (EQ card / instrumented) |
| `mention_relations` | Pre-merge relation sum |
| `unique_nodes` U | Graph unique entities |
| `unique_edges` | Graph unique relations |
| `ents_per_1k_chars` | `1000*M/chars` and/or `1000*U/chars` |
| `mode` | `geometry-only` \| `live-mistral` |
| `elapsed_s` | Wall time |
| `llm_model` / `embed_model` | Pins used |

## Procedure

### Step 0 — Preconditions

```bash
test -n "$MISTRAL_API_KEY"
test -f papers/light_rag_2410.05779v3.pdf
test -f zz_test_docs/academic_papers/lighrag_2410.05779v3.pymupdf.gold.md
# LightRAG checkout
test -d /Users/raphaelmansuy/Github/03-working/LightRAG/lightrag
```

### Step 1 — Geometry (no LLM)

```bash
python3 specs/115-extraction-chunk-kg/experiments/geometry_probe.py
```

Expect: N(1200)=13, N(800)=20, N(600)=26 on gold MD via real LightRAG F chunker.

### Step 2 — Live LightRAG (Arm C)

```bash
python3 specs/115-extraction-chunk-kg/experiments/run_lightrag_mistral.py
```

Writes `measurements/lightrag_live.json` with N, U, edges, timings.

### Step 3 — Live EdgeQuake (Arms A + B)

```bash
# Requires DATABASE_URL (make postgres-start) + MISTRAL_API_KEY
python3 specs/115-extraction-chunk-kg/experiments/run_edgequake_mistral.py
```

Creates isolated workspaces `spec115-fair` and `spec115-product`, ingests **gold MD** (and optionally PDF), records document M + AGE U.

### Step 4 — Report

Fill [05-execution-report.md](05-execution-report.md) and [measurements/SUMMARY.md](measurements/SUMMARY.md).

## Pass / interpret rules

| Observation | Supports |
|-------------|----------|
| N_B / N_A ≈ 1.5 on gold | **H-C1** |
| M_B / M_A ≈ N_B / N_A | **H-C2** |
| U_A ≈ U_C (±30%) under fair pins | **not H-C5** |
| U_A ≫ U_C under fair pins | **H-C5** (true over-extract) |
| EQ card M ≫ AGE U | SPEC-108 LAW-X1 (expected) |
| Pdf N ≠ F N at same size | **H-C4** |

## Honesty labels

- Never label heuristic stride as “production chunker.”
- Never compare EQ card M to LR U without saying so.
- Never claim PDF-path parity when extract arm used gold MD.
