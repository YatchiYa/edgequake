# 010 — Smoke → Full Runbook (Easy to Run)

**Cross-ref:** [000](./000-index.md) · [003](./003-fair-evaluation-protocol.md) · [009](./009-implementation-plan.md) · [012](./012-acceptance-criteria-and-scorecard.md)

> Until `make bench047-*` exists, use the **Manual path**. After EQ-047-10, prefer Make targets.

---

## 0. Prerequisites

```bash
# Tools
docker --version
rustc --version
python3 --version   # >= 3.10 recommended for harness

# Keys
export MISTRAL_API_KEY=...     # required
export OPENAI_API_KEY=...      # required for official extractor
# OR
export BENCH047_EXTRACTOR=mistral   # all-Mistral path (label in scorecard)

# Cache
export EDGEQUAKE_BENCH_CACHE="${EDGEQUAKE_BENCH_CACHE:-$HOME/.cache/edgequake/bench047}"
mkdir -p "$EDGEQUAKE_BENCH_CACHE"
```

License reminder: MMLongBench-Doc data is **CC BY-NC 4.0** — research use only.

---

## 1. Start EdgeQuake (Mistral hybrid profile)

```bash
cd /path/to/edgequake

make postgres-start
# Wait until healthy
docker ps | grep edgequake-postgres

# Locked bench profile (overrides Makefile pixtral default for vision)
export EDGEQUAKE_LLM_PROVIDER=mistral
export EDGEQUAKE_LLM_MODEL=mistral-small-latest
export EDGEQUAKE_EMBEDDING_PROVIDER=mistral
export MISTRAL_EMBEDDING_MODEL=mistral-embed
export EDGEQUAKE_VISION_PROVIDER=mistral
export EDGEQUAKE_VISION_MODEL=mistral-small-latest
export EDGEQUAKE_EMBEDDING_BATCH_SIZE=16

make backend-bg
curl -s http://localhost:8080/health | python3 -m json.tool
```

**Doctor checks (manual until CLI exists):**

1. `llm_provider_name` includes mistral  
2. embedding model `mistral-embed` / dimension 1024  
3. vision model is Small (not silently pixtral unless you chose that profile)  
4. upload of a 1-page image PDF produces non-empty markdown with visual content  

---

## 2. Stage A — SMOKE (10 documents)

### Make path (target state)

```bash
make bench047-smoke
cat specs/047-rag-evaluation/e2e/artifacts/smoke/SUMMARY.md
cat specs/047-rag-evaluation/e2e/artifacts/smoke/scorecard.json | python3 -m json.tool | head
```

### Manual path (pre-Make)

```bash
# 1) Download dataset
python3 - <<'PY'
# placeholder until tools/bench047 ships — use HF datasets
from datasets import load_dataset
ds = load_dataset("yubo2333/MMLongBench-Doc/data", split="train")
print(len(ds), ds[0].keys())
PY

# 2) Restrict to fixtures/smoke_doc_ids_v1.txt
# 3) Create workspace, upload those PDFs with vision
# 4) Query each question with mode=hybrid
# 5) Extract short answers, score with vendored eval_score
# 6) Write scorecard.json + SUMMARY.md
```

### What “good smoke” looks like

| Check | Expect |
|-------|--------|
| `valid` | `true` |
| `ingest_coverage` | ≥ 0.9 |
| `n_docs` | 10 |
| `n_questions` | > 0 (typically dozens) |
| Overall Acc / F1 | finite numbers in SUMMARY |
| Banner | RAG adaptation note present |
| Time | recorded; no requirement to beat LVLM F1 |

**Progression signal:** save `scorecard.json` as baseline `smoke-v0`. Later EdgeQuake versions compare F1 delta.

---

## 3. Stage B — CORE (~40 documents)

```bash
make bench047-core
# or: bench047 core --i-accept-cost
```

Compare:

```bash
# target UX
bench047 report e2e/artifacts/core --compare e2e/artifacts/smoke
```

Expect: more stable slice estimates; chart/image and cross-page gaps become visible.

---

## 4. Stage C — FULL (135 / 1091)

```bash
make bench047-full
# long-running; use --resume if interrupted
tail -f specs/047-rag-evaluation/e2e/artifacts/full/logs/heartbeat.jsonl
```

Publish: attach `scorecard.json` + `SUMMARY.md` to release notes with dataset revision + EdgeQuake VERSION.

---

## 5. Easy evaluation checklist (human)

Open `SUMMARY.md` and answer:

1. Is `valid: true`? If no → fix ops, do not interpret F1.  
2. What is Overall F1? Acc?  
3. Which slice is worst (cross-page / chart / unanswerable)?  
4. Did ingest_coverage drop vs last run?  
5. Cost and p95 latency acceptable?

If yes to (1) and you can answer (2)–(3) in under a minute, the harness met the “easy to evaluate” bar.

---

## 6. Ablation quick commands (after Phase 2)

```bash
bench047 smoke --profile P1_naive
bench047 smoke --profile P5_text_parse
bench047 smoke --profile P6_oracle_pages
```

Never mix ablation F1 into the primary progression chart without labeling.

---

## 7. Troubleshooting

| Symptom | Action |
|---------|--------|
| Health not mistral | re-export env; restart backend |
| Embed dim ≠ 1024 | check `MISTRAL_EMBEDDING_MODEL` |
| PDFs Failed | grep backend log; check vision key/rate limits |
| All chart Acc=0 | suspect vision drop (EQ-047-03) |
| Extractor Failed | check OPENAI_API_KEY; retry; switch mistral judge |
| F1=0 with valid preds | answer_format mismatch / extractor not short-form |

---

## 8. One-page command card

```bash
export MISTRAL_API_KEY=... OPENAI_API_KEY=...
export EDGEQUAKE_LLM_PROVIDER=mistral EDGEQUAKE_LLM_MODEL=mistral-small-latest
export EDGEQUAKE_EMBEDDING_PROVIDER=mistral MISTRAL_EMBEDDING_MODEL=mistral-embed
export EDGEQUAKE_VISION_PROVIDER=mistral EDGEQUAKE_VISION_MODEL=mistral-small-latest
make postgres-start && make backend-bg
make bench047-smoke && cat specs/047-rag-evaluation/e2e/artifacts/smoke/SUMMARY.md
```

Next: [011 Complementary Benchmarks](./011-complementary-benchmarks-methodology.md).
