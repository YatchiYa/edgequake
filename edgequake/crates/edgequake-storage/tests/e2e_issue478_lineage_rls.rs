//! Issue #478: document lineage discovery on a large AGE graph.
//!
//! The tenant role used to index-scan every node in the tenant
//! (`idx_*_tenant_id`) because the row-security guard calls `agtype_to_json`.
//! Each test explains one discovery query and fails if that plan is a
//! sequential scan or a tenant-wide index scan, or if it exceeds the 2s
//! discovery statement timeout.
//!
//! Run:
//!   cargo test -p edgequake-storage --features postgres \
//!     --test e2e_issue478_lineage_rls -- --test-threads=1

#![cfg(feature = "postgres")]

#[path = "support/postgres_test_config.rs"]
mod postgres_test_config;

use edgequake_storage::traits::{EdgeListFilter, GraphScanOps, GraphStorage, NodeListFilter};
use edgequake_storage::PostgresAGEGraphStorage;
use postgres_test_config::{contract_pg_pool, require_or_skip_postgres};
use std::sync::OnceLock;
use std::time::Instant;
use uuid::Uuid;

const NOISE_NODES: i64 = 160_000;
const TARGET_NODES: i64 = 20;
const TARGET_EDGES: i64 = 5;
const OTHER_NODES: i64 = 30;
const OTHER_EDGES: i64 = 3;
const DOC: &str = "doc-target";
const OTHER_DOC: &str = "doc-b-only";
/// Same budget as `SOURCE_DISCOVERY_STATEMENT_TIMEOUT_MS`.
const DISCOVERY_BUDGET_MS: f64 = 2_000.0;

struct Fixture {
    pool: sqlx::PgPool,
    graph: String,
    storage: PostgresAGEGraphStorage,
    tenant_a: Uuid,
    workspace_a: Uuid,
    tenant_b: Uuid,
    workspace_b: Uuid,
    nodes: i64,
    edges: i64,
}

/// One runtime for the process. Each `#[tokio::test]` builds its own runtime, and a
/// pool created on a finished runtime cannot be checked out by the next test.
fn runtime() -> &'static tokio::runtime::Runtime {
    static RT: OnceLock<tokio::runtime::Runtime> = OnceLock::new();
    RT.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("issue 478 runtime")
    })
}

fn fixture() -> Option<&'static Fixture> {
    static FIXTURE: OnceLock<Option<Fixture>> = OnceLock::new();
    FIXTURE
        .get_or_init(|| runtime().block_on(build_fixture()))
        .as_ref()
}

/// Scratch DB name. The shared `{dev}_test` database has already dropped Apache AGE.
const AGE_DATABASE: &str = "issue478_lineage";

fn base_database_url() -> Option<String> {
    if let Ok(url) = std::env::var("DATABASE_URL") {
        let url = url.trim().to_string();
        if !url.is_empty() {
            return Some(url);
        }
    }
    std::fs::read_to_string("/tmp/edgequake-db-url")
        .ok()
        .map(|url| url.trim().to_string())
        .filter(|url| !url.is_empty())
}

fn rewrite_database(url: &str, name: &str) -> String {
    let (base, rest) = url.rsplit_once('/').expect("database url");
    match rest.split_once('?') {
        Some((_, query)) => format!("{base}/{name}?{query}"),
        None => format!("{base}/{name}"),
    }
}

fn migration_files() -> Vec<(i64, String, std::path::PathBuf)> {
    let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../migrations");
    let mut files = Vec::new();
    for ent in std::fs::read_dir(&dir).expect("migrations dir") {
        let ent = ent.expect("migration dirent");
        let name = ent.file_name().to_string_lossy().into_owned();
        let Some((num, rest)) = name.split_once('_') else {
            continue;
        };
        if !rest.ends_with(".sql") {
            continue;
        }
        let Ok(version) = num.parse::<i64>() else {
            continue;
        };
        let description = rest.trim_end_matches(".sql").replace('_', " ");
        files.push((version, description, ent.path()));
    }
    files.sort_by_key(|(version, _, _)| *version);
    files
}

fn checksums_by_file() -> std::collections::HashMap<String, String> {
    let lock = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../migrations/checksums.lock"),
    )
    .expect("checksums.lock");
    let mut map = std::collections::HashMap::new();
    for line in lock.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut parts = line.split_whitespace();
        let Some(hash) = parts.next() else { continue };
        let Some(file) = parts.next() else { continue };
        map.insert(file.to_string(), hash.to_string());
    }
    map
}

