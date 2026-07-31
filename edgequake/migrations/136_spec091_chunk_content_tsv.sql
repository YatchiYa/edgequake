-- SPEC-091 RM2: lexical spine on typed chunks (portable PG16/17/18 STORED generated).

ALTER TABLE public.chunks
    ADD COLUMN IF NOT EXISTS content_tsv tsvector
    GENERATED ALWAYS AS (to_tsvector('english', coalesce(content, ''))) STORED;

CREATE INDEX IF NOT EXISTS idx_chunks_content_tsv
    ON public.chunks USING gin (content_tsv);

COMMENT ON COLUMN public.chunks.content_tsv IS
    'SPEC-091 RM2: BM25/tsquery lexical index over chunk content (LAW-D6 generated from content).';
