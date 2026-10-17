/**
 * SPEC-165 — browser client for the agents platform (`/api/agents/*` → agents `/v1/*`).
 * Same auth/scope plumbing as the rest of the app: buildHeaders + ensureSessionScope,
 * one silent refresh on 401.
 */
'use client';

import { buildHeaders, refreshAccessToken } from '@/lib/api/client';
import { ensureSessionScope } from '@/lib/api/session-scope';
import { useTenantStore } from '@/stores/use-tenant-store';
import { useQuery } from '@tanstack/react-query';
import { useEffect, useRef } from 'react';

export type ToolCategory = 'knowledge' | 'graph' | 'documents' | 'artifacts';

export interface Agent {
  id: string;
  /** `system` = the agent architect: runnable, hidden from galleries/pickers. */
  kind: 'builtin' | 'custom' | 'system';
  name: string;
  description: string;
  icon: string;
  instructions: string;
  model: string;
  tools: string[];
  starters: string[];
  temperature: number;
  owner_id: string | null;
  visibility: 'private' | 'workspace';
  workspace_id: string | null;
  created_at: string;
  updated_at: string;
  editable: boolean;
}

export type AgentInput = Pick<Agent, 'name' | 'description' | 'icon' | 'instructions' | 'model' | 'tools' | 'starters' | 'temperature' | 'visibility'>;

export interface AgentTool {
  id: string;
  name: string;
  description: string;
  category: ToolCategory;
}

export interface AgentModel {
  id: string;
  label: string;
  default: boolean;
}

export interface Conversation {
  id: string;
  agent_id: string;
  title: string;
  created_at: string;
  updated_at: string;
}

export interface ToolActivity {
  id: string;
  name: string;
  status: 'running' | 'done' | 'error';
  args?: Record<string, unknown>;
}

export interface ArtifactRef {
  name: string;
  mime_type: string;
  version: number;
}

/** One `propose_agent_spec` result of the agent architect (SSE `agent_spec`). */
export interface AgentSpecProposal {
  spec: AgentInput;
  rationale: string;
  tests: string[];
}

export const ARCHITECT_ID = 'agent_architect';

/** Builder form state sent with architect messages (data for the model, not instructions). */
export interface ArchitectContext {
  draft_agent?: Partial<AgentInput>;
  editing_agent_id?: string;
}

/** One sub-question of an investigation plan (SSE `plan`). */
export interface PlanItem {
  id: string;
  label: string;
}

/** Progress of one step of a run (SSE `step`): `plan`, `r1..rN` (searches), `synth`, `graph`. */
export interface RunStep {
  id: string;
  label: string;
  status: 'running' | 'done' | 'error';
  detail?: string;
  sources_count?: number;
  /** Milliseconds since the start of the run when this status was reached. */
  elapsed_ms?: number;
}

/** A passage the answer is grounded on (SSE `sources`). */
export interface SourceRef {
  document_id: string;
  title: string;
  /** Clean excerpt (~280 chars) around the query terms. */
  snippet: string;
  /** Heading path of the passage in its document. */
  section?: string;
  /** The whole passage (markdown). */
  content?: string;
  score: number;
  chunk_id?: string;
  /** Citation number used in the answer ([n]). */
  ref?: number;
  /** Investigation steps (`r1`…) that found this passage. */
  steps?: string[];
}

export interface Message {
  id: string;
  role: 'user' | 'assistant';
  /** First agent of the turn (the one the conversation talks to). */
  author: string;
  /** Every agent that contributed to this turn, in order (routing → specialist → sub-agents). */
  agents?: string[];
  content: string;
  tools: ToolActivity[];
  artifacts: ArtifactRef[];
  created_at: string;
  /** Architect bubbles only: the spec proposed in this turn. */
  agent_spec?: AgentSpecProposal;
  plan?: PlanItem[];
  steps?: RunStep[];
  sources?: SourceRef[];
}

export interface ConversationDetail extends Conversation {
  messages: Message[];
}

export class AgentsApiError extends Error {
  constructor(
    message: string,
    readonly status: number,
  ) {
    super(message);
  }
}

/** Raw fetch through the proxy with the session's bearer + tenant/workspace headers. */
export async function agentsFetch(path: string, init: RequestInit = {}): Promise<Response> {
  const send = async () => {
    await ensureSessionScope();
    return fetch(`/api/agents${path}`, { ...init, headers: buildHeaders(init.headers, init.body) });
  };
  let res = await send();
  if (res.status === 401 && (await refreshAccessToken())) res = await send();
  return res;
}

export async function agentsJson<T>(path: string, init: RequestInit = {}): Promise<T> {
  const res = await agentsFetch(path, init);
  if (!res.ok) {
    const body = (await res.json().catch(() => ({}))) as { error?: string };
    throw new AgentsApiError(body.error || `HTTP ${res.status}`, res.status);
  }
  return (res.status === 204 ? undefined : await res.json()) as T;
}

