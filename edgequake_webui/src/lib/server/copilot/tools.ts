/**
 * EdgeQuake Copilot — server-side tools.
 *
 * Every tool call goes to the EdgeQuake API with the *caller's* bearer token and scope
 * headers, so the backend's membership/tenant isolation is the only authority: the
 * copilot can never see or change more than the signed-in user can.
 *
 * Graph mutations are never executed by the model directly: `propose_graph_changes`
 * returns a proposal the user must approve in the UI (prompt-injection guard — document
 * text cannot make the copilot edit the graph on its own).
 */

export interface CallerScope {
  authorization: string | null;
  tenantId: string | null;
  workspaceId: string | null;
  /** Aborts backend calls when the browser disconnects. */
  signal?: AbortSignal;
}

export type GraphOperation =
  | { op: "merge_entities"; sources: string[]; target: string; strategy?: "prefer_target" | "prefer_source" | "merge" }
  | { op: "create_entity"; name: string; type: string; description: string }
  | { op: "update_entity"; name: string; type?: string; description?: string }
  | { op: "delete_entity"; name: string }
  | { op: "create_relationship"; source: string; target: string; keywords: string; description: string; weight?: number }
  | { op: "update_relationship"; id: string; keywords?: string; description?: string; weight?: number }
  | { op: "delete_relationship"; id: string };

export interface OperationResult {
  index: number;
  ok: boolean;
  summary: string;
  /** Entity names touched (for graph highlight after apply). */
  entities: string[];
}

const MAX_TOOL_RESULT_CHARS = 7000;

export function backendBaseUrl(): string {
  // Server-to-server: prefer the private network address (e.g. http://api:8080 in Compose)
  // over the public origin, which may sit behind TLS the Node runtime does not trust.
  const raw =
    process.env.EDGEQUAKE_INTERNAL_API_URL?.trim() ||
    process.env.EDGEQUAKE_API_URL?.trim() ||
    process.env.NEXT_PUBLIC_API_URL?.trim() ||
    "http://127.0.0.1:8090";
  return raw.replace(/\/$/, "");
}

export class BackendError extends Error {
  constructor(
    public status: number,
    message: string,
  ) {
    super(message);
  }
}

async function call<T = unknown>(
  scope: CallerScope,
  method: string,
  path: string,
  body?: unknown,
): Promise<T> {
  const headers: Record<string, string> = { "X-Edgequake-Client": "copilot" };
  if (scope.authorization) headers.Authorization = scope.authorization;
  if (scope.tenantId) headers["X-Tenant-ID"] = scope.tenantId;
  if (scope.workspaceId) headers["X-Workspace-ID"] = scope.workspaceId;
  if (body !== undefined) headers["Content-Type"] = "application/json";
  const res = await fetch(`${backendBaseUrl()}/api/v1${path}`, {
    method,
    headers,
    body: body === undefined ? undefined : JSON.stringify(body),
    signal: scope.signal
      ? AbortSignal.any([scope.signal, AbortSignal.timeout(120_000)])
      : AbortSignal.timeout(120_000),
  });
  const text = await res.text();
  if (!res.ok) {
    let message = text.slice(0, 400);
    try {
      const parsed = JSON.parse(text) as { message?: string; details?: { diagnostics?: { reason?: string } } };
      message = [parsed.message, parsed.details?.diagnostics?.reason].filter(Boolean).join(": ") || message;
    } catch {
      /* keep raw */
    }
    throw new BackendError(res.status, message || res.statusText);
  }
  return (text ? JSON.parse(text) : {}) as T;
}

/** Path segment: encoded, and never `.`/`..` (URL parsing would climb to another route). */
const enc = (segment: string) => {
  const trimmed = segment.trim();
  if (!trimmed || trimmed === "." || trimmed === "..") throw new Error(`Invalid identifier: "${segment}"`);
  return encodeURIComponent(trimmed);
};

/** Names go to the backend verbatim: it owns canonical normalization + exact lookup. */
const entityRef = (name: string) => name.trim();

function clip(value: unknown): string {
  const text = typeof value === "string" ? value : JSON.stringify(value);
  return text.length > MAX_TOOL_RESULT_CHARS
    ? `${text.slice(0, MAX_TOOL_RESULT_CHARS)}… [truncated]`
    : text;
}

