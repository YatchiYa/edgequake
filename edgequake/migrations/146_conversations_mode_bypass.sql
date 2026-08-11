-- ============================================================================
-- Migration 146: Allow conversations.mode = 'bypass' (Chat mode / FEAT0106)
-- Version: 1.0.0 — 2026-08-11
--
-- PURPOSE:
--   Chat mode persists ConversationMode::Bypass. The CHECK constraint only
--   allowed local|global|hybrid|naive|mix, so creating a Chat conversation
--   failed with: violates check constraint "valid_mode".
--
-- IDEMPOTENT: DROP CONSTRAINT IF EXISTS (both historical names) + ADD.
-- ============================================================================

ALTER TABLE conversations
    DROP CONSTRAINT IF EXISTS valid_mode;

ALTER TABLE conversations
    DROP CONSTRAINT IF EXISTS conversations_valid_mode;

ALTER TABLE conversations
    ADD CONSTRAINT valid_mode CHECK (
        mode IN ('local', 'global', 'hybrid', 'naive', 'mix', 'bypass')
    );

COMMENT ON COLUMN conversations.mode IS
    'local | global | hybrid | naive | mix | bypass (Chat; no KG retrieval)';
