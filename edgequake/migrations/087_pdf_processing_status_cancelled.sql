-- ============================================================================
-- Migration 087: Allow pdf_documents.processing_status = 'cancelled' (SPEC-057 P0)
-- Version: 1.0.0 — 2026-07-17
--
-- PURPOSE:
--   User/system cancel must not map to 'failed'. Extend valid_processing_status
--   CHECK to include 'cancelled'.
--
-- IDEMPOTENT: DROP CONSTRAINT IF EXISTS + ADD CONSTRAINT.
-- ============================================================================

ALTER TABLE pdf_documents
    DROP CONSTRAINT IF EXISTS valid_processing_status;

ALTER TABLE pdf_documents
    ADD CONSTRAINT valid_processing_status CHECK (
        processing_status IN ('pending', 'processing', 'completed', 'failed', 'cancelled')
    );

COMMENT ON COLUMN pdf_documents.processing_status IS
    'pending | processing | completed | failed | cancelled (SPEC-057)';
