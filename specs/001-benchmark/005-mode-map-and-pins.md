# 005 — Mode Map & Pins

**Cross-ref:** [SPEC-047/017 naming traps](../047-rag-evaluation/017-lightrag-vs-edgequake-query-pipeline-assessment.md)

---

## 1. Headline mode map (locked)

| SUT | Query mode | Meaning |
|-----|------------|---------|
| EdgeQuake | **`mix`** | Multi-arm RRF; **publish Acc** forces always-on local+global+naive (`EDGEQUAKE_MIX_ARM_GATE=false`) |
| LightRAG | **`mix`** | KG local+global + naive vector context (always all three) |

**Profile id (publish Acc):** `P0_mistral_mix_lrlike_arms_v2` (or medium: `P0_mistral_medium_mix_lrlike_arms_v2`)

> **Naming trap:** Production EQ Mix still intent-gates Factual → naive-only when `EDGEQUAKE_MIX_ARM_GATE` is unset/true. Fair dual-SUT Acc requires the **server** pin `EDGEQUAKE_MIX_ARM_GATE=false` (`make bench001-backend-lrlike` / Acc make targets). Scorecard records `pins.mix_arm_gate`.

Do **not** headline-compare EQ `hybrid` to LR `hybrid` — names collide but semantics differ:

| Name | LightRAG | EdgeQuake |
|------|----------|-----------|
| `hybrid` | local + global KG merge | local + global + naive round-robin |
| `mix` | KG + naive vector | multi-arm RRF + intent gate |
| `naive` | vector-only | vector-only |

Ablations (labeled, never mixed into headline): `P1_hybrid`, `P2_naive`.

---

## 2. Provider pins (defaults = Mistral Small + mistral-embed)

All roles are **parameters** (CLI flag or env). Headline defaults:

| Role | CLI | Env (priority) | Default |
|------|-----|----------------|---------|
| LLM provider | `--llm-provider` | `BENCH001_LLM_PROVIDER` → `EDGEQUAKE_LLM_PROVIDER` | `mistral` |
| LLM model | `--llm-model` | `BENCH001_LLM_MODEL` → `EDGEQUAKE_LLM_MODEL` → `MISTRAL_MODEL` | **`mistral-small-latest`** |
| Vision provider | `--vision-provider` | `BENCH001_VISION_PROVIDER` → `EDGEQUAKE_VISION_PROVIDER` | `mistral` |
| Vision model | `--vision-model` | `BENCH001_VISION_MODEL` → `EDGEQUAKE_VISION_MODEL` | **`mistral-small-latest`** |
| Embed provider | `--embedding-provider` | `BENCH001_EMBEDDING_PROVIDER` → `EDGEQUAKE_EMBEDDING_PROVIDER` | `mistral` |
| Embed model | `--embedding-model` | `BENCH001_EMBEDDING_MODEL` → `MISTRAL_EMBEDDING_MODEL` | **`mistral-embed`** |
| Embed dim | `--embedding-dim` | `BENCH001_EMBEDDING_DIM` | **1024** |
| LLM base URL | `--llm-base-url` | `BENCH001_LLM_BASE_URL` | `https://api.mistral.ai/v1` |
| **Judge** provider | `--judge-provider` | `BENCH001_JUDGE_PROVIDER` | same as LLM |
| **Judge** model | `--judge-model` | `BENCH001_JUDGE_MODEL` | same as LLM (default mistral-small-latest) |
| Judge base URL | `--judge-base-url` | `BENCH001_JUDGE_BASE_URL` | same as LLM base URL |
| Judge metric embed | `--judge-embedding-model` | `BENCH001_JUDGE_EMBEDDING_MODEL` | **`mistral-embed`** (Acc cosine term). Paper parity: `BAAI/bge-large-en-v1.5` |
| Judge embed backend | `--judge-embed-backend` | `BENCH001_JUDGE_EMBED_BACKEND` | `auto` → API for mistral-embed, HF for BGE |
| Judge temperature | `--judge-temperature` | `BENCH001_JUDGE_TEMPERATURE` | `0.0` |
| Acc factuality weight | `--acc-factuality-weight` | `BENCH001_ACC_FACTUALITY_WEIGHT` | `0.75` (`Acc = w·F1 + (1-w)·embed_cos`) |
| Answer style | `--answer-style` | `BENCH001_ANSWER_STYLE` | **`gold`** (short gold-like answers; Acc F1). Also: `concise` / `default` / `verbose` |
| Chunk size (fair Acc) | — | `EDGEQUAKE_CHUNK_SIZE` + `EDGEQUAKE_ADAPTIVE_CHUNKING=0` (server) | **1200** / overlap **100** (`pins.chunk_token_size`); production may keep adaptive on |
| Adaptive chunking | — | `EDGEQUAKE_ADAPTIVE_CHUNKING` (server) | **`false`** for publish Acc (`pins.adaptive_chunking`); production default `true` |
| retrieve_topk (matched) | — | `BENCH001_RETRIEVE_TOPK` | **30** (paper H.2; both SUTs) |
| Publish fairness | — | `BENCH001_PUBLISH_FAIRNESS` | `1` → L2 required, empty≤5%, LR-like Mix arms |
| EQ Mix arm gate | — | `EDGEQUAKE_MIX_ARM_GATE` (server) | **`false`** for publish Acc (`pins.mix_arm_gate=false`); production default `true` |
| EQ related chunks | — | `EDGEQUAKE_RELATED_CHUNK_NUMBER` (server) | **`5`** (LightRAG `RELATED_CHUNK_NUMBER` parity; `pins.related_chunk_number`) |
| EQ Mix fusion | — | `EDGEQUAKE_MIX_FUSION` (server) | **`rrf`** default; ablation `round_robin` (`pins.mix_fusion`) |
| Orphan retract on recover | — | `EDGEQUAKE_ORPHAN_RETRACT_ON_RECOVER` (server) | **`0`** for Acc backends (avoid wiping warm indexes on restart) |
| EQ query concurrency | — | `BENCH001_QUERY_CONCURRENCY` | Acc default **4** (`pins.eq_query_concurrency`) |
| LR query concurrency | — | `BENCH001_LR_QUERY_CONCURRENCY` | Acc default **1** (`pins.lr_query_concurrency_effective`; do not raise past proven Acc∥=1) |
| Embed concurrency (ingest) | — | `EDGEQUAKE_ALLOW_LOCAL_HIGH_CONCURRENCY` | Set **`1`** on Acc backends if startup logs clamp workers as `extract_provider=ollama` despite Mistral — speeds force-ingest; does not change Acc query semantics |
| Ingest max chars | — | `BENCH001_INGEST_MAX_CHARS` | smoke-fast Acc default **100000** (`pins.ingest_max_chars`); `0` = full ~1.05MB medical blob. Caps isolate EQ workspace / LR dir as `*-c{N}` |
| Ingest timeout | — | `BENCH001_INGEST_TIMEOUT_S` | smoke-fast Acc default **1800**s; fail-closed on doc `failed` (no silent hang) |
| Profile id | — | auto | `P0_mistral_mix_lrlike_arms_v2` when publish fairness + gate off |