/// Build a private database that still has Apache AGE. Migration 171 is recorded
/// and not executed, so the shared test database's dropped extension stays put.
async fn provision_age_database() {
    let Some(admin_url) = base_database_url() else {
        return;
    };
    let maint = rewrite_database(&admin_url, "postgres");
    let maint_pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(1)
        .connect(&maint)
        .await
        .expect("maintenance connection");
    let exists: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM pg_database WHERE datname = $1)")
            .bind(AGE_DATABASE)
            .fetch_one(&maint_pool)
            .await
            .expect("database exists");
    if !exists {
        sqlx::query(&format!("CREATE DATABASE {AGE_DATABASE}"))
            .execute(&maint_pool)
            .await
            .expect("create lineage database");
    }
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(1)
        .connect(&rewrite_database(&admin_url, AGE_DATABASE))
        .await
        .expect("lineage database");
    let ready: bool = sqlx::query_scalar(
        "SELECT to_regclass('public._sqlx_migrations') IS NOT NULL
            AND EXISTS (SELECT 1 FROM pg_extension WHERE extname = 'age')
            AND EXISTS (SELECT 1 FROM public._sqlx_migrations WHERE version = 207 AND success)",
    )
    .fetch_one(&pool)
    .await
    .unwrap_or(false);
    if ready {
        pool.close().await;
        maint_pool.close().await;
        return;
    }
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS public._sqlx_migrations (
            version BIGINT PRIMARY KEY,
            description TEXT NOT NULL,
            installed_on TIMESTAMPTZ NOT NULL DEFAULT now(),
            success BOOLEAN NOT NULL,
            checksum BYTEA NOT NULL,
            execution_time BIGINT NOT NULL
        )",
    )
    .execute(&pool)
    .await
    .expect("migration ledger");
    let checksums = checksums_by_file();
    for (version, description, path) in migration_files() {
        let file_name = path.file_name().unwrap().to_string_lossy().into_owned();
        let applied: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM public._sqlx_migrations WHERE version = $1 AND success)",
        )
        .bind(version)
        .fetch_one(&pool)
        .await
        .unwrap_or(false);
        if applied {
            continue;
        }
        // 171 drops AGE. This proof needs the extension, so record the step and skip it.
        if version != 171 {
            let sql = std::fs::read_to_string(&path).expect("read migration");
            sqlx::raw_sql("SET search_path TO public")
                .execute(&pool)
                .await
                .expect("search_path");
            sqlx::raw_sql(&sql)
                .execute(&pool)
                .await
                .unwrap_or_else(|e| {
                    panic!("migration {file_name} failed: {e}");
                });
        }
        let hash = checksums
            .get(&file_name)
            .unwrap_or_else(|| panic!("missing checksum for {file_name}"));
        sqlx::query(
            "INSERT INTO public._sqlx_migrations (version, description, success, checksum, execution_time)
             VALUES ($1, $2, TRUE, decode($3, 'hex'), 0)",
        )
        .bind(version)
        .bind(&description)
        .bind(hash)
        .execute(&pool)
        .await
        .unwrap_or_else(|e| panic!("record migration {file_name}: {e}"));
    }
    pool.close().await;
    maint_pool.close().await;
}

fn assert_safe_ident(name: &str) {
    let ok = name
        .chars()
        .next()
        .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
        && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_');
    assert!(ok, "refusing unsafe graph name {name}");
}

