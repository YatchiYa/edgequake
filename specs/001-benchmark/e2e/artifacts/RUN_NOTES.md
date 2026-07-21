# SPEC-001 RUN_NOTES

## Provider pin (locked)

| Role | Value |
|------|-------|
| Profile | `P0_mistral_mix` |
| LLM | `mistral` / `mistral-small-latest` |
| Vision | `mistral` / `mistral-small-latest` |
| Embed | `mistral` / `mistral-embed` (1024-d) |

## 2026-07-19 — Mistral pin retarget + dry-run

| Item | Value |
|------|-------|
| Command | `BENCH001_DRY_RUN=1 make bench001-smoke` |
| Fixture | `smoke_question_ids_v1` (n=40, medical, seed=42) |
| Dataset revision | `dc3a111e77dbaf8bbaf51ef331f3cfc9b1b5c546` |
| Result | `valid=false` (`dry_run;judge:rouge_proxy`) — expected for offline plumbing |
| Artifacts | [`smoke/SUMMARY.md`](./smoke/SUMMARY.md), [`smoke/scorecard.json`](./smoke/scorecard.json) |

### Live dual-SUT

```bash
export MISTRAL_API_KEY=...
export LLM_API_KEY=$MISTRAL_API_KEY
export EDGEQUAKE_API_URL=http://127.0.0.1:8080
export BENCH001_LIGHTRAG_REPO=/path/to/LightRAG   # optional sibling

# Backend must also run under Mistral pins (or rely on make targets that export them
# for the harness — still restart backend with the same env):
export EDGEQUAKE_LLM_PROVIDER=mistral EDGEQUAKE_LLM_MODEL=mistral-small-latest
export EDGEQUAKE_EMBEDDING_PROVIDER=mistral MISTRAL_EMBEDDING_MODEL=mistral-embed
export EDGEQUAKE_VISION_PROVIDER=mistral EDGEQUAKE_VISION_MODEL=mistral-small-latest
export VLM_PROCESS_ENABLE=true

make postgres-start && make backend-bg
make bench001-doctor
make bench001-smoke
```

