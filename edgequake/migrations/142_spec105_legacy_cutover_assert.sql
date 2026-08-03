-- SPEC-105 LAW-L5 — post–SPEC-091 legacy cutover assert (expandable).
--
-- Runs after 141. Does NOT replace migrations 125 / 126 / 131.
--
-- Upgrade from ≤0.22:
--   1) migrate expandable (106…)
--   2) migrate --confirm-drop (125/126/131) when durable rows remain
--   3) this migration asserts emptiness and drops empty leftover tables
--
-- Behavior:
--   * Any public.eq_%_kv or eq_%_vectors (non-stats) with ≥1 row → RAISE
--     (operator must finish confirm-drop first).
--   * Empty leftover tables → DROP TABLE IF EXISTS.
--   * Upsert server_config legacy_stores_forbidden = true.

DO $$
DECLARE
    rec RECORD;
    row_cnt BIGINT;
    dropped INT := 0;
BEGIN
    FOR rec IN
        SELECT c.relname AS table_name
        FROM pg_class c
        JOIN pg_namespace n ON n.oid = c.relnamespace
        WHERE n.nspname = 'public'
          AND c.relkind = 'r'
          AND (
                c.relname LIKE 'eq\_%\_kv' ESCAPE '\'
             OR (
                    c.relname LIKE 'eq\_%\_vectors' ESCAPE '\'
                AND c.relname NOT LIKE '%\_stats' ESCAPE '\'
             )
          )
        ORDER BY 1
    LOOP
        -- Identifier already constrained by LIKE eq_* shape.
        EXECUTE format('SELECT COUNT(*)::bigint FROM public.%I', rec.table_name)
            INTO row_cnt;
        IF row_cnt > 0 THEN
            RAISE EXCEPTION
                'SPEC-105 migration 142: public.% still has % row(s). '
                'Finish SPEC-091 irreversible drops first: '
                'edgequake migrate --confirm-drop (125/126/131). '
                '142 never deletes durable legacy data.',
                rec.table_name, row_cnt;
        END IF;
        EXECUTE format('DROP TABLE IF EXISTS public.%I CASCADE', rec.table_name);
        dropped := dropped + 1;
        RAISE NOTICE 'SPEC-105: dropped empty legacy table public.%', rec.table_name;
    END LOOP;

    RAISE NOTICE 'SPEC-105 migration 142: empty legacy tables dropped=%', dropped;
END $$;

INSERT INTO server_config (key, value, updated_at)
VALUES (
    'legacy_stores_forbidden',
    'true'::jsonb,
    NOW()
)
ON CONFLICT (key) DO UPDATE
SET value = EXCLUDED.value,
    updated_at = NOW();
