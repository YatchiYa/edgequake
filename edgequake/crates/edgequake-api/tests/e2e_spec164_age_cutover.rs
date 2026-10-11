//! SPEC-164 gates that need Apache AGE: full migration train, mirror trigger,
//! shadow equality, ledger flip back to AGE, then confirm-drop (171).
//!
//! Greenfield without the extension is `EDGEQUAKE_SPEC164_NO_AGE=1` on a
//! server where `CREATE EXTENSION age` fails.

use std::time::Instant;

use edgequake_api::state::migration_bootstrap::{
    run_postgres_expandable_migrations, run_postgres_migrations,
};
use edgequake_storage::{
    GraphStorage, GraphStorageAnalyticsOps, GraphStorageMutateOps, GraphStorageReadOps,
    PostgresAGEGraphStorage, PostgresConfig,
};
use serde_json::{json, Value};
use sqlx::postgres::PgPoolOptions;
use uuid::Uuid;

#[tokio::test]
async fn age_cutover_proves_mirror_shadow_ledger_flip_and_confirm_drop() {
    let Some(admin) = admin_url() else {
        eprintln!("SKIP spec164 age cutover: set DATABASE_URL");
        return;
    };
    if std::env::var("EDGEQUAKE_SPEC164_NO_AGE").is_ok() {
        eprintln!("SKIP age cutover: EDGEQUAKE_SPEC164_NO_AGE is set");
        return;
    }
    let db = format!("spec164age_{}", &Uuid::new_v4().simple().to_string()[..12]);
    let url = provision(&admin, &db).await;
    let pool = PgPoolOptions::new()
        .max_connections(2)
        .connect(&url)
        .await
        .expect("db connect");

    std::env::set_var("EDGEQUAKE_MIGRATE_CLI", "1");
    run_postgres_expandable_migrations(&pool)
        .await
        .expect("expandable train through 170, 171 still pending");
    let applied_max: i64 =
        sqlx::query_scalar("SELECT COALESCE(MAX(version), 0) FROM _sqlx_migrations")
            .fetch_one(&pool)
            .await
            .expect("applied max");
    assert!(applied_max >= 170, "170 must apply, got {applied_max}");
    assert_ne!(
        applied_max, 171,
        "expandable migrate must not apply confirm-drop 171"
    );
    let age_on: bool =
        sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM pg_extension WHERE extname = 'age')")
            .fetch_one(&pool)
            .await
            .expect("age probe");
    assert!(age_on, "AGE must still be installed before confirm-drop");

    let (host, port, user, password) = parse_url(&url);
    let tenant = Uuid::new_v4();
    let workspace = Uuid::new_v4();
    sqlx::query("INSERT INTO public.tenants (tenant_id, name, slug) VALUES ($1, 'spec164', $2)")
        .bind(tenant)
        .bind(format!("t{}", &tenant.simple().to_string()[..12]))
        .execute(&pool)
        .await
        .expect("tenant");
    sqlx::query(
        "INSERT INTO public.workspaces (workspace_id, tenant_id, name, slug) VALUES ($1, $2, 'spec164', 'ws')",
    )
    .bind(workspace)
    .bind(tenant)
    .execute(&pool)
    .await
    .expect("workspace");
    let config = PostgresConfig::new(host, port, &db, user, password)
        .with_namespace("spec164")
        .with_max_connections(4);
    let storage = PostgresAGEGraphStorage::new(config);
    storage
        .initialize()
        .await
        .expect("initialize graph on schema 170");
    let graph = storage.graph_name().to_string();

    let mirror_props = format!(
        r#"{{"node_id":"MIRROR_ONLY","tenant_id":"{tenant}","workspace_id":"{workspace}","entity_type":"Person","description":"from-trigger"}}"#
    );
    sqlx::query(&format!(
        r#"INSERT INTO {graph}."Node" (id, properties, eq_node_id)
           VALUES (public.eq_next_node_id($1), $2::ag_catalog.agtype, 'MIRROR_ONLY')"#
    ))
    .bind(&graph)
    .bind(&mirror_props)
    .execute(&pool)
    .await
    .expect("direct AGE insert (v0.34-shaped writer)");
    let mirrored: Option<String> = sqlx::query_scalar(
        "SELECT properties->>'description' FROM public.eq_graph_nodes
         WHERE namespace = 'spec164' AND node_id = 'MIRROR_ONLY'",
    )
    .fetch_optional(&pool)
    .await
    .expect("mirror read")
    .flatten();
    assert_eq!(
        mirrored.as_deref(),
        Some("from-trigger"),
        "mirror trigger must copy the AGE row into eq_graph_nodes"
    );

    storage
        .upsert_nodes_batch(&[
            node("ALICE", &tenant, &workspace, "alice"),
            node("BOB", &tenant, &workspace, "bob"),
            node("CAROL", &tenant, &workspace, "carol"),
        ])
        .await
        .expect("upsert");
    storage
        .upsert_edges_batch(&[
            edge("ALICE", "BOB", "KNOWS", &tenant, &workspace),
            edge("BOB", "CAROL", "KNOWS", &tenant, &workspace),
        ])
        .await
        .expect("edges");

    let report = storage.run_spec164_cutover().await.expect("cutover");
    assert_eq!(
        report.mismatches, 0,
        "shadow neighbor sets must match: {report:?}"
    );
    assert!(report.ready, "ledger must be ready when shadow is clean");
    let shadow_ok: bool =
        sqlx::query_scalar("SELECT shadow_ok FROM public.eq_graph_cutover WHERE id = 1")
            .fetch_one(&pool)
            .await
            .expect("shadow flag");
    assert!(shadow_ok);

    let tid = tenant.to_string();
    let wid = workspace.to_string();
    let mut plain_ms = serde_json::Map::new();
    measure_queries(&storage, &mut plain_ms, &tid, &wid).await;
    let plain_alice = description(&storage, "ALICE").await;

    sqlx::query(&format!(
        r#"ALTER TABLE {graph}."Node" DISABLE TRIGGER eq_graph_mirror_node"#
    ))
    .execute(&pool)
    .await
    .expect("disable mirror");
    sqlx::query(
        "UPDATE public.eq_graph_nodes
         SET properties = jsonb_set(properties, '{description}', '\"PLAIN_MARKER\"')
         WHERE namespace = 'spec164' AND node_id = 'ALICE'",
    )
    .execute(&pool)
    .await
    .expect("mark plain");
    sqlx::query(&format!(
        r#"UPDATE {graph}."Node"
           SET properties = (jsonb_set(ag_catalog.agtype_to_json(properties)::jsonb, '{{description}}', '"AGE_MARKER"'))::text::ag_catalog.agtype
           WHERE eq_node_id = 'ALICE'"#
    ))
    .execute(&pool)
    .await
    .expect("mark age");

    sqlx::query("UPDATE public.eq_graph_cutover SET status = 'age' WHERE id = 1")
        .execute(&pool)
        .await
        .expect("flip ledger to age");
    let mut age_ms = serde_json::Map::new();
    measure_queries(&storage, &mut age_ms, &tid, &wid).await;
    assert_eq!(
        description(&storage, "ALICE").await.as_deref(),
        Some("AGE_MARKER"),
        "status=age must read Apache AGE, not the plain copy"
    );
    let neighbors = storage
        .get_neighbors("ALICE", 2, Some(&tid), Some(&wid))
        .await
        .expect("age neighbors");
    let ids: Vec<_> = neighbors.iter().map(|n| n.id.as_str()).collect();
    assert!(ids.contains(&"BOB") && ids.contains(&"CAROL"), "{ids:?}");

    sqlx::query("UPDATE public.eq_graph_cutover SET status = 'ready' WHERE id = 1")
        .execute(&pool)
        .await
        .expect("flip ledger to ready");
    assert_eq!(
        description(&storage, "ALICE").await.as_deref(),
        Some("PLAIN_MARKER"),
        "status=ready must read the plain table"
    );
    assert_ne!(plain_alice.as_deref(), Some("AGE_MARKER"));

    sqlx::query(&format!(
        r#"ALTER TABLE {graph}."Node" ENABLE TRIGGER eq_graph_mirror_node"#
    ))
    .execute(&pool)
    .await
    .expect("enable mirror");

    run_postgres_migrations(&pool)
        .await
        .expect("confirm-drop train applies 171");
    let age_after: bool =
        sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM pg_extension WHERE extname = 'age')")
            .fetch_one(&pool)
            .await
            .expect("age after 171");
    assert!(!age_after, "171 must drop Apache AGE");
    assert!(storage.has_node("ALICE").await.expect("plain still serves"));

    let config2 = PostgresConfig::new(
        parse_url(&url).0,
        parse_url(&url).1,
        &db,
        parse_url(&url).2,
        parse_url(&url).3,
    )
    .with_namespace("spec164")
    .with_max_connections(2);
    let booted = PostgresAGEGraphStorage::new(config2);
    booted
        .initialize()
        .await
        .expect("boot after 171 must not require CREATE EXTENSION age");
    let age_recreated: bool =
        sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM pg_extension WHERE extname = 'age')")
            .fetch_one(&pool)
            .await
            .expect("age recreate probe");
    assert!(
        !age_recreated,
        "boot must not recreate AGE after the ledger marks it absent"
    );
    assert!(booted.has_node("ALICE").await.expect("post-171 read"));

    let body = serde_json::json!({
        "plain_ready_ms": plain_ms,
        "age_ledger_ms": age_ms,
    });
    std::fs::write(
        "/tmp/spec164-age-measurements.json",
        serde_json::to_string_pretty(&body).unwrap(),
    )
    .expect("write measurements");
    eprintln!("{}", serde_json::to_string_pretty(&body).unwrap());

    pool.close().await;
    drop_db(&admin, &db).await;
}

