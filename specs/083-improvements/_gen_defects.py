#!/usr/bin/env python3
"""Generate per-defect study markdown for SPEC-083. Run from repo root or this dir."""
from __future__ import annotations

from pathlib import Path

ROOT = Path(__file__).resolve().parent
DEFECTS = ROOT / "defects"

# id, title, prio, status, cluster, sprint, laws, locus, why, root, solution, edges, tests, related
# status: CONFIRMED | PARTIAL | FIXED | RETRACTED
CATALOG: list[dict] = [
    # --- P0 ---
    dict(
        id="P0",
        title="SPEC-062 eq_* denorm DDL blocked on large AGE graphs",
        prio="P0",
        status="PARTIAL",
        cluster="00-schema-readiness",
        sprint=0,
        laws="LAW-2",
        locus="migrations/support/092/apply.sql; graph_lifecycle.rs ensure_indexes; nodes_ops/read.rs",
        why="On large production graphs, ACCESS EXCLUSIVE for ADD COLUMN loses to long agtype queries. Columns never appear → chat SQL errors and ingest ON CONFLICT loops. Small PPD graphs hide the bug.",
        root="Hot-path readers and writers assume denormalized columns exist. DDL was coupled to boot/ensure_indexes under query load without a readiness gate or property fallback. lock_timeout=5s fails fast but leaves schema incomplete.",
        solution="(1) Offline/low-load DDL with lock_timeout + retry + NOTICE. (2) Boot gate: refuse query/ingest if eq_* missing. (3) Temporary COALESCE(eq_*, props->>) fallback until gate green. (4) Keep SPEC-069 single-flight. Shared primitive: SchemaReadiness.",
        edges="178k+ nodes; concurrent BFS; partial backfill NULL eq_*; multi-graph workspace",
        tests="e2e_schema_ready_refuses_traffic; e2e_degrees_match_property_fallback; contract_eq_columns_present_after_reconcile",
        related="X-03, C-20, D-30",
        diagram="""
  long_agtype_query --holds--> AccessShare
       |
       v
  ALTER ADD COLUMN --needs--> AccessExclusive --TIMEOUT--> columns missing
       |
       v
  pg_node_degrees_batch(eq_*) --> ERROR / empty --> chat broken
""",
    ),
    dict(
        id="X-03",
        title="Four eq_* consumers without gate or fallback",
        prio="P0",
        status="CONFIRMED",
        cluster="00-schema-readiness",
        sprint=0,
        laws="LAW-2",
        locus="nodes_ops/read.rs:148-171; edges_ops.rs:360-365; scan_ops.rs:321-448",
        why="Absence of eq_* columns or NULL backfill silently breaks degree/incident-edge queries or excludes rows — production chat failure mode.",
        root="Readers bound exclusively to denormalized columns without catalog probe or property-path fallback. Prefix scans with IS NOT NULL drop non-backfilled edges.",
        solution="SchemaReadiness probe; dual SQL path (eq_* vs props); never filter out NULL eq_* without property OR; metric when fallback used.",
        edges="column missing vs NULL; mixed backfill; empty node_ids",
        tests="contract_incident_edges_fallback; e2e_chat_local_mode_without_eq_columns_degraded",
        related="P0",
        diagram="""
  read_path --> eq_source_id IN (...)
       |              |
       |         column missing --> SQL ERROR
       |         NULL eq_* --> silent miss
       v
  no COALESCE(props->>'source_id')
""",
    ),
    # --- Security S-01..S-13 ---
    dict(
        id="S-01",
        title="WebSocket has no tenant isolation",
        prio="P1",
        status="CONFIRMED",
        cluster="01-tenant-isolation",
        sprint=1,
        laws="LAW-1,LAW-4",
        locus="edgequake-api/src/handlers/websocket.rs:53-66,166-256",
        why="Any authenticated client on /ws/pipeline/progress receives all tenants' progress events — cross-tenant data leak.",
        root="authorize_ws_upgrade validates token then discards Claims. Broadcast fan-out is global; filter is track_id-only on the other endpoint.",
        solution="WsSession { tenant_id, workspace_id, user_id } from JWT; ProgressEvent carries scope; filter before send. DRY with TenantContext.",
        edges="missing claims; API-key auth; Lagged under filter",
        tests="e2e_ws_tenant_a_never_sees_tenant_b; contract_progress_event_has_workspace",
        related="S-02,C-24,X-23",
        diagram="""
  JWT valid --> identity dropped --> subscribe(global)
       |
       v
  TenantA event --> TenantB socket  (LEAK)
""",
    ),
    dict(
        id="S-02",
        title="track_id ownership never verified on WS/PDF progress/cancel",
        prio="P1",
        status="PARTIAL",
        cluster="01-tenant-isolation",
        sprint=1,
        laws="LAW-1,LAW-4",
        locus="pdf_upload/status.rs:262-278; websocket.rs cancel; task_scope.rs:9-29 (REST OK)",
        why="Tenant can observe or cancel another tenant's upload/task via guessed track_id.",
        root="REST uses get_task_for_context; WS/PDF progress/cancel bypass it.",
        solution="All track_id entrypoints call get_task_for_context (or equivalent). 404 on mismatch.",
        edges="race after task delete; admin override",
        tests="e2e_cancel_foreign_track_id_404; e2e_pdf_progress_foreign_404",
        related="S-01",
        diagram="""
  REST GET /tasks/{id} --> workspace check OK
  WS/PDF /progress/{id} --> no check --> cross-tenant
""",
    ),
    dict(
        id="S-03",
        title="RLS inert: transaction-local GUC without BEGIN",
        prio="P1",
        status="CONFIRMED",
        cluster="01-tenant-isolation",
        sprint=1,
        laws="LAW-1",
        locus="migrations/001_init_database.sql:434-444; rls.rs:220-254; conversation.rs acquire then INSERT",
        why="Policies never see current_tenant_id → isolation relies solely on app WHERE. Forgotten WHERE = leak.",
        root="set_config(..., true) is transaction-local. Autocommit SELECT set_tenant_context() ends the tx → GUCs cleared before next statement. Also ENABLE without FORCE lets table owner bypass.",
        solution="with_rls_transaction: BEGIN → set_tenant_context → work → COMMIT. FORCE ROW LEVEL SECURITY. Align docker/init.sql is_local with migrations.",
        edges="pool checkout reuse; nested tx; SECURITY DEFINER",
        tests="e2e_rls_guc_visible_on_following_insert; e2e_owner_forced_rls",
        related="S-04,S-05,S-06,X-37",
        diagram="""
  set_config(is_local=true) in autocommit
       |
       v  statement ends
  GUC cleared --> INSERT sees NULL --> policy fail-open / no filter
""",
    ),
    dict(
        id="S-04",
        title="RLS fail-open on NULL tenant",
        prio="P1",
        status="CONFIRMED",
        cluster="01-tenant-isolation",
        sprint=1,
        laws="LAW-1,LAW-4",
        locus="001_init_database.sql:507-516; support/081 AGE policies; 085 mm_assets",
        why="Rows with NULL tenant_id (or empty GUC OR clauses) visible to every tenant.",
        root="USING (tenant_id IS NULL OR …) and empty-GUC OR true patterns.",
        solution="Fail-closed policies; NOT NULL tenant_id on new rows; backfill/quarantine NULL rows; remove empty-GUC OR.",
        edges="legacy NULL rows; migration order",
        tests="e2e_null_tenant_row_invisible; contract_policy_has_no_null_or",
        related="S-03,S-06",
        diagram="""
  policy: tenant_id IS NULL OR match
       |
       v
  orphan row --> visible to ALL
""",
    ),
    dict(
        id="S-05",
        title="Incoherent RLS GUC namespaces",
        prio="P1",
        status="CONFIRMED",
        cluster="01-tenant-isolation",
        sprint=1,
        laws="LAW-1,LAW-3",
        locus="001 set_tenant_context app.current_*; 012 app.tenant_id; support/081 edgequake.tenant_id; session.rs",
        why="Setting one namespace leaves others empty → AGE/audit policies inert or wrong.",
        root="Three parallel GUC vocabularies; set_tenant_context only sets app.current_*.",
        solution="SSOT: app.current_tenant_id / app.current_workspace_id / app.current_user_id only. Migrate all policies. One setter.",
        edges="AGE session setup with tenant_id=None",
        tests="contract_single_guc_namespace; e2e_age_rls_sees_app_current",
        related="S-03,X-37",
        diagram="""
  set_tenant_context --> app.current_*
  AGE policy ---------> edgequake.tenant_id  (never set)
  audit policy --------> app.tenant_id       (never set)
""",
    ),
    dict(
        id="S-06",
        title="document_originals has no RLS",
        prio="P1",
        status="CONFIRMED",
        cluster="01-tenant-isolation",
        sprint=1,
        laws="LAW-1",
        locus="migrations/082_add_document_originals.sql",
        why="Source PDF/binary bytes readable across workspaces if any query omits WHERE.",
        root="M082 created table without ENABLE ROW LEVEL SECURITY (mm_assets later claimed to mirror it — irony).",
        solution="ENABLE+FORCE RLS; workspace policy fail-closed; app WHERE retained.",
        edges="FK delete cascade; download handler",
        tests="e2e_document_originals_cross_workspace_denied",
        related="S-03,S-04",
        diagram="""
  pdf_documents RLS ON
  document_originals RLS OFF --> binary leak surface
""",
    ),
    dict(
        id="S-07",
        title="No access-token revocation; iss/aud unchecked",
        prio="P1",
        status="CONFIRMED",
        cluster="02-auth-transport",
        sprint=1,
        laws="LAW-4",
        locus="edgequake-auth/src/jwt.rs:75-91,162-168",
        why="Stolen access token valid ~24h; logout only kills refresh.",
        root="jti minted but never stored; iss/aud None; validation skips them.",
        solution="Require iss/aud; jti denylist (or denylist+TTL store) on logout; short access TTL.",
        edges="clock skew leeway; multi-instance denylist needs Redis/PG",
        tests="e2e_logout_rejects_access_jti; contract_jwt_requires_iss_aud",
        related="S-08,S-09",
        diagram="""
  logout --> revoke refresh only
  access(jti) --> still valid until exp
""",
    ),
    dict(
        id="S-08",
        title="Role::parse fail-open to User",
        prio="P1",
        status="CONFIRMED",
        cluster="02-auth-transport",
        sprint=1,
        laws="LAW-4",
        locus="edgequake-auth/src/types.rs:26-30; jwt.rs role()",
        why="Tampered/unknown role becomes User instead of reject — privilege weirdness and silent auth bugs.",
        root="unwrap_or_default() on parse.",
        solution="Role::try_parse → Result; JWT validation fails closed on unknown role. Update tests that expect fail-open.",
        edges="legacy tokens with bad role",
        tests="contract_unknown_role_rejected",
        related="S-07",
        diagram="""
  role=\"sudo\" --> parse --> User (WRONG)
  role=\"sudo\" --> try_parse --> Err --> 401
""",
    ),
    dict(
        id="S-09",
        title="Default JWT_SECRET does not block startup",
        prio="P1",
        status="CONFIRMED",
        cluster="02-auth-transport",
        sprint=1,
        laws="LAW-4",
        locus="startup_security.rs:39-65; auth config DEFAULT_INSECURE_JWT_SECRET",
        why="Production can boot with known secret; quickstart DEV_MODE compounds open API.",
        root="Warn-only unless EDGEQUAKE_STRICT_STARTUP=1; length rule undocumented in code.",
        solution="Fail startup on default/short secret unless EDGEQUAKE_DEV_MODE with explicit banner. Enforce >=32 bytes.",
        edges="test harness secrets; docker compose",
        tests="contract_startup_rejects_default_secret; e2e_dev_mode_banner",
        related="S-10",
        diagram="""
  JWT_SECRET unset --> default --> warn --> serve (BAD)
  prod path -------> Fatal if default/short
""",
    ),
    dict(
        id="S-10",
        title="CORS Any/Any/Any by default; WS Origin fail-open",
        prio="P1",
        status="PARTIAL",
        cluster="02-auth-transport",
        sprint=1,
        laws="LAW-4",
        locus="server.rs:74-94; middleware.rs:553-557",
        why="Browser cross-origin abuse; WS without Origin accepted.",
        root="Missing EDGEQUAKE_CORS_ORIGINS → AllowOrigin::Any; ws_validate_origin allows missing Origin.",
        solution="Prod fail-closed (require allow-list). Dev may Any with DEV_MODE. Require Origin on WS in prod.",
        edges="native apps no Origin; mobile",
        tests="contract_cors_default_fail_closed_prod; e2e_ws_missing_origin_rejected_prod",
        related="S-09",
        diagram="""
  cors_origins=None --> Any/Any/Any
  WS Origin absent --> allow (fail-open)
""",
    ),
    dict(
        id="S-11",
        title="Rate limit keyed on raw x-tenant-id header",
        prio="P1",
        status="CONFIRMED",
        cluster="02-auth-transport",
        sprint=1,
        laws="LAW-4,LAW-3",
        locus="middleware.rs:630-638; limiter.rs cleanup never called",
        why="Spoof header → fresh bucket; bypass limits. DashMap never cleaned → memory leak. N replicas = N× limit without Redis.",
        root="Auth runs before rate limit but key ignores Claims; cleanup_stale_buckets unused.",
        solution="Key = authenticated tenant_id (or user_id) from Claims; schedule cleanup; document Redis for multi-replica.",
        edges="anonymous routes; API key without tenant",
        tests="e2e_rate_limit_ignores_spoofed_header; contract_cleanup_scheduled",
        related="S-01",
        diagram="""
  auth OK --> rate_limit(x-tenant-id spoof) --> new bucket
  Claims.tenant_id unused
""",
    ),
    dict(
        id="S-12",
        title="Filename unsanitized; MIME from extension only",
        prio="P1",
        status="CONFIRMED",
        cluster="02-auth-transport",
        sprint=1,
        laws="LAW-4",
        locus="file_upload.rs:63-67; file_validation.rs:111-151; pdf magic only",
        why="Path tricks in stored names; content/MIME mismatch; no AV.",
        root="Multipart filename trusted; get_mime_type(extension) only.",
        solution="sanitize_filename (strip path, control chars); magic-byte sniff for allowed types; reject mismatch. AV = ops optional hook.",
        edges="unicode names; double extension",
        tests="contract_filename_strips_path; e2e_exe_as_pdf_rejected",
        related="D-51,D-44",
        diagram="""
  upload \"../../x.pdf\" --> stored as-is
  \"x.exe\" renamed .pdf --> MIME application/pdf (WRONG)
""",
    ),
    dict(
        id="S-13",
        title="eval() on benchmark dataset content",
        prio="P1",
        status="CONFIRMED",
        cluster="02-auth-transport",
        sprint=1,
        laws="LAW-4",
        locus="tools/bench047/bench047/mmlongbench_eval_score.py:137-179",
        why="Malicious/modified dataset executes arbitrary Python.",
        root="eval used to parse list-like strings.",
        solution="ast.literal_eval or json.loads only; ban eval.",
        edges="vendor copy duplicate",
        tests="contract_no_eval_in_bench047; unit_literal_eval_lists",
        related="",
        diagram="""
  dataset string --> eval --> arbitrary code
  dataset string --> literal_eval --> data only
""",
    ),
]

