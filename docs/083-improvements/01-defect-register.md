# SPEC-083 — Defect Register (code-audited)

> Canonical IDs from the v0.20.2 French register, re-verified at HEAD 2026-07-23.
> Per-defect studies: [`defects/`](defects/). Principles: [`00-first-principles.md`](00-first-principles.md).

## Summary

| Metric | Count |
|--------|------:|
| Studies written | 90 |
| CONFIRMED | 0 |
| PARTIAL | 0 |
| FIXED | 89 |
| RETRACTED | 1 |

## Full table

| ID | Title | Prio | Audit | Cluster | Sprint | Study |
|----|-------|------|-------|---------|--------|-------|
| P0 | SPEC-062 eq_* denorm DDL blocked on large AGE graphs | P0 | FIXED | 00-schema-readiness | 0 | [P0.md](defects/P0.md) |
| X-03 | Four eq_* consumers without gate or fallback | P0 | FIXED | 00-schema-readiness | 0 | [X-03.md](defects/X-03.md) |
| C-14 | Entity normalization: article/possessive/case bugs | P1 | FIXED | 03-graph-identity | 2 | [C-14.md](defects/C-14.md) |
| C-17 | Gleaning calls complete() without CompletionOptions | P1 | FIXED | 04-pipeline-reliability | 3 | [C-17.md](defects/C-17.md) |
| C-18 | CHUNK_MAX_RETRIES=0 means zero attempts | P1 | FIXED | 04-pipeline-reliability | 2 | [C-18.md](defects/C-18.md) |
| D-50 | .env.example hardcodes VISION_PROVIDER=openai | P1 | FIXED | 06-ops-ci-sdk | 1 | [D-50.md](defects/D-50.md) |
| S-01 | WebSocket has no tenant isolation | P1 | FIXED | 01-tenant-isolation | 1 | [S-01.md](defects/S-01.md) |
| S-02 | track_id ownership never verified on WS/PDF progress/cancel | P1 | FIXED | 01-tenant-isolation | 1 | [S-02.md](defects/S-02.md) |
| S-03 | RLS inert: transaction-local GUC without BEGIN | P1 | FIXED | 01-tenant-isolation | 1 | [S-03.md](defects/S-03.md) |
| S-04 | RLS fail-open on NULL tenant | P1 | FIXED | 01-tenant-isolation | 1 | [S-04.md](defects/S-04.md) |
| S-05 | Incoherent RLS GUC namespaces | P1 | FIXED | 01-tenant-isolation | 1 | [S-05.md](defects/S-05.md) |
| S-06 | document_originals has no RLS | P1 | FIXED | 01-tenant-isolation | 1 | [S-06.md](defects/S-06.md) |
| S-07 | No access-token revocation; iss/aud unchecked | P1 | FIXED | 02-auth-transport | 1 | [S-07.md](defects/S-07.md) |
| S-08 | Role::parse fail-open to User | P1 | FIXED | 02-auth-transport | 1 | [S-08.md](defects/S-08.md) |
| S-09 | Default JWT_SECRET does not block startup | P1 | FIXED | 02-auth-transport | 1 | [S-09.md](defects/S-09.md) |
| S-10 | CORS Any/Any/Any by default; WS Origin fail-open | P1 | FIXED | 02-auth-transport | 1 | [S-10.md](defects/S-10.md) |
| S-11 | Rate limit keyed on raw x-tenant-id header | P1 | FIXED | 02-auth-transport | 1 | [S-11.md](defects/S-11.md) |
| S-12 | Filename unsanitized; MIME from extension only | P1 | FIXED | 02-auth-transport | 1 | [S-12.md](defects/S-12.md) |
| S-13 | eval() on benchmark dataset content | P1 | FIXED | 02-auth-transport | 1 | [S-13.md](defects/S-13.md) |
| X-06 | LLM layer: no jitter, single WaitAndRetry, no circuit breaker | P1 | FIXED | 04-pipeline-reliability | 3 | [X-06.md](defects/X-06.md) |
| X-07 | LLM retry reimplemented via substring matching | P1 | FIXED | 04-pipeline-reliability | 3 | [X-07.md](defects/X-07.md) |
| X-10 | No client L2 normalization of embeddings | P1 | FIXED | 04-pipeline-reliability | 3 | [X-10.md](defects/X-10.md) |
| X-16 | empty_on_missing_json silent empty extraction | P1 | FIXED | 04-pipeline-reliability | 1 | [X-16.md](defects/X-16.md) |
| X-28 | Checkpoint content_hash is 64-bit over first 64KiB | P1 | FIXED | 04-pipeline-reliability | 2 | [X-28.md](defects/X-28.md) |
| X-29 | No task state machine guards transitions | P1 | FIXED | 04-pipeline-reliability | 2 | [X-29.md](defects/X-29.md) |
| X-30 | Circuit breaker / failure class via string matching | P1 | FIXED | 04-pipeline-reliability | 3 | [X-30.md](defects/X-30.md) |
| X-35 | Accuracy degrades with corpus size (0.458@40) | P1 | FIXED | 07-accuracy-explain | 5 | [X-35.md](defects/X-35.md) |
| X-37 | Three multi-tenant isolation models | P1 | FIXED | 01-tenant-isolation | 1 | [X-37.md](defects/X-37.md) |
| C-15 | Pdf/Markdown chunker offsets not rebased | P2 | FIXED | 04-pipeline-reliability | 2 | [C-15.md](defects/C-15.md) |
| C-16 | Atomic blocks ignore size guard | P2 | FIXED | 04-pipeline-reliability | 2 | [C-16.md](defects/C-16.md) |
| C-20 | Contract test native_upsert vacuous / contradictory | P2 | FIXED | 00-schema-readiness | 0 | [C-20.md](defects/C-20.md) |
| C-21 | batch_fetch_chunk_contents is N+1 | P2 | FIXED | 04-pipeline-reliability | 3 | [C-21.md](defects/C-21.md) |
| C-22 | KV upsert non-transactional across batches | P2 | FIXED | 04-pipeline-reliability | 2 | [C-22.md](defects/C-22.md) |
| C-23 | Document dedup broken: indexed vs completed | P2 | FIXED | 04-pipeline-reliability | 2 | [C-23.md](defects/C-23.md) |
| C-24 | matches_track_id ignores Deletion* events | P2 | FIXED | 01-tenant-isolation | 1 | [C-24.md](defects/C-24.md) |
| C-27 | Tenant cache last_accessed never refreshed (FIFO≠LRU) | P2 | FIXED | 06-ops-ci-sdk | 4 | [C-27.md](defects/C-27.md) |
| C-28 | cosine_similarity panics on dimension mismatch | P2 | FIXED | 05-query-fusion | 2 | [C-28.md](defects/C-28.md) |
| D-30 | Graph is not a multigraph: edge key omits type | P2 | FIXED | 03-graph-identity | 2 | [D-30.md](defects/D-30.md) |
| D-31 | Relation weight (a+b)/2 order-dependent non-associative | P2 | FIXED | 03-graph-identity | 2 | [D-31.md](defects/D-31.md) |
| D-32 | Entity type first-wins forever, no conflict log | P2 | FIXED | 03-graph-identity | 2 | [D-32.md](defects/D-32.md) |
| D-33 | source_ids cap before lineage computation | P2 | FIXED | 03-graph-identity | 2 | [D-33.md](defects/D-33.md) |
| D-34 | Double gate merger(1200) vs summarizer(4000) | P2 | FIXED | 03-graph-identity | 3 | [D-34.md](defects/D-34.md) |
| D-37 | chunk_score carries three incompatible scales | P2 | FIXED | 05-query-fusion | 4 | [D-37.md](defects/D-37.md) |
| D-38 | query_vec embeds history+question | P2 | FIXED | 05-query-fusion | 3 | [D-38.md](defects/D-38.md) |
| D-39 | min_score skipped on fused / preserve_order paths | P2 | FIXED | 05-query-fusion | 3 | [D-39.md](defects/D-39.md) |
| D-40 | QueryStats vs QueryStreamStats diverged | P2 | FIXED | 05-query-fusion | 4 | [D-40.md](defects/D-40.md) |
| D-41 | Progress percent unweighted average of phases | P2 | FIXED | 06-ops-ci-sdk | 4 | [D-41.md](defects/D-41.md) |
| D-42 | ETA resets after serialization; progress process-local | P2 | FIXED | 06-ops-ci-sdk | 4 | [D-42.md](defects/D-42.md) |
| D-44 | PDF 100 MiB contract unreachable; error text wrong | P2 | FIXED | 06-ops-ci-sdk | 4 | [D-44.md](defects/D-44.md) |
| D-45 | audit_logs defined 4×; partitions never scheduled | P2 | FIXED | 06-ops-ci-sdk | 2 | [D-45.md](defects/D-45.md) |
| D-48 | SDK workflows nested under sdks/*/.github never run | P2 | FIXED | 06-ops-ci-sdk | 4 | [D-48.md](defects/D-48.md) |
| D-49 | sed -i '' BSD-only in publish targets | P2 | FIXED | 06-ops-ci-sdk | 4 | [D-49.md](defects/D-49.md) |
| D-51 | Multipart 100% in RAM; batch uncapped file count | P2 | FIXED | 06-ops-ci-sdk | 4 | [D-51.md](defects/D-51.md) |
| D-52 | Extraction cache never sets (100% miss) | P2 | FIXED | 04-pipeline-reliability | 4 | [D-52.md](defects/D-52.md) |
| D-53 | Three divergent token estimators; no real tokenizer | P2 | FIXED | 04-pipeline-reliability | 3 | [D-53.md](defects/D-53.md) |
| X-01 | Migration 002 entirely dead | P2 | FIXED | 06-ops-ci-sdk | 4 | [X-01.md](defects/X-01.md) |
| X-02 | Boot repairs migration checksums (fragile) | P2 | FIXED | 06-ops-ci-sdk | 4 | [X-02.md](defects/X-02.md) |
| X-05 | BM25 label is ts_rank_cd; english config hard-coded | P2 | FIXED | 05-query-fusion | 4 | [X-05.md](defects/X-05.md) |
| X-08 | Three contradictory embedding batch clamps | P2 | FIXED | 04-pipeline-reliability | 3 | [X-08.md](defects/X-08.md) |
| X-09 | Diamond dependency: two edgequake-llm versions in lock | P2 | FIXED | 06-ops-ci-sdk | 4 | [X-09.md](defects/X-09.md) |
| X-11 | Scan and Reindex task types unimplemented | P2 | FIXED | 06-ops-ci-sdk | 5 | [X-11.md](defects/X-11.md) |
| X-13 | Duplicate page markers without SSOT | P2 | FIXED | 04-pipeline-reliability | 3 | [X-13.md](defects/X-13.md) |
| X-14 | LightRAG separator cascade never active in prod | P2 | FIXED | 04-pipeline-reliability | 3 | [X-14.md](defects/X-14.md) |
| X-17 | Entity resolution exact-match only | P2 | FIXED | 03-graph-identity | 4 | [X-17.md](defects/X-17.md) |
| X-18 | No partial tolerance on embedding batches | P2 | FIXED | 04-pipeline-reliability | 3 | [X-18.md](defects/X-18.md) |
| X-19 | No pipeline backpressure / token-bucket rate limit | P2 | FIXED | 04-pipeline-reliability | 3 | [X-19.md](defects/X-19.md) |
| X-20 | Citations coupled to context by position index | P2 | FIXED | 05-query-fusion | 3 | [X-20.md](defects/X-20.md) |
| X-23 | WebSocket Lagged swallowed | P2 | FIXED | 01-tenant-isolation | 1 | [X-23.md](defects/X-23.md) |
| X-24 | main.rs AUTO_RESUME comment stale (default ON) | P2 | FIXED | 06-ops-ci-sdk | 4 | [X-24.md](defects/X-24.md) |
| X-25 | OpenAPI build gate blind to routes.rs mounts | P2 | FIXED | 06-ops-ci-sdk | 4 | [X-25.md](defects/X-25.md) |
| X-27 | Frontend has no middleware.ts server guard | P2 | FIXED | 06-ops-ci-sdk | 4 | [X-27.md](defects/X-27.md) |
| X-31 | Shutdown has no drain timeout | P2 | FIXED | 06-ops-ci-sdk | 4 | [X-31.md](defects/X-31.md) |
| X-32 | Decorative CI gates | P2 | FIXED | 06-ops-ci-sdk | 4 | [X-32.md](defects/X-32.md) |
| X-33 | SDKs locked at 0.4.0 while server 0.20.2 | P2 | FIXED | 06-ops-ci-sdk | 4 | [X-33.md](defects/X-33.md) |
| X-34 | Golden set loaded/counted never evaluated | P2 | FIXED | 07-accuracy-explain | 5 | [X-34.md](defects/X-34.md) |
| X-36 | Three divergent configuration systems | P2 | FIXED | 06-ops-ci-sdk | 4 | [X-36.md](defects/X-36.md) |
| C-19 | drop_workspace_table missing prefix (RETRACTED) | P3 | RETRACTED | 08-dead-code | 4 | [C-19.md](defects/C-19.md) |
| C-25 | Anthropic provider ignores ImageData::from_url | P3 | FIXED | 04-pipeline-reliability | 4 | [C-25.md](defects/C-25.md) |
| C-26 | MAX_SOURCE_IDS=300 declared never applied | P3 | FIXED | 03-graph-identity | 2 | [C-26.md](defects/C-26.md) |
| D-35 | Docs say weighted sum; Mix fusion uses max | P3 | FIXED | 05-query-fusion | 4 | [D-35.md](defects/D-35.md) |
| D-36 | EDGEQUAKE_SPARSE_FUSION=weighted is sparse-first | P3 | FIXED | 05-query-fusion | 4 | [D-36.md](defects/D-36.md) |
| D-46 | OTEL layer mounted before env_filter | P3 | FIXED | 06-ops-ci-sdk | 4 | [D-46.md](defects/D-46.md) |
| D-47 | make postgres-start does not exist | P3 | FIXED | 06-ops-ci-sdk | 4 | [D-47.md](defects/D-47.md) |
| D-54 | Louvain phase-1 only; extractive community reports | P3 | FIXED | 07-accuracy-explain | 5 | [D-54.md](defects/D-54.md) |
| X-04 | Vector module docs claim L2/IP; code cosine-only | P3 | FIXED | 05-query-fusion | 4 | [X-04.md](defects/X-04.md) |
| X-12 | PDF concurrency match decorative (all arms return 2) | P3 | FIXED | 06-ops-ci-sdk | 4 | [X-12.md](defects/X-12.md) |
| X-15 | OTHER missing from default entity types | P3 | FIXED | 03-graph-identity | 4 | [X-15.md](defects/X-15.md) |
| X-21 | ExplainTrace nonexistent | P3 | FIXED | 07-accuracy-explain | 5 | [X-21.md](defects/X-21.md) |
| X-22 | SSE Thinking event never emitted | P3 | FIXED | 05-query-fusion | 3 | [X-22.md](defects/X-22.md) |
| X-26 | schema.d.ts unused by webui/SDKs | P3 | FIXED | 06-ops-ci-sdk | 4 | [X-26.md](defects/X-26.md) |

## Notes

- Register skips **D-43** (D-42 → D-44).
- **X-01…X-12** = official complementary IDs from the register body.
- **X-13…X-27** = page-6 unnumbered promotions (do not collide with X-01…X-12).
- **C-19** RETRACTED; **X-15** FIXED (OTHER in defaults); **D-50** FIXED (`.env.example` vision empty).
- Verify/E2E closure (2026-07-23): S-03 call-sites + D-33 lineage + X-30 typed classifiers + D-49 SED_INPLACE + Cluster 00–05 matrix tests green.
- Double-check closure (2026-07-23): API session/identity/pdf_lineage migrated off autocommit acquire → `with_optional_pg_rls`; defect Audit headers + docs pack synced to this register SSOT.
- Explicit **CONFIRMED** backlog (out of scope this pass): D-45, D-54, X-17 ingest fuzzy, X-34/X-35 Acc nightly, X-13 full SSOT, D-48 SDK nested workflows, X-36 config `resolve()`.
- Missing companion from source report recreated: [INCIDENT-PROD-DIAGNOSIS.md](INCIDENT-PROD-DIAGNOSIS.md).
- Full-pack audit (2026-07-23): demoted **X-30** (ingest still string taxonomy) and **X-32** (CI still has continue-on-error on postgres/e2e gates) to PARTIAL; X-06 kept FIXED (jittered embed retry + CircuitBreakerOpen).
- Full closure Wave 0 (2026-07-23): PARTIAL → FIXED: P0,S-10,X-30,D-44,X-18,X-19,X-20,X-25,X-32.
- Full closure Wave A (2026-07-23): C-16,X-08,X-13,X-14,D-52,X-31,C-25,C-26,D-51 → FIXED.
- Full closure Wave B (2026-07-23): D-35,D-37,D-40,X-04,X-05,X-22 → FIXED.
- Full closure Wave C (2026-07-23): X-01/02/09/11/12/23/26/27/33/36,D-42/45/46/48 → FIXED.
- Full closure Wave D (2026-07-23): D-32,X-17,D-54,X-34,X-35 → FIXED. Register now 89 FIXED / 0 PARTIAL / 0 CONFIRMED / 1 RETRACTED.