#[tokio::test]
async fn full_train_through_171_on_fresh_database() {
    let Some(admin) = admin_url() else {
        eprintln!("SKIP spec164 full train: set DATABASE_URL");
        return;
    };
    let no_age = std::env::var("EDGEQUAKE_SPEC164_NO_AGE").is_ok();
    let db = format!(
        "spec164train_{}",
        &Uuid::new_v4().simple().to_string()[..12]
    );
    let url = provision(&admin, &db).await;
    let pool = PgPoolOptions::new()
        .max_connections(2)
        .connect(&url)
        .await
        .expect("db connect");
    std::env::set_var("EDGEQUAKE_MIGRATE_CLI", "1");
    run_postgres_migrations(&pool)
        .await
        .unwrap_or_else(|e| panic!("full train through 171 failed (no_age={no_age}): {e}"));
    let max_version: i64 =
        sqlx::query_scalar("SELECT COALESCE(MAX(version), 0) FROM _sqlx_migrations")
            .fetch_one(&pool)
            .await
            .expect("max");
    assert_eq!(max_version, 171, "train must include confirm-drop 171");
    let age_on: bool =
        sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM pg_extension WHERE extname = 'age')")
            .fetch_one(&pool)
            .await
            .expect("age");
    assert!(!age_on, "171 leaves no age extension (no_age={no_age})");
    let ready: String =
        sqlx::query_scalar("SELECT status FROM public.eq_graph_cutover WHERE id = 1")
            .fetch_one(&pool)
            .await
            .expect("ledger");
    assert_eq!(ready, "ready");

    let (host, port, user, password) = parse_url(&url);
    let storage = PostgresAGEGraphStorage::new(
        PostgresConfig::new(host, port, &db, user, password)
            .with_namespace("train164")
            .with_max_connections(2),
    );
    storage.initialize().await.expect("plain boot");
    let tenant = Uuid::new_v4();
    let workspace = Uuid::new_v4();
    storage
        .upsert_nodes_batch(&[node("N1", &tenant, &workspace, "one")])
        .await
        .expect("plain write after full train");
    assert!(storage.has_node("N1").await.expect("read"));
    let age_after_boot: bool =
        sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM pg_extension WHERE extname = 'age')")
            .fetch_one(&pool)
            .await
            .expect("age after boot");
    assert!(!age_after_boot);

    pool.close().await;
    drop_db(&admin, &db).await;
}