async fn build_fixture() -> Option<Fixture> {
    if base_database_url().is_none() {
        eprintln!("SKIP: no DATABASE_URL / POSTGRES_PASSWORD");
        return None;
    }
    provision_age_database().await;
    std::env::set_var("EDGEQUAKE_TEST_DATABASE", AGE_DATABASE);
    // Migration 170 marks a graph-less database `ready`. A ready ledger ignores
    // this variable, so the scratch database stays on the in-progress ledger.
    // That is not a finished cutover: migration 171 is recorded and not applied.
    std::env::set_var("EDGEQUAKE_GRAPH_READ", "age");
    let cfg = require_or_skip_postgres("issue478")?;
    let graph = cfg.age_graph_name();
    assert_safe_ident(&graph);
    let pool = contract_pg_pool(&cfg).await;
    sqlx::query(
        "UPDATE public.eq_graph_cutover
            SET status = 'age',
                age_absent = false,
                shadow_ok = false,
                backfill_complete = false,
                updated_at = now(),
                detail = jsonb_build_object('reason', 'issue478_scratch_not_ready')
          WHERE id = 1",
    )
    .execute(&pool)
    .await
    .expect(
        "issue478 ledger must stay off ready so EDGEQUAKE_GRAPH_READ=age creates the AGE graph",
    );
    let storage = PostgresAGEGraphStorage::new(cfg);
    storage
        .initialize()
        .await
        .expect("AGE graph, labels, and lineage indexes");

    let tenant_a = Uuid::new_v4();
    let workspace_a = Uuid::new_v4();
    let tenant_b = Uuid::new_v4();
    let workspace_b = Uuid::new_v4();
    seed_scope(&pool, tenant_a, workspace_a, "issue478-a").await;
    seed_scope(&pool, tenant_b, workspace_b, "issue478-b").await;

    let mut conn = pool.acquire().await.expect("insert connection");
    sqlx::query("LOAD 'age'")
        .execute(&mut *conn)
        .await
        .expect("LOAD age");
    sqlx::query("SET statement_timeout = 0")
        .execute(&mut *conn)
        .await
        .expect("disable statement timeout for the load");
    // Mirror and id-sync triggers rewrite every row into the plain graph.
    // The discovery proof only reads the AGE labels, and eq_* is filled by the INSERT.
    for label in ["Node", "EDGE"] {
        sqlx::query(&format!(
            r#"ALTER TABLE {graph}."{label}" DISABLE TRIGGER USER"#
        ))
        .execute(&mut *conn)
        .await
        .unwrap_or_else(|e| panic!("disable {label} triggers: {e}"));
    }

    let started = Instant::now();
    insert_nodes(
        &mut conn,
        &graph,
        tenant_a,
        workspace_a,
        NOISE_NODES,
        "noise-",
        "noise-doc",
        true,
    )
    .await;
    insert_nodes(
        &mut conn,
        &graph,
        tenant_a,
        workspace_a,
        TARGET_NODES,
        "target-",
        DOC,
        false,
    )
    .await;
    insert_nodes(
        &mut conn,
        &graph,
        tenant_b,
        workspace_b,
        OTHER_NODES,
        "b-",
        OTHER_DOC,
        false,
    )
    .await;
    insert_noise_edges(&mut conn, &graph, tenant_a, workspace_a).await;
    insert_target_edges(&mut conn, &graph, tenant_a, workspace_a).await;
    insert_other_edges(&mut conn, &graph, tenant_b, workspace_b).await;
    sqlx::query(&format!("ANALYZE {graph}.\"Node\""))
        .execute(&mut *conn)
        .await
        .expect("analyze nodes");
    sqlx::query(&format!("ANALYZE {graph}.\"EDGE\""))
        .execute(&mut *conn)
        .await
        .expect("analyze edges");
    for label in ["Node", "EDGE"] {
        sqlx::query(&format!(
            r#"ALTER TABLE {graph}."{label}" ENABLE TRIGGER USER"#
        ))
        .execute(&mut *conn)
        .await
        .unwrap_or_else(|e| panic!("enable {label} triggers: {e}"));
    }
    let nodes: i64 = sqlx::query_scalar(&format!("SELECT count(*) FROM {graph}.\"Node\""))
        .fetch_one(&mut *conn)
        .await
        .expect("node count");
    let edges: i64 = sqlx::query_scalar(&format!("SELECT count(*) FROM {graph}.\"EDGE\""))
        .fetch_one(&mut *conn)
        .await
        .expect("edge count");
    eprintln!(
        "ISSUE478 graph={graph} nodes={nodes} edges={edges} load_ms={}",
        started.elapsed().as_millis()
    );
    assert!(
        nodes >= NOISE_NODES + TARGET_NODES,
        "synthetic graph must be large enough to show the tenant scan, got {nodes}"
    );
    assert!(edges >= NOISE_NODES, "synthetic edges got {edges}");

    Some(Fixture {
        pool,
        graph,
        storage,
        tenant_a,
        workspace_a,
        tenant_b,
        workspace_b,
        nodes,
        edges,
    })
}

