//! PostgreSQL page layout storage (SPEC-128).

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::error::{Result, StorageError};
use crate::page_layout_storage::*;

pub struct PostgresPageLayoutStorage {
    pool: PgPool,
}

impl PostgresPageLayoutStorage {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(FromRow)]
struct PageRow {
    page_id: Uuid,
    document_id: Uuid,
    workspace_id: Uuid,
    page_number: i32,
    width_pt: f64,
    height_pt: f64,
    rotation: i16,
    cropbox_pdf: Option<Value>,
    raster_width_px: Option<i32>,
    raster_height_px: Option<i32>,
    layout_model: Option<String>,
    layout_status: String,
    #[sqlx(default)]
    region_count: i32,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(FromRow)]
struct RegionRow {
    region_id: Uuid,
    page_id: Uuid,
    document_id: Uuid,
    workspace_id: Uuid,
    class: String,
    source: String,
    bbox_pdf: Value,
    confidence: Option<f32>,
    reading_order: Option<i32>,
    asset_path: Option<String>,
    extra: Value,
    created_at: DateTime<Utc>,
}

fn parse_bbox(v: &Value) -> LayoutBBoxPdf {
    LayoutBBoxPdf {
        x0: v.get("x0").and_then(|x| x.as_f64()).unwrap_or(0.0),
        y0: v.get("y0").and_then(|x| x.as_f64()).unwrap_or(0.0),
        x1: v.get("x1").and_then(|x| x.as_f64()).unwrap_or(0.0),
        y1: v.get("y1").and_then(|x| x.as_f64()).unwrap_or(0.0),
    }
}

impl From<PageRow> for DocumentPage {
    fn from(r: PageRow) -> Self {
        Self {
            page_id: r.page_id,
            document_id: r.document_id,
            workspace_id: r.workspace_id,
            page_number: r.page_number,
            width_pt: r.width_pt,
            height_pt: r.height_pt,
            rotation: r.rotation,
            cropbox_pdf: r.cropbox_pdf.as_ref().map(parse_bbox),
            raster_width_px: r.raster_width_px,
            raster_height_px: r.raster_height_px,
            layout_model: r.layout_model,
            layout_status: r.layout_status,
            region_count: r.region_count,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}

impl From<RegionRow> for PageLayoutRegion {
    fn from(r: RegionRow) -> Self {
        Self {
            region_id: r.region_id,
            page_id: r.page_id,
            document_id: r.document_id,
            workspace_id: r.workspace_id,
            class: r.class,
            source: r.source,
            bbox_pdf: parse_bbox(&r.bbox_pdf),
            confidence: r.confidence,
            reading_order: r.reading_order,
            asset_path: r.asset_path,
            extra: r.extra,
            created_at: r.created_at,
        }
    }
}

#[async_trait]
impl DocumentPageLayoutStorage for PostgresPageLayoutStorage {
    async fn replace_document_pages(&self, request: ReplaceDocumentPagesRequest) -> Result<()> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| StorageError::Database(format!("layout tx begin: {e}")))?;
        sqlx::query("DELETE FROM document_pages WHERE document_id = $1 AND workspace_id = $2")
            .bind(request.document_id)
            .bind(request.workspace_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| StorageError::Database(format!("layout delete pages: {e}")))?;

        for p in request.pages {
            let crop = p
                .cropbox_pdf
                .map(|b| serde_json::json!({"x0": b.x0, "y0": b.y0, "x1": b.x1, "y1": b.y1}));
            let page_id: Uuid = sqlx::query_scalar(
                r#"
                INSERT INTO document_pages (
                    document_id, workspace_id, page_number, width_pt, height_pt,
                    rotation, cropbox_pdf, raster_width_px, raster_height_px,
                    layout_model, layout_status
                ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)
                RETURNING page_id
                "#,
            )
            .bind(request.document_id)
            .bind(request.workspace_id)
            .bind(p.page_number)
            .bind(p.width_pt)
            .bind(p.height_pt)
            .bind(p.rotation)
            .bind(crop)
            .bind(p.raster_width_px)
            .bind(p.raster_height_px)
            .bind(&p.layout_model)
            .bind(&p.layout_status)
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| StorageError::Database(format!("layout insert page: {e}")))?;

            for r in p.regions {
                let bbox = serde_json::json!({
                    "x0": r.bbox_pdf.x0,
                    "y0": r.bbox_pdf.y0,
                    "x1": r.bbox_pdf.x1,
                    "y1": r.bbox_pdf.y1
                });
                sqlx::query(
                    r#"
                    INSERT INTO page_layout_regions (
                        page_id, document_id, workspace_id, class, source,
                        bbox_pdf, confidence, reading_order, asset_path, extra
                    ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)
                    "#,
                )
                .bind(page_id)
                .bind(request.document_id)
                .bind(request.workspace_id)
                .bind(&r.class)
                .bind(&r.source)
                .bind(bbox)
                .bind(r.confidence)
                .bind(r.reading_order)
                .bind(&r.asset_path)
                .bind(&r.extra)
                .execute(&mut *tx)
                .await
                .map_err(|e| StorageError::Database(format!("layout insert region: {e}")))?;
            }
        }