For official Acc, clone [GraphRAG-Benchmark](https://github.com/GraphRAG-Bench/GraphRAG-Benchmark) and set:

```bash
export BENCH001_GRAPHRAG_BENCH_REPO=/path/to/GraphRAG-Benchmark
```

Judge LLM defaults to `mistral-small-latest` via `https://api.mistral.ai/v1`. Acc cosine term defaults to **`mistral-embed`** (same family as SUT). Paper-parity Acc uses `--judge-embedding-model BAAI/bge-large-en-v1.5 --judge-embed-backend hf_bge`.


## 2026-07-19 — Live dual-SUT smoke (Mistral pins)

| Item | Value |
|------|-------|
| Fixture | `smoke_question_ids_v1` (n=40, medical) |
| Dataset revision | `dc3a111e77dbaf8bbaf51ef331f3cfc9b1b5c546` |
| Profile | `P0_mistral_mix` |
| Query concurrency | 8 |
| Eval concurrency | 8 (`generation_eval --max_concurrent`) |
| Predictions | EQ 40/40 non-empty · LR 40/40 non-empty |
| Judge | **`generation_eval`** → **`valid=true`** |
| EQ Acc | **0.2289** |
| LR Acc | **0.2311** |
| Δ (EQ − LR) | **−0.0023** |
| EQ Acc by type | Fact 0.2261 · Reasoning 0.2303 · Summarize 0.2300 · Creative 0.2291 |
| LR Acc by type | Fact 0.2311 · Reasoning 0.2313 · Summarize 0.2317 · Creative 0.2305 |

### Artifacts

- [`smoke/SUMMARY.md`](./smoke/SUMMARY.md)
- [`smoke/scorecard.json`](./smoke/scorecard.json)
- [`smoke/predictions_eq.json`](./smoke/predictions_eq.json)
- [`smoke/predictions_lr.json`](./smoke/predictions_lr.json)

### Speed knobs

```bash
BENCH001_QUERY_CONCURRENCY=8    # SUT mix queries (LR often needs 1–2)
BENCH001_EVAL_CONCURRENCY=16    # official judge samples (gen∥retrieval + qtypes)
# Larger SUT+judge ablation:
make bench001-smoke-fast-large  # mistral-large-latest, eval∥=24
```


## 2026-07-19 — Acc relevance (P15): F1/cos decompose + canary + paper rescore

| Item | Value |
|------|-------|
| Acc components | `overall_f1` + `overall_cos` in scorecard; `--detailed_output` always on live gen eval |
| F1 JSON fix | Mistral-safe parse for statement classification (was silent F1=0 → Acc≈0.25 floor) |
| Canary | `make bench001-acc-canary` → **passed=true** under `mistral-medium-latest` |
| Rescore smoke-fast | `smoke-fast-rescore-20260719T033933Z` — valid; EQ Acc **0.632** (F1 0.519 / cos 0.972); LR Acc **0.692** (F1 0.603 / cos 0.957) |
| Paper track | `make bench001-smoke-paper` (`P0_paper`, GPT-4o-mini + BGE; needs `OPENAI_API_KEY`) |
| Fail closed | `acc_components_missing` under publish fairness |

## 2026-07-19 — Acc-lift medium + gold + parallel judgment

| Item | Value |
|------|-------|
| Commands | `make bench001-smoke-fast-acc` → `make bench001-smoke-acc` |
| Profile | `P0_mistral_medium_mix_v2` |
| Pins | SUT+judge `mistral-medium-latest`, embed `mistral-embed`, `--answer-style gold`, eval∥=24 |
| smoke-fast archive | `history/smoke-fast-20260719T032233Z` — valid, EQ Acc 0.2431 / LR 0.2393 |
| smoke n=40 archive | `history/smoke-20260719T032752Z` — valid, EQ Acc **0.2402** / LR **0.2411** / Δ **−0.0008** |
| L2 (n=40, prior medium) | EQ recall ~0.71 / LR ~0.86 (retrieval OK; Acc not retrieval-bound) |
| Score wall vs prior smoke | **136 s → ~50 s** (eval 4→24 + gen∥retrieval + qtypes∥) |
| Shape check | Gold length; no `[n]` citations after prompt rule 7; ROUGE-L Fact ~0.37 |
| Acc read | Still ~0.24 → statement-F1 near floor (wrong facts vs gold, e.g. biopsy ≠ hematopathologist). Acc ≈ 0.25×embed_cos. Next content lever / optional `P0_paper` — not more model size. |

## 2026-07-19 — Large Mistral + parallel judgment (`smoke-fast`, n=8)

| Item | Value |
|------|-------|
| Command | `make bench001-smoke-fast-large` |
| Profile | `P0_mistral_large_mix_v2` |
| SUT+judge LLM | `mistral-large-latest` (embed stays `mistral-embed`) |
| Answer style | `gold` |
| Eval concurrency | **24** (gen∥retrieval + parallel question types) |
| Archive | `history/smoke-fast-20260719T030730Z` |
| Valid | **true** (L2 present) |
| EQ Acc / LR Acc / Δ | **0.2423 / 0.2423 / +0.0000** |
| Score wall (e_conc 4→24) | **115 s → 88 s** (~23% faster judgment phase) |

First principles: upsizing LLM is fair only when both SUTs + judge share the pin; judgment is post-hoc on frozen predictions so parallelism cannot change Acc definitions.


## 2026-07-19 — Fast smoke gate (`smoke-fast`, n=8)

| Item | Value |
|------|-------|
| Command | `make bench001-smoke-fast` / `bench001.cli smoke-fast --query-only` |
| API | `http://127.0.0.1:8090` |
| Fixture | `smoke_fast_question_ids_v1` (2×4 types) |
| Wall clock | **~99 s** (query∥ + judge∥, concurrency 12) |
| Context export | EQ empty-ctx **0.0** · LR empty-ctx **0.0** |
| Judge | `generation_eval` → **valid=true** |
| EQ Acc | **0.2280** |
| LR Acc | **0.2303** |
| Δ | **−0.0023** |

Artifacts: [`smoke-fast/SUMMARY.md`](./smoke-fast/SUMMARY.md)

Speed recipe: warm indexes + `--query-only` + EQ∥LR + conc 12. Full smoke (n=40) remains the release gate.