function short(text: unknown, max = 280): string | undefined {
  if (typeof text !== "string") return undefined;
  return text.length > max ? `${text.slice(0, max)}…` : text;
}

// ---------------------------------------------------------------------------
// Tool schemas (OpenAI function-calling format)
// ---------------------------------------------------------------------------

const operationSchema = {
  type: "object",
  properties: {
    op: {
      type: "string",
      enum: [
        "merge_entities",
        "create_entity",
        "update_entity",
        "delete_entity",
        "create_relationship",
        "update_relationship",
        "delete_relationship",
      ],
    },
    sources: { type: "array", items: { type: "string" }, description: "merge_entities: entities folded into target" },
    target: { type: "string", description: "merge_entities: surviving entity; create_relationship: target entity" },
    strategy: { type: "string", enum: ["prefer_target", "prefer_source", "merge"] },
    name: { type: "string", description: "entity name (create/update/delete_entity)" },
    type: { type: "string", description: "entity type, e.g. PERSON, ORGANIZATION, CONCEPT" },
    description: { type: "string" },
    source: { type: "string", description: "create_relationship: source entity" },
    keywords: { type: "string", description: "relationship keywords; first keyword becomes the relation type" },
    weight: { type: "number" },
    id: { type: "string", description: "relationship id exactly as returned by a read tool" },
  },
  required: ["op"],
};

export const TOOL_DEFINITIONS = [
  {
    type: "function",
    function: {
      name: "search_graph",
      description: "Find entities in the current workspace's knowledge graph by name/keyword.",
      parameters: {
        type: "object",
        properties: {
          query: { type: "string" },
          limit: { type: "integer", minimum: 1, maximum: 50 },
          entity_type: { type: "string" },
        },
        required: ["query"],
      },
    },
  },
  {
    type: "function",
    function: {
      name: "get_entity",
      description: "Full details of one entity: description, type, incoming/outgoing relationships, document references.",
      parameters: { type: "object", properties: { name: { type: "string" } }, required: ["name"] },
    },
  },
  {
    type: "function",
    function: {
      name: "get_neighborhood",
      description: "Entities and relationships (with relationship ids) around an entity, up to depth 3.",
      parameters: {
        type: "object",
        properties: { name: { type: "string" }, depth: { type: "integer", minimum: 1, maximum: 3 } },
        required: ["name"],
      },
    },
  },
  {
    type: "function",
    function: {
      name: "get_relationship",
      description: "Details of one relationship by id.",
      parameters: { type: "object", properties: { id: { type: "string" } }, required: ["id"] },
    },
  },
  {
    type: "function",
    function: {
      name: "graph_overview",
      description: "Most connected entities of the workspace graph plus totals. Use for 'what is in my graph' questions.",
      parameters: {
        type: "object",
        properties: { max_nodes: { type: "integer", minimum: 10, maximum: 200 } },
      },
    },
  },
  {
    type: "function",
    function: {
      name: "list_documents",
      description: "List documents in the workspace (optionally filter by file-name pattern or status).",
      parameters: {
        type: "object",
        properties: {
          pattern: { type: "string" },
          status: { type: "string", enum: ["pending", "processing", "completed", "failed", "partial_failure"] },
          page_size: { type: "integer", minimum: 1, maximum: 50 },
        },
      },
    },
  },
  {
    type: "function",
    function: {
      name: "get_document",
      description: "Metadata and a content excerpt of one document by id.",
      parameters: { type: "object", properties: { id: { type: "string" } }, required: ["id"] },
    },
  },
  {
    type: "function",
    function: {
      name: "ask_knowledge_base",
      description:
        "Answer a question from the workspace documents with EdgeQuake RAG (returns answer + sources). Use for content questions.",
      parameters: {
        type: "object",
        properties: {
          question: { type: "string" },
          mode: { type: "string", enum: ["hybrid", "local", "global", "naive", "mix"] },
        },
        required: ["question"],
      },
    },
  },
  {
    type: "function",
    function: {
      name: "workspace_status",
      description: "Workspace statistics (documents, entities, relationships) and ingestion pipeline status.",
      parameters: { type: "object", properties: {} },
    },
  },
  {
    type: "function",
    function: {
      name: "propose_graph_changes",
      description:
        "Propose knowledge-graph edits (merge, split, regroup, rename, retype, create/delete entities and relationships). " +
        "Nothing is applied until the user clicks Approve. Splitting a relationship = delete it and create the better " +
        "relationships/intermediate entity. Splitting an entity = create the new entities, recreate its relationships on " +
        "the right one, then delete the original. Operations run in order.",
      parameters: {
        type: "object",
        properties: {
          title: { type: "string", description: "short imperative summary shown on the approval card" },
          rationale: { type: "string", description: "why these changes improve the graph" },
          operations: { type: "array", items: operationSchema, minItems: 1, maxItems: 40 },
        },
        required: ["title", "operations"],
      },
    },
  },
  {
    type: "function",
    function: {
      name: "navigate",
      description:
        "Open a page of the EdgeQuake app for the user: /, /documents, /documents/{id}, /query, /graph, /knowledge, /workspace, /pipeline, /costs, /settings.",
      parameters: { type: "object", properties: { path: { type: "string" } }, required: ["path"] },
    },
  },
  {
    type: "function",
    function: {
      name: "highlight_nodes",
      description: "Show and highlight entities on the user's graph view (opens the graph page if needed).",
      parameters: {
        type: "object",
        properties: { names: { type: "array", items: { type: "string" }, minItems: 1 } },
        required: ["names"],
      },
    },
  },
] as const;

