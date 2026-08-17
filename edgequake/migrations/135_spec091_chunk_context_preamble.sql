-- SPEC-091 RM2: optional contextual preamble for embedding (Anthropic-style).
-- Populated when EDGEQUAKE_CONTEXTUAL_CHUNK=on; nullable otherwise.

ALTER TABLE public.chunks
    ADD COLUMN IF NOT EXISTS context_preamble text;

COMMENT ON COLUMN public.chunks.context_preamble IS
    'SPEC-091 RM2: optional document/section context prepended at embed time '
    'when EDGEQUAKE_CONTEXTUAL_CHUNK=on (LAW-RM7 quality contract).';