**Force-ingest honesty:** Acc Makefile targets default to `--query-only`. Set `BENCH001_FORCE_INGEST=1` only when the chunk pin (or corpus) changes. Pin `EDGEQUAKE_API_URL` / `.edgequake-dev-ports.env` to the Acc backend port before force-ingest — a stale ports file silently points uploads at a dead port.

**Progress:** force-ingest prints `ingest heartbeat` every ~10s with `pct` / `eta` / `stage` from document `stage_progress`, and updates `progress.json` `ingest_eq`. Do not treat task=`indexed` as done while `display_status=storing`.

### Why mistral-embed for the judge (not BGE by default)?

Official Acc = **0.75 × statement-F1 + 0.25 × embed_cosine(answer, gold)**.

| Choice | Pros | Cons |
|--------|------|------|
| **mistral-embed** (default) | Same family as SUT retrieval; no local HF download; coherent `P0_mistral_mix` lineage | Not identical to paper Table-2 numbers |
| **BGE-large** (paper) | Matches published GraphRAG-Bench eval README / Table 2 | Extra local model; different vector space than SUT |

Use BGE only when you need **paper-comparable Acc**:

```bash
python3 -m bench001.cli smoke-fast --query-only \
  --judge-embedding-model BAAI/bge-large-en-v1.5 \
  --judge-embed-backend hf_bge
```

### How to get closer Acc while keeping a **Mistral** judge

Acc ≈ **0.75 × statement-F1 + 0.25 × embed_cos**. With L2 Evidence Recall already ~0.95, Acc is limited by **answer shape vs gold**, not retrieval. Keep one confound change per run.

| Priority | Lever | Keep Mistral? | Command / pin |
|----------|-------|---------------|---------------|
| **1** | **Gold-format SUT answers** (anti-refusal, 1-sentence facts) | Yes | `--answer-style gold` (default) |
| **2** | **Stronger same-family judge** for statement decomposition | Yes | `--judge-model mistral-medium-latest` |
| **3** | Optional: stronger SUT generator (fair EQ↔LR) | Yes | `--llm-model mistral-medium-latest` (both SUTs) |
| **4** | Keep L2 gates; do not “fix Acc” by dropping retrieval | Yes | `BENCH001_PUBLISH_FAIRNESS=1` |
| **5** | Paper Table-2 Acc | No (OpenAI judge) | `--judge-model gpt-4o-mini` + BGE embed |