export const TOOL_LABELS: Record<string, string> = {
  search_graph: "Searching the graph",
  get_entity: "Reading entity",
  get_neighborhood: "Exploring neighborhood",
  get_relationship: "Reading relationship",
  graph_overview: "Scanning the graph",
  list_documents: "Listing documents",
  get_document: "Reading document",
  ask_knowledge_base: "Querying the knowledge base",
  workspace_status: "Checking workspace",
  propose_graph_changes: "Preparing changes",
  navigate: "Opening page",
  highlight_nodes: "Highlighting on graph",
};

// ---------------------------------------------------------------------------
// Read tools
// ---------------------------------------------------------------------------

type Args = Record<string, unknown>;
const str = (v: unknown) => (typeof v === "string" ? v : "");
const num = (v: unknown, d: number, max: number) =>
  Math.min(typeof v === "number" && Number.isFinite(v) ? Math.max(1, Math.floor(v)) : d, max);

interface GraphNode {
  id: string;
  label?: string;
  node_type?: string;
  entity_type?: string;
  description?: string;
  degree?: number | { total?: number };
}
const degreeOf = (n: GraphNode) => (typeof n.degree === "number" ? n.degree : (n.degree?.total ?? 0));
const slimNode = (n: GraphNode) => ({
  name: n.label ?? n.id,
  type: n.node_type ?? n.entity_type,
  degree: degreeOf(n),
  description: short(n.description, 160),
});

