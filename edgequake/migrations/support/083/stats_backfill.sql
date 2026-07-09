-- ============================================================================
-- Repair: copy relationship_count from documents.metadata JSONB into the column
-- when the column is 0/NULL but JSONB has a positive value (legacy stats path drift).
-- Idempotent. Safe to run on every bootstrap.
-- ============================================================================
UPDATE documents
SET relationship_count = COALESCE(
      NULLIF((metadata->>'relationship_count')::int, 0),
      relationship_count
    ),
    entity_count = CASE
      WHEN entity_count = 0
           AND COALESCE((metadata->>'entity_count')::int, 0) > 0
      THEN (metadata->>'entity_count')::int
      ELSE entity_count
    END,
    chunk_count = CASE
      WHEN chunk_count = 0
           AND COALESCE((metadata->>'chunk_count')::int, 0) > 0
      THEN (metadata->>'chunk_count')::int
      ELSE chunk_count
    END,
    updated_at = NOW()
WHERE (
    (COALESCE(relationship_count, 0) = 0
     AND COALESCE((metadata->>'relationship_count')::int, 0) > 0)
 OR (COALESCE(entity_count, 0) = 0
     AND COALESCE((metadata->>'entity_count')::int, 0) > 0)
 OR (COALESCE(chunk_count, 0) = 0
     AND COALESCE((metadata->>'chunk_count')::int, 0) > 0)
);
