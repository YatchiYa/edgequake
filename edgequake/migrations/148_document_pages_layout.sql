-- ============================================================================
-- Migration 148: document_pages + page_layout_regions (SPEC-128 overlay)
-- ============================================================================
-- Persist per-page PDF user-space layout. bbox_norm is derived at read.
-- RLS mirrors document_mm_assets (workspace isolation, fail-closed).

SET search_path = public;

CREATE TABLE IF NOT EXISTS document_pages (
    page_id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    document_id        UUID NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    workspace_id       UUID NOT NULL REFERENCES workspaces(workspace_id) ON DELETE CASCADE,
    page_number        INT NOT NULL CHECK (page_number >= 1),
    width_pt           DOUBLE PRECISION NOT NULL CHECK (width_pt > 0),
    height_pt          DOUBLE PRECISION NOT NULL CHECK (height_pt > 0),
    rotation           SMALLINT NOT NULL DEFAULT 0,
    cropbox_pdf        JSONB NULL,
    raster_width_px    INT NULL,
    raster_height_px   INT NULL,
    layout_model       TEXT NULL,
    layout_status      TEXT NOT NULL DEFAULT 'extracted',
    created_at         TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at         TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (document_id, page_number)
);

CREATE INDEX IF NOT EXISTS idx_document_pages_workspace_doc
    ON document_pages (workspace_id, document_id);

CREATE TABLE IF NOT EXISTS page_layout_regions (
    region_id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    page_id            UUID NOT NULL REFERENCES document_pages(page_id) ON DELETE CASCADE,
    document_id        UUID NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    workspace_id       UUID NOT NULL REFERENCES workspaces(workspace_id) ON DELETE CASCADE,
    class              TEXT NOT NULL,
    source             TEXT NOT NULL,
    bbox_pdf           JSONB NOT NULL,
    confidence         REAL NULL,
    reading_order      INT NULL,
    asset_path         TEXT NULL,
    extra              JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at         TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_page_layout_regions_doc_page
    ON page_layout_regions (document_id, page_id);

CREATE INDEX IF NOT EXISTS idx_page_layout_regions_doc_class
    ON page_layout_regions (document_id, class);

COMMENT ON TABLE document_pages IS
    'Per-page PDF geometry for SPEC-128 layout overlay. Cascade-deletes with documents.';
COMMENT ON TABLE page_layout_regions IS
    'Page-scoped layout regions in PDF user space. bbox_norm is derived at read (LAW-128-4).';
COMMENT ON COLUMN page_layout_regions.bbox_pdf IS
    'PDF user-space box {x0,y0,x1,y1}; never store bbox_norm.';

ALTER TABLE document_pages ENABLE ROW LEVEL SECURITY;
ALTER TABLE page_layout_regions ENABLE ROW LEVEL SECURITY;

DROP POLICY IF EXISTS document_pages_workspace_select ON document_pages;
DROP POLICY IF EXISTS document_pages_workspace_insert ON document_pages;
DROP POLICY IF EXISTS document_pages_workspace_update ON document_pages;
DROP POLICY IF EXISTS document_pages_workspace_delete ON document_pages;

CREATE POLICY document_pages_workspace_select ON document_pages
    FOR SELECT
    USING (
        workspace_id::text = COALESCE(current_setting('app.current_workspace_id', true), '')
        OR current_setting('app.current_workspace_id', true) IS NULL
        OR current_setting('app.current_workspace_id', true) = ''
    );

CREATE POLICY document_pages_workspace_insert ON document_pages
    FOR INSERT
    WITH CHECK (
        workspace_id::text = COALESCE(current_setting('app.current_workspace_id', true), '')
        OR current_setting('app.current_workspace_id', true) IS NULL
        OR current_setting('app.current_workspace_id', true) = ''
    );

CREATE POLICY document_pages_workspace_update ON document_pages
    FOR UPDATE
    USING (
        workspace_id::text = COALESCE(current_setting('app.current_workspace_id', true), '')
        OR current_setting('app.current_workspace_id', true) IS NULL
        OR current_setting('app.current_workspace_id', true) = ''
    );

CREATE POLICY document_pages_workspace_delete ON document_pages
    FOR DELETE
    USING (
        workspace_id::text = COALESCE(current_setting('app.current_workspace_id', true), '')
        OR current_setting('app.current_workspace_id', true) IS NULL
        OR current_setting('app.current_workspace_id', true) = ''
    );

DROP POLICY IF EXISTS page_layout_regions_workspace_select ON page_layout_regions;
DROP POLICY IF EXISTS page_layout_regions_workspace_insert ON page_layout_regions;
DROP POLICY IF EXISTS page_layout_regions_workspace_update ON page_layout_regions;
DROP POLICY IF EXISTS page_layout_regions_workspace_delete ON page_layout_regions;

CREATE POLICY page_layout_regions_workspace_select ON page_layout_regions
    FOR SELECT
    USING (
        workspace_id::text = COALESCE(current_setting('app.current_workspace_id', true), '')
        OR current_setting('app.current_workspace_id', true) IS NULL
        OR current_setting('app.current_workspace_id', true) = ''
    );

CREATE POLICY page_layout_regions_workspace_insert ON page_layout_regions
    FOR INSERT
    WITH CHECK (
        workspace_id::text = COALESCE(current_setting('app.current_workspace_id', true), '')
        OR current_setting('app.current_workspace_id', true) IS NULL
        OR current_setting('app.current_workspace_id', true) = ''
    );

CREATE POLICY page_layout_regions_workspace_update ON page_layout_regions
    FOR UPDATE
    USING (
        workspace_id::text = COALESCE(current_setting('app.current_workspace_id', true), '')
        OR current_setting('app.current_workspace_id', true) IS NULL
        OR current_setting('app.current_workspace_id', true) = ''
    );

CREATE POLICY page_layout_regions_workspace_delete ON page_layout_regions
    FOR DELETE
    USING (
        workspace_id::text = COALESCE(current_setting('app.current_workspace_id', true), '')
        OR current_setting('app.current_workspace_id', true) IS NULL
        OR current_setting('app.current_workspace_id', true) = ''
    );

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'edgequake') THEN
        GRANT SELECT, INSERT, UPDATE, DELETE ON document_pages TO edgequake;
        GRANT SELECT, INSERT, UPDATE, DELETE ON page_layout_regions TO edgequake;
    END IF;
END $$;