export async function runReadTool(name: string, args: Args, scope: CallerScope): Promise<string> {
  switch (name) {
    case "search_graph": {
      const q = new URLSearchParams({ q: str(args.query), limit: String(num(args.limit, 10, 50)) });
      if (str(args.entity_type)) q.set("entity_type", str(args.entity_type));
      const data = await call<{ nodes: GraphNode[]; total_matches?: number }>(scope, "GET", `/graph/nodes/search?${q}`);
      return clip({ total_matches: data.total_matches, entities: data.nodes.map(slimNode) });
    }
    case "get_entity":
      return clip(await call(scope, "GET", `/graph/entities/${enc(entityRef(str(args.name)))}`));
    case "get_neighborhood": {
      const depth = num(args.depth, 1, 3);
      const data = await call<{ nodes: GraphNode[]; edges: unknown[] }>(
        scope,
        "GET",
        `/graph/entities/${enc(entityRef(str(args.name)))}/neighborhood?depth=${depth}`,
      );
      return clip({ entities: data.nodes.map(slimNode), relationships: data.edges });
    }
    case "get_relationship":
      return clip(await call(scope, "GET", `/graph/relationships/${enc(str(args.id))}`));
    case "graph_overview": {
      const max = num(args.max_nodes, 80, 200);
      const data = await call<{ nodes: GraphNode[]; total_nodes?: number; total_edges?: number }>(
        scope,
        "GET",
        `/graph?max_nodes=${max}&depth=1`,
      );
      const top = [...data.nodes].sort((a, b) => degreeOf(b) - degreeOf(a)).slice(0, 40).map(slimNode);
      return clip({ total_entities: data.total_nodes, total_relationships: data.total_edges, top_entities: top });
    }
    case "list_documents": {
      const q = new URLSearchParams({ page: "1", page_size: String(num(args.page_size, 20, 50)) });
      if (str(args.pattern)) q.set("document_pattern", str(args.pattern));
      if (str(args.status)) q.set("status", str(args.status));
      const data = await call<{
        documents: Array<Record<string, unknown>>;
        total?: number;
        status_counts?: unknown;
      }>(scope, "GET", `/documents?${q}`);
      return clip({
        total: data.total,
        status_counts: data.status_counts,
        documents: data.documents.map((d) => ({
          id: d.id,
          title: d.title ?? d.file_name,
          status: d.status,
          entities: d.entity_count,
          chunks: d.chunk_count,
          created_at: d.created_at,
        })),
      });
    }
    case "get_document": {
      const d = await call<Record<string, unknown>>(scope, "GET", `/documents/${enc(str(args.id))}`);
      return clip({ ...d, content: short(d.content, 3500), lineage: undefined });
    }
    case "ask_knowledge_base": {
      const data = await call<{ answer?: string; sources?: Array<Record<string, unknown>> }>(scope, "POST", "/query", {
        query: str(args.question),
        mode: str(args.mode) || "hybrid",
        include_references: true,
      });
      return clip({
        answer: data.answer,
        sources: (data.sources ?? []).slice(0, 8).map((s) => ({
          document_id: s.document_id,
          title: s.title ?? s.file_name ?? s.source_type,
          snippet: short(s.snippet ?? s.content, 220),
        })),
      });
    }
    case "workspace_status": {
      const [stats, pipeline] = await Promise.all([
        scope.workspaceId
          ? call(scope, "GET", `/workspaces/${enc(scope.workspaceId)}/stats`).catch((e: Error) => ({ error: e.message }))
          : Promise.resolve(null),
        call(scope, "GET", "/pipeline/status").catch((e: Error) => ({ error: e.message })),
      ]);
      return clip({ stats, pipeline });
    }
    default:
      throw new Error(`Unknown tool ${name}`);
  }
}

// ---------------------------------------------------------------------------
// Approved graph operations
// ---------------------------------------------------------------------------

/** Exact, human-readable effect of an operation (approval card: nothing hidden). */
export function describeOperation(op: GraphOperation): string {
  const quote = (text: string | undefined) => (text ? ` “${short(text, 160)}”` : "");
  switch (op.op) {
    case "merge_entities":
      return `Merge ${op.sources.join(", ")} → ${op.target} (${op.strategy ?? "merge"} descriptions)`;
    case "create_entity":
      return `Create entity ${op.name} (${op.type})${quote(op.description)}`;
    case "update_entity":
      return `Update entity ${op.name}${op.type ? ` · type → ${op.type}` : ""}${op.description ? ` · description →${quote(op.description)}` : ""}`;
    case "delete_entity":
      return `Delete entity ${op.name} and all its relationships`;
    case "create_relationship":
      return `Link ${op.source} —[${op.keywords.split(",")[0]?.trim()}]→ ${op.target}${quote(op.description)}`;
    case "update_relationship":
      return `Update relationship ${op.id}${op.keywords ? ` · keywords → ${op.keywords}` : ""}${op.weight !== undefined ? ` · weight → ${op.weight}` : ""}${op.description ? ` · description →${quote(op.description)}` : ""}`;
    case "delete_relationship":
      return `Remove relationship ${op.id}`;
  }
}