# Functional C
CATALOG += [
    dict(
        id="C-14",
        title="Entity normalization: article/possessive/case bugs",
        prio="P1",
        status="CONFIRMED",
        cluster="03-graph-identity",
        sprint=2,
        laws="LAW-6",
        locus="edgequake-storage/src/entity_id.rs:198-219",
        why="Duplicate graph nodes for same real-world entity (THE COMPANY vs The Company; John's vs John; curly apostrophe).",
        root="Article strip case-sensitive before upper; both strip_suffix arms ASCII 0x27; no NFC/U+2019; case fold after strips in wrong order.",
        solution="NFC → casefold → strip articles/possessives (ASCII + U+2019) → underscore → UPPER. Dedup migration merge duplicates. DRY: single normalize_entity_name SSOT.",
        edges="CJK; hyphenated names; already-written dupes",
        tests="unit_normalize_THE_COMPANY; unit_normalize_curly_apostrophe; e2e_merge_duplicate_nodes_migration",
        related="D-32,X-17",
        diagram="""
  \"THE COMPANY\" -/strip/-> THE_COMPANY
  \"The Company\" --strip--> COMPANY  => TWO NODES
  fix: casefold first --> company --> strip --> COMPANY
""",
    ),
    dict(
        id="C-15",
        title="Pdf/Markdown chunker offsets not rebased",
        prio="P2",
        status="CONFIRMED",
        cluster="04-pipeline-reliability",
        sprint=2,
        laws="LAW-3",
        locus="chunker/page_aware.rs:155-177; markdown_chunking.rs:30-54",
        why="Character lineage wrong → citation/highlight offsets point at wrong spans.",
        root="Sub-chunkers return offsets relative to segment; wrappers never add base_offset; structure_indice may rewrite content.",
        solution="Add base_offset when pushing sub-chunks; contract test full-doc slice equals chunk text.",
        edges="overlapping pages; empty segments",
        tests="e2e_page_aware_offsets_rebase; e2e_markdown_block_offsets",
        related="X-13",
        diagram="""
  doc[0..N] --> seg@base --> chunk(start=0) --> stored start=0 (WRONG)
  fix: stored start = base + local
""",
    ),
    dict(
        id="C-16",
        title="Atomic blocks ignore size guard",
        prio="P2",
        status="CONFIRMED",
        cluster="04-pipeline-reliability",
        sprint=2,
        laws="LAW-2",
        locus="chunker/recursive.rs:384-390",
        why="Huge table → single chunk > embedder window → whole batch fails.",
        root="atomic regions forced as one piece; no max size split.",
        solution="If atomic.len > max_embed_chars, split with overlap while preserving atomic marker metadata; partial embed tolerance (X-18).",
        edges="nested atomic; tiny chunk_size",
        tests="e2e_huge_table_splits; unit_atomic_respects_max",
        related="X-08,X-18",
        diagram="""
  atomic table 2MB --> one chunk --> embed 400 --> FAIL batch
""",
    ),
    dict(
        id="C-17",
        title="Gleaning calls complete() without CompletionOptions",
        prio="P1",
        status="CONFIRMED",
        cluster="04-pipeline-reliability",
        sprint=3,
        laws="LAW-3",
        locus="extractor/gleaning.rs:199-205 vs extractor/llm.rs complete_with_options",
        why="Reasoning models burn budget on CoT during gleaning; inconsistent extraction quality/cost.",
        root="Gleaning path forgot shared extraction_completion_options.",
        solution="DRY: call complete_with_options(extraction_completion_options(...)) in gleaning.",
        edges="providers without options",
        tests="contract_gleaning_uses_completion_options",
        related="X-07",
        diagram="""
  base extract --> complete_with_options(temp=0, max_tokens, reasoning=none)
  gleaning -------> complete() bare
""",
    ),
    dict(
        id="C-18",
        title="CHUNK_MAX_RETRIES=0 means zero attempts",
        prio="P1",
        status="CONFIRMED",
        cluster="04-pipeline-reliability",
        sprint=2,
        laws="LAW-3",
        locus="pipeline/extraction.rs:302; config allows 0",
        why="Chunks skipped silently with no error when env set to 0.",
        root="for attempt in 1..=max_retries empty when 0; name says retries not attempts.",
        solution="Rename to max_attempts with min 1, OR treat 0 as 1 attempt; reject invalid config at boot.",
        edges="fail-fast tests expecting 0",
        tests="e2e_chunk_max_retries_zero_still_attempts_once_or_rejects",
        related="X-06",
        diagram="""
  max_retries=0 --> 1..=0 --> no loop --> no extract, no error
""",
    ),
    dict(
        id="C-19",
        title="drop_workspace_table missing prefix (RETRACTED)",
        prio="P3",
        status="RETRACTED",
        cluster="08-dead-code",
        sprint=4,
        laws="LAW-8",
        locus="workspace_vector.rs:204-226; workspace_crud.rs:515-518",
        why="N/A — false positive from v0.18 report.",
        root="Affirmation erroneous; prefix eq_ present via format!.",
        solution="No code change. Keep stub for traceability.",
        edges="n/a",
        tests="resource_safety_proof drop_workspace_table contracts",
        related="",
        diagram="""
  report claimed missing prefix --> code has public.eq_{ns}... --> RETRACTED
""",
    ),
    dict(
        id="C-20",
        title="Contract test native_upsert vacuous / contradictory",
        prio="P2",
        status="CONFIRMED",
        cluster="00-schema-readiness",
        sprint=0,
        laws="LAW-8",
        locus="contract_spec054_query_postgres_perf.rs:103-109",
        why="CI green while upsert arbiter drifted to eq_* indexes — false confidence.",
        root="assert!(legacy || contains(\"source_id\")) always true; SPEC-060 removed string asserted elsewhere.",
        solution="Assert ON CONFLICT (eq_*) / idx_edge_eq_source_target; fail if legacy-only.",
        edges="dual path during migration",
        tests="contract_native_upsert_eq_arbiter",
        related="P0,X-03",
        diagram="""
  test OR source_id --> always pass
  code ON CONFLICT (eq_*) --> untested
""",
    ),
    dict(
        id="C-21",
        title="batch_fetch_chunk_contents is N+1",
        prio="P2",
        status="CONFIRMED",
        cluster="04-pipeline-reliability",
        sprint=3,
        laws="LAW-3",
        locus="chunk_content.rs:30-42; KVStorage::get_by_ids exists",
        why="Query latency linear in chunk count.",
        root="Loop get_by_id despite batch API.",
        solution="Use get_by_ids / get_by_ids_ordered.",
        edges="empty ids; missing keys",
        tests="contract_batch_fetch_uses_get_by_ids; bench_chunk_fetch",
        related="",
        diagram="""
  for id in ids { get_by_id }  --> N round-trips
  get_by_ids(ids) -------------> 1 round-trip
""",
    ),
    dict(
        id="C-22",
        title="KV upsert non-transactional across batches",
        prio="P2",
        status="CONFIRMED",
        cluster="04-pipeline-reliability",
        sprint=2,
        laws="LAW-2",
        locus="adapters/postgres/kv.rs:257-289",
        why="Mid-batch failure leaves partial writes; inconsistent document state.",
        root="Per-chunk execute on pool; no BEGIN/COMMIT around multi-batch upsert.",
        solution="Single transaction for upsert(data); or outbox pattern. Rollback on error.",
        edges="huge payloads; statement_timeout",
        tests="e2e_kv_upsert_all_or_nothing",
        related="C-23",
        diagram="""
  batch1 COMMIT --> batch2 FAIL --> partial KV
""",
    ),
    dict(
        id="C-23",
        title="Document dedup broken: indexed vs completed",
        prio="P2",
        status="CONFIRMED",
        cluster="04-pipeline-reliability",
        sprint=2,
        laws="LAW-3",
        locus="document_reingest.rs:65-71 vs status_updates completed→indexed; reprocess_admission",
        why="Dedup/re-ingest predicates miss modern status → duplicates or blocked transitions.",
        root="Status vocabulary split across KV/PG/UI without SSOT.",
        solution="DocumentStatus enum SSOT; map indexed↔completed explicitly; align allowlists and unique indexes.",
        edges="partial_failure; processed legacy",
        tests="e2e_dedup_matches_completed_and_indexed",
        related="X-29",
        diagram="""
  unique WHERE status='indexed'  (dead)
  runtime status='completed' --> never matches
""",
    ),
    dict(
        id="C-24",
        title="matches_track_id ignores Deletion* events",
        prio="P2",
        status="CONFIRMED",
        cluster="01-tenant-isolation",
        sprint=1,
        laws="LAW-3",
        locus="websocket.rs:573-580; websocket_types Deletion*",
        why="Deletion progress never reaches track-scoped WS clients.",
        root="Match arms omit Deletion*/BulkDeletion* which carry track_id.",
        solution="Exhaustive match on all variants with track_id/task_id; compile-fail on new variants (_ => false banned).",
        edges="bulk deletion",
        tests="contract_matches_track_id_deletion_variants",
        related="S-01,X-23",
        diagram="""
  DeletionPhase{track_id} --> _ => false --> client silent
""",
    ),
    dict(
        id="C-25",
        title="Anthropic provider ignores ImageData::from_url",
        prio="P3",
        status="CONFIRMED",
        cluster="04-pipeline-reliability",
        sprint=4,
        laws="LAW-3",
        locus="edgequake-llm anthropic.rs:908-912; traits ImageData::from_url",
        why="URL images sent as bogus base64 → invalid Anthropic requests.",
        root="Always source_type=base64; OpenAI uses to_api_url().",
        solution="Branch on mime_type==url / source kind like OpenAI path.",
        edges="data URLs",
        tests="unit_anthropic_url_image_source",
        related="",
        diagram="""
  ImageData::from_url --> Anthropic forces base64 (WRONG)
""",
    ),
    dict(
        id="C-26",
        title="MAX_SOURCE_IDS=300 declared never applied",
        prio="P3",
        status="CONFIRMED",
        cluster="03-graph-identity",
        sprint=2,
        laws="LAW-3,LAW-7",
        locus="entity.rs:50; relationship.rs:65; live cap merge_limits 200",
        why="Misleading constant; real cap elsewhere → confusion and drift.",
        root="Dead const; add_source uncapped; pipeline has separate 200.",
        solution="Delete dead const or wire SSOT MAX_SOURCE_IDS from merge_limits; single number.",
        edges="D-33 order",
        tests="contract_single_source_id_cap",
        related="D-33",
        diagram="""
  MAX_SOURCE_IDS=300 (dead)
  DEFAULT_MAX_SOURCE_IDS=200 (live)
""",
    ),
    dict(
        id="C-27",
        title="Tenant cache last_accessed never refreshed (FIFO≠LRU)",
        prio="P2",
        status="CONFIRMED",
        cluster="06-ops-ci-sdk",
        sprint=4,
        laws="LAW-3",
        locus="tenant_manager.rs:254-259,480-487",
        why="Hot tenants evicted before cold ones under load.",
        root="Cache hit returns without updating last_accessed.",
        solution="Touch last_accessed on hit; or use LRU crate.",
        edges="clock; concurrent hits",
        tests="unit_tenant_cache_lru_touch",
        related="",
        diagram="""
  hit --> return Arc (no touch) --> eviction by insert time
""",
    ),
    dict(
        id="C-28",
        title="cosine_similarity panics on dimension mismatch",
        prio="P2",
        status="CONFIRMED",
        cluster="05-query-fusion",
        sprint=2,
        laws="LAW-4",
        locus="edgequake-core/src/types/embedding.rs:83-88",
        why="Unexpected embedding dim kills thread instead of error path.",
        root="assert_eq! in library method.",
        solution="cosine_similarity -> Result; callers handle Err. Keep relevancy_prune safe path.",
        edges="zero norms already handled",
        tests="unit_cosine_dim_mismatch_is_err",
        related="X-10",
        diagram="""
  dim 1536 vs 768 --> panic
  dim mismatch -----> Err(DimensionMismatch)
""",
    ),
]

