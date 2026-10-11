-- SPEC-164 / v0.35 expand: plain adjacency tables beside Apache AGE.
-- No DROP. Mirror triggers copy committed AGE rows. Ledger stays "age"
-- until backfill + shadow (eq_graph_shadow_mismatches) mark it ready.
-- Greenfield (no ag_catalog): ledger is ready immediately (S8).

CREATE TABLE IF NOT EXISTS public.eq_graph_namespace (
    graph_name text PRIMARY KEY,
    namespace  text NOT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS eq_graph_namespace_ns_uidx
    ON public.eq_graph_namespace (namespace);

CREATE TABLE IF NOT EXISTS public.eq_graph_nodes (
    namespace    text NOT NULL,
    node_id      text NOT NULL,
    tenant_id    text NOT NULL DEFAULT '',
    workspace_id text NOT NULL DEFAULT '',
    entity_type  text,
    properties   jsonb NOT NULL DEFAULT '{}'::jsonb,
    PRIMARY KEY (namespace, node_id)
);

CREATE TABLE IF NOT EXISTS public.eq_graph_edges (
    namespace    text NOT NULL,
    source_id    text NOT NULL,
    target_id    text NOT NULL,
    rel_type     text NOT NULL DEFAULT '',
    tenant_id    text NOT NULL DEFAULT '',
    workspace_id text NOT NULL DEFAULT '',
    weight       double precision,
    properties   jsonb NOT NULL DEFAULT '{}'::jsonb,
    PRIMARY KEY (namespace, source_id, target_id, rel_type)
);

CREATE INDEX IF NOT EXISTS idx_eq_graph_edges_source
    ON public.eq_graph_edges (namespace, source_id);
CREATE INDEX IF NOT EXISTS idx_eq_graph_edges_target
    ON public.eq_graph_edges (namespace, target_id);
CREATE INDEX IF NOT EXISTS idx_eq_graph_nodes_tenant_ws
    ON public.eq_graph_nodes (namespace, tenant_id, workspace_id);
CREATE INDEX IF NOT EXISTS idx_eq_graph_edges_tenant_ws
    ON public.eq_graph_edges (namespace, tenant_id, workspace_id);
CREATE INDEX IF NOT EXISTS idx_eq_graph_nodes_source_ids_gin
    ON public.eq_graph_nodes USING gin ((properties -> 'source_ids'));
CREATE INDEX IF NOT EXISTS idx_eq_graph_nodes_source_chunk_ids_gin
    ON public.eq_graph_nodes USING gin ((properties -> 'source_chunk_ids'));
CREATE INDEX IF NOT EXISTS idx_eq_graph_edges_source_ids_gin
    ON public.eq_graph_edges USING gin ((properties -> 'source_ids'));
CREATE INDEX IF NOT EXISTS idx_eq_graph_edges_source_chunk_ids_gin
    ON public.eq_graph_edges USING gin ((properties -> 'source_chunk_ids'));

CREATE TABLE IF NOT EXISTS public.eq_graph_cutover (
    id                   smallint PRIMARY KEY DEFAULT 1 CHECK (id = 1),
    status               text NOT NULL DEFAULT 'age'
        CHECK (status IN ('age', 'backfilling', 'ready')),
    age_absent           boolean NOT NULL DEFAULT false,
    shadow_ok            boolean NOT NULL DEFAULT false,
    backfill_complete    boolean NOT NULL DEFAULT false,
    unbackfilled_graphs  integer NOT NULL DEFAULT 0,
    updated_at           timestamptz NOT NULL DEFAULT now(),
    detail               jsonb NOT NULL DEFAULT '{}'::jsonb
);

INSERT INTO public.eq_graph_cutover (id)
VALUES (1)
ON CONFLICT (id) DO NOTHING;

CREATE TABLE IF NOT EXISTS public.eq_graph_backfill (
    graph_name      text PRIMARY KEY,
    namespace       text NOT NULL,
    nodes_copied    bigint NOT NULL DEFAULT 0,
    edges_copied    bigint NOT NULL DEFAULT 0,
    orphans_skipped bigint NOT NULL DEFAULT 0,
    shadow_ok       boolean NOT NULL DEFAULT false,
    finished_at     timestamptz
);

-- Depth-bounded neighbor id set on plain tables (product caps: depth ≤ 3, K ≤ 500).
CREATE OR REPLACE FUNCTION public.eq_graph_neighbor_ids_plain(
    p_namespace text,
    p_start text,
    p_depth int,
    p_cap int
) RETURNS text[]
LANGUAGE plpgsql STABLE AS $$
DECLARE
    visited text[] := ARRAY[p_start];
    frontier text[] := ARRAY[p_start];
    next_f text[];
    nbr text;
    d int;
BEGIN
    IF p_start IS NULL OR p_depth < 1 THEN
        RETURN ARRAY[]::text[];
    END IF;
    FOR d IN 1..LEAST(p_depth, 3) LOOP
        next_f := ARRAY[]::text[];
        FOR nbr IN
            SELECT endpoint
            FROM (
                SELECT e.target_id AS endpoint
                FROM public.eq_graph_edges e
                WHERE e.namespace = p_namespace AND e.source_id = ANY(frontier)
                UNION
                SELECT e.source_id
                FROM public.eq_graph_edges e
                WHERE e.namespace = p_namespace AND e.target_id = ANY(frontier)
            ) s
            WHERE NOT (endpoint = ANY(visited))
            ORDER BY endpoint
        LOOP
            EXIT WHEN cardinality(visited) >= p_cap;
            visited := visited || nbr;
            next_f := next_f || nbr;
        END LOOP;
        frontier := next_f;
        EXIT WHEN cardinality(frontier) = 0 OR cardinality(visited) >= p_cap;
    END LOOP;
    RETURN ARRAY(SELECT x FROM unnest(visited) AS x ORDER BY x);
END;
$$;

-- Same hop shape on an AGE "EDGE" child table. Dynamic graph name is an identifier.
CREATE OR REPLACE FUNCTION public.eq_graph_neighbor_ids_age(
    p_graph text,
    p_start text,
    p_depth int,
    p_cap int
) RETURNS text[]
LANGUAGE plpgsql STABLE AS $$
DECLARE
    visited text[] := ARRAY[p_start];
    frontier text[] := ARRAY[p_start];
    next_f text[];
    nbr text;
    d int;
BEGIN
    IF p_graph !~ '^[A-Za-z_][A-Za-z0-9_]*$' THEN
        RAISE EXCEPTION 'refusing unsafe graph name';
    END IF;
    IF p_start IS NULL OR to_regclass(format('%I."EDGE"', p_graph)) IS NULL THEN
        RETURN ARRAY(SELECT x FROM unnest(visited) AS x ORDER BY x);
    END IF;
    FOR d IN 1..LEAST(p_depth, 3) LOOP
        next_f := ARRAY[]::text[];
        FOR nbr IN EXECUTE format(
            $q$
            SELECT endpoint FROM (
                SELECT COALESCE(e.eq_target_id, ag_catalog.agtype_to_json(e.properties)->>'target_id') AS endpoint
                FROM %I."EDGE" e
                WHERE COALESCE(e.eq_source_id, ag_catalog.agtype_to_json(e.properties)->>'source_id') = ANY($1)
                UNION
                SELECT COALESCE(e.eq_source_id, ag_catalog.agtype_to_json(e.properties)->>'source_id')
                FROM %I."EDGE" e
                WHERE COALESCE(e.eq_target_id, ag_catalog.agtype_to_json(e.properties)->>'target_id') = ANY($1)
            ) s
            WHERE endpoint IS NOT NULL AND NOT (endpoint = ANY($2))
            ORDER BY endpoint
            $q$, p_graph, p_graph)
        USING frontier, visited
        LOOP
            EXIT WHEN cardinality(visited) >= p_cap;
            visited := visited || nbr;
            next_f := next_f || nbr;
        END LOOP;
        frontier := next_f;
        EXIT WHEN cardinality(frontier) = 0 OR cardinality(visited) >= p_cap;
    END LOOP;
    RETURN ARRAY(SELECT x FROM unnest(visited) AS x ORDER BY x);
END;
$$;

-- H-164-02: 50 starts, depth 2, cap 500. Returns mismatch count.
CREATE OR REPLACE FUNCTION public.eq_graph_shadow_mismatches(
    p_namespace text,
    p_graph text
) RETURNS integer
LANGUAGE plpgsql STABLE AS $$
DECLARE
    s text;
    mismatches integer := 0;
    plain_ids text[];
    age_ids text[];
BEGIN
    FOR s IN
        SELECT node_id
        FROM public.eq_graph_nodes
        WHERE namespace = p_namespace
        ORDER BY md5(node_id || '164')
        LIMIT 50
    LOOP
        plain_ids := public.eq_graph_neighbor_ids_plain(p_namespace, s, 2, 500);
        age_ids := public.eq_graph_neighbor_ids_age(p_graph, s, 2, 500);
        IF plain_ids IS DISTINCT FROM age_ids THEN
            mismatches := mismatches + 1;
        END IF;
    END LOOP;
    RETURN mismatches;
END;
$$;

CREATE OR REPLACE FUNCTION public.eq_graph_mirror_node()
RETURNS trigger
LANGUAGE plpgsql AS $$
DECLARE
    props jsonb;
    nid text;
    ns text;
BEGIN
    IF TG_OP = 'DELETE' THEN
        props := ag_catalog.agtype_to_json(OLD.properties)::jsonb;
        nid := COALESCE(OLD.eq_node_id, props->>'node_id');
    ELSE
        props := ag_catalog.agtype_to_json(NEW.properties)::jsonb;
        nid := COALESCE(NEW.eq_node_id, props->>'node_id');
    END IF;
    SELECT namespace INTO ns FROM public.eq_graph_namespace WHERE graph_name = TG_TABLE_SCHEMA;
    IF ns IS NULL THEN
        ns := TG_TABLE_SCHEMA;
    END IF;
    IF nid IS NULL OR nid = '' THEN
        IF TG_OP = 'DELETE' THEN RETURN OLD; ELSE RETURN NEW; END IF;
    END IF;
    IF TG_OP = 'DELETE' THEN
        DELETE FROM public.eq_graph_nodes WHERE namespace = ns AND node_id = nid;
        DELETE FROM public.eq_graph_edges
         WHERE namespace = ns AND (source_id = nid OR target_id = nid);
        RETURN OLD;
    END IF;
    INSERT INTO public.eq_graph_nodes (
        namespace, node_id, tenant_id, workspace_id, entity_type, properties
    ) VALUES (
        ns,
        nid,
        COALESCE(props->>'tenant_id', ''),
        COALESCE(props->>'workspace_id', ''),
        NULLIF(props->>'entity_type', ''),
        props
    )
    ON CONFLICT (namespace, node_id) DO UPDATE SET
        tenant_id = EXCLUDED.tenant_id,
        workspace_id = EXCLUDED.workspace_id,
        entity_type = EXCLUDED.entity_type,
        properties = EXCLUDED.properties;
    RETURN NEW;
END;
$$;

CREATE OR REPLACE FUNCTION public.eq_graph_mirror_edge()
RETURNS trigger
LANGUAGE plpgsql AS $$
DECLARE
    props jsonb;
    ns text;
    src text;
    tgt text;
    rel text;
BEGIN
    IF TG_OP = 'DELETE' THEN
        props := ag_catalog.agtype_to_json(OLD.properties)::jsonb;
        src := COALESCE(OLD.eq_source_id, props->>'source_id');
        tgt := COALESCE(OLD.eq_target_id, props->>'target_id');
        rel := COALESCE(NULLIF(OLD.eq_rel_type, ''), NULLIF(props->>'relation_type', ''), '');
    ELSE
        props := ag_catalog.agtype_to_json(NEW.properties)::jsonb;
        src := COALESCE(NEW.eq_source_id, props->>'source_id');
        tgt := COALESCE(NEW.eq_target_id, props->>'target_id');
        rel := COALESCE(NULLIF(NEW.eq_rel_type, ''), NULLIF(props->>'relation_type', ''), '');
    END IF;
    SELECT namespace INTO ns FROM public.eq_graph_namespace WHERE graph_name = TG_TABLE_SCHEMA;
    IF ns IS NULL THEN
        ns := TG_TABLE_SCHEMA;
    END IF;
    IF src IS NULL OR tgt IS NULL THEN
        IF TG_OP = 'DELETE' THEN RETURN OLD; ELSE RETURN NEW; END IF;
    END IF;
    IF TG_OP = 'DELETE' THEN
        DELETE FROM public.eq_graph_edges
         WHERE namespace = ns AND source_id = src AND target_id = tgt AND rel_type = rel;
        RETURN OLD;
    END IF;
    INSERT INTO public.eq_graph_edges (
        namespace, source_id, target_id, rel_type, tenant_id, workspace_id, weight, properties
    ) VALUES (
        ns, src, tgt, rel,
        COALESCE(props->>'tenant_id', ''),
        COALESCE(props->>'workspace_id', ''),
        CASE
            WHEN COALESCE(props->>'weight', '') ~ '^-?[0-9]+(\.[0-9]+)?$'
            THEN (props->>'weight')::double precision
            ELSE NULL
        END,
        props
    )
    ON CONFLICT (namespace, source_id, target_id, rel_type) DO UPDATE SET
        tenant_id = EXCLUDED.tenant_id,
        workspace_id = EXCLUDED.workspace_id,
        weight = EXCLUDED.weight,
        properties = EXCLUDED.properties;
    RETURN NEW;
END;
$$;

CREATE OR REPLACE FUNCTION public.eq_graph_install_mirror_triggers(
    p_graph text,
    p_namespace text
) RETURNS void
LANGUAGE plpgsql AS $$
BEGIN
    IF p_graph !~ '^[A-Za-z_][A-Za-z0-9_]*$' THEN
        RAISE EXCEPTION 'refusing unsafe graph name';
    END IF;
    INSERT INTO public.eq_graph_namespace (graph_name, namespace)
    VALUES (p_graph, p_namespace)
    ON CONFLICT (graph_name) DO UPDATE SET namespace = EXCLUDED.namespace;

    IF to_regclass(format('%I."Node"', p_graph)) IS NOT NULL
       AND EXISTS (
           SELECT 1 FROM information_schema.columns
           WHERE table_schema = p_graph AND table_name = 'Node' AND column_name = 'eq_node_id'
       ) THEN
        EXECUTE format('DROP TRIGGER IF EXISTS eq_graph_mirror_node ON %I."Node"', p_graph);
        EXECUTE format(
            'CREATE TRIGGER eq_graph_mirror_node AFTER INSERT OR UPDATE OR DELETE ON %I."Node" FOR EACH ROW EXECUTE FUNCTION public.eq_graph_mirror_node()',
            p_graph
        );
    END IF;

    IF to_regclass(format('%I."EDGE"', p_graph)) IS NOT NULL
       AND EXISTS (
           SELECT 1 FROM information_schema.columns
           WHERE table_schema = p_graph AND table_name = 'EDGE' AND column_name = 'eq_source_id'
       ) THEN
        EXECUTE format('DROP TRIGGER IF EXISTS eq_graph_mirror_edge ON %I."EDGE"', p_graph);
        EXECUTE format(
            'CREATE TRIGGER eq_graph_mirror_edge AFTER INSERT OR UPDATE OR DELETE ON %I."EDGE" FOR EACH ROW EXECUTE FUNCTION public.eq_graph_mirror_edge()',
            p_graph
        );
    END IF;
END;
$$;

CREATE OR REPLACE FUNCTION public.eq_graph_install_plain_rls()
RETURNS void
LANGUAGE plpgsql AS $$
DECLARE
    t text;
    predicate text;
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'edgequake_tenant_access') THEN
        RETURN;
    END IF;
    IF to_regprocedure('public.current_tenant_id()') IS NULL THEN
        RETURN;
    END IF;
    predicate := 'tenant_id = public.current_tenant_id()::text AND (public.current_workspace_id() IS NULL OR workspace_id = public.current_workspace_id()::text)';
    FOREACH t IN ARRAY ARRAY['eq_graph_nodes', 'eq_graph_edges'] LOOP
        EXECUTE format('GRANT SELECT, INSERT, UPDATE, DELETE ON public.%I TO edgequake_tenant_access', t);
        EXECUTE format('ALTER TABLE public.%I ENABLE ROW LEVEL SECURITY', t);
        EXECUTE format('ALTER TABLE public.%I FORCE ROW LEVEL SECURITY', t);
        IF NOT EXISTS (
            SELECT 1 FROM pg_policies
            WHERE schemaname = 'public' AND tablename = t AND policyname = 'tenant_access_guard'
        ) THEN
            EXECUTE format(
                'CREATE POLICY tenant_access_allow ON public.%I FOR ALL TO edgequake_tenant_access USING (true) WITH CHECK (true)',
                t
            );
            EXECUTE format(
                'CREATE POLICY tenant_access_guard ON public.%I AS RESTRICTIVE FOR ALL TO PUBLIC USING (%s) WITH CHECK (%s)',
                t, predicate, predicate
            );
        END IF;
    END LOOP;