async fn seed_scope(pool: &sqlx::PgPool, tenant: Uuid, workspace: Uuid, prefix: &str) {
    let suffix = tenant.as_simple().to_string();
    sqlx::query("INSERT INTO tenants (tenant_id, name, slug) VALUES ($1, $2, $3)")
        .bind(tenant)
        .bind(format!("{prefix} tenant"))
        .bind(format!("{prefix}-{suffix}"))
        .execute(pool)
        .await
        .expect("seed tenant");
    sqlx::query(
        "INSERT INTO workspaces (workspace_id, tenant_id, name, slug, is_active) VALUES ($1, $2, $3, $4, TRUE)",
    )
    .bind(workspace)
    .bind(tenant)
    .bind(format!("{prefix} workspace"))
    .bind(format!("{prefix}-ws-{suffix}"))
    .execute(pool)
    .await
    .expect("seed workspace");
}

#[allow(clippy::too_many_arguments)]
async fn insert_nodes(
    conn: &mut sqlx::PgConnection,
    graph: &str,
    tenant: Uuid,
    workspace: Uuid,
    count: i64,
    id_prefix: &str,
    source_doc: &str,
    per_row_source: bool,
) {
    // The column default builds the graphid. AGE has no bigint→graphid cast.
    let sql = format!(
        r#"
        INSERT INTO {graph}."Node" (properties, eq_node_id)
        SELECT
          (json_build_object(
              'node_id', $4::text || gs.i::text,
              'tenant_id', $1::text,
              'workspace_id', $2::text,
              'entity_type', 'ISSUE478',
              'source_ids', CASE
                  WHEN $6::bool THEN json_build_array($5::text || '-' || gs.i::text)
                  ELSE json_build_array($5::text)
              END,
              'source_chunk_ids', CASE
                  WHEN $6::bool THEN json_build_array($5::text || '-' || gs.i::text || '-chunk-0')
                  ELSE json_build_array($5::text || '-chunk-0')
              END
          ))::text::ag_catalog.agtype,
          $4::text || gs.i::text
        FROM generate_series(1, $3) AS gs(i)
        "#
    );
    sqlx::query(&sql)
        .bind(tenant.to_string())
        .bind(workspace.to_string())
        .bind(count)
        .bind(id_prefix)
        .bind(source_doc)
        .bind(per_row_source)
        .execute(&mut *conn)
        .await
        .unwrap_or_else(|e| panic!("insert nodes {id_prefix}: {e}"));
}

async fn insert_noise_edges(
    conn: &mut sqlx::PgConnection,
    graph: &str,
    tenant: Uuid,
    workspace: Uuid,
) {
    let sql = format!(
        r#"
        INSERT INTO {graph}."EDGE"
            (start_id, end_id, properties, eq_source_id, eq_target_id, eq_rel_type)
        SELECT
          src.id,
          tgt.id,
          (json_build_object(
              'tenant_id', $1::text,
              'workspace_id', $2::text,
              'source_id', src.eq_node_id,
              'target_id', tgt.eq_node_id,
              'relation_type', 'RELATED_TO',
              'source_ids', json_build_array('noise-doc-' || gs.i::text),
              'source_chunk_ids', json_build_array('noise-doc-' || gs.i::text || '-chunk-0')
          ))::text::ag_catalog.agtype,
          src.eq_node_id,
          tgt.eq_node_id,
          'RELATED_TO'
        FROM generate_series(1, $3) AS gs(i)
        JOIN {graph}."Node" src ON src.eq_node_id = 'noise-' || gs.i::text
        JOIN {graph}."Node" tgt ON tgt.eq_node_id = 'noise-' || ((gs.i % $3) + 1)::text
        "#
    );
    sqlx::query(&sql)
        .bind(tenant.to_string())
        .bind(workspace.to_string())
        .bind(NOISE_NODES)
        .execute(&mut *conn)
        .await
        .expect("insert noise edges");
}

