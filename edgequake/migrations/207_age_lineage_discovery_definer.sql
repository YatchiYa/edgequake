-- GitHub #478: per-document lineage discovery under forced AGE row security.
--
-- The guard compares agtype_to_json(properties)->>'tenant_id'. That
-- expression is not leakproof, and neither is current_tenant_id(), so the
-- tenant role cannot use idx_*_source_ids_gin. A ~160k-node graph then
-- scans the label and hits SOURCE_DISCOVERY_STATEMENT_TIMEOUT_MS (2s).
--
-- These functions run as the table owner (SECURITY DEFINER), which bypasses
-- forced row security, and apply the session tenant themselves. The GIN
-- probe stays an index condition. When app.current_tenant_id is unset, no
-- tenant predicate is added (the administration scan).

CREATE OR REPLACE FUNCTION public.eq_age_graph_name_ok(p_graph text)
RETURNS boolean
LANGUAGE sql
IMMUTABLE
PARALLEL SAFE
AS $$
    SELECT p_graph ~ '^[A-Za-z_][A-Za-z0-9_]*$'
$$;

CREATE OR REPLACE FUNCTION public.eq_age_discover_nodes(
    p_graph text,
    p_exact text[],
    p_prefixes text[],
    p_probe_limit integer
) RETURNS TABLE(props jsonb)
LANGUAGE plpgsql
STABLE
SECURITY DEFINER
SET search_path = pg_catalog, public, ag_catalog
AS $fn$
DECLARE
    v_tenant text := NULLIF(current_setting('app.current_tenant_id', true), '');
    v_workspace text := NULLIF(current_setting('app.current_workspace_id', true), '');
    v_limit integer := LEAST(GREATEST(COALESCE(p_probe_limit, 1), 1), 256);
BEGIN
    IF NOT public.eq_age_graph_name_ok(p_graph) THEN
        RAISE EXCEPTION 'refusing unsafe graph name';
    END IF;
    IF to_regclass(format('%I."Node"', p_graph)) IS NULL THEN
        RETURN;
    END IF;

    RETURN QUERY EXECUTE format(
        $q$
        WITH probes AS MATERIALIZED (
            SELECT probe_id FROM unnest($1::text[]) AS t(probe_id)
            UNION
            SELECT (p.prefix || gs.i::text) AS probe_id
            FROM unnest($2::text[]) AS p(prefix)
            CROSS JOIN generate_series(0, $3::int - 1) AS gs(i)
        ),
        probe_ids AS MATERIALIZED (
            SELECT array_agg(probe_id) AS ids FROM probes
        ),
        hits AS MATERIALIZED (
            SELECT v.properties
            FROM %I."Node" v
            CROSS JOIN probe_ids p
            WHERE p.ids IS NOT NULL
              AND (
                    (ag_catalog.agtype_to_json(v.properties)::jsonb -> 'source_ids') ?| p.ids
                 OR (ag_catalog.agtype_to_json(v.properties)::jsonb -> 'source_chunk_ids') ?| p.ids
              )
            LIMIT 5000
        )
        SELECT ag_catalog.agtype_to_json(h.properties)::jsonb
        FROM hits h
        WHERE ($4::text IS NULL OR ag_catalog.agtype_to_json(h.properties) ->> 'tenant_id' = $4)
          AND ($5::text IS NULL OR ag_catalog.agtype_to_json(h.properties) ->> 'workspace_id' = $5)
        $q$,
        p_graph
    )
    USING p_exact, p_prefixes, v_limit, v_tenant, v_workspace;
END;
$fn$;

CREATE OR REPLACE FUNCTION public.eq_age_discover_edges(
    p_graph text,
    p_exact text[],
    p_prefixes text[],
    p_probe_limit integer
) RETURNS TABLE(props jsonb, source_id text, target_id text)
LANGUAGE plpgsql
STABLE
SECURITY DEFINER
SET search_path = pg_catalog, public, ag_catalog
AS $fn$
DECLARE
    v_tenant text := NULLIF(current_setting('app.current_tenant_id', true), '');
    v_workspace text := NULLIF(current_setting('app.current_workspace_id', true), '');
    v_limit integer := LEAST(GREATEST(COALESCE(p_probe_limit, 1), 1), 256);
    v_src text;
    v_tgt text;
BEGIN
    IF NOT public.eq_age_graph_name_ok(p_graph) THEN
        RAISE EXCEPTION 'refusing unsafe graph name';
    END IF;
    IF to_regclass(format('%I."EDGE"', p_graph)) IS NULL THEN
        RETURN;
    END IF;

    IF EXISTS (
        SELECT 1
        FROM pg_attribute a
        JOIN pg_class c ON c.oid = a.attrelid
        JOIN pg_namespace n ON n.oid = c.relnamespace
        WHERE n.nspname = p_graph
          AND c.relname = 'EDGE'
          AND a.attname = 'eq_source_id'
          AND NOT a.attisdropped
    ) THEN
        v_src := 'COALESCE(e.eq_source_id, ag_catalog.agtype_to_json(e.properties)->>''source_id'')';
        v_tgt := 'COALESCE(e.eq_target_id, ag_catalog.agtype_to_json(e.properties)->>''target_id'')';
    ELSE
        v_src := '(ag_catalog.agtype_to_json(e.properties)->>''source_id'')';
        v_tgt := '(ag_catalog.agtype_to_json(e.properties)->>''target_id'')';
    END IF;

    RETURN QUERY EXECUTE format(
        $q$
        WITH probes AS MATERIALIZED (
            SELECT probe_id FROM unnest($1::text[]) AS t(probe_id)
            UNION
            SELECT (p.prefix || gs.i::text) AS probe_id
            FROM unnest($2::text[]) AS p(prefix)
            CROSS JOIN generate_series(0, $3::int - 1) AS gs(i)
        ),
        probe_ids AS MATERIALIZED (
            SELECT array_agg(probe_id) AS ids FROM probes
        ),
        hits AS MATERIALIZED (
            SELECT e.properties, %s AS source_id, %s AS target_id
            FROM %I."EDGE" e
            CROSS JOIN probe_ids p
            WHERE p.ids IS NOT NULL
              AND %s IS NOT NULL
              AND %s IS NOT NULL
              AND (
                    (ag_catalog.agtype_to_json(e.properties)::jsonb -> 'source_ids') ?| p.ids
                 OR (ag_catalog.agtype_to_json(e.properties)::jsonb -> 'source_chunk_ids') ?| p.ids
              )
            LIMIT 5000
        )
        SELECT ag_catalog.agtype_to_json(h.properties)::jsonb, h.source_id, h.target_id
        FROM hits h
        WHERE ($4::text IS NULL OR ag_catalog.agtype_to_json(h.properties) ->> 'tenant_id' = $4)
          AND ($5::text IS NULL OR ag_catalog.agtype_to_json(h.properties) ->> 'workspace_id' = $5)
        $q$,
        v_src, v_tgt, p_graph, v_src, v_tgt
    )
    USING p_exact, p_prefixes, v_limit, v_tenant, v_workspace;
END;
$fn$;

REVOKE ALL ON FUNCTION public.eq_age_graph_name_ok(text) FROM PUBLIC;
REVOKE ALL ON FUNCTION public.eq_age_discover_nodes(text, text[], text[], integer) FROM PUBLIC;
REVOKE ALL ON FUNCTION public.eq_age_discover_edges(text, text[], text[], integer) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION public.eq_age_discover_nodes(text, text[], text[], integer) TO edgequake_tenant_access;
GRANT EXECUTE ON FUNCTION public.eq_age_discover_edges(text, text[], text[], integer) TO edgequake_tenant_access;
