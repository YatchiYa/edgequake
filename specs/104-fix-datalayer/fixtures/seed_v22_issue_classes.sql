-- SPEC-104 synthetic seed for V22 issue classes (safe for empty/dev DB).
-- Does NOT require production dump. Run against edgequake DB as role edgequake.

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- ---------------------------------------------------------------------------
-- Issue #1 / #2 context: orphan workspace-shaped KV table (UUID)
-- INV-D2 will probe workspaces.id (bug) once per such table.
-- ---------------------------------------------------------------------------
DO $$
DECLARE
  orphan_ws uuid := 'aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee';
  tbl text := format('eq_%s_kv', orphan_ws);
BEGIN
  EXECUTE format(
    'CREATE TABLE IF NOT EXISTS public.%I (
       key TEXT PRIMARY KEY,
       value JSONB,
       created_at TIMESTAMPTZ DEFAULT NOW()
     )',
    tbl
  );
  -- Intentionally NO matching workspaces.workspace_id row.
END $$;

-- ---------------------------------------------------------------------------
-- Issue #3: indexed document without chunk KV keys
-- ---------------------------------------------------------------------------
INSERT INTO documents (
  id, workspace_id, tenant_id, title, content, status, created_at, updated_at
)
SELECT
  'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb'::uuid,
  w.workspace_id,
  w.tenant_id,
  'SPEC-104 INV-03 seed (no chunks)',
  'synthetic body for SPEC-104',
  'indexed',
  NOW(),
  NOW()
FROM workspaces w
ORDER BY w.created_at
LIMIT 1
ON CONFLICT (id) DO UPDATE
  SET status = 'indexed',
      title = EXCLUDED.title,
      content = EXCLUDED.content;

-- Ensure default KV has no {id}-chunk-% for that document (delete if any).
DO $$
BEGIN
  IF EXISTS (
    SELECT 1 FROM information_schema.tables
    WHERE table_schema = 'public' AND table_name = 'eq_eq_default_kv'
  ) THEN
    DELETE FROM eq_eq_default_kv
    WHERE key LIKE 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb-chunk-%';
  END IF;
END $$;

-- ---------------------------------------------------------------------------
-- Issue #2: document that AGE graph is NOT named "edgequake"
-- (informational SELECT for operators; safe if AGE missing)
-- ---------------------------------------------------------------------------
-- SELECT name FROM ag_catalog.ag_graph;

-- ---------------------------------------------------------------------------
-- Issue #4: use API concurrent POST /api/v1/tenants {"name":"Novagen Orga"}
-- Issue #5: requires scale; see measurements/ and analytics_ops e2e.
-- ---------------------------------------------------------------------------