async fn insert_target_edges(
    conn: &mut sqlx::PgConnection,
    graph: &str,
    tenant: Uuid,
    workspace: Uuid,
) {
    let sql = format!(
        r#"
        INSERT INTO {graph}."EDGE"
            (start_id, end_id, properties, eq_source_id, eq_target_id, eq_rel_type)
        SELECT
          src.id,
          tgt.id,
          (json_build_object(
              'tenant_id', $1::text,
              'workspace_id', $2::text,
              'source_id', src.eq_node_id,
              'target_id', tgt.eq_node_id,
              'relation_type', 'RELATED_TO',
              'source_ids', json_build_array($4::text),
              'source_chunk_ids', json_build_array($4::text || '-chunk-0'),
              'source_chunk_id', $4::text || '-chunk-0',
              'source_document_id', $4::text
          ))::text::ag_catalog.agtype,
          src.eq_node_id,
          tgt.eq_node_id,
          'RELATED_TO'
        FROM generate_series(1, $3) AS gs(i)
        JOIN {graph}."Node" src ON src.eq_node_id = 'target-' || gs.i::text
        JOIN {graph}."Node" tgt ON tgt.eq_node_id = 'target-' || $5::text
        "#
    );
    sqlx::query(&sql)
        .bind(tenant.to_string())
        .bind(workspace.to_string())
        .bind(TARGET_EDGES)
        .bind(DOC)
        .bind(TARGET_NODES.to_string())
        .execute(&mut *conn)
        .await
        .expect("insert target edges");
}

async fn insert_other_edges(
    conn: &mut sqlx::PgConnection,
    graph: &str,
    tenant: Uuid,
    workspace: Uuid,
) {
    let sql = format!(
        r#"
        INSERT INTO {graph}."EDGE"
            (start_id, end_id, properties, eq_source_id, eq_target_id, eq_rel_type)
        SELECT
          src.id,
          tgt.id,
          (json_build_object(
              'tenant_id', $1::text,
              'workspace_id', $2::text,
              'source_id', src.eq_node_id,
              'target_id', tgt.eq_node_id,
              'relation_type', 'RELATED_TO',
              'source_ids', json_build_array($4::text),
              'source_chunk_ids', json_build_array($4::text || '-chunk-0'),
              'source_chunk_id', $4::text || '-chunk-0',
              'source_document_id', $4::text
          ))::text::ag_catalog.agtype,
          src.eq_node_id,
          tgt.eq_node_id,
          'RELATED_TO'
        FROM generate_series(1, $3) AS gs(i)
        JOIN {graph}."Node" src ON src.eq_node_id = 'b-1'
        JOIN {graph}."Node" tgt ON tgt.eq_node_id = 'b-' || (gs.i + 1)::text
        "#
    );
    sqlx::query(&sql)
        .bind(tenant.to_string())
        .bind(workspace.to_string())
        .bind(OTHER_EDGES)
        .bind(OTHER_DOC)
        .execute(&mut *conn)
        .await
        .expect("insert other-tenant edges");
}

fn probes(doc: &str) -> (Vec<String>, Vec<String>) {
    (vec![doc.to_string()], vec![format!("{doc}-chunk-")])
}

async fn open_tenant<'a>(
    pool: &'a sqlx::PgPool,
    tenant: Option<Uuid>,
    workspace: Option<Uuid>,
) -> sqlx::Transaction<'a, sqlx::Postgres> {
    let mut tx = pool.begin().await.expect("begin");
    sqlx::query("SELECT set_config('statement_timeout', '30000', true)")
        .execute(&mut *tx)
        .await
        .expect("statement timeout");
    let tenant_text = tenant.map(|id| id.to_string()).unwrap_or_default();
    let workspace_text = workspace.map(|id| id.to_string()).unwrap_or_default();
    sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
        .bind(&tenant_text)
        .execute(&mut *tx)
        .await
        .expect("tenant guc");
    sqlx::query("SELECT set_config('app.current_workspace_id', $1, true)")
        .bind(&workspace_text)
        .execute(&mut *tx)
        .await
        .expect("workspace guc");
    sqlx::query("SET LOCAL ROLE edgequake_tenant_access")
        .execute(&mut *tx)
        .await
        .expect("SET ROLE edgequake_tenant_access");
    tx
}

async fn explain_query(
    pool: &sqlx::PgPool,
    graph: &str,
    kind: &str,
    tenant: Option<Uuid>,
    workspace: Option<Uuid>,
    doc: &str,
) -> String {
    let mut tx = open_tenant(pool, tenant, workspace).await;
    let (exact, prefixes) = probes(doc);
    let plan: String =
        sqlx::query_scalar("SELECT public.eq_age_lineage_explain($1, $2, $3, $4, $5)")
            .bind(kind)
            .bind(graph)
            .bind(&exact)
            .bind(&prefixes)
            .bind(256_i32)
            .fetch_one(&mut *tx)
            .await
            .unwrap_or_else(|e| panic!("explain {kind}: {e}"));
    tx.rollback().await.expect("rollback explain");
    plan
}