# Design D-30..D-54 (skip D-43 gap in register — we still document D-44 as PDF limit)
CATALOG += [
    dict(
        id="D-30",
        title="Graph is not a multigraph: edge key omits type",
        prio="P2",
        status="CONFIRMED",
        cluster="03-graph-identity",
        sprint=2,
        laws="LAW-3,LAW-6",
        locus="edges_ops.rs ON CONFLICT (eq_source_id, eq_target_id)",
        why="Alice-KNOWS-Bob and Alice-WORKS_WITH-Bob overwrite — semantic loss at ingest.",
        root="Unique index/arbiter excludes relation type.",
        solution="Unique (eq_source_id, eq_target_id, rel_type); migrate; update merge keys.",
        edges="null type; case of type",
        tests="e2e_multigraph_two_rel_types_persist",
        related="D-31,C-20",
        diagram="""
  (A,B,KNOWS) upsert
  (A,B,WORKS_WITH) ON CONFLICT (A,B) --> overwrites
""",
    ),
    dict(
        id="D-31",
        title="Relation weight (a+b)/2 order-dependent non-associative",
        prio="P2",
        status="CONFIRMED",
        cluster="03-graph-identity",
        sprint=2,
        laws="LAW-3",
        locus="merger/relationship.rs:619-627; divergent vector/graph policies",
        why="Weight has no stable meaning; 3 merges of 1.0 → 0.9375 not 1.0.",
        root="Exponential smoothing α=0.5; three dedup layers disagree (max vs mean).",
        solution="SSOT WeightPolicy: store (sum, count) or use max; document; one policy across vector/graph.",
        edges="zero weight; negative",
        tests="unit_weight_associative; contract_single_weight_policy",
        related="D-30,D-37",
        diagram="""
  w=(w+1)/2 three times: 0.5 -> 0.75 -> 0.875 -> 0.9375
  true mean of ones -> 1.0
""",
    ),
    dict(
        id="D-32",
        title="Entity type first-wins forever, no conflict log",
        prio="P2",
        status="CONFIRMED",
        cluster="03-graph-identity",
        sprint=2,
        laws="LAW-3",
        locus="merger/entity.rs keep first type; update_entity_node never writes entity_type",
        why="Wrong type from first chunk frozen; no observability.",
        root="Update path omits entity_type; no conflict metric.",
        solution="Majority vote or confidence; always log conflicts; allow type update when evidence stronger.",
        edges="OTHER vs CONCEPT; X-15",
        tests="e2e_entity_type_conflict_logged_and_resolved",
        related="C-14,X-15",
        diagram="""
  chunk1 type=ORG --> stored
  chunk2 type=PERSON --> ignored silently
""",
    ),
    dict(
        id="D-33",
        title="source_ids cap before lineage computation",
        prio="P2",
        status="CONFIRMED",
        cluster="03-graph-identity",
        sprint=2,
        laws="LAW-7",
        locus="merger/entity.rs:430-440; merge_limits",
        why="Documents whose chunks fall outside cap vanish from lineage/scope filters.",
        root="Truncate chunk_ids then derive source_document_ids.",
        solution="Compute full lineage/document set first; then cap stored source_ids with deterministic policy (e.g. newest).",
        edges="cap=0; single doc many chunks",
        tests="e2e_lineage_includes_docs_beyond_source_cap",
        related="C-26",
        diagram="""
  chunks[1..500] --> cap 200 --> docs from capped only --> docZ missing
""",
    ),
    dict(
        id="D-34",
        title="Double gate merger(1200) vs summarizer(4000)",
        prio="P2",
        status="CONFIRMED",
        cluster="03-graph-identity",
        sprint=3,
        laws="LAW-3",
        locus="description_merge.rs:213-224; summarizer.rs:202-207",
        why="NeedsLlm in [1200,4000] never calls LLM — contradiction resolution skipped. Jaccard case-sensitive underestimates similarity.",
        root="Two independent gates; no shared TokenGate SSOT.",
        solution="One threshold SSOT; Jaccard normalize case/punct; if NeedsLlm then summarizer must LLM.",
        edges="empty descriptions",
        tests="unit_needs_llm_always_summarizes; unit_jaccard_normalized",
        related="D-53",
        diagram="""
  merger NeedsLlm@1200 --> summarizer simple_merge@<4000 --> no LLM
""",
    ),
    dict(
        id="D-35",
        title="Docs say weighted sum; Mix fusion uses max",
        prio="P3",
        status="CONFIRMED",
        cluster="05-query-fusion",
        sprint=4,
        laws="LAW-3,LAW-8",
        locus="modes/mix.rs:328-334; QueryEngineConfig docs",
        why="Operators tune weights expecting sum; behavior is max — wrong relevance.",
        root="Doc/code drift.",
        solution="Either implement weighted sum or rename docs/API to max-after-minmax. Prefer rename+enum clarity.",
        edges="zero arms",
        tests="contract_mix_fusion_semantics_documented",
        related="D-36,D-37",
        diagram="""
  docs: sum(w_i * s_i)
  code: max(contribution)
""",
    ),
    dict(
        id="D-36",
        title="EDGEQUAKE_SPARSE_FUSION=weighted is sparse-first",
        prio="P3",
        status="CONFIRMED",
        cluster="05-query-fusion",
        sprint=4,
        laws="LAW-3",
        locus="sparse_retrieval.rs:161-173",
        why="Misnamed mode; operators think weights apply.",
        root="Weighted arm returns sparse order; rrf added in SPEC-076 separately.",
        solution="Rename weighted→sparse_first; keep rrf; document.",
        edges="empty sparse",
        tests="contract_fusion_mode_names",
        related="D-35,D-39",
        diagram="""
  mode=weighted --> ignore vector ranks --> sparse-first
""",
    ),
    dict(
        id="D-37",
        title="chunk_score carries three incompatible scales",
        prio="P2",
        status="CONFIRMED",
        cluster="05-query-fusion",
        sprint=4,
        laws="LAW-3",
        locus="relevancy_prune; graph_ppr; fusion RRF; mix minmax",
        why="Comparing/thresholding scores across arms is meaningless.",
        root="No ScoreScale enum; f32 overloaded.",
        solution="Typed scores {Cosine, PprShare, Rrf, MinMax}; convert at fusion boundary.",
        edges="min_score D-39",
        tests="unit_score_scale_no_cross_compare",
        related="D-39,D-35",
        diagram="""
  cosine[-1,1] vs RRF[0,w] vs PPR share --> same f32 field
""",
    ),
    dict(
        id="D-38",
        title="query_vec embeds history+question",
        prio="P2",
        status="CONFIRMED",
        cluster="05-query-fusion",
        sprint=3,
        laws="LAW-3",
        locus="query_pipeline.rs:432-487",
        why="Conversation history pollutes retrieval vector; CLIC propagates to HL/LL slots.",
        root="keyword_query = query_with_conversation_context then embed_one.",
        solution="Embed question-only (optionally rewrite); keep history for prompt only.",
        edges="empty history; multilingual",
        tests="e2e_query_vec_matches_question_only_embedding",
        related="D-39,X-20",
        diagram="""
  history+question --> embed --> retrieve (polluted)
  question ---------> embed --> retrieve
  history ---------------------> prompt only
""",
    ),
    dict(
        id="D-39",
        title="min_score skipped on fused / preserve_order paths",
        prio="P2",
        status="CONFIRMED",
        cluster="05-query-fusion",
        sprint=3,
        laws="LAW-3",
        locus="sparse_retrieval / chunk_retrieval preserve_order",
        why="Low-similarity chunks pass when graph order wins — silent threshold bypass.",
        root="Filter applied only on some arms.",
        solution="Always apply min_score in declared scale; if preserve_order, filter then stable-sort.",
        edges="all filtered empty",
        tests="e2e_min_score_enforced_on_rrf",
        related="D-37,D-36",
        diagram="""
  fuse --> preserve_order --> skip min_score --> junk chunks
""",
    ),
    dict(
        id="D-40",
        title="QueryStats vs QueryStreamStats diverged",
        prio="P2",
        status="CONFIRMED",
        cluster="05-query-fusion",
        sprint=4,
        laws="LAW-3",
        locus="query_types / edgequake-query types / core query stats",
        why="Streaming path lacks arm diagnostics — explainability gone for SSE clients.",
        root="Three shapes evolved independently.",
        solution="One QueryStats SSOT; stream mirrors fields; shared builder.",
        edges="partial stream abort",
        tests="contract_stream_stats_superset",
        related="X-21,X-22",
        diagram="""
  sync QueryStats { arms_run, ... }
  stream QueryStreamStats { subset } --> gap
""",
    ),
    dict(
        id="D-41",
        title="Progress percent unweighted average of phases",
        prio="P2",
        status="CONFIRMED",
        cluster="06-ops-ci-sdk",
        sprint=4,
        laws="LAW-3",
        locus="progress/mod.rs equal stage weights",
        why="Upload (seconds) weighs as Extraction (hours) — bar structurally false.",
        root="TODO equal weights.",
        solution="PhaseWeights SSOT (time or work units); configure per pipeline kind.",
        edges="skipped phases",
        tests="unit_progress_weighted",
        related="D-42",
        diagram="""
  mean([1,1,1,1,1,1]) --> 50% after upload done (misleading)
""",
    ),
    dict(
        id="D-42",
        title="ETA resets after serialization; progress process-local",
        prio="P2",
        status="CONFIRMED",
        cluster="06-ops-ci-sdk",
        sprint=4,
        laws="LAW-3",
        locus="progress.rs avg_item_time_ms serde(skip); HashMap process-local",
        why="Restart loses bar; ETA jumps to zero after serialize.",
        root="Non-persisted fields; in-memory map.",
        solution="Persist tracker snapshot in task/KV; include avg_item_time_ms.",
        edges="multi-replica",
        tests="e2e_progress_survives_restart",
        related="D-41",
        diagram="""
  serialize --> skip avg --> deserialize --> ETA 0
  restart --> empty HashMap --> bar lost
""",
    ),
    dict(
        id="D-44",
        title="PDF 100 MiB contract unreachable; error text wrong",
        prio="P2",
        status="PARTIAL",
        cluster="06-ops-ci-sdk",
        sprint=4,
        laws="LAW-3",
        locus="DefaultBodyLimit 50MiB; validation.rs 100MiB dead; messages mention 10MB",
        why="Docs/validators disagree; users hit opaque 413; wrong error strings.",
        root="Three limits (body, validator, copy). Runtime SSOT already 50 MiB in budget/FE.",
        solution="Single MAX_UPLOAD_BYTES=50MiB everywhere; delete dead 100MiB; fix messages.",
        edges="multipart overhead",
        tests="contract_upload_limit_ssot_50mib",
        related="S-12,D-51",
        diagram="""
  validate allows 100MiB --> body limit 50MiB --> dead code
""",
    ),
    dict(
        id="D-45",
        title="audit_logs defined 4×; partitions never scheduled",
        prio="P2",
        status="CONFIRMED",
        cluster="06-ops-ci-sdk",
        sprint=2,
        laws="LAW-2,LAW-3",
        locus="001,012,docker/init,specs schema; unbounded audit channel",
        why="INSERTs break past last pre-created partition; audit loss on error/shutdown.",
        root="Multiple CREATE TABLE; no partition maintenance job; unbounded channel drop-on-error.",
        solution="One migration SSOT; pg_partman or cron create partitions; bounded channel + shutdown flush.",
        edges="timezone month boundaries",
        tests="e2e_audit_insert_next_month_partition; contract_single_audit_definition",
        related="",
        diagram="""
  4 CREATE TABLE definitions
  partitions for 12 months --> month 13 INSERT FAIL
""",
    ),
    dict(
        id="D-46",
        title="OTEL layer mounted before env_filter",
        prio="P3",
        status="CONFIRMED",
        cluster="06-ops-ci-sdk",
        sprint=4,
        laws="LAW-3",
        locus="observability/subscriber.rs:117-124",
        why="Cannot reduce exported trace volume via RUST_LOG.",
        root="Intentional order comment; filter after OTLP bridge.",
        solution="Apply EnvFilter before OTLP layer (or layered filter on export).",
        edges="dynamic reload",
        tests="contract_otel_respects_rust_log",
        related="",
        diagram="""
  registry --> otel --> env_filter  (otel unfiltered)
  registry --> env_filter --> otel
""",
    ),
    dict(
        id="D-47",
        title="make postgres-start does not exist",
        prio="P3",
        status="CONFIRMED",
        cluster="06-ops-ci-sdk",
        sprint=4,
        laws="LAW-8",
        locus="Makefile db-start; AGENTS/CONTRIBUTING say postgres-start",
        why="Onboarding broken.",
        root="Docs/Makefile drift.",
        solution="Alias postgres-start: db-start; update docs to prefer db-start.",
        edges="test-postgres-start distinct",
        tests="contract_makefile_has_postgres_start_alias",
        related="",
        diagram="""
  docs: make postgres-start --> missing target
  Makefile: db-start
""",
    ),
    dict(
        id="D-48",
        title="SDK workflows nested under sdks/*/.github never run",
        prio="P2",
        status="CONFIRMED",
        cluster="06-ops-ci-sdk",
        sprint=4,
        laws="LAW-8",
        locus="sdks/*/.github/workflows/",
        why="PyPI/npm/crates/NuGet/Ruby publish CI never executes.",
        root="GitHub Actions only reads root .github/workflows.",
        solution="Move/call reusable workflows at repo root; matrix per SDK.",
        edges="secrets permissions",
        tests="contract_no_nested_github_workflows_or_root_dispatch",
        related="X-33,D-49",
        diagram="""
  sdks/python/.github/workflows --> IGNORED by GHA
  .github/workflows/sdk-*.yml --> runs
""",
    ),
    dict(
        id="D-49",
        title="sed -i '' BSD-only in publish targets",
        prio="P2",
        status="CONFIRMED",
        cluster="06-ops-ci-sdk",
        sprint=4,
        laws="LAW-8",
        locus="Makefile; scripts/bump-version.sh",
        why="Linux CI/release machines break on sed -i ''.",
        root="macOS sed dialect hard-coded.",
        solution="Portable sed helper (sed -i.bak || sed -i) or use Python/perl.",
        edges="busybox sed",
        tests="contract_no_sed_i_empty_string",
        related="D-48",
        diagram="""
  macOS sed -i '' OK
  GNU sed -i '' FAIL
""",
    ),
    dict(
        id="D-50",
        title=".env.example hardcodes VISION_PROVIDER=openai",
        prio="P1",
        status="CONFIRMED",
        cluster="06-ops-ci-sdk",
        sprint=1,
        laws="LAW-4",
        locus=".env.example:36",
        why="Users believing local Ollama still send PDF pages to OpenAI — data leak/cost.",
        root="Example defaults to cloud vision.",
        solution="Default empty or ollama; comment cloud opt-in clearly.",
        edges="CI needing openai",
        tests="contract_env_example_vision_not_openai_by_default",
        related="S-09",
        diagram="""
  copy .env.example --> VISION=openai --> PDFs leave machine
""",
    ),
    dict(
        id="D-51",
        title="Multipart 100% in RAM; batch uncapped file count",
        prio="P2",
        status="CONFIRMED",
        cluster="06-ops-ci-sdk",
        sprint=4,
        laws="LAW-2",
        locus="pdf_upload/upload.rs:113-119; file_upload.rs",
        why="OOM on large/batch uploads; only global body limit protects.",
        root="field.bytes().await to Vec; accumulate all files before processing.",
        solution="Stream to temp file; max files per batch; process sequentially.",
        edges="slow clients",
        tests="e2e_batch_file_cap; e2e_upload_streams_to_temp",
        related="D-44,S-12",
        diagram="""
  N files --> Vec<(name, bytes)> all in RAM --> then process
""",
    ),
    dict(
        id="D-52",
        title="Extraction cache never sets (100% miss)",
        prio="P2",
        status="CONFIRMED",
        cluster="04-pipeline-reliability",
        sprint=4,
        laws="LAW-3",
        locus="pipeline/cache.rs:358 TODO",
        why="~478 LOC overhead, zero hits.",
        root="get without set; TODO left.",
        solution="Wire set on success OR delete CachedExtractor (prefer delete until needed).",
        edges="cache key stability",
        tests="contract_cache_set_or_module_removed",
        related="dead-code §5",
        diagram="""
  get --> miss --> compute --> (no set) --> forever miss
""",
    ),
    dict(
        id="D-53",
        title="Three divergent token estimators; no real tokenizer",
        prio="P2",
        status="CONFIRMED",
        cluster="04-pipeline-reliability",
        sprint=3,
        laws="LAW-3",
        locus="embeddings ~2.5; text_utils len/4; summarizer; tiktoken in workspace unused by pipeline",
        why="Same chunk_size yields different real sizes; FR/CJK miscounted; embed/LLM limits wrong.",
        root="Heuristic estimators; pipeline doesn't depend on tiktoken-rs.",
        solution="TokenEstimator trait; default tiktoken cl100k; one dependency in pipeline.",
        edges="model-specific encodings",
        tests="unit_token_estimator_ssot; e2e_chunk_size_respects_tokenizer",
        related="D-34,X-08",
        diagram="""
  Fixed chunk_size=800 --> 3200 bytes heuristic
  Recursive -------------> ~800 words different scale
""",
    ),
    dict(
        id="D-54",
        title="Louvain phase-1 only; extractive community reports",
        prio="P3",
        status="CONFIRMED",
        cluster="07-accuracy-explain",
        sprint=5,
        laws="LAW-3",
        locus="storage/community.rs:315-384",
        why="No multi-level hierarchy; 'reports' are formatting not LLM summaries — global mode weak.",
        root="Phase-1 only implemented; reports extractive.",
        solution="Phase-2 aggregation optional feature; real report generation behind flag.",
        edges="tiny graphs",
        tests="unit_louvain_hierarchy_levels",
        related="X-35",
        diagram="""
  Louvain phase1 --> flat communities
  missing phase2 --> no hierarchy
""",
    ),
]