```bash
# Recommended Acc-lift (fair: medium on BOTH SUTs + judge, gold answers, eval∥24)
make bench001-smoke-fast-acc   # n=8 gate
make bench001-smoke-acc        # n=40 release Acc-lift

# Larger SUT+judge ablation
make bench001-smoke-fast-large
```

**Judgment parallelism (wall-time):** EQ∥LR scoring already; within each SUT, `generation_eval` ∥ `retrieval_eval`; within each eval, samples + question types concurrent via `--eval-concurrency` / `BENCH001_EVAL_CONCURRENCY` (default **16**, max **64**).

**Do not:** raise `--acc-factuality-weight` to hide bad F1, or compare to paper 0.63 without `P0_paper`.

### Acc relevance gates (P15)

```bash
make bench001-acc-canary          # paraphrase high / wrong-fact low (judge-only)
make bench001-smoke-fast-acc      # decision Acc with F1+cos in SUMMARY
make bench001-smoke-acc           # n=40
make bench001-smoke-paper         # P0_paper rescore (needs OPENAI_API_KEY + BGE)
# or: python3 -m bench001.cli rescore --source smoke --profile-id P0_paper ...
```

Publishable Acc claims require `overall_f1` + `overall_cos` on both SUTs (fail closed: `acc_components_missing`).

### How to tune the judge (knobs)

1. **Answer style** (`--answer-style gold`, default) — primary Acc lever under F1.  
2. **Judge LLM** — `mistral-medium-latest` for nuanced statement F1; `mistral-small-latest` for cheap smoke.  
3. **Temperature** — keep `0` for reproducibility.  
4. **Factuality weight** — leave `0.75` unless ablating the Acc mix itself.  
5. **Metric embed** — `mistral-embed` for stack coherence; BGE for paper parity.

> **Law:** Both SUTs share the same resolved SUT pins. Judge may differ (e.g. GPT-4o-mini judge vs Mistral SUT) but **must** appear in scorecard `pins.lineage`. Headline `P0_mistral_mix` = defaults above; non-default stacks auto-tag `P0_custom_<llm>_<embed>`.

Example — paper-comparable judge, Mistral SUT:

```bash
python3 -m bench001.cli smoke \
  --judge-provider openai --judge-model gpt-4o-mini \
  --judge-base-url https://api.openai.com/v1
# SUT remains mistral-small-latest + mistral-embed unless overridden
```

---

## 3. EdgeQuake API contract

Backend must be started with Mistral env (Makefile `bench001-smoke` exports these):

```bash
export EDGEQUAKE_LLM_PROVIDER=mistral EDGEQUAKE_LLM_MODEL=mistral-small-latest
export EDGEQUAKE_EMBEDDING_PROVIDER=mistral MISTRAL_EMBEDDING_MODEL=mistral-embed
export EDGEQUAKE_VISION_PROVIDER=mistral EDGEQUAKE_VISION_MODEL=mistral-small-latest
export VLM_PROCESS_ENABLE=true
# Fair Acc ↔ LightRAG mix: always-on local+global+naive (server process)
export EDGEQUAKE_MIX_ARM_GATE=false
# or: make bench001-backend-lrlike
```

```http
POST /api/v1/documents
{ "content": "...", "title": "medical", "async_processing": true,
  "chunk_options": { "chunk_size": 1200 } }

POST /api/v1/query
{ "query": "<question>", "mode": "mix" }
```

Workspace isolation via `X-Workspace-Id: bench001-smoke` (or env `EDGEQUAKE_WORKSPACE`).

---

## 4. LightRAG contract

Harness adapter (`bench001.lightrag_runner`) configures LightRAG with Mistral via OpenAI-compatible API:

- `llm_model_name` / complete: `mistral-small-latest`
- `embedding_func`: `mistral-embed` @ `https://api.mistral.ai/v1` (dim **1024**)
- query mode: `mix`

Requires `MISTRAL_API_KEY` and `BENCH001_LIGHTRAG_REPO` (or sibling `../LightRAG`).

---

## 5. Scorecard pin block (required keys)

See [006](./006-scorecard-schema.md) `pins` — must include SUT `llm_*` / `vision_*` / `embedding_*`, judge `judge_*`, and `lineage` (compact model IDs for the run).
