# 029 — Ingest parity audit (Horizon B1)

**Status:** Tooling shipped · run on warm Acc workspace before B2  
**Cross-ref:** [028 First-principles beat roadmap](./028-first-principles-beat-roadmap.md) · [030 B2 gleaning](./030-ingest-gleaning-parity.md) · [017 Beat LightRAG](./017-beat-lightrag.md)

---

## 1. Why

Acc fair pins freeze ingest (chunk 1200/100, adaptive off) so query ablations stay one-confound. That is correct for science and **incomplete for product ceiling**: if EQ extraction misses entity↔chunk `source_id` links that LightRAG has, **no Mix / CE / packing knob recovers that gold**.

Horizon B starts with a **paired audit**, not a silent re-ingest.

---

## 2. What we measure

| Signal | Source |
|--------|--------|
| Entity counts | EQ AGE `Node` vs LR `kv_store_entity_chunks` |
| Relation linkage | EQ `EDGE` count vs LR `kv_store_relation_chunks` |
| Name overlap | Normalized UPPERCASE entity Jaccard |
| Linkage density | Mean chunks / entity; zero-chunk entity rate |
| Chunk ID equality | **Not** expected — EQ UUID-chunk vs LR `doc-*-chunk-*` namespaces |

Gold evidence membership (Complex misses) is a follow-up once chunk namespaces are aligned via a **forced re-ingest** workspace.

---

## 3. How to run

```bash
# Defaults: warm_workspace.json + ~/.cache/edgequake/bench001/lightrag/smoke
python3 tools/bench001/scripts/audit_eq_lr_ingest.py

# Explicit
BENCH001_EQ_WORKSPACE_ID=8b359190-0733-4949-994c-f39eca074d79 \
BENCH001_LR_STAGE=smoke \
DATABASE_URL='postgresql://edgequake:edgequake_secret@localhost:5432/edgequake' \
python3 tools/bench001/scripts/audit_eq_lr_ingest.py
```

Artifacts: `specs/001-benchmark/e2e/artifacts/ingest-audit/<utc>/SUMMARY.md` + `audit_report.json`.

Requires: `psycopg2-binary`, Postgres with AGE default graph, LR Acc cache stage present.

---

## 4. Decision rules → B2 / B3

| Finding | Next |
|---------|------|
| EQ entity count ≪ LR **or** EQ coverage of LR names ≪ 0.5 | **B2** gleaning / merge / extract prompts |
| High EQ zero-chunk rate | Fix `source_ids` attach on extract/merge |
| Name overlap OK but Fact/Summarize ER still ≪ LR−0.03 after A* | **B3** structure-aware chunking (labeled) + forced rebuild |
| Overlap OK and L2 gates pass on A* | Stay on query path; do not re-ingest |

---

## 5. Re-ingest plan (when B2/B3 promote)

1. New workspace id (never overwrite warm Acc silently).  
2. Labeled ingest pins in scorecard (`adaptive`, chunk size, extract model).  
3. Re-run Acc **query-only** A0/A1 on new workspace.  
4. Keep Acc publication ingest freeze until promote criteria met.

---

## 6. Non-goals

- Do not change Acc ingest pins during Horizon A query ablations.  
- Do not claim Beat from ingest-only without Acc CI gates.  
- Do not treat chunk-id set equality as a pass/fail (namespaces differ today).
