# 010 — Smoke → Core Runbook

**Cross-ref:** [000](./000-index.md) · [003](./003-fair-evaluation-protocol.md) · [005](./005-mode-map-and-pins.md)

---

## Prerequisites

```bash
# EdgeQuake backend (Postgres required) — start with Mistral pins
export MISTRAL_API_KEY=...
export EDGEQUAKE_LLM_PROVIDER=mistral EDGEQUAKE_LLM_MODEL=mistral-small-latest
export EDGEQUAKE_EMBEDDING_PROVIDER=mistral MISTRAL_EMBEDDING_MODEL=mistral-embed
export EDGEQUAKE_VISION_PROVIDER=mistral EDGEQUAKE_VISION_MODEL=mistral-small-latest
export VLM_PROCESS_ENABLE=true

make postgres-start
make backend-bg   # or make dev-bg
curl -s http://127.0.0.1:8080/health | python3 -m json.tool

# Keys / URLs
export LLM_API_KEY=$MISTRAL_API_KEY   # GraphRAG-Bench generation_eval (Mistral-compatible)
export EDGEQUAKE_API_URL=http://127.0.0.1:8080
export EDGEQUAKE_API_KEY=...          # if auth enabled (X-API-Key / Bearer)

# Optional LightRAG sibling
export BENCH001_LIGHTRAG_REPO=/path/to/LightRAG
```

> `make bench001-smoke` / `bench001-core` export the Mistral LLM / vision / embed env vars automatically for the harness process. The **backend** must also be running under the same pins.

---

## Install & doctor

```bash
make bench001-install
make bench001-doctor
```

Doctor checks: HF hub, fixture files, EQ `/health`, **`MISTRAL_API_KEY`**, LightRAG import (warn if missing), prints `P0_mistral_mix` pins.

---

## Freeze smoke

```bash
make bench001-freeze-smoke
```

Downloads medical corpus + questions into `~/.cache/edgequake/bench001/` and verifies all 40 smoke IDs exist.

---

## Fast smoke gate (recommended first)

~8 stratified questions, query-only against warm indexes, concurrency 12. Target wall **≤ 3 min**.

```bash
export EDGEQUAKE_API_URL=http://127.0.0.1:8090   # use make sync-dev-ports backend
export BENCH001_EQ_WORKSPACE_ID=<warm-workspace-uuid>
make bench001-smoke-fast
# artifacts: specs/001-benchmark/e2e/artifacts/smoke-fast/
```

## Smoke (default launch, n=40)

```bash
make bench001-smoke
```

Equivalent:

```bash
cd tools/bench001
python3 -m bench001.cli smoke --api "$EDGEQUAKE_API_URL"
```

Useful flags:

| Flag | Effect |
|------|--------|
| `--query-only` | Skip ingest; reuse indexes |
| `--eq-only` | Skip LightRAG (scorecard `valid: false`) |
| `--lr-only` | Skip EdgeQuake |
| `--dry-run` | Offline plumbing; writes `smoke-dry-run/` |
| `--force-ingest` | Rebuild indexes |
| `--max-questions N` | Truncate (debug dir; `valid: false`) |
| `--llm-provider/model` | SUT LLM (default `mistral` / `mistral-small-latest`) |
| `--vision-provider/model` | Vision (default mistral-small-latest) |
| `--embedding-provider/model/dim` | Embed (default `mistral` / `mistral-embed` / 1024) |
| `--judge-provider/model` | Official judge LLM (default = SUT LLM) |
| `--judge-base-url` | Judge API base (default = SUT base URL) |
| `--judge-embedding-model` | Acc cosine embed (default `mistral-embed`; paper: BGE) |
| `--judge-embed-backend` | `auto` / `openai_compat` / `hf_bge` |
| `--judge-temperature` | Judge LLM temperature (default `0`) |
| `--acc-factuality-weight` | Acc mix weight on statement-F1 (default `0.75`) |
| `--answer-style` | `gold` (default) / `concise` / `default` / `verbose` |
| `BENCH001_PUBLISH_FAIRNESS` | `1` (default): profile `P0_mistral_mix_v2`, top-k=30, L2 required, empty≤5% |
| `BENCH001_RETRIEVE_TOPK` | Matched retrieval budget (default **30**) |
| `BENCH001_EVAL_CONCURRENCY` | Judge sample fan-out (default **16**, max 64); gen∥retrieval + qtypes parallel |
| `make bench001-acc-canary` | Acc instrument canaries (judge-only; must pass before Acc claims) |
| `make bench001-smoke-fast-acc` | Acc-lift gate n=8: gold + `mistral-medium-latest` both SUTs+judge, eval∥=24 |
| `make bench001-smoke-acc` | Acc-lift smoke n=40 (same pins; query-only; parallel judgment) |
| `make bench001-smoke-paper` | Paper-track Acc rescore on frozen smoke preds (`P0_paper`) |
| `make bench001-smoke-fast-large` | Ablation: `mistral-large-latest` SUT+judge, eval∥=24 |

**Ladder:** `acc-canary` → `smoke-fast-acc` → `smoke-acc` → optional `smoke-paper`. SUMMARY must show Acc / F1 / cos.

All provider/judge pins are written to `scorecard.pins.lineage` and `SUMMARY.md` → **Model lineage**.

**Publishable smoke (n=40):**

```bash
export BENCH001_PUBLISH_FAIRNESS=1
export BENCH001_EQ_WORKSPACE_ID=<warm-workspace>
export BENCH001_QUERY_CONCURRENCY=4 BENCH001_EVAL_CONCURRENCY=4
python3 -m bench001.cli smoke --api "$EDGEQUAKE_API_URL" --query-only
# Require: valid=true, L2 retrieval in SUMMARY, profile P0_mistral_mix_v2
```

Artifacts: `specs/001-benchmark/e2e/artifacts/smoke/`

---

## Core (cost-gated)

```bash
make bench001-core
# or:
python3 -m bench001.cli core --api "$EDGEQUAKE_API_URL" --i-accept-cost
```

---

## Report & progression

```bash
python3 -m bench001.cli report smoke
python3 -m bench001.cli report smoke --compare history/smoke-<utc>
# Ladder ledger (auto-updated each run):
cat specs/001-benchmark/e2e/artifacts/PROGRESS.md
# Live phase ticks:
cat specs/001-benchmark/e2e/artifacts/smoke/progress.json
```

After fixing harness bugs (e.g. context export), re-score with:

```bash
# Warm indexes already present:
python3 -m bench001.cli smoke --query-only --api "$EDGEQUAKE_API_URL"
```

---

## Expected smoke timings

| Path | Target |
|------|--------|
| `--dry-run` | < 30 s |
| `--query-only` (warm) | ≤ 20 min |
| Cold ingest + query (medical) | ≤ 60 min |

---

## Troubleshooting

| Symptom | Fix |
|---------|-----|
| `MISTRAL_API_KEY` missing | Export key; doctor will fail closed |
| Wrong embed dim / provider | Restart backend with `EDGEQUAKE_EMBEDDING_PROVIDER=mistral` + `mistral-embed` |
| `DATABASE_URL not set` | Use `make backend-bg` |
| LightRAG missing | Set `BENCH001_LIGHTRAG_REPO` or run `--eq-only` for EQ-only debug |
| Judge fails | Set `LLM_API_KEY=$MISTRAL_API_KEY`; or harness falls back to local rouge Acc with `judge: rouge_proxy` and `valid: false` |
| Medical ingest timeout | Increase poll timeout; medical context ~1M chars — use `async_processing` |
