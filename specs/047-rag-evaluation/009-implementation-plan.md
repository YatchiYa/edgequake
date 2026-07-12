# 009 — Implementation Plan (Tickets)

**Cross-ref:** all lenses · [010](./010-smoke-then-full-runbook.md) · [012](./012-acceptance-criteria-and-scorecard.md)

---

## Delivery phases

```text
Phase 0  Foundations (vision flag, doctor, download, scorer)
Phase 1  Smoke harness (10 docs) ← first user-visible win
Phase 2  Core + ablations
Phase 3  Full + nightly + version compare
Phase 4  Complementary benches (011) — separate specs if large
```

---

## Ticket board

### Phase 0 — Foundations

| ID | Title | Done when |
|----|-------|-----------|
| EQ-047-01 | Add `tools/bench047` package skeleton + CLI | `bench047 --help` works |
| EQ-047-02 | Vendor `eval_score.py` + unit parity tests | Int/Float/ANLS/List cases pass |
| EQ-047-03 | Fix Mistral Small vision capability / fail-closed | `doctor` detects vision OK; no silent drop |
| EQ-047-04 | Dataset download + checksum manifest | cache populated; NC notice printed |
| EQ-047-05 | Freeze `fixtures/smoke_doc_ids_v1.txt` (10) | stratified list committed + rationale doc |
| EQ-047-06 | Scorecard JSON Schema + SUMMARY renderer | schema validates empty+sample |

### Phase 1 — Smoke (priority)

| ID | Title | Done when |
|----|-------|-----------|
| EQ-047-07 | Workspace + ingest client (vision on) | 10 PDFs → Completed/Failed logged |
| EQ-047-08 | Hybrid query loop + predictions.jsonl | all Qs for smoke docs answered |
| EQ-047-09 | Extractor (GPT-4o official + Mistral alt) | short answers produced |
| EQ-047-10 | Score + scorecard + SUMMARY | `make bench047-smoke` green path |
| EQ-047-11 | Resume + cost gate flags | kill/resume proven |
| EQ-047-12 | Runbook dry-run on real machine | [010](./010-smoke-then-full-runbook.md) executed once; artifacts saved locally |

### Phase 2 — Core + science

| ID | Title | Done when |
|----|-------|-----------|
| EQ-047-13 | `fixtures/core_doc_ids_v1.txt` (~40) | committed |
| EQ-047-14 | Ablation profiles P1–P6 | runnable via `--profile` |
| EQ-047-15 | Retrieval page_hit diagnostics | best-effort field in JSONL |
| EQ-047-16 | Experiments E1–E4 reports | markdown tables in artifacts |

### Phase 3 — Full + ops

| ID | Title | Done when |
|----|-------|-----------|
| EQ-047-17 | `bench047 full` + heartbeat | completes or cleanly checkpoints |
| EQ-047-18 | Nightly smoke GHA workflow | manual+nightly documented |
| EQ-047-19 | `report --compare` | delta table |
| EQ-047-20 | Pin dataset revision in CI | reproducible |

### Phase 4 — Expand (after MMLongBench smoke green)

| ID | Title | Done when |
|----|-------|-----------|
| EQ-047-21 | MultiHop-RAG adapter sketch | separate fixture + metrics |
| EQ-047-22 | UniDoc-Bench / LongDocURL evaluation note | go/no-go in 011 |
| EQ-047-23 | GraphRAG-Bench link to SPEC-046 ACC | single dashboard pointer |

---

## Suggested implementation order (do not reorder casually)

```text
01 → 02 → 03 → 04 → 05 → 06 → 07 → 08 → 09 → 10 → 12
                                              ↘ 11
Then 13–16, then 17–20, then 21–23.
```

---

## Definition of “SPEC-047 implemented”

1. `make bench047-smoke` produces valid scorecard on a clean machine with keys.  
2. Vision fail-closed proven.  
3. Official scoring semantics preserved (unit tests).  
4. Progression path to core/full documented and runnable.  
5. Complementary methodology published in [011](./011-complementary-benchmarks-methodology.md) (this pack — done at spec time).

---

## Out of scope for first merge

- WebUI benchmark dashboard  
- Automatic publishing to HF leaderboard  
- Non-Mistral multi-provider matrix in one run  

Next: [010 Runbook](./010-smoke-then-full-runbook.md).
