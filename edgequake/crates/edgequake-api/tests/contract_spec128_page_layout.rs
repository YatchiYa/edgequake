//! SPEC-128 page layout persist + HTTP + bbox_norm (always-on, memory storage).

use axum::body::Body;
use axum::http::{Request, StatusCode};
use edgequake_api::{AppState, Server, ServerConfig};
use edgequake_storage::{
    bbox_norm_from_pdf, bbox_norm_iou, DocumentPageLayoutStorage, LayoutBBoxNorm, LayoutBBoxPdf,
    MemoryPageLayoutStorage, ReplaceDocumentPagesRequest, UpsertDocumentPage,
    UpsertPageLayoutRegion,
};
use serde_json::{json, Value};
use tower::ServiceExt;
use uuid::Uuid;

const TENANT: &str = "aaaaaaaa-0019-0019-0019-aaaaaaaaaaaa";
const USER: &str = "bbbbbbbb-0019-0019-0019-bbbbbbbbbbbb";
const WORKSPACE: &str = "cccccccc-0019-0019-0019-cccccccccccc";

fn golden_bbox() -> LayoutBBoxPdf {
    LayoutBBoxPdf {
        x0: 61.2,
        y0: 396.0,
        x1: 306.0,
        y1: 594.0,
    }
}

fn golden_norm() -> LayoutBBoxNorm {
    LayoutBBoxNorm {
        x: 0.1,
        y: 0.25,
        w: 0.4,
        h: 0.25,
    }
}

fn sample_pages(page_number: i32) -> Vec<UpsertDocumentPage> {
    vec![UpsertDocumentPage {
        page_number,
        width_pt: 612.0,
        height_pt: 792.0,
        rotation: 0,
        cropbox_pdf: None,
        raster_width_px: Some(1545),
        raster_height_px: Some(2000),
        layout_model: Some("l0-l1".into()),
        layout_status: "extracted".into(),
        regions: vec![
            UpsertPageLayoutRegion {
                class: "figure".into(),
                source: "l1_paint".into(),
                bbox_pdf: golden_bbox(),
                confidence: Some(0.9),
                reading_order: Some(1),
                asset_path: Some("assets/page-0003-fig-01.png".into()),
                extra: serde_json::json!({"figure_kind": "diagram"}),
            },
            UpsertPageLayoutRegion {
                class: "abandon".into(),
                source: "l3_filter".into(),
                bbox_pdf: LayoutBBoxPdf {
                    x0: 10.0,
                    y0: 700.0,
                    x1: 80.0,
                    y1: 770.0,
                },
                confidence: Some(0.8),
                reading_order: Some(0),
                asset_path: Some("assets/logo.png".into()),
                extra: serde_json::json!({"figure_kind": "logo"}),
            },
        ],
    }]
}

async fn seed(store: &dyn DocumentPageLayoutStorage, doc: Uuid, ws: Uuid, page_number: i32) {
    store
        .replace_document_pages(ReplaceDocumentPagesRequest {
            document_id: doc,
            workspace_id: ws,
            pages: sample_pages(page_number),
        })
        .await
        .unwrap();
}

fn test_router(state: AppState) -> axum::Router {
    Server::new(
        ServerConfig {
            host: "127.0.0.1".into(),
            port: 0,
            enable_cors: false,
            enable_compression: false,
            enable_swagger: true,
        },
        state,
    )
    .build_router()
}

async fn get_json(app: &axum::Router, uri: &str, workspace_id: &str) -> (StatusCode, Value) {
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(uri)
                .header("X-Tenant-ID", TENANT)
                .header("X-User-ID", USER)
                .header("X-Workspace-ID", workspace_id)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body = serde_json::from_slice(&bytes).unwrap_or_else(|_| json!({}));
    (status, body)
}

#[test]
fn contract_spec128_bbox_norm_matches_formula() {
    let n = bbox_norm_from_pdf(golden_bbox(), 612.0, 792.0, 0);
    assert!(bbox_norm_iou(n, golden_norm()) >= 0.99);
}

