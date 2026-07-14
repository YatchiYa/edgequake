-- ============================================================================
-- Migration 084: Document multimodal assets (page PNGs / chart crops)
-- Durable storage for vision page drawings previously filesystem-only.
-- ============================================================================

SET search_path = public;

CREATE TABLE IF NOT EXISTS document_mm_assets (
    document_id UUID NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    workspace_id UUID NOT NULL REFERENCES workspaces(workspace_id) ON DELETE CASCADE,
    asset_path VARCHAR(512) NOT NULL,
    content_type VARCHAR(100) NOT NULL DEFAULT 'image/png',
    file_size_bytes BIGINT NOT NULL,
    asset_data BYTEA NOT NULL,
    asset_kind VARCHAR(32) NOT NULL DEFAULT 'page_full',
    page_num INTEGER,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (document_id, asset_path)
);

CREATE INDEX IF NOT EXISTS idx_document_mm_assets_workspace
    ON document_mm_assets(workspace_id);

CREATE INDEX IF NOT EXISTS idx_document_mm_assets_doc_page
    ON document_mm_assets(document_id, page_num);

COMMENT ON TABLE document_mm_assets IS
    'Vision page / chart-crop PNG assets for markdown viewer and multimodal analyze (SPEC-047 MV-28 durable)';
COMMENT ON COLUMN document_mm_assets.asset_path IS
    'Relative path SSOT from drawing_tags (e.g. assets/page-0001.png)';
COMMENT ON COLUMN document_mm_assets.asset_kind IS
    'page_full | page_chart_crop';
