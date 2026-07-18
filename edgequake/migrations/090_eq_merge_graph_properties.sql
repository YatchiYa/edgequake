-- ============================================================================
-- Migration 090: eq_merge_graph_properties (SPEC-058 Wave 2)
-- Version: 1.0.0 — 2026-07-18
--
-- PURPOSE:
--   Race-safe merge of AGE node/edge properties on native UPSERT.
--   App-level merge_source_ids is defense-in-depth; concurrent workers
--   previously lost source_ids via `properties = EXCLUDED.properties` LWW.
--
-- SEMANTICS:
--   * Scalar keys: incoming wins (jsonb ||).
--   * source_ids / source_chunk_ids: order-preserving union (existing first).
--   * Keys only in existing are preserved when absent from incoming.
--
-- IDEMPOTENT: CREATE OR REPLACE FUNCTION.
-- ============================================================================

SET search_path = public;

CREATE OR REPLACE FUNCTION eq_merge_text_json_arrays(
  existing jsonb,
  incoming jsonb
) RETURNS jsonb
LANGUAGE sql
IMMUTABLE
AS $$
  SELECT COALESCE(
    (
      SELECT jsonb_agg(to_jsonb(t.val) ORDER BY t.ord)
      FROM (
        SELECT DISTINCT ON (x.val) x.val, x.ord
        FROM (
          SELECT e.val, e.ord
          FROM jsonb_array_elements_text(COALESCE(existing, '[]'::jsonb))
                 WITH ORDINALITY AS e(val, ord)
          UNION ALL
          SELECT i.val, (SELECT COALESCE(MAX(ord), 0) FROM jsonb_array_elements_text(COALESCE(existing, '[]'::jsonb)) WITH ORDINALITY AS _(v, ord)) + i.ord
          FROM jsonb_array_elements_text(COALESCE(incoming, '[]'::jsonb))
                 WITH ORDINALITY AS i(val, ord)
        ) AS x
        ORDER BY x.val, x.ord
      ) AS t
    ),
    '[]'::jsonb
  );
$$;

CREATE OR REPLACE FUNCTION eq_merge_graph_properties(
  existing jsonb,
  incoming jsonb
) RETURNS jsonb
LANGUAGE plpgsql
IMMUTABLE
AS $$
DECLARE
  result jsonb;
  e jsonb := COALESCE(existing, '{}'::jsonb);
  i jsonb := COALESCE(incoming, '{}'::jsonb);
BEGIN
  -- Incoming scalars win; keys only in existing remain.
  result := e || i;

  IF (e ? 'source_ids') OR (i ? 'source_ids') THEN
    result := jsonb_set(
      result,
      '{source_ids}',
      eq_merge_text_json_arrays(e->'source_ids', i->'source_ids')
    );
  END IF;

  IF (e ? 'source_chunk_ids') OR (i ? 'source_chunk_ids') THEN
    result := jsonb_set(
      result,
      '{source_chunk_ids}',
      eq_merge_text_json_arrays(e->'source_chunk_ids', i->'source_chunk_ids')
    );
  END IF;

  RETURN result;
END;
$$;

COMMENT ON FUNCTION eq_merge_graph_properties(jsonb, jsonb) IS
  'SPEC-058: merge AGE property maps with source_ids/source_chunk_ids union';
