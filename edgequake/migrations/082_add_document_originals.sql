-- ============================================================================
-- Migration 082: Document Originals Table
-- Stores raw upload bytes for non-PDF documents (images, files).
-- ============================================================================

SET search_path = public;

CREATE TABLE IF NOT EXISTS document_originals (
    document_id UUID PRIMARY KEY REFERENCES documents(id) ON DELETE CASCADE,
    workspace_id UUID NOT NULL REFERENCES workspaces(workspace_id) ON DELETE CASCADE,
    filename VARCHAR(512) NOT NULL,
    content_type VARCHAR(100) NOT NULL,
    file_size_bytes BIGINT NOT NULL,
    original_data BYTEA NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_document_originals_workspace
    ON document_originals(workspace_id);

COMMENT ON TABLE document_originals IS
    'Raw original upload bytes for non-PDF documents (images, text files, etc.)';