# Register X-01..X-12 (official)
CATALOG += [
    dict(
        id="X-01",
        title="Migration 002 entirely dead",
        prio="P2",
        status="CONFIRMED",
        cluster="06-ops-ci-sdk",
        sprint=4,
        laws="LAW-3",
        locus="migrations 001/002/026",
        why="tasks PK semantics diverge from intended 002 design; confusion for operators.",
        root="001 creates tasks with UUID PK; 002 IF NOT EXISTS no-op; 026 comments confirm.",
        solution="Document SSOT schema; squash/repair migration notes; do not re-run 002 blindly.",
        edges="fresh vs upgraded DBs",
        tests="contract_tasks_pk_documented",
        related="X-02",
        diagram="""
  001 creates tasks(id UUID PK)
  002 CREATE IF NOT EXISTS --> no-op --> dead
""",
    ),
    dict(
        id="X-02",
        title="Boot repairs migration checksums (fragile)",
        prio="P2",
        status="CONFIRMED",
        cluster="06-ops-ci-sdk",
        sprint=4,
        laws="LAW-2",
        locus="state/migration_bootstrap",
        why="New drift panics startup; repair masks history.",
        root="M071/M078 checksum drift between versions patched at boot.",
        solution="Freeze checksums.lock process; CI gate; avoid silent repair — fail with runbook.",
        edges="forked forks",
        tests="contract_checksum_drift_fails_loud",
        related="X-01,P0",
        diagram="""
  drift --> boot rewrite checksum --> starts
  new drift --> panic
""",
    ),
    dict(
        id="X-04",
        title="Vector module docs claim L2/IP; code cosine-only",
        prio="P3",
        status="CONFIRMED",
        cluster="05-query-fusion",
        sprint=4,
        laws="LAW-8",
        locus="vector/capabilities.rs",
        why="Operators configure non-cosine metrics that do nothing.",
        root="Docs overclaim; indexes always cosine ops.",
        solution="Docs/API expose cosine-only until real ops exist.",
        edges="pgvector ops classes",
        tests="contract_vector_metric_cosine_only",
        related="X-10",
        diagram="""
  docs: Cosine|L2|IP
  code: cosine only
""",
    ),
    dict(
        id="X-05",
        title="BM25 label is ts_rank_cd; english config hard-coded",
        prio="P2",
        status="CONFIRMED",
        cluster="05-query-fusion",
        sprint=4,
        laws="LAW-3,LAW-8",
        locus="fts.rs",
        why="French corpus stemmed as English; ranking not true BM25.",
        root="Marketing name; text search config literal 'english'.",
        solution="Rename to cover-density/tsvector; configurable FTS language; optional ParadeDB later.",
        edges="mixed language docs",
        tests="e2e_fts_language_config",
        related="D-36",
        diagram="""
  label BM25 --> ts_rank_cd(english)
""",
    ),
    dict(
        id="X-06",
        title="LLM layer: no jitter, single WaitAndRetry, no circuit breaker",
        prio="P1",
        status="CONFIRMED",
        cluster="04-pipeline-reliability",
        sprint=3,
        laws="LAW-5",
        locus="llm-error / worker retry paths",
        why="Thundering herd on 429; weak retry; cascading failure.",
        root="Pure exponential backoff; WaitAndRetry once; no breaker in LLM layer.",
        solution="RetryExecutor + full jitter; breaker; use typed retry_strategy.",
        edges="local Ollama overload",
        tests="unit_retry_has_jitter; e2e_breaker_opens",
        related="X-07,X-30",
        diagram="""
  N workers 429 --> sync retry --> herd
  jitter+breaker --> shed load
""",
    ),
    dict(
        id="X-07",
        title="LLM retry reimplemented via substring matching",
        prio="P1",
        status="CONFIRMED",
        cluster="04-pipeline-reliability",
        sprint=3,
        laws="LAW-5",
        locus="pipeline/helpers/embeddings.rs:204; LlmError::retry_strategy unused",
        why="Provider wording changes → wrong retry/Unknown; costly duplicate logic.",
        root="Typed strategy exists; EdgeQuake string-matches '429'/'rate limit'.",
        solution="Only LlmError::retry_strategy / RetryExecutor; delete substring paths.",
        edges="wrapped errors",
        tests="contract_no_substring_retry_matching; unit_typed_429",
        related="X-06,X-30",
        diagram="""
  LlmError.retry_strategy()  (unused)
  err.to_string().contains(\"429\") (used)
""",
    ),
    dict(
        id="X-08",
        title="Three contradictory embedding batch clamps",
        prio="P2",
        status="CONFIRMED",
        cluster="04-pipeline-reliability",
        sprint=3,
        laws="LAW-3",
        locus="safety_limits; wrapper; Makefile Mistral 16",
        why="Same limit three numbers → 400 errors or underutilization.",
        root="Trait 2048 / wrapper 256 / Makefile 16; env missing from .env.example.",
        solution="One EDGEQUAKE_EMBEDDING_BATCH_SIZE SSOT; provider.max_batch_size min'd once.",
        edges="provider-specific",
        tests="contract_embed_batch_ssot",
        related="D-53,C-16",
        diagram="""
  2048 vs 256 vs 16 --> contradiction
""",
    ),
    dict(
        id="X-09",
        title="Diamond dependency: two edgequake-llm versions in lock",
        prio="P2",
        status="CONFIRMED",
        cluster="06-ops-ci-sdk",
        sprint=4,
        laws="LAW-3",
        locus="Cargo.lock; pdf2md",
        why="Subtle trait/type mismatches; duplicate code size.",
        root="pdf2md pulled older llm; recurrence of 0.5.1 bug.",
        solution="Align pdf2md / [patch] / upgrade; cargo tree gate in CI.",
        edges="semver",
        tests="contract_single_edgequake_llm_version",
        related="",
        diagram="""
  app --> llm 0.10.1
  pdf2md --> llm 0.6.x  (diamond)
""",
    ),
    dict(
        id="X-10",
        title="No client L2 normalization of embeddings",
        prio="P1",
        status="CONFIRMED",
        cluster="04-pipeline-reliability",
        sprint=3,
        laws="LAW-3",
        locus="LLM/pipeline embeddings; Ollama vs OpenAI",
        why="Local similarities wrong vs OpenAI-normalized vectors.",
        root="Contract silent on who normalizes; no client normalize.",
        solution="Always L2-normalize on write and query (SSOT); document.",
        edges="zero vector",
        tests="e2e_ollama_cosine_after_l2; unit_normalize",
        related="C-28,X-04",
        diagram="""
  OpenAI: pre-normalized
  Ollama: raw --> cosine wrong without L2
""",
    ),
    dict(
        id="X-11",
        title="Scan and Reindex task types unimplemented",
        prio="P2",
        status="CONFIRMED",
        cluster="06-ops-ci-sdk",
        sprint=5,
        laws="LAW-3",
        locus="processor/task_impl.rs",
        why="Cannot reindex after embedding model change without full reingest.",
        root="Variants return UnsupportedOperation.",
        solution="Implement reindex job: walk chunks → re-embed → swap alias; scan for orphan cleanup.",
        edges="partial failure mid-reindex",
        tests="e2e_reindex_embedding_model_change",
        related="X-35",
        diagram="""
  Task::Reindex --> UnsupportedOperation
""",
    ),
    dict(
        id="X-12",
        title="PDF concurrency match decorative (all arms return 2)",
        prio="P3",
        status="CONFIRMED",
        cluster="06-ops-ci-sdk",
        sprint=4,
        laws="LAW-8",
        locus="pdf concurrency match 0..=49=>2 ... =>2",
        why="Readers believe adaptive concurrency; always 2.",
        root="Placeholder arms never filled.",
        solution="Real schedule by pages/VRAM or delete match and document constant.",
        edges="local vs cloud vision",
        tests="contract_pdf_concurrency_schedule",
        related="",
        diagram="""
  match pages { 0..=49=>2, 50..=199=>2, ... =>2 }
""",
    ),
]