async fn measure_queries(
    storage: &PostgresAGEGraphStorage,
    out: &mut serde_json::Map<String, Value>,
    tid: &str,
    wid: &str,
) {
    time(out, "get_node", storage.get_node("ALICE")).await;
    time(
        out,
        "get_neighbors",
        storage.get_neighbors("ALICE", 2, Some(tid), Some(wid)),
    )
    .await;
    time(
        out,
        "get_edges_for_node_set",
        storage.get_edges_for_node_set(
            &["ALICE".into(), "BOB".into(), "CAROL".into()],
            Some(tid),
            Some(wid),
        ),
    )
    .await;
    time(
        out,
        "search_nodes",
        storage.search_nodes("alice", 10, None, Some(tid), Some(wid)),
    )
    .await;
    time(out, "node_count", storage.node_count()).await;
}

async fn time<T>(
    out: &mut serde_json::Map<String, Value>,
    name: &str,
    fut: impl std::future::Future<Output = T>,
) {
    let start = Instant::now();
    let _ = fut.await;
    out.insert(name.into(), json!(start.elapsed().as_secs_f64() * 1000.0));
}

async fn description(storage: &PostgresAGEGraphStorage, id: &str) -> Option<String> {
    storage
        .get_node(id)
        .await
        .expect("get")
        .and_then(|n| n.properties.get("description").cloned())
        .and_then(|v| v.as_str().map(str::to_string))
}

