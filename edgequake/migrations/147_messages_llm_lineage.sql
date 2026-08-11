-- SPEC-032 / query metadata bar: persist LLM provider + model on messages
-- so history can show lineage next to tokens/sec (with query mode).

ALTER TABLE messages
    ADD COLUMN IF NOT EXISTS llm_provider VARCHAR(100),
    ADD COLUMN IF NOT EXISTS llm_model VARCHAR(255);

COMMENT ON COLUMN messages.llm_provider IS 'LLM provider used for this assistant response (lineage)';
COMMENT ON COLUMN messages.llm_model IS 'LLM model used for this assistant response (lineage)';
