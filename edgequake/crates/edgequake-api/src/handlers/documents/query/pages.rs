//! GET `/documents/{id}/pages` and `/documents/{id}/pages/{n}/layout` (SPEC-128).

use axum::extract::{Path, State};
use axum::Json;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use edgequake_storage::{bbox_norm_from_pdf, DocumentPage, PageLayoutRegion};

use crate::error::{ApiError, ApiResult};
use crate::middleware::TenantContext;
use crate::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DocumentPageSummary {
    pub page_number: i32,
    pub width_pt: f64,
    pub height_pt: f64,
    pub rotation: i16,
    pub layout_status: String,
    pub region_count: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DocumentPagesResponse {
    pub document_id: String,
    pub pages: Vec<DocumentPageSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct LayoutBBoxPdfDto {
    pub x0: f64,
    pub y0: f64,
    pub x1: f64,
    pub y1: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct LayoutBBoxNormDto {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PageLayoutRegionDto {
    pub region_id: String,
    pub class: String,
    pub source: String,
    pub bbox_pdf: LayoutBBoxPdfDto,
    pub bbox_norm: LayoutBBoxNormDto,
    pub confidence: Option<f32>,
    pub reading_order: Option<i32>,
    pub asset_path: Option<String>,
    pub extra: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PageLayoutResponse {
    pub document_id: String,
    pub page_number: i32,
    pub width_pt: f64,
    pub height_pt: f64,
    pub rotation: i16,
    pub layout_model: Option<String>,
    pub layout_status: String,
    pub regions: Vec<PageLayoutRegionDto>,
}

fn workspace_uuid(tenant: &TenantContext) -> ApiResult<Uuid> {
    tenant
        .workspace_id_uuid()
        .ok_or_else(|| ApiError::BadRequest("workspace required".into()))
}

fn document_uuid(document_id: &str) -> ApiResult<Uuid> {
    Uuid::parse_str(document_id)
        .map_err(|e| ApiError::BadRequest(format!("invalid document_id: {e}")))
}

fn page_summary(p: DocumentPage) -> DocumentPageSummary {
    DocumentPageSummary {
        page_number: p.page_number,
        width_pt: p.width_pt,
        height_pt: p.height_pt,
        rotation: p.rotation,
        layout_status: p.layout_status,
        region_count: Some(p.region_count),
    }
}

fn region_dto(page: &DocumentPage, r: PageLayoutRegion) -> PageLayoutRegionDto {
    let bbox_norm = bbox_norm_from_pdf(r.bbox_pdf, page.width_pt, page.height_pt, page.rotation);
    PageLayoutRegionDto {
        region_id: r.region_id.to_string(),
        class: r.class,
        source: r.source,
        bbox_pdf: LayoutBBoxPdfDto {
            x0: r.bbox_pdf.x0,
            y0: r.bbox_pdf.y0,
            x1: r.bbox_pdf.x1,
            y1: r.bbox_pdf.y1,
        },
        bbox_norm: LayoutBBoxNormDto {
            x: bbox_norm.x,
            y: bbox_norm.y,
            w: bbox_norm.w,
            h: bbox_norm.h,
        },
        confidence: r.confidence,
        reading_order: r.reading_order,
        asset_path: r.asset_path,
        extra: r.extra,
    }
}

fn page_layout_storage(
    state: &AppState,
) -> ApiResult<&dyn edgequake_storage::DocumentPageLayoutStorage> {
    state
        .storage
        .page_layout_storage
        .as_deref()
        .ok_or_else(|| ApiError::Internal("page layout storage not initialized".into()))
}

/// GET `/api/v1/documents/{document_id}/pages`
#[utoipa::path(
    get,
    path = "/api/v1/documents/{document_id}/pages",
    tag = "Documents",
    params(("document_id" = String, Path, description = "Document UUID")),
    responses(
        (status = 200, description = "Page list (empty if layout not extracted yet)", body = DocumentPagesResponse),
        (status = 400, description = "Invalid document id or missing workspace")
    )
)]
pub async fn list_document_pages(
    State(state): State<AppState>,
    tenant: TenantContext,
    Path(document_id): Path<String>,
) -> ApiResult<Json<DocumentPagesResponse>> {
    let ws = workspace_uuid(&tenant)?;
    let doc = document_uuid(&document_id)?;
    let storage = page_layout_storage(&state)?;
    let pages = storage
        .list_document_pages(&ws, &doc)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(DocumentPagesResponse {
        document_id,
        pages: pages.into_iter().map(page_summary).collect(),
    }))
}

/// GET `/api/v1/documents/{document_id}/pages/{page_number}/layout`
#[utoipa::path(
    get,
    path = "/api/v1/documents/{document_id}/pages/{page_number}/layout",
    tag = "Documents",
    params(
        ("document_id" = String, Path, description = "Document UUID"),
        ("page_number" = i32, Path, description = "1-indexed page")
    ),
    responses(
        (status = 200, description = "Page layout", body = PageLayoutResponse),
        (status = 404, description = "Page not found")
    )
)]
pub async fn get_document_page_layout(
    State(state): State<AppState>,
    tenant: TenantContext,
    Path((document_id, page_number)): Path<(String, i32)>,
) -> ApiResult<Json<PageLayoutResponse>> {
    if page_number < 1 {
        return Err(ApiError::BadRequest("page_number must be >= 1".into()));
    }
    let ws = workspace_uuid(&tenant)?;
    let doc = document_uuid(&document_id)?;
    let storage = page_layout_storage(&state)?;
    let bundle = storage
        .get_page_layout(&ws, &doc, page_number)
        .await
        .map_err(ApiError::from)?
        .ok_or_else(|| ApiError::NotFound(format!("layout page {page_number} not found")))?;
    let regions = bundle
        .regions
        .into_iter()
        .map(|r| region_dto(&bundle.page, r))
        .collect();
    Ok(Json(PageLayoutResponse {
        document_id,
        page_number: bundle.page.page_number,
        width_pt: bundle.page.width_pt,
        height_pt: bundle.page.height_pt,
        rotation: bundle.page.rotation,
        layout_model: bundle.page.layout_model,
        layout_status: bundle.page.layout_status,
        regions,
    }))
}