# Page-6 promotions X-13..X-27
PAGE6 = [
    ("X-13", "Duplicate page markers without SSOT", "P2", "CONFIRMED", "04-pipeline-reliability", "Vision/EdgeParse duplicate <|-- edgequake-page:N -->| PageAware reparses — divergence breaks page lineage.", "Two backends stamp markers; no single writer.", "SSOT PageMarkerWriter; strip-before-restamp; contract identical marker grammar.", "C-15"),
    ("X-14", "LightRAG separator cascade never active in prod", "P2", "CONFIRMED", "04-pipeline-reliability", "CJK/last-resort char split unused; tests mask by passing cascade.", "ChunkerConfig.default() non-empty ASCII separators; default_recursive_separators only if empty.", "Default separators = LightRAG cascade including final \"\"; tests use production default.", "D-53"),
    ("X-15", "OTHER missing from default entity types", "P3", "FIXED", "03-graph-identity", "Was forcing CONCEPT fallback; heterogeneous entities collapsed.", "Defaults omitted OTHER while prompt mentioned it.", "Verify prompts/mod.rs includes OTHER; keep regression test.", "D-32"),
    ("X-16", "empty_on_missing_json silent empty extraction", "P1", "CONFIRMED", "04-pipeline-reliability", "Doc marked processed with 0 entities when LLM returns non-JSON.", "empty_on_missing_json:true swallows parse failure.", "Fail chunk/doc or quarantine; never success with silent empty unless explicit allow.", "C-17"),
    ("X-17", "Entity resolution exact-match only", "P2", "PARTIAL", "03-graph-identity", "ORG vs ORGANIZATION duplicates; embeddings unused for identity.", "Merge by normalized EntityId only; no fuzzy/embedding blocking.", "Blocking keys + embedding similarity threshold optional; stem/accent fold in normalize.", "C-14"),
    ("X-18", "No partial tolerance on embedding batches", "P2", "PARTIAL", "04-pipeline-reliability", "One sub-batch error fails entire collect; waste.", "collect::<Result<Vec<_>>>() fail-fast; truncate policy exists separately.", "Per-sub-batch Result; retry/skip with metrics; don't fail whole doc on one batch.", "C-16,X-08"),
    ("X-19", "No pipeline backpressure / token-bucket rate limit", "P2", "PARTIAL", "04-pipeline-reliability", "Memory blowups; provider quota storms.", "buffer_unordered bounds concurrency not memory; tenant_limiter is semaphore.", "Admission control + token bucket per provider; bound in-flight bytes.", "X-06"),
    ("X-20", "Citations coupled to context by position index", "P2", "PARTIAL", "05-query-fusion", "Reorder after format breaks [N]↔reference_id silently.", "format_query_context uses i+1; dual lists in some modes.", "Stable citation_id on chunks; never renumber after format.", "D-38"),
    ("X-21", "ExplainTrace nonexistent", "P3", "CONFIRMED", "07-accuracy-explain", "No structured explainability; only QueryStats postmortem.", "Spec 0003 still Proposed; no ExplainTrace type.", "MVP ExplainTrace from arm stats; API field optional.", "D-40,X-35"),
    ("X-22", "SSE Thinking event never emitted", "P3", "CONFIRMED", "05-query-fusion", "Dead protocol variant; casing inconsistent WS vs SSE.", "Variant defined; production stream omits Thinking.", "Emit or remove; unify event casing SSOT.", "D-40"),
    ("X-23", "WebSocket Lagged swallowed", "P2", "CONFIRMED", "01-tenant-isolation", "Clients miss events without notice; track filter after recv worsens loss.", "broadcast Lagged → warn+continue at bridge/handler.", "Send LagNotification or disconnect; per-tenant channels; filter before enqueue if possible.", "S-01,C-24"),
    ("X-24", "main.rs AUTO_RESUME comment stale (default ON)", "P2", "CONFIRMED", "06-ops-ci-sdk", "Operators think manual resume; boot may relaunch LLM jobs (cost).", "Comment says manual; hydrate default ON in 0.20.", "Fix comment+docs; make default explicit in logs.", ""),
    ("X-25", "OpenAPI build gate blind to routes.rs mounts", "P2", "PARTIAL", "06-ops-ci-sdk", "Annotated but unmounted (or wrong path) handlers pass checks.", "build.rs checks handlers↔openapi not Axum routes; some parity tests exist.", "Include routes.rs path inventory in gate; single route registry SSOT.", "X-26"),
    ("X-26", "schema.d.ts unused by webui/SDKs", "P3", "CONFIRMED", "06-ops-ci-sdk", "OpenAPI codegen gated but clients hand-written — drift.", "No imports of schema.d.ts; SDKs ignore snapshot.", "Generate TS client from OpenAPI OR drop dead artifact; SDKs consume SSOT.", "X-25,X-33"),
    ("X-27", "Frontend has no middleware.ts server guard", "P2", "CONFIRMED", "06-ops-ci-sdk", "Authz only client-side — deep links/API misuse.", "No Next middleware.ts in webui.", "Add middleware for session/auth redirects; never trust client-only guards.", "S-01"),
]

