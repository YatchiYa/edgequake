# 030 — Ingest gleaning / breadcrumb parity (Horizon B2)

**Status:** Code shipped · labeled re-ingest **done** · **no Parity promote** (L2 miss)  
**Cross-ref:** [029 Ingest audit](./029-ingest-parity-audit.md) · [028 Roadmap](./028-first-principles-beat-roadmap.md) · [031 B3](./031-structure-aware-chunking.md) · LightRAG extract (gleaning + `---Section Context---`)

---

## 1. B1 root cause (warm Acc WS `8b359190-…`)

| Signal | Finding |
|--------|---------|
| Counts | EQ **429** nodes vs LR **3580** entity keys |
| Jaccard (norm) | **0.009** — misleading alone |
| Soft-overlap | **~66%** of EQ names substring-match an LR name → concepts often present under different surface forms |
| Zero-chunk EQ | **62 / 429** entities lack `source_chunk_ids` |
| Acc text upload | Title `bench001-…` **without `.md`**, `mime_type=None` → `ChunkStrategy::Recursive` → **no heading breadcrumbs** in extract/glean prompts |

LR Acc ingest uses section-context injection + gleaning (`MAX_GLEANING=1`). EQ already has both code paths (`section_context.rs`, `GleaningExtractor`) but Acc bench001 uploads were not selecting Markdown chunking.

---

## 2. Code changes (this step)

| Change | Why |
|--------|-----|
| `text_upload.rs` → `mime_type=text/markdown` | Drive Markdown strategy for API text uploads |
| `resolve_admission_chunk_strategy` | Markdown when `source_type`/`document_type` is markdown even without `.md` title |
| `bench001/client.py` | Explicit `enable_gleaning`, `max_gleaning`, `chunk_strategy=markdown`, `.md` title suffix |
| Audit soft-overlap metric | Don’t over-read Jaccard alone |

---

## 3. Labeled re-ingest experiment (executed)

**Never** overwrite the warm Acc peer workspace silently.

```bash
# Fresh workspace + full-corpus force-ingest under B2 pins, then A1 query-only
unset BENCH001_EQ_WORKSPACE_ID
export BENCH001_CHUNK_STRATEGY=markdown
export BENCH001_ENABLE_GLEANING=1
export BENCH001_MAX_GLEANING=1
export BENCH001_PUBLICATION=1
export BENCH001_INGEST_MAX_CHARS=0
make bench001-b2-reingest
# If A1 hits pool timeouts under concurrency=8:
export BENCH001_ACC_QUERY_CONCURRENCY=4
./tools/bench001/scripts/run_p_ladder_acc.sh a1
```

**Pins (labeled):** chunk 1200/100 · adaptive off · markdown strategy · gleaning=1 · query pack = A1 `rr_cer`.

### Results

| Artifact | Role |
|----------|------|
| WS `e0270f5f-0b6c-4e90-882f-5f9b0eac8cff` | B2 graph (warm peer `8b359190-…` preserved) |
| Audit [`20260720T070838Z`](../e2e/artifacts/ingest-audit/20260720T070838Z/) | EQ 392 nodes · soft-overlap **0.640** · zero-chunk **11.2%** |
| Force-ingest [`T070837Z`](../e2e/artifacts/history/smoke-20260720T070837Z/) | Flat query pack Acc 0.721 (not A1) |
| A1 invalid [`T071121Z`](../e2e/artifacts/history/smoke-20260720T071121Z/) | Pool timeout empties — discard |
| **A1 clean [`T071732Z`](../e2e/artifacts/history/smoke-20260720T071732Z/)** | Acc **0.785** (Δ+0.006) · Complex Δ **−0.006** · ctx **0.494** · recall **0.928** |

**Success (promote B2 workspace):**

| Criterion | Result |
|-----------|--------|
| soft-overlap ≥ 0.75 **or** ents ≥ 0.5× LR | **FAIL** |
| Zero-chunk ≤ 5% | **FAIL** (11.2%) |
| A1 ctx ≥ 0.50 | **FAIL** (0.494) |
| A1 recall ≥ LR−0.03 | **FAIL** (0.928 vs 0.932) |
| A1 Acc ≥ A4−0.02 | **PASS** (0.785 ≥ 0.747) |

**Call:** Markdown + gleaning **raises Acc/Complex** vs pre-B2 A1/A4 but does **not** close entity density or L2 gates. Proceed to **B3** ([031](./031-structure-aware-chunking.md)). Keep `e0270f5f-…` as B2 candidate for A1 packing; do not claim Beat/Parity.

---

## 4. Non-goals

- Soft Mix Acc fishing  
- Silent Acc ingest pin changes on the existing warm peer  
- Claiming Beat from ingest-only without Acc CI gates  