        tx.commit()
            .await
            .map_err(|e| StorageError::Database(format!("layout tx commit: {e}")))?;
        Ok(())
    }

    async fn list_document_pages(
        &self,
        workspace_id: &Uuid,
        document_id: &Uuid,
    ) -> Result<Vec<DocumentPage>> {
        let rows = sqlx::query_as::<_, PageRow>(
            r#"
            SELECT page_id, document_id, workspace_id, page_number, width_pt, height_pt,
                   rotation, cropbox_pdf, raster_width_px, raster_height_px,
                   layout_model, layout_status, created_at, updated_at,
                   (SELECT COUNT(*)::int FROM page_layout_regions r WHERE r.page_id = document_pages.page_id)
                       AS region_count
            FROM document_pages
            WHERE document_id = $1 AND workspace_id = $2
            ORDER BY page_number
            "#,
        )
        .bind(document_id)
        .bind(workspace_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| StorageError::Database(format!("list document pages: {e}")))?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn get_page_layout(
        &self,
        workspace_id: &Uuid,
        document_id: &Uuid,
        page_number: i32,
    ) -> Result<Option<PageLayoutBundle>> {
        let page = sqlx::query_as::<_, PageRow>(
            r#"
            SELECT page_id, document_id, workspace_id, page_number, width_pt, height_pt,
                   rotation, cropbox_pdf, raster_width_px, raster_height_px,
                   layout_model, layout_status, created_at, updated_at
            FROM document_pages
            WHERE document_id = $1 AND workspace_id = $2 AND page_number = $3
            "#,
        )
        .bind(document_id)
        .bind(workspace_id)
        .bind(page_number)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| StorageError::Database(format!("get document page: {e}")))?;
        let Some(page) = page else {
            return Ok(None);
        };
        let mut page: DocumentPage = page.into();
        let rows = sqlx::query_as::<_, RegionRow>(
            r#"
            SELECT region_id, page_id, document_id, workspace_id, class, source,
                   bbox_pdf, confidence, reading_order, asset_path, extra, created_at
            FROM page_layout_regions
            WHERE page_id = $1
            ORDER BY reading_order NULLS LAST, created_at
            "#,
        )
        .bind(page.page_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| StorageError::Database(format!("get page regions: {e}")))?;
        let regions: Vec<PageLayoutRegion> = rows.into_iter().map(Into::into).collect();
        page.region_count = regions.len() as i32;
        Ok(Some(PageLayoutBundle { page, regions }))
    }

    async fn delete_pages_for_document(
        &self,
        workspace_id: &Uuid,
        document_id: &Uuid,
    ) -> Result<u64> {
        let res =
            sqlx::query("DELETE FROM document_pages WHERE document_id = $1 AND workspace_id = $2")
                .bind(document_id)
                .bind(workspace_id)
                .execute(&self.pool)
                .await
                .map_err(|e| StorageError::Database(format!("delete document pages: {e}")))?;
        Ok(res.rows_affected())
    }
}