#[tokio::test]
async fn contract_spec128_memory_layout_round_trip() {
    let store = MemoryPageLayoutStorage::new();
    let doc = Uuid::new_v4();
    let ws = Uuid::parse_str(WORKSPACE).unwrap();
    seed(&store, doc, ws, 3).await;
    let bundle = store
        .get_page_layout(&ws, &doc, 3)
        .await
        .unwrap()
        .expect("page 3");
    assert_eq!(bundle.regions.len(), 2);
    assert_eq!(bundle.page.region_count, 2);
    let n = bbox_norm_from_pdf(
        bundle.regions[0].bbox_pdf,
        bundle.page.width_pt,
        bundle.page.height_pt,
        bundle.page.rotation,
    );
    assert!(bbox_norm_iou(n, golden_norm()) >= 0.99);
}

#[tokio::test]
async fn contract_spec128_memory_cascade_delete() {
    let store = MemoryPageLayoutStorage::new();
    let doc = Uuid::new_v4();
    let ws = Uuid::parse_str(WORKSPACE).unwrap();
    seed(&store, doc, ws, 1).await;
    let n = store.delete_pages_for_document(&ws, &doc).await.unwrap();
    assert_eq!(n, 1);
    assert!(store.get_page_layout(&ws, &doc, 1).await.unwrap().is_none());
}

#[test]
fn contract_spec128_vision_rs_calls_prune() {
    let src = include_str!("../../edgequake-pdf/src/backend/vision.rs");
    assert!(
        src.contains("apply_filter_result_or_keep") || src.contains("apply_filter_to_figure_map"),
        "G-prune: vision convert must rebuild figure_map after filter"
    );
    assert!(
        src.contains("write_sidecar_from_assets"),
        "SPEC-128: persist layout sidecar before bbox drop"
    );
}

#[tokio::test]
async fn contract_spec128_http_get_layout_page_3() {
    let state = AppState::test_state();
    let store = state
        .storage
        .page_layout_storage
        .as_ref()
        .expect("memory page layout storage");
    let doc = Uuid::new_v4();
    let ws = Uuid::parse_str(WORKSPACE).unwrap();
    seed(store.as_ref(), doc, ws, 3).await;
    let app = test_router(state);
    let (status, body) = get_json(
        &app,
        &format!("/api/v1/documents/{doc}/pages/3/layout"),
        WORKSPACE,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "body={body}");
    assert_eq!(body["page_number"], 3);
    let regions = body["regions"].as_array().expect("regions");
    assert_eq!(regions.len(), 2);
    let n = &regions[0]["bbox_norm"];
    let got = LayoutBBoxNorm {
        x: n["x"].as_f64().unwrap(),
        y: n["y"].as_f64().unwrap(),
        w: n["w"].as_f64().unwrap(),
        h: n["h"].as_f64().unwrap(),
    };
    assert!(
        bbox_norm_iou(got, golden_norm()) >= 0.99,
        "GET layout bbox_norm {got:?}"
    );
    assert!(regions.iter().any(|r| r["class"] == "abandon"));

    let (status, body) = get_json(&app, &format!("/api/v1/documents/{doc}/pages"), WORKSPACE).await;
    assert_eq!(status, StatusCode::OK, "body={body}");
    assert_eq!(body["pages"][0]["region_count"], 2);
}