/** Validate model output into typed operations (drops malformed entries). */
export function parseOperations(raw: unknown): GraphOperation[] {
  if (!Array.isArray(raw)) return [];
  const out: GraphOperation[] = [];
  for (const item of raw) {
    if (!item || typeof item !== "object") continue;
    const o = item as Args;
    switch (o.op) {
      case "merge_entities": {
        const sources = Array.isArray(o.sources) ? o.sources.filter((s): s is string => typeof s === "string" && !!s) : [];
        if (sources.length && str(o.target)) {
          const strategy = ["prefer_target", "prefer_source", "merge"].includes(str(o.strategy))
            ? (str(o.strategy) as "prefer_target" | "prefer_source" | "merge")
            : "merge";
          out.push({ op: "merge_entities", sources, target: str(o.target), strategy });
        }
        break;
      }
      case "create_entity":
        if (str(o.name) && str(o.type))
          out.push({ op: "create_entity", name: str(o.name), type: str(o.type), description: str(o.description) });
        break;
      case "update_entity":
        if (str(o.name) && (str(o.type) || str(o.description)))
          out.push({
            op: "update_entity",
            name: str(o.name),
            type: str(o.type) || undefined,
            description: str(o.description) || undefined,
          });
        break;
      case "delete_entity":
        if (str(o.name)) out.push({ op: "delete_entity", name: str(o.name) });
        break;
      case "create_relationship":
        if (str(o.source) && str(o.target) && str(o.keywords))
          out.push({
            op: "create_relationship",
            source: str(o.source),
            target: str(o.target),
            keywords: str(o.keywords),
            description: str(o.description) || str(o.keywords),
            weight: typeof o.weight === "number" ? o.weight : undefined,
          });
        break;
      case "update_relationship":
        if (str(o.id))
          out.push({
            op: "update_relationship",
            id: str(o.id),
            keywords: str(o.keywords) || undefined,
            description: str(o.description) || undefined,
            weight: typeof o.weight === "number" ? o.weight : undefined,
          });
        break;
      case "delete_relationship":
        if (str(o.id)) out.push({ op: "delete_relationship", id: str(o.id) });
        break;
    }
  }
  return out;
}

const COPILOT_SOURCE = "copilot";

async function applyOne(op: GraphOperation, scope: CallerScope): Promise<string[]> {
  const n = entityRef;
  switch (op.op) {
    case "merge_entities": {
      for (const source of op.sources) {
        if (n(source).toUpperCase() === n(op.target).toUpperCase()) continue;
        await call(scope, "POST", "/graph/entities/merge", {
          source_entity: n(source),
          target_entity: n(op.target),
          merge_strategy: op.strategy ?? "merge",
        });
      }
      return [n(op.target)];
    }
    case "create_entity":
      await call(scope, "POST", "/graph/entities", {
        entity_name: n(op.name),
        entity_type: op.type.toUpperCase(),
        description: op.description,
        source_id: COPILOT_SOURCE,
      });
      return [n(op.name)];
    case "update_entity":
      await call(scope, "PUT", `/graph/entities/${enc(n(op.name))}`, {
        entity_type: op.type?.toUpperCase(),
        description: op.description,
      });
      return [n(op.name)];
    case "delete_entity":
      await call(scope, "DELETE", `/graph/entities/${enc(n(op.name))}?confirm=true&delete_relationships=true`);
      return [];
    case "create_relationship":
      await call(scope, "POST", "/graph/relationships", {
        src_id: n(op.source),
        tgt_id: n(op.target),
        keywords: op.keywords,
        description: op.description,
        weight: op.weight ?? 1.0,
        source_id: COPILOT_SOURCE,
      });
      return [n(op.source), n(op.target)];
    case "update_relationship":
      await call(scope, "PUT", `/graph/relationships/${enc(op.id)}`, {
        keywords: op.keywords,
        description: op.description,
        weight: op.weight,
      });
      return [];
    case "delete_relationship":
      await call(scope, "DELETE", `/graph/relationships/${enc(op.id)}`);
      return [];
  }
}

/**
 * Apply approved operations in order. Stops at the first failure so a split never
 * deletes the original after its replacement failed to be created.
 */
export async function applyOperations(ops: GraphOperation[], scope: CallerScope): Promise<OperationResult[]> {
  const results: OperationResult[] = [];
  for (const [index, op] of ops.entries()) {
    try {
      const entities = await applyOne(op, scope);
      results.push({ index, ok: true, summary: describeOperation(op), entities });
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      results.push({ index, ok: false, summary: `${describeOperation(op)} — failed: ${message}`, entities: [] });
      for (const [rest, skipped] of ops.slice(index + 1).entries()) {
        results.push({
          index: index + 1 + rest,
          ok: false,
          summary: `${describeOperation(skipped)} — skipped (previous step failed)`,
          entities: [],
        });
      }
      break;
    }
  }
  return results;
}
