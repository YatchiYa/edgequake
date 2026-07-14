-- ============================================================================
-- Migration 085: document_mm_assets RLS + stable asset_id (REST by document + id)
-- ============================================================================

SET search_path = public;

-- Partial page index (lineage scans) — idempotent replace of non-partial index from 084
DROP INDEX IF EXISTS idx_document_mm_assets_doc_page;
CREATE INDEX IF NOT EXISTS idx_document_mm_assets_doc_page
    ON document_mm_assets(document_id, page_num)
    WHERE page_num IS NOT NULL;

COMMENT ON TABLE document_mm_assets IS
    'Vision page / chart-crop PNG assets for markdown viewer and multimodal analyze (SPEC-047 durable). Cascade-deletes with document lineage.';
COMMENT ON COLUMN document_mm_assets.page_num IS
    '1-indexed PDF page for page→asset lineage (matches chunks.page_start)';

-- Workspace isolation (mirrors pdf_documents / document_originals pattern)
ALTER TABLE document_mm_assets ENABLE ROW LEVEL SECURITY;

DROP POLICY IF EXISTS document_mm_assets_workspace_select ON document_mm_assets;
DROP POLICY IF EXISTS document_mm_assets_workspace_insert ON document_mm_assets;
DROP POLICY IF EXISTS document_mm_assets_workspace_update ON document_mm_assets;
DROP POLICY IF EXISTS document_mm_assets_workspace_delete ON document_mm_assets;

CREATE POLICY document_mm_assets_workspace_select ON document_mm_assets
    FOR SELECT
    USING (
        workspace_id::text = COALESCE(current_setting('app.current_workspace_id', true), '')
        OR current_setting('app.current_workspace_id', true) IS NULL
        OR current_setting('app.current_workspace_id', true) = ''
    );

CREATE POLICY document_mm_assets_workspace_insert ON document_mm_assets
    FOR INSERT
    WITH CHECK (
        workspace_id::text = COALESCE(current_setting('app.current_workspace_id', true), '')
        OR current_setting('app.current_workspace_id', true) IS NULL
        OR current_setting('app.current_workspace_id', true) = ''
    );

CREATE POLICY document_mm_assets_workspace_update ON document_mm_assets
    FOR UPDATE
    USING (
        workspace_id::text = COALESCE(current_setting('app.current_workspace_id', true), '')
        OR current_setting('app.current_workspace_id', true) IS NULL
        OR current_setting('app.current_workspace_id', true) = ''
    );

CREATE POLICY document_mm_assets_workspace_delete ON document_mm_assets
    FOR DELETE
    USING (
        workspace_id::text = COALESCE(current_setting('app.current_workspace_id', true), '')
        OR current_setting('app.current_workspace_id', true) IS NULL
        OR current_setting('app.current_workspace_id', true) = ''
    );

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'edgequake') THEN
        GRANT SELECT, INSERT, UPDATE, DELETE ON document_mm_assets TO edgequake;
    END IF;
END $$;

-- Stable asset_id for REST GET /documents/{id}/assets/{asset_id}
ALTER TABLE document_mm_assets
    ADD COLUMN IF NOT EXISTS asset_id VARCHAR(128);

UPDATE document_mm_assets
SET asset_id = regexp_replace(
        regexp_replace(asset_path, '^.*\/', ''),
        '\.[^.]+$',
        ''
    )
WHERE asset_id IS NULL OR asset_id = '';

DO $$
BEGIN
    IF EXISTS (
        SELECT 1 FROM document_mm_assets
        WHERE asset_id IS NULL OR btrim(asset_id) = ''
    ) THEN
        RAISE EXCEPTION 'Migration 085 FAILED: asset_id backfill left empty rows';
    END IF;
END $$;

ALTER TABLE document_mm_assets
    ALTER COLUMN asset_id SET NOT NULL;

CREATE UNIQUE INDEX IF NOT EXISTS idx_document_mm_assets_doc_asset_id
    ON document_mm_assets(document_id, asset_id);

COMMENT ON COLUMN document_mm_assets.asset_id IS
    'Stable REST id (filename stem): page-0001 | page-0001-chart — SSOT with drawing_tags::asset_id_from_rel_path';
