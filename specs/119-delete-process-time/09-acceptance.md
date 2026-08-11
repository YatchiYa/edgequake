# 09 — Acceptance

## Checklist

| ID | Criterion | Pass? |
|----|-----------|-------|
| A1 | SPEC-119 doc pack complete (00–10 + lenses) | Yes |
| A2 | GitHub #375 investigation + fix + ops comments | Yes |
| A3 | Singular SQL uses btree-matching `->>'…'` (no `::jsonb` on singular path) | Yes |
| A4 | `idx_edge_source_chunk_id` + `idx_edge_source_document_id` in `ensure_indexes` | Yes |
| A5 | Migration `145_spec119_edge_singular_citation_indexes.sql` present | Yes |
| A6 | Contract: indexes exist after ensure_indexes | Yes |
| A7 | EXPLAIN Index Cond for chunk_id, document_id, and OR (BitmapOr) | Yes |
| A8 | Wall-budget e2e (200 singular edges + EXPLAIN ANALYZE OR) under 2s | Yes (~11ms discovery; ~0.4ms EXPLAIN) |
| A9 | Modern GIN path / SPEC-071 still green | Yes |
| A10 | Symptom F singular match (memory) + retract_checked clears singular edges | Yes |
| A11 | DRY timeout SSOT (`graph_cleanup_timeout`) — no raw Detail in user copy | Yes |
| A12 | Reprocess uses `retract_document_indexes_checked` (fail-closed on discovery timeout) | Yes |

## Honest limits (still true)

| Item | Status |
|------|--------|
| Automated e2e does not synthesize 220k edges | Scale proof = live EXPLAIN on large `"EDGE"` when graph ≥10k + manual 69k Index Scan |
| Marker migration does not CREATE indexes | Requires `ensure_indexes` (restart / graph init / upsert) |
| Frontend i18n keys | Not added; API/worker strings are product-facing |
| Uncommitted until push | Spec links on GitHub resolve only after merge to the linked branch |

## Live verification (2026-08-11)

| Metric | Before | After |
|--------|--------|-------|
| Plan (chunk_id) | Seq Scan ~288ms @ 69k | Index Scan ~0.3ms |
| Plan (OR) | Seq Scan | BitmapOr of both singular btrees |
| Expression | `::jsonb->>'…'` | `->>'…'` matches index |

## Definition of done

A1–A12 checked; ops comment on #375 explains fleet apply path; remaining limits listed above (not hidden).