async fn count_query(
    pool: &sqlx::PgPool,
    graph: &str,
    kind: &str,
    tenant: Option<Uuid>,
    workspace: Option<Uuid>,
    doc: &str,
) -> i64 {
    let mut tx = open_tenant(pool, tenant, workspace).await;
    let (exact, prefixes) = probes(doc);
    let sql = match kind {
        "nodes" => "SELECT count(*) FROM public.eq_age_lineage_discover_nodes($1, $2, $3, $4)",
        "edges" => "SELECT count(*) FROM public.eq_age_lineage_discover_edges($1, $2, $3, $4)",
        "edges_singular" => {
            "SELECT count(*) FROM public.eq_age_lineage_discover_edges_singular($1, $2, $3, $4)"
        }
        other => panic!("unknown kind {other}"),
    };
    let count: i64 = sqlx::query_scalar(sql)
        .bind(graph)
        .bind(&exact)
        .bind(&prefixes)
        .bind(256_i32)
        .fetch_one(&mut *tx)
        .await
        .unwrap_or_else(|e| panic!("count {kind}: {e}"));
    tx.rollback().await.expect("rollback count");
    count
}

fn execution_ms(plan: &str) -> f64 {
    plan.lines()
        .filter_map(|line| line.trim().strip_prefix("Execution Time:"))
        .filter_map(|rest| rest.trim().strip_suffix("ms"))
        .filter_map(|number| number.trim().parse::<f64>().ok())
        .next_back()
        .unwrap_or_else(|| panic!("plan has no execution time:\n{plan}"))
}

/// Fail when the plan is the pre-fix tenant scan (or any sequential scan).
fn assert_indexed_plan(plan: &str, indexes: &[&str], forbidden: &[&str]) -> f64 {
    assert!(
        !plan.contains("Seq Scan"),
        "lineage plan sequential-scanned:\n{plan}"
    );
    for index in forbidden {
        assert!(
            !plan.contains(index),
            "lineage plan scanned the whole tenant via {index}:\n{plan}"
        );
    }
    for index in indexes {
        assert!(
            plan.contains(index),
            "lineage plan did not use {index}:\n{plan}"
        );
    }
    let ms = execution_ms(plan);
    assert!(
        ms < DISCOVERY_BUDGET_MS,
        "execution {ms} ms exceeds the {DISCOVERY_BUDGET_MS} ms discovery timeout:\n{plan}"
    );
    ms
}

fn write_plan(kind: &str, plan: &str, rows: i64, other_rows: i64, ms: f64) {
    let dir = "/tmp/issue478";
    let _ = std::fs::create_dir_all(dir);
    let body = format!(
        "kind={kind}\nrows={rows}\nother_tenant_rows={other_rows}\nexecution_ms={ms}\n\n{plan}\n"
    );
    std::fs::write(format!("{dir}/after-{kind}.txt"), body).expect("write plan");
    eprintln!("ISSUE478 {kind} rows={rows} other_tenant_rows={other_rows} execution_ms={ms}");
}