for pid, title, prio, status, cluster, why, root, solution, related in PAGE6:
    sprint = 1 if prio == "P1" else (3 if "pipeline" in cluster or "query" in cluster else 4)
    if cluster.startswith("07"):
        sprint = 5
    if cluster.startswith("01"):
        sprint = 1
    CATALOG.append(
        dict(
            id=pid,
            title=title,
            prio=prio,
            status=status,
            cluster=cluster,
            sprint=sprint,
            laws="LAW-3" if status != "FIXED" else "LAW-8",
            locus="see register page-6 / cluster doc",
            why=why,
            root=root,
            solution=solution,
            edges="see cluster EDGE_CASES",
            tests=f"contract_{pid.lower().replace('-', '_')}; e2e_{pid.lower().replace('-', '_')}",
            related=related,
            diagram=f"\n  {pid}: {title}\n       |\n       v\n  root → symptom → fix\n",
        )
    )

# X-28..X-37
CATALOG += [
    dict(
        id="X-28",
        title="Checkpoint content_hash is 64-bit over first 64KiB",
        prio="P1",
        status="PARTIAL",
        cluster="04-pipeline-reliability",
        sprint=2,
        laws="LAW-2",
        locus="PipelineCheckpoint hash first 65536 bytes truncated",
        why="Suffix-only edits pass checkpoint validation → resume on stale state.",
        root="Performance shortcut fingerprint incomplete.",
        solution="Full SHA-256 (or chunked Merkle); never truncate to 8 bytes for correctness path.",
        edges="empty files; streaming uploads",
        tests="e2e_checkpoint_rejects_suffix_change",
        related="X-29",
        diagram="""
  hash(first 64KiB)[0..8] --> collision/suffix miss
  sha256(full) -------------> safe
""",
    ),
    dict(
        id="X-29",
        title="No task state machine guards transitions",
        prio="P1",
        status="CONFIRMED",
        cluster="04-pipeline-reliability",
        sprint=2,
        laws="LAW-2",
        locus="tasks update_task / mark_success",
        why="Cancelled→Success possible; lost update races.",
        root="update_task without FROM-status guard / version.",
        solution="Explicit FSM + UPDATE ... WHERE status IN (...); optimistic version column.",
        edges="retry vs cancel race",
        tests="e2e_cancelled_cannot_mark_success; e2e_optimistic_lock",
        related="C-23",
        diagram="""
  Cancelled --mark_success--> Success (allowed today)
  FSM: Cancelled -/-> Success
""",
    ),
    dict(
        id="X-30",
        title="Circuit breaker / failure class via string matching",
        prio="P1",
        status="CONFIRMED",
        cluster="04-pipeline-reliability",
        sprint=3,
        laws="LAW-5",
        locus="worker.rs; ingestion_reliability contains(\"timeout\")",
        why="Business message with 'timeout' trips breaker; provider wording change → Unknown retryable.",
        root="Hard threshold 3; English substring classification.",
        solution="Typed IngestionFailureClass from LlmError/io kinds; no contains.",
        edges="localized errors",
        tests="unit_failure_class_typed; unit_breaker_ignores_business_timeout_word",
        related="X-06,X-07",
        diagram="""
  msg.contains(\"timeout\") --> breaker++
  typed Timeout error -----> breaker++
""",
    ),
    dict(
        id="X-31",
        title="Shutdown has no drain timeout",
        prio="P2",
        status="CONFIRMED",
        cluster="06-ops-ci-sdk",
        sprint=4,
        laws="LAW-2",
        locus="worker.rs / main server path",
        why="2h PDF blocks process exit — bad deploys/k8s.",
        root="No with_graceful_shutdown drain deadline.",
        solution="Shutdown budget; cancel tasks cooperatively; force after T.",
        edges="in-flight LLM calls",
        tests="e2e_shutdown_drains_or_cancels_within_budget",
        related="X-29",
        diagram="""
  SIGTERM --> wait forever on long PDF
  SIGTERM --> drain≤T --> cancel --> exit
""",
    ),
    dict(
        id="X-32",
        title="Decorative CI gates",
        prio="P2",
        status="CONFIRMED",
        cluster="06-ops-ci-sdk",
        sprint=4,
        laws="LAW-8",
        locus=".github/workflows continue-on-error; frontend-test echo fallback",
        why="CI cannot fail on audit/lint/tests — false green.",
        root="continue-on-error; `|| echo No tests`; clippy --lib only; no dependabot/CODEOWNERS.",
        solution="Make gates blocking; remove echo fallback; clippy all-targets; add dependabot+CODEOWNERS.",
        edges="flaky e2e quarantine explicitly",
        tests="contract_ci_no_continue_on_error_critical; contract_frontend_test_must_run",
        related="D-48,X-34",
        diagram="""
  cargo audit continue-on-error --> green
  pnpm test || echo --> green
""",
    ),
    dict(
        id="X-33",
        title="SDKs locked at 0.4.0 while server 0.20.2",
        prio="P2",
        status="CONFIRMED",
        cluster="06-ops-ci-sdk",
        sprint=4,
        laws="LAW-3,LAW-8",
        locus="sdks/*/version",
        why="Divergent surfaces (PDF top-level vs nested); clients broken vs server.",
        root="Hand-written SDKs; no openapi-generator; nested CI never publishes.",
        solution="Version align; generate from OpenAPI where feasible; root workflows.",
        edges="breaking API",
        tests="contract_sdk_major_matches_server_policy",
        related="D-48,X-26",
        diagram="""
  server 0.20.2
  sdks 0.4.0 --> drift
""",
    ),
    dict(
        id="X-34",
        title="Golden set loaded/counted never evaluated",
        prio="P2",
        status="CONFIRMED",
        cluster="07-accuracy-explain",
        sprint=5,
        laws="LAW-8",
        locus="tests/fixtures/spec025_golden_qa.json; skeleton metrics",
        why="Quality gate is count≥50 and tautological heuristic fixtures.",
        root="No live engine scoring in CI.",
        solution="Real eval job (nightly) with Acc/F1 thresholds; keep count as smoke only.",
        edges="cost of LLM judge",
        tests="nightly_golden_acc_gate",
        related="X-35,X-32",
        diagram="""
  load 50 synthetics --> assert len>=50 --> pass
  never score answers
""",
    ),
    dict(
        id="X-35",
        title="Accuracy degrades with corpus size (0.458@40)",
        prio="P1",
        status="PARTIAL",
        cluster="07-accuracy-explain",
        sprint=5,
        laws="LAW-3",
        locus="specs/001-benchmark; SPEC-055",
        why="System quality falls as docs grow — product risk; marketed numbers from @5 docs.",
        root="Multi-factor: D-38 pollution, D-30 collapse, normalization, retrieval fusion, community phase-1.",
        solution="Track Acc@N curve as gate; fix retrieval/identity clusters first; publish honest metrics.",
        edges="domain shift",
        tests="bench_acc_at_n_regression_gate",
        related="D-38,D-30,C-14,D-54,X-34",
        diagram="""
  Acc@5 0.549 --> @10 0.529 --> @40 0.458 (monotone drop)
""",
    ),
    dict(
        id="X-36",
        title="Three divergent configuration systems",
        prio="P2",
        status="CONFIRMED",
        cluster="06-ops-ci-sdk",
        sprint=4,
        laws="LAW-3",
        locus="core/config.rs; EdgeQuakeConfig; Workspace resolution",
        why="gpt model names and entity_types differ by source — unpredictable prod.",
        root="Multiple from_env / builders / workspace overrides without precedence SSOT.",
        solution="One EdgeQuakeConfig::resolve() precedence: explicit arg > env > workspace > defaults; delete dead paths.",
        edges="hot reload",
        tests="contract_config_precedence",
        related="D-50",
        diagram="""
  Config(7 env) vs EdgeQuakeConfig vs Workspace --> 3 truths
""",
    ),
    dict(
        id="X-37",
        title="Three multi-tenant isolation models",
        prio="P1",
        status="CONFIRMED",
        cluster="01-tenant-isolation",
        sprint=1,
        laws="LAW-1,LAW-3",
        locus="vectors per-table; graph properties; relational RLS inert; KV prefix only",
        why="Inconsistent isolation; KV has no real tenant boundary; 8-hex workspace collision assumed.",
        root="Organic evolution without IsolationPolicy SSOT.",
        solution="Document+enforce: RLS real (S-03..S-06); graph props+workspace ids; KV prefix+RLS or workspace partition; lengthen vector table suffix.",
        edges="migration of existing KV keys",
        tests="e2e_kv_cross_tenant_denied; e2e_vector_table_suffix_collision_resistant",
        related="S-03,S-05,S-01",
        diagram="""
  vectors | graph props | RLS(inert) | KV prefixes
     \\         |           |          /
      +---- IsolationPolicy SSOT ----+
""",
    ),
]