const send = (method: string, body?: unknown): RequestInit => ({
  method,
  body: body === undefined ? undefined : JSON.stringify(body),
});

export const agentsApi = {
  listAgents: () => agentsJson<{ items: Agent[] }>('/agents?include=system').then((r) => r.items),
  getAgent: (id: string) => agentsJson<Agent>(`/agents/${encodeURIComponent(id)}`),
  createAgent: (input: AgentInput) => agentsJson<Agent>('/agents', send('POST', input)),
  updateAgent: (id: string, input: AgentInput) => agentsJson<Agent>(`/agents/${encodeURIComponent(id)}`, send('PUT', input)),
  deleteAgent: (id: string) => agentsJson<void>(`/agents/${encodeURIComponent(id)}`, send('DELETE')),
  duplicateAgent: (id: string) => agentsJson<Agent>(`/agents/${encodeURIComponent(id)}/duplicate`, send('POST', {})),
  listTools: () => agentsJson<{ items: AgentTool[] }>('/tools').then((r) => r.items),
  listModels: () => agentsJson<{ items: AgentModel[] }>('/models').then((r) => r.items),
  listConversations: (q = '') =>
    agentsJson<{ items: Conversation[] }>(`/conversations?limit=50${q ? `&q=${encodeURIComponent(q)}` : ''}`).then((r) => r.items),
  getConversation: (id: string) => agentsJson<ConversationDetail>(`/conversations/${encodeURIComponent(id)}`),
  createConversation: (agentId: string) => agentsJson<Conversation>('/conversations', send('POST', { agent_id: agentId })),
  renameConversation: (id: string, title: string) =>
    agentsJson<Conversation>(`/conversations/${encodeURIComponent(id)}`, send('PATCH', { title })),
  deleteConversation: (id: string) => agentsJson<void>(`/conversations/${encodeURIComponent(id)}`, send('DELETE')),
  artifactUrl: (conversationId: string, a: ArtifactRef) =>
    `/conversations/${encodeURIComponent(conversationId)}/artifacts/${encodeURIComponent(a.name)}?version=${a.version}`,
};

/** Query keys are workspace-scoped: workspace-visible agents depend on the current scope. */
export const agentsKeys = {
  all: ['agents-platform'] as const,
  agents: (ws: string | null) => ['agents-platform', ws, 'agents'] as const,
  agent: (ws: string | null, id: string) => ['agents-platform', ws, 'agent', id] as const,
  tools: ['agents-platform', 'tools'] as const,
  models: ['agents-platform', 'models'] as const,
  conversations: (ws: string | null) => ['agents-platform', ws, 'conversations'] as const,
  conversation: (id: string) => ['agents-platform', 'conversation', id] as const,
};

export function useWorkspaceId(): string | null {
  return useTenantStore((s) => s.selectedWorkspaceId);
}

const withoutSystem = (items: Agent[]) => items.filter((a) => a.kind !== 'system');

/** Agents for pickers and galleries (system agents excluded). */
export function useAgents() {
  const ws = useWorkspaceId();
  return useQuery({ queryKey: agentsKeys.agents(ws), queryFn: agentsApi.listAgents, staleTime: 30_000, select: withoutSystem });
}

/** Every agent incl. system ones (name/icon lookups). Same cache entry as useAgents. */
export function useAllAgents() {
  const ws = useWorkspaceId();
  return useQuery({ queryKey: agentsKeys.agents(ws), queryFn: agentsApi.listAgents, staleTime: 30_000 });
}

/**
 * Conversations live in the workspace they were created in: run `onLeave` when the
 * user switches to another workspace (not on the initial hydration from null).
 */
export function useOnWorkspaceChange(onLeave: () => void) {
  const ws = useWorkspaceId();
  const first = useRef(ws);
  const cb = useRef(onLeave);
  useEffect(() => {
    cb.current = onLeave;
  });
  useEffect(() => {
    if (first.current === null) first.current = ws;
    else if (ws !== first.current) {
      first.current = ws;
      cb.current();
    }
  }, [ws]);
}

export function useAgentTools() {
  return useQuery({ queryKey: agentsKeys.tools, queryFn: agentsApi.listTools, staleTime: 5 * 60_000 });
}

export function useAgentModels() {
  return useQuery({ queryKey: agentsKeys.models, queryFn: agentsApi.listModels, staleTime: 5 * 60_000 });
}

/** `q` filters server-side; the unfiltered list is the one other views patch optimistically. */
export function useConversations(q = '') {
  const ws = useWorkspaceId();
  return useQuery({
    queryKey: [...agentsKeys.conversations(ws), q],
    queryFn: () => agentsApi.listConversations(q),
    staleTime: 15_000,
  });
}

export function useConversation(id: string) {
  return useQuery({ queryKey: agentsKeys.conversation(id), queryFn: () => agentsApi.getConversation(id), retry: 1 });
}