#[tokio::test]
async fn contract_spec128_http_page_bounds() {
    let state = AppState::test_state();
    let store = state.storage.page_layout_storage.as_ref().unwrap();
    let doc = Uuid::new_v4();
    let ws = Uuid::parse_str(WORKSPACE).unwrap();
    seed(store.as_ref(), doc, ws, 1).await;
    let app = test_router(state);
    let (status, _) = get_json(
        &app,
        &format!("/api/v1/documents/{doc}/pages/0/layout"),
        WORKSPACE,
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    let (status, _) = get_json(
        &app,
        &format!("/api/v1/documents/{doc}/pages/999/layout"),
        WORKSPACE,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn contract_spec128_http_rls_other_workspace() {
    let state = AppState::test_state();
    let store = state.storage.page_layout_storage.as_ref().unwrap();
    let doc = Uuid::new_v4();
    let ws = Uuid::parse_str(WORKSPACE).unwrap();
    seed(store.as_ref(), doc, ws, 1).await;
    let app = test_router(state);
    let other = "dddddddd-0019-0019-0019-dddddddddddd";
    let (status, _) = get_json(
        &app,
        &format!("/api/v1/documents/{doc}/pages/1/layout"),
        other,
    )
    .await;
    assert_eq!(
        status,
        StatusCode::NOT_FOUND,
        "G-rls: other workspace must not see layout"
    );
}

#[cfg(feature = "postgres")]
mod postgres_cascade {
    use super::*;
    use edgequake_storage::PostgresPageLayoutStorage;
    use sqlx::postgres::PgPoolOptions;

    async fn try_pool() -> Option<sqlx::PgPool> {
        let url = std::env::var("DATABASE_URL").ok()?;
        PgPoolOptions::new()
            .max_connections(2)
            .connect(&url)
            .await
            .ok()
    }

    #[tokio::test]
    async fn contract_spec128_postgres_delete_document_cascades_layout() {
        let Some(pool) = try_pool().await else {
            eprintln!("SKIP postgres cascade: DATABASE_URL not reachable");
            return;
        };
        let tenant = Uuid::new_v4();
        let workspace = Uuid::new_v4();
        let doc = Uuid::new_v4();
        if sqlx::query(
            "INSERT INTO tenants (tenant_id, name, slug) VALUES ($1, $2, $3) ON CONFLICT DO NOTHING",
        )
        .bind(tenant)
        .bind(format!("t-{tenant}"))
        .bind(format!("t-{tenant}"))
        .execute(&pool)
        .await
        .is_err()
        {
            eprintln!("SKIP postgres cascade: tenants insert failed (schema?)");
            return;
        }
        if sqlx::query(
            "INSERT INTO workspaces (workspace_id, tenant_id, name, slug) VALUES ($1,$2,$3,$4) ON CONFLICT DO NOTHING",
        )
        .bind(workspace)
        .bind(tenant)
        .bind(format!("w-{workspace}"))
        .bind(format!("w-{workspace}"))
        .execute(&pool)
        .await
        .is_err()
        {
            eprintln!("SKIP postgres cascade: workspaces insert failed");
            let _ = sqlx::query("DELETE FROM tenants WHERE tenant_id = $1")
                .bind(tenant)
                .execute(&pool)
                .await;
            return;
        }
        if sqlx::query(
            "INSERT INTO documents (id, tenant_id, workspace_id, title, content, status) VALUES ($1,$2,$3,$4,$5,'indexed')",
        )
            .bind(doc)
            .bind(tenant)
            .bind(workspace)
            .bind("spec128-cascade")
            .bind("x")
            .execute(&pool)
            .await
            .is_err()
        {
            eprintln!("SKIP postgres cascade: documents insert failed");
            let _ = sqlx::query("DELETE FROM workspaces WHERE workspace_id = $1")
                .bind(workspace)
                .execute(&pool)
                .await;
            let _ = sqlx::query("DELETE FROM tenants WHERE tenant_id = $1")
                .bind(tenant)
                .execute(&pool)
                .await;
            return;
        }
        let store = PostgresPageLayoutStorage::new(pool.clone());
        seed(&store, doc, workspace, 1).await;
        let before = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM page_layout_regions WHERE document_id = $1",
        )
        .bind(doc)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert!(before >= 1);
        sqlx::query("DELETE FROM documents WHERE id = $1")
            .bind(doc)
            .execute(&pool)
            .await
            .unwrap();
        let after = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM page_layout_regions WHERE document_id = $1",
        )
        .bind(doc)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(after, 0, "G-cascade: layout regions must die with document");
        let _ = sqlx::query("DELETE FROM workspaces WHERE workspace_id = $1")
            .bind(workspace)
            .execute(&pool)
            .await;
        let _ = sqlx::query("DELETE FROM tenants WHERE tenant_id = $1")
            .bind(tenant)
            .execute(&pool)
            .await;
    }
}