#[test]
fn issue478_node_discovery_uses_lineage_gin() {
    let Some(fx) = fixture() else {
        return;
    };
    runtime().block_on(async {
        let plan = explain_query(
            &fx.pool,
            &fx.graph,
            "nodes",
            Some(fx.tenant_a),
            Some(fx.workspace_a),
            DOC,
        )
        .await;
        let ms = assert_indexed_plan(
            &plan,
            &["idx_node_source_ids_gin", "idx_node_source_chunk_ids_gin"],
            &["idx_node_tenant_id"],
        );
        let rows = count_query(
            &fx.pool,
            &fx.graph,
            "nodes",
            Some(fx.tenant_a),
            Some(fx.workspace_a),
            DOC,
        )
        .await;
        let other = count_query(
            &fx.pool,
            &fx.graph,
            "nodes",
            Some(fx.tenant_b),
            Some(fx.workspace_b),
            DOC,
        )
        .await;
        let blocked = count_query(&fx.pool, &fx.graph, "nodes", None, None, DOC).await;
        assert_eq!(rows, TARGET_NODES, "tenant A node citations");
        assert_eq!(other, 0, "tenant B must not see tenant A nodes");
        assert_eq!(blocked, 0, "tenant role with no tenant must see nothing");
        let filter_a = NodeListFilter {
            tenant_id: Some(fx.tenant_a.to_string()),
            workspace_id: Some(fx.workspace_a.to_string()),
            ..NodeListFilter::default()
        };
        let filter_b = NodeListFilter {
            tenant_id: Some(fx.tenant_b.to_string()),
            workspace_id: Some(fx.workspace_b.to_string()),
            ..NodeListFilter::default()
        };
        let found_a = fx
            .storage
            .find_nodes_by_source_prefixes(&filter_a, &[DOC.to_string()])
            .await
            .expect("storage node discovery stays inside the discovery timeout");
        let found_b = fx
            .storage
            .find_nodes_by_source_prefixes(&filter_b, &[DOC.to_string()])
            .await
            .expect("storage node discovery for the other tenant");
        assert_eq!(found_a.len() as i64, TARGET_NODES);
        assert!(found_b.is_empty(), "storage leaked tenant A nodes");
        write_plan("nodes", &plan, rows, other, ms);
        eprintln!(
            "ISSUE478 nodes graph_nodes={} graph_edges={}",
            fx.nodes, fx.edges
        );
    });
}

#[test]
fn issue478_edge_discovery_uses_lineage_gin() {
    let Some(fx) = fixture() else {
        return;
    };
    runtime().block_on(async {
        let plan = explain_query(
            &fx.pool,
            &fx.graph,
            "edges",
            Some(fx.tenant_a),
            Some(fx.workspace_a),
            DOC,
        )
        .await;
        let ms = assert_indexed_plan(
            &plan,
            &["idx_edge_source_ids_gin", "idx_edge_source_chunk_ids_gin"],
            &["idx_edge_tenant_id"],
        );
        let rows = count_query(
            &fx.pool,
            &fx.graph,
            "edges",
            Some(fx.tenant_a),
            Some(fx.workspace_a),
            DOC,
        )
        .await;
        let other = count_query(
            &fx.pool,
            &fx.graph,
            "edges",
            Some(fx.tenant_b),
            Some(fx.workspace_b),
            DOC,
        )
        .await;
        assert_eq!(rows, TARGET_EDGES, "tenant A edge citations");
        assert_eq!(other, 0, "tenant B must not see tenant A edges");
        let filter_a = EdgeListFilter {
            tenant_id: Some(fx.tenant_a.to_string()),
            workspace_id: Some(fx.workspace_a.to_string()),
            relationship_type: None,
        };
        let found_a = fx
            .storage
            .find_edges_by_source_prefixes(&filter_a, &[DOC.to_string()])
            .await
            .expect("storage edge discovery stays inside the discovery timeout");
        assert_eq!(found_a.len() as i64, TARGET_EDGES);
        write_plan("edges", &plan, rows, other, ms);
    });
}

#[test]
fn issue478_singular_edge_discovery_uses_citation_btree() {
    let Some(fx) = fixture() else {
        return;
    };
    runtime().block_on(async {
        let plan = explain_query(
            &fx.pool,
            &fx.graph,
            "edges_singular",
            Some(fx.tenant_a),
            Some(fx.workspace_a),
            DOC,
        )
        .await;
        let ms = assert_indexed_plan(
            &plan,
            &["idx_edge_source_chunk_id", "idx_edge_source_document_id"],
            &["idx_edge_tenant_id"],
        );
        let rows = count_query(
            &fx.pool,
            &fx.graph,
            "edges_singular",
            Some(fx.tenant_a),
            Some(fx.workspace_a),
            DOC,
        )
        .await;
        let other = count_query(
            &fx.pool,
            &fx.graph,
            "edges_singular",
            Some(fx.tenant_b),
            Some(fx.workspace_b),
            DOC,
        )
        .await;
        // UNION ALL returns the same edge from source_chunk_id and source_document_id.
        assert_eq!(rows, TARGET_EDGES * 2, "tenant A singular citations");
        assert_eq!(
            other, 0,
            "tenant B must not see tenant A singular citations"
        );
        write_plan("edges_singular", &plan, rows, other, ms);
    });
}
