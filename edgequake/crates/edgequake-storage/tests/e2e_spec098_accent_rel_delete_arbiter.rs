//! SPEC-098 LAW-098-13: delete by trigger `eq_rel_type` formula when
//! `properties.relation_type` bytes differ from stored `eq_rel_type`
//! (French accent drift). Postgres `UPPER` is the equality SSOT — no Rust case
//! heuristics, no flaky "close enough" matching.
#![cfg(feature = "postgres")]

#[path = "support/postgres_test_config.rs"]
mod postgres_test_config;

use std::collections::HashMap;
use std::sync::Arc;

use edgequake_storage::traits::{GraphStorage, GraphStorageMutateOps};
use edgequake_storage::PostgresAGEGraphStorage;
use postgres_test_config::require_or_skip_postgres;
use serde_json::json;
use sqlx::Row;

const PROP_REL: &str = "REPRéSENTE"; // mixed accent — science_one prop shape

#[tokio::test]
async fn e2e_spec098_accent_rel_delete_survives_prop_eq_drift() {
    let Some(config) = require_or_skip_postgres("e2e098_accent_del") else {
        return;
    };
    let prev = std::env::var("EDGEQUAKE_NATIVE_GRAPH_WRITES").ok();
    std::env::set_var("EDGEQUAKE_NATIVE_GRAPH_WRITES", "1");

    let graph = Arc::new(PostgresAGEGraphStorage::new(config.clone()));
    graph.initialize().await.expect("graph init");
    let g = graph.graph_name().to_string();
    let pool = postgres_test_config::contract_pg_pool(&config).await;

    // SSOT probe: same UPPER the trigger uses must map prop → stored key.
    let upper_prop: String = sqlx::query_scalar("SELECT UPPER($1::text)")
        .bind(PROP_REL)
        .fetch_one(&pool)
        .await
        .expect("UPPER probe");
    assert_ne!(
        upper_prop.as_str(),
        PROP_REL,
        "precondition: DB UPPER must change accent case (ctype/locale); got {upper_prop:?}"
    );

    let src = "ACCENT_DEL_SRC".to_string();
    let tgt = "ACCENT_DEL_TGT".to_string();
    for id in [&src, &tgt] {
        graph
            .upsert_node(
                id,
                HashMap::from([("entity_type".into(), json!("CONCEPT"))]),
            )
            .await
            .expect("node");
    }

    let mut props = HashMap::new();
    props.insert("relation_type".into(), json!(PROP_REL));
    props.insert("description".into(), json!("accent arbiter"));
    graph
        .upsert_edge(&src, &tgt, props)
        .await
        .expect("upsert edge");

    // Trigger prefers existing eq_rel_type on UPDATE OF properties → column stays
    // UPPER while property bytes become mixed-case accent (science_one shape).
    sqlx::query(&format!(
        r#"
        UPDATE {g}."EDGE" e
        SET properties = (
          jsonb_set(
            ag_catalog.agtype_to_json(e.properties)::jsonb,
            '{{relation_type}}',
            to_jsonb($3::text),
            true
          )
        )::text::ag_catalog.agtype
        WHERE e.eq_source_id = $1 AND e.eq_target_id = $2
        "#
    ))
    .bind(&src)
    .bind(&tgt)
    .bind(PROP_REL)
    .execute(&pool)
    .await
    .expect("force drift");

    let row = sqlx::query(&format!(
        r#"
        SELECT eq_rel_type,
               ag_catalog.agtype_to_json(properties)::jsonb->>'relation_type' AS prop_rel,
               encode(convert_to(eq_rel_type, 'UTF8'), 'hex') AS eq_hex,
               encode(convert_to(
                 ag_catalog.agtype_to_json(properties)::jsonb->>'relation_type', 'UTF8'
               ), 'hex') AS prop_hex
        FROM {g}."EDGE"
        WHERE eq_source_id = $1 AND eq_target_id = $2
        "#
    ))
    .bind(&src)
    .bind(&tgt)
    .fetch_one(&pool)
    .await
    .expect("read drift");

    let eq_rel: String = row.get("eq_rel_type");
    let prop_rel: String = row.get("prop_rel");
    let eq_hex: String = row.get("eq_hex");
    let prop_hex: String = row.get("prop_hex");
    assert_ne!(
        eq_hex, prop_hex,
        "precondition: byte-level drift eq={eq_rel:?} prop={prop_rel:?}"
    );
    assert_eq!(
        eq_rel, upper_prop,
        "eq_rel_type must equal DB UPPER(prop) — trigger SSOT"
    );
    assert_eq!(prop_rel, PROP_REL);

    // Delete with the *raw property* label — SQL UPPER is SSOT (not Rust upper).
    graph
        .delete_edges_batch(&[(src.clone(), tgt.clone(), prop_rel.clone())])
        .await
        .expect("delete with drifted prop rel");

    let left: i64 = sqlx::query_scalar(&format!(
        r#"SELECT COUNT(*)::bigint FROM {g}."EDGE"
           WHERE eq_source_id = $1 AND eq_target_id = $2"#
    ))
    .bind(&src)
    .bind(&tgt)
    .fetch_one(&pool)
    .await
    .expect("count");
    assert_eq!(left, 0, "accent-drifted exclusive edge must be deleted");

    let _ = graph.clear().await;
    match prev {
        Some(v) => std::env::set_var("EDGEQUAKE_NATIVE_GRAPH_WRITES", v),
        None => std::env::remove_var("EDGEQUAKE_NATIVE_GRAPH_WRITES"),
    }
}