END;
$$;

SELECT public.eq_graph_install_plain_rls();

-- Seed namespace + triggers for graphs that already exist. Empty / no AGE → ready.
DO $$
DECLARE
    age_ns boolean;
    g record;
    ngraphs integer := 0;
    derived text;
BEGIN
    SELECT EXISTS (SELECT 1 FROM pg_namespace WHERE nspname = 'ag_catalog') INTO age_ns;
    IF age_ns THEN
        BEGIN
            FOR g IN EXECUTE 'SELECT name::text AS name FROM ag_catalog.ag_graph' LOOP
                ngraphs := ngraphs + 1;
                IF g.name LIKE 'eq\_eq\_%\_graph' ESCAPE '\' THEN
                    derived := substring(g.name FROM 7 FOR char_length(g.name) - 12);
                ELSE
                    derived := g.name;
                END IF;
                PERFORM public.eq_graph_install_mirror_triggers(g.name, derived);
            END LOOP;
        EXCEPTION WHEN undefined_table OR invalid_schema_name THEN
            age_ns := false;
            ngraphs := 0;
        END;
    END IF;

    IF NOT age_ns OR ngraphs = 0 THEN
        UPDATE public.eq_graph_cutover
           SET status = 'ready',
               age_absent = NOT age_ns,
               shadow_ok = true,
               backfill_complete = true,
               unbackfilled_graphs = 0,
               updated_at = now(),
               detail = jsonb_build_object('reason', 'greenfield_or_empty')
         WHERE id = 1;
    ELSE
        UPDATE public.eq_graph_cutover
           SET status = 'age',
               age_absent = false,
               shadow_ok = false,
               backfill_complete = false,
               unbackfilled_graphs = ngraphs,
               updated_at = now(),
               detail = jsonb_build_object('reason', 'awaiting_backfill', 'graphs', ngraphs)
         WHERE id = 1;
    END IF;
END $$;
