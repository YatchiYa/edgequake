# SPEC-073 — Recommendations (locked)

No silent flips. No floor raises from this assessment alone.  
Scale order: [`005-industry-scale-playbook.md`](005-industry-scale-playbook.md) (July 2026 industry) ∩ EdgeQuake measured floors.

## Do

1. **Keep** workspace → document → chunk **relational ownership** (`workspaces` / `documents` / `chunks` / PDF / lineage).
2. **Keep** denormalized `workspace_id`, `tenant_id`, `document_id` on every vector upsert (columns + metadata). Wave-2 **requires** columns-only filter shapes for partial-HNSW implication.
3. **Keep** the split serving path: KV for chunk text SSOT, `eq_*_vectors` for ANN — until a measured bake-off proves otherwise.
4. **Follow the industry scale order** without skipping to external ANN:
   - Schema denorm → HNSW → **halfvec** → **partial HNSW** (Wave-2) → residency/warmup → **DiskANN** only when RAM/concurrent demands it
5. **Keep** the evidence-led product ladder:
   - Default supported: Wave-2 shared+partial @**100k**
   - Opt-in: dedicated DiskANN @**150k** with `query_search_list_size≥400` (SPEC-072)
6. **Prioritize reliability hardening** over new ANN floors (checklist below).
7. **Warm** hot-workspace partial HNSW (`make wave2-greenfield-env` / `scripts/wave2_warmup.sh` / admin warmup) so `/ready` catalog presence matches real traffic.
8. **Measure filtered recall@k** (with `workspace_id`) whenever changing `ef_search`, iterative_scan, or DiskANN list size — unfiltered demos hide the filter trap.

## Do not

1. **Do not** silently merge KV + vectors into one `document_chunks` table in product boot or migrations (this pack).
2. **Do not** treat `public.documents` / `public.chunks` as the RAG ANN corpus.
3. **Do not** raise `highest_green_N` from this study alone.
4. **Do not** silent-flip `halfvec`, partial HNSW, vectorscale, or DiskANN on existing DBs.
5. **Do not** claim dedicated HNSW unlocks mid-scale concurrent (SPEC-069 disproved).
6. **Do not** run DiskANN @150k at default `query_search_list_size=100` (recall fails — SPEC-072).
7. **Do not** equate document counts with vector counts in FAQ/limits copy.

## Retract completeness checklist (reliability)

When a document is deleted, cancelled, orphaned, or failed, verify **all** surfaces clear for that `document_id` / workspace:

| Surface | Check |
|---------|-------|
| Relational | `documents` / `chunks` / `pdf_documents` / lineage links gone or cascade |
| KV | Chunk keys for document removed |
| Vectors | `delete_by_document` (column + metadata keys) removes chunk/entity/rel rows as policy requires |
| AGE | Nodes/edges retract or `source_ids` / `source_chunk_ids` updated (SPEC-058/059 merge rules) |
| Indexes | No orphan ANN rows left for deleted docs; hot partial HNSW still consistent |

Compensate must only delete **created** entity/rel vectors (atomic `upsert_report_created`) — never shared updates.

## Wave-2 denorm guard (reliability)

On every vector upsert path:

- [ ] `metadata` contains `workspace_id` / `document_id` (or `source_document_id`)
- [ ] Materialized columns populated from metadata (`storage_impl` INSERT … COALESCE)
- [ ] Filtered search uses columns-only when Wave-2 on (no JSONB `OR` that breaks implication)
- [ ] EXPLAIN shows Index Scan on partial/HNSW (or DiskANN), not Seq+Sort on the workspace slice

## Future option (not this pack) — SPEC-074 candidate

**Unified serving schema bake-off** (`document_chunks` with text + embedding co-located) only if:

1. Full-gate recall/latency/concurrent ≥ Wave-2 baseline @100k
2. Retract story is **simpler** (fewer saga surfaces) without regressions
3. FTS + ANN co-location does not blow TOAST / residency
4. Migration is opt-in greenfield — never silent rewrite of existing corpora

**Research-backed next work (see [`006`](006-research-evidence-improvements.md)):**

1. **P0:** Document + set DiskANN `query_rescore` with `query_search_list_size≥400` (pgvectorscale official tip).
2. **P0:** Automate retract e2e (delete document → zero vectors/KV/AGE orphans).
3. **P1:** Exact reorder / hybrid RRF bake-off; iterative_scan-only vs partial HNSW quantification.
4. **P1/P2:** Binary quantize+rerank; Filtered-DiskANN workspace labels — full gate only.

EdgeQuake already enables `hnsw.iterative_scan=relaxed_order` on filtered queries ([`search_tuning.rs`](../../edgequake/crates/edgequake-storage/src/adapters/postgres/vector/search_tuning.rs)); partial HNSW remains the Wave-2 shape win.

## Operator one-liner

> Document linked to workspace is the **control plane**; denorm `workspace_id` on vectors is the **index plane**; KV/AGE are **content/graph planes**. Reliability is retract across all planes. Scale = cut bytes → shape ANN to workspace → keep it resident → DiskANN when RAM cliffs — then measure.