def render(d: dict) -> str:
    return f"""# {d['id']} — {d['title']}

> **Priority**: {d['prio']}  
> **Audit status**: {d['status']}  
> **Cluster**: [`{d['cluster']}`](../clusters/{d['cluster']}/)  
> **Sprint**: {d['sprint']}  
> **Laws**: {d['laws']}  
> **Cross-refs**: {d['related'] or '—'}

---

## 1. WHY

{d['why']}

---

## 2. Audit (code is law)

| Field | Value |
|-------|-------|
| Primary locus | `{d['locus']}` |
| Verdict | **{d['status']}** |
| Verified against | HEAD audit 2026-07-23 (v0.20.2 lineage) |

---

## 3. Root cause (first principles)

{d['root']}

---

## 4. ASCII causal diagram

```
{d['diagram'].rstrip()}
```

---

## 5. Solution (SOLID + DRY)

{d['solution']}

| Principle | Application |
|-----------|-------------|
| S | Own the invariant in one module named in the solution |
| D | Depend on SSOT helpers (SchemaReadiness, TenantContext, RetryExecutor, TokenEstimator, …) |
| DRY | No second copy of normalize / retry / GUC / score scale |

---

## 6. Edge cases

{d['edges']}

---

## 7. E2E / contract tests

{d['tests']}

---

## 8. Cross-refs

- Cluster: [`../clusters/{d['cluster']}/`](../clusters/{d['cluster']}/)
- Roadmap sprint {d['sprint']}: [`../03-implementation-roadmap.md`](../03-implementation-roadmap.md)
- Matrix: [`../02-cross-ref-matrix.md`](../02-cross-ref-matrix.md)
- Related: {d['related'] or '—'}
"""


def main() -> None:
    DEFECTS.mkdir(parents=True, exist_ok=True)
    ids = []
    for d in CATALOG:
        path = DEFECTS / f"{d['id']}.md"
        path.write_text(render(d), encoding="utf-8")
        ids.append(d["id"])
    print(f"wrote {len(ids)} defect studies")
    # uniqueness check
    assert len(ids) == len(set(ids)), "duplicate ids"
    (ROOT / "_generated_ids.txt").write_text("\n".join(ids) + "\n", encoding="utf-8")


if __name__ == "__main__":
    main()
