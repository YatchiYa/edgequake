# 003 — Fair Evaluation Protocol (RAG Adaptation)

**Cross-ref:** [001](./001-first-principles.md) · [002](./002-benchmark-deep-dive-mmlongbench.md) · [010 Runbook](./010-smoke-then-full-runbook.md) · [012 Scorecard](./012-acceptance-criteria-and-scorecard.md)

---

## 1. Protocol overview

```text
┌──────────────┐   ┌──────────────┐   ┌──────────────┐   ┌──────────────┐
│ 0. Prepare   │──▶│ 1. Ingest    │──▶│ 2. Query     │──▶│ 3. Score     │
│ cache+subset │   │ real PDFs    │   │ hybrid RAG   │   │ extract+rule │
└──────────────┘   └──────────────┘   └──────────────┘   └──────────────┘
```

Stages 2–3 mirror MATHVISTA / MMLongBench-Doc three-step evaluation; stage 1 replaces “render all pages to LVLM context” with EdgeQuake ingestion.

---

## 2. Stage 0 — Prepare (deterministic)

### 2.1 Download

```bash
# Pseudocode — implemented by bench047 download tool
huggingface-cli download yubo2333/MMLongBench-Doc \
  --repo-type dataset \
  --local-dir "$EDGEQUAKE_BENCH_CACHE/mmlongbench-doc"
```

Record: dataset revision, download UTC, byte size, file count.

### 2.2 Smoke subset (10 documents)

**Goal:** high-signal, stratified, cheap enough to iterate daily.

Selection algorithm (seed = `047-smoke-v1`):

1. Parse all Q&A; group by `doc_id`.  
2. Require each selected doc to have ≥1 answerable and prefer mix of sources.  
3. Greedy cover until 10 docs maximizing diversity of:
   - `doc_type`
   - presence of chart/image evidence
   - presence of cross-page questions
   - presence of unanswerable questions  
4. Freeze IDs in `fixtures/smoke_doc_ids_v1.txt` (committed).  
5. Include **all questions** for those 10 docs (not a random Q subsample) so Acc/F1 within-doc is meaningful.

Core subset (~40 docs): extend smoke with seed `047-core-v1`, same algorithm, committed list.

Full: all docs / all questions.

### 2.3 Workspace

```text
workspace_slug = bench047-{stage}-{git_sha8}-{utc_compact}
```

Fresh workspace per run. Never reuse a dirty graph.

---

## 3. Stage 1 — Ingest (real PDFs)

### 3.1 Locked env profile

```bash
export EDGEQUAKE_LLM_PROVIDER=mistral
export EDGEQUAKE_LLM_MODEL=mistral-small-latest
export EDGEQUAKE_EMBEDDING_PROVIDER=mistral
export MISTRAL_EMBEDDING_MODEL=mistral-embed
export EDGEQUAKE_VISION_PROVIDER=mistral
export EDGEQUAKE_VISION_MODEL=mistral-small-latest
export EDGEQUAKE_EMBEDDING_BATCH_SIZE=16   # mistral-embed token budget safety
```

### 3.2 Upload contract

For each PDF:

1. `POST /api/v1/documents` (multipart) with vision enabled.  
2. Poll until `Completed` or `Failed`.  
3. Persist per-doc record: `doc_id`, edgequake_uuid, status, pages, chunks, entities, cost, latency, error.

**Fail closed:** if vision capability is disabled in client, abort run with `INVALID_VISION_CONFIG`.

### 3.3 Ingest acceptance (smoke)

| Gate | Threshold |
|------|-----------|
| Docs completed | ≥ 9 / 10 |
| Mean time to complete | recorded (no hard fail) |
| Any `Failed` | listed; smoke may continue if ≥9 OK, but scorecard `ingest_ok=false` if <9 |

Questions whose PDF failed ingest are **excluded** from scoring and counted under `skipped_ingest_failed`.

---

## 4. Stage 2 — Query (hybrid)

### 4.1 Request

```http
POST /api/v1/query
Content-Type: application/json

{
  "query": "<official question string>",
  "mode": "hybrid",
  "workspace": "<bench workspace>"
}
```

Optional but recommended for analysis (not for changing answers):

- return sources / chunk ids  
- store full response text + latency + token usage  

### 4.2 Blindness rules

- Do **not** pass `evidence_pages` to the API.  
- Do **not** restrict retrieval filters to gold pages in primary mode.  
- Temperature: provider default or `0` if configurable — pin in scorecard.

### 4.3 Unanswerable behavior

Prompting may include a system instruction (EdgeQuake query system prompt) that allows saying the answer is not in the knowledge base. Record the exact system prompt hash. Do not specially case gold-unanswerable questions in the client.

---

## 5. Stage 3 — Score

### 5.1 Extract

```text
a_short = extract_answer(question, a_long, prompt=OFFICIAL_PROMPT, model=EXTRACTOR)
```

Default extractor: `gpt-4o` (official). Alternative: `mistral-small-latest` under profile `extractor=mistral`.

### 5.2 Rule score

Port / call upstream `eval_score(gt, pred, answer_format)`.

### 5.3 Aggregates

Compute exactly as `eval_acc_and_f1` + `show_results` slices:

- Overall Acc, Overall F1  
- Single-page Acc  
- Cross-page Acc  
- Unanswerable Acc  
- Per evidence source Acc  
- Per document type Acc  

### 5.4 RAG diagnostics (extra, clearly separated)

| Metric | Definition |
|--------|------------|
| `retrieval_hit@k` | Proxy: any returned source page intersects gold `evidence_pages` (when page metadata available) |
| `ingest_coverage` | completed_docs / selected_docs |
| `answer_empty_rate` | fraction empty / error responses |
| `cost_usd_total` | sum provider costs if available |
| `p50/p95_query_latency_ms` | query wall time |

These do **not** replace Acc/F1.

---

## 6. Reporting honesty banner (mandatory in Markdown summary)

```markdown
> **Task note:** This is an EdgeQuake **RAG adaptation** of MMLongBench-Doc
> (ingest + hybrid retrieve + generate). It is **not** comparable to the
> official LVLM leaderboard without caveats. Official LVLM GPT-4o F1 ≈ 44.9%
> is a difficulty reference only.
```

---

## 7. Reproducibility checklist

- [ ] Dataset revision pinned  
- [ ] Fixture list SHA  
- [ ] EdgeQuake git SHA + VERSION  
- [ ] models.toml hashes for mistral entries  
- [ ] Env profile dump (secrets redacted)  
- [ ] Extractor model + prompt SHA  
- [ ] `eval_score.py` SHA  
- [ ] RNG seed for any sampling  
- [ ] Artifact directory path  

---

## 8. Invalid run taxonomy

| Code | Meaning |
|------|---------|
| `INVALID_VISION_CONFIG` | Vision model cannot receive images |
| `INVALID_EMBED_DIM` | Embedding dim ≠ 1024 for mistral-embed path |
| `INVALID_DATASET` | Q&A/PDF mismatch or corrupt cache |
| `INVALID_EXTRACTOR` | Extractor API failed >5% |
| `INVALID_WORKSPACE` | Contaminated / wrong workspace |
| `PARTIAL_INGEST` | < stage threshold completed |

Invalid runs may still write diagnostics but must set `"valid": false` in scorecard.

Next: lens docs [004](./004-ai-engineer-lens.md)–[008](./008-product-sre-lens.md), then [009](./009-implementation-plan.md).