fn node(
    id: &str,
    tenant: &Uuid,
    workspace: &Uuid,
    description: &str,
) -> (String, std::collections::HashMap<String, Value>) {
    let mut properties = std::collections::HashMap::new();
    properties.insert("tenant_id".into(), json!(tenant.to_string()));
    properties.insert("workspace_id".into(), json!(workspace.to_string()));
    properties.insert("entity_type".into(), json!("Person"));
    properties.insert("description".into(), json!(description));
    properties.insert("source_ids".into(), json!(["doc-chunk-0"]));
    (id.to_string(), properties)
}

fn edge(
    source: &str,
    target: &str,
    rel: &str,
    tenant: &Uuid,
    workspace: &Uuid,
) -> (String, String, std::collections::HashMap<String, Value>) {
    let mut properties = std::collections::HashMap::new();
    properties.insert("tenant_id".into(), json!(tenant.to_string()));
    properties.insert("workspace_id".into(), json!(workspace.to_string()));
    properties.insert("relation_type".into(), json!(rel));
    (source.into(), target.into(), properties)
}

async fn provision(admin: &str, db: &str) -> String {
    let admin_pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(admin)
        .await
        .expect("admin");
    sqlx::query(&format!("CREATE DATABASE {db}"))
        .execute(&admin_pool)
        .await
        .expect("create database");
    admin_pool.close().await;
    replace_db(admin, db)
}

async fn drop_db(admin: &str, db: &str) {
    let admin_pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(admin)
        .await
        .expect("admin drop");
    sqlx::query(&format!(
        "SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE datname = '{db}' AND pid <> pg_backend_pid()"
    ))
    .execute(&admin_pool)
    .await
    .ok();
    sqlx::query(&format!("DROP DATABASE IF EXISTS {db}"))
        .execute(&admin_pool)
        .await
        .ok();
}

fn admin_url() -> Option<String> {
    let url = std::env::var("DATABASE_URL")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .or_else(|| std::fs::read_to_string("/tmp/edgequake-db-url").ok())?;
    let url = url.trim().to_string();
    if url.is_empty() {
        None
    } else {
        Some(url)
    }
}

fn replace_db(url: &str, db: &str) -> String {
    let q = url.find('?').map(|i| &url[i..]).unwrap_or("");
    let without_q = url.trim_end_matches(q);
    let slash = without_q.rfind('/').expect("db slash");
    format!("{}{}{}", &without_q[..slash + 1], db, q)
}

fn parse_url(url: &str) -> (String, u16, String, String) {
    let without_scheme = url.split("://").nth(1).expect("scheme");
    let (auth, host_path) = without_scheme.split_once('@').expect("auth");
    let (user, password) = auth.split_once(':').expect("user");
    let (host_port, _db) = host_path.split_once('/').expect("host");
    let (host, port) = match host_port.split_once(':') {
        Some((h, p)) => (h.to_string(), p.parse().unwrap_or(5432)),
        None => (host_port.to_string(), 5432),
    };
    (host, port, user.to_string(), password.to_string())
}
