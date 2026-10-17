/**
 * SPEC-165 — pure chat logic (unit-tested): SSE frame parsing, the stream event
 * reducer that turns `/messages` events into ONE assistant message per turn, and the
 * sidebar's date grouping.
 */
import type { AgentSpecProposal, ArtifactRef, Conversation, Message, PlanItem, RunStep, SourceRef } from './api';

export type StreamEvent =
  | { type: 'message_start'; author: string }
  | { type: 'delta'; author?: string; text: string }
  | { type: 'tool_start'; id: string; author?: string; name: string; args?: Record<string, unknown> }
  | { type: 'tool_end'; id: string; ok: boolean }
  | ({ type: 'artifact' } & ArtifactRef)
  | ({ type: 'agent_spec' } & AgentSpecProposal)
  | { type: 'plan'; items: PlanItem[] }
  | ({ type: 'step' } & RunStep)
  | { type: 'sources'; items: SourceRef[] }
  | { type: 'title'; title: string }
  | { type: 'error'; message: string }
  | { type: 'done' };

export interface RunState {
  /** Live messages of this run: the user's message, then the turn's single assistant message. */
  messages: Message[];
  /** Author of the last streamed text (a new author starts a new paragraph). */
  textAuthor?: string;
  /** Date.now() when the run started (live elapsed time). */
  startedAt?: number;
  title: string | null;
  error: string | null;
  done: boolean;
}

/** Split an SSE buffer into complete `data:` payloads; returns the unconsumed tail. */
export function parseSseFrames(buffer: string): { events: StreamEvent[]; rest: string } {
  const frames = buffer.replace(/\r\n/g, '\n').split('\n\n');
  const rest = frames.pop() ?? '';
  const events: StreamEvent[] = [];
  for (const frame of frames) {
    const data = frame
      .split('\n')
      .filter((l) => l.startsWith('data:'))
      .map((l) => l.slice(5).replace(/^ /, ''))
      .join('\n');
    if (!data) continue;
    try {
      const parsed = JSON.parse(data) as StreamEvent;
      if (parsed && typeof parsed.type === 'string') events.push(parsed);
    } catch {
      /* ignore malformed frame */
    }
  }
  return { events, rest };
}

let seq = 0;
const localId = () => `live-${Date.now().toString(36)}-${(seq++).toString(36)}`;

function newAssistant(author: string): Message {
  return { id: localId(), role: 'assistant', author, agents: [author], content: '', tools: [], artifacts: [], created_at: new Date().toISOString() };
}

export function userMessage(content: string): Message {
  return { id: localId(), role: 'user', author: 'user', content, tools: [], artifacts: [], created_at: new Date().toISOString() };
}

export function initialRun(content: string): RunState {
  return { messages: [userMessage(content)], title: null, error: null, done: false, startedAt: Date.now() };
}

/**
 * The turn's single assistant message, created on first sight. Every (sub-)agent of the
 * turn — coordinator, specialist, planner, synthesizer — writes into it; `agents` keeps
 * who contributed, in order.
 */
function withAssistant(messages: Message[], author: string | undefined, update: (m: Message) => Message): Message[] {
  const last = messages[messages.length - 1];
  if (last?.role !== 'assistant') return [...messages, update(newAssistant(author || 'assistant'))];
  const agents = last.agents ?? [last.author];
  const seen = !author || agents.includes(author) ? last : { ...last, agents: [...agents, author] };
  return [...messages.slice(0, -1), update(seen)];
}

/** Upsert a step by id (later events refine earlier ones). */
export function upsertStep(steps: RunStep[] | undefined, step: RunStep): RunStep[] {
  const list = steps ?? [];
  const i = list.findIndex((s) => s.id === step.id);
  return i < 0 ? [...list, step] : list.map((s, j) => (j === i ? { ...s, ...step } : s));
}

const sourceKey = (s: SourceRef) => `${s.chunk_id ?? ''}|${s.document_id}`;

/** Append sources not seen yet (same chunk of the same document = same source). */
export function mergeSources(have: SourceRef[] | undefined, items: SourceRef[]): SourceRef[] {
  const out = [...(have ?? [])];
  const keys = new Set(out.map(sourceKey));
  for (const s of items) {
    if (!keys.has(sourceKey(s))) {
      keys.add(sourceKey(s));
      out.push(s);
    }
  }
  return out;
}

export function reduceRun(state: RunState, event: StreamEvent): RunState {
  switch (event.type) {
    case 'message_start':
      return { ...state, messages: withAssistant(state.messages, event.author, (m) => m) };
    case 'delta': {
      const author = event.author ?? state.textAuthor;
      const sep = state.textAuthor && author !== state.textAuthor ? '\n\n' : '';
      return {
        ...state,
        textAuthor: author,
        messages: withAssistant(state.messages, event.author, (m) => ({ ...m, content: m.content ? m.content + sep + event.text : event.text })),
      };
    }
    case 'tool_start':
      return {
        ...state,
        messages: withAssistant(state.messages, event.author, (m) => ({
          ...m,
          tools: [...m.tools, { id: event.id, name: event.name, status: 'running', args: event.args }],
        })),
      };
    case 'tool_end':
      return {
        ...state,
        messages: state.messages.map((m) =>
          m.tools.some((t) => t.id === event.id)
            ? { ...m, tools: m.tools.map((t) => (t.id === event.id ? { ...t, status: event.ok ? 'done' : 'error' } : t)) }
            : m,
        ),
      };
    case 'artifact': {
      const ref: ArtifactRef = { name: event.name, mime_type: event.mime_type, version: event.version };
      return {
        ...state,
        messages: withAssistant(state.messages, undefined, (m) => ({
          ...m,
          // A new version of the same artifact replaces the card.
          artifacts: [...m.artifacts.filter((a) => a.name !== ref.name), ref],
        })),
      };
    }
    case 'agent_spec': {
      const proposal: AgentSpecProposal = { spec: event.spec, rationale: event.rationale, tests: event.tests ?? [] };
      return { ...state, messages: withAssistant(state.messages, undefined, (m) => ({ ...m, agent_spec: proposal })) };
    }
    case 'plan':
      return { ...state, messages: withAssistant(state.messages, undefined, (m) => ({ ...m, plan: event.items })) };
    case 'step': {
      const step: RunStep = { id: event.id, label: event.label, status: event.status, detail: event.detail, sources_count: event.sources_count, elapsed_ms: event.elapsed_ms };
      for (const k of Object.keys(step) as (keyof RunStep)[]) if (step[k] === undefined) delete step[k];
      return { ...state, messages: withAssistant(state.messages, undefined, (m) => ({ ...m, steps: upsertStep(m.steps, step) })) };
    }
    case 'sources':
      return { ...state, messages: withAssistant(state.messages, undefined, (m) => ({ ...m, sources: mergeSources(m.sources, event.items) })) };
    case 'title':
      return { ...state, title: event.title };
    case 'error':
      return { ...state, error: event.message };
    case 'done':
      return { ...state, done: true };
    default:
      return state;
  }
}

/** Run finished (done, error or abort): tools still "running" can no longer complete. */
export function settleRun(state: RunState): RunState {
  const settle = <T extends { status: string }>(items: T[] | undefined) =>
    items?.some((t) => t.status === 'running') ? items.map((t) => (t.status === 'running' ? { ...t, status: 'error' } : t)) : items;
  return {
    ...state,
    done: true,
    messages: state.messages.map((m) =>
      m.tools.some((t) => t.status === 'running') || m.steps?.some((s) => s.status === 'running')
        ? { ...m, tools: settle(m.tools) ?? [], steps: settle(m.steps) }
        : m,
    ),
  };
}

export type DateGroup = 'today' | 'yesterday' | 'previous7' | 'older';

/** Group conversations (already newest first) by `updated_at` relative to local midnight. */
export function groupByDate(items: Conversation[], now: Date = new Date()): { group: DateGroup; items: Conversation[] }[] {
  const midnight = new Date(now.getFullYear(), now.getMonth(), now.getDate()).getTime();
  const day = 86_400_000;
  const order: DateGroup[] = ['today', 'yesterday', 'previous7', 'older'];
  const buckets = new Map<DateGroup, Conversation[]>();
  for (const c of items) {
    const t = new Date(c.updated_at).getTime();
    const g: DateGroup = t >= midnight ? 'today' : t >= midnight - day ? 'yesterday' : t >= midnight - 7 * day ? 'previous7' : 'older';
    buckets.set(g, [...(buckets.get(g) ?? []), c]);
  }
  return order.filter((g) => buckets.has(g)).map((g) => ({ group: g, items: buckets.get(g)! }));
}

/** "explorer_agent" → "Explorer agent" when the author is not a known agent id. */
export function humanizeAuthor(author: string): string {
  const s = author.replace(/[_-]+/g, ' ').trim();
  return s ? s[0].toUpperCase() + s.slice(1) : author;
}

/**
 * A chunk as displayed in Sources: its leading markdown headings become a short section
 * path (document title dropped, last two levels kept), the rest is the passage text.
 */
export function splitSnippet(snippet: string): { section: string; text: string } {
  const lines = snippet.split('\n');
  const heads: string[] = [];
  let i = 0;
  for (; i < lines.length; i++) {
    const l = lines[i].trim();
    if (/^#{1,6}\s/.test(l)) heads.push(l.replace(/^#+\s*/, ''));
    else if (l && !/^(-{3,}|\*{3,})$/.test(l)) break;
  }
  const section = (heads.length > 1 ? heads.slice(1) : heads).slice(-2).join(' › ');
  const body = lines.slice(i).filter((l) => !/^\s*```/.test(l));
  return { section, text: body.join('\n').replace(/\*\*/g, '').trim() };
}

/** Sources per document, documents by relevance (their best passage), passages best first. */
export function groupSourcesByDocument(sources: SourceRef[]): { id: string; title: string; best: number; items: SourceRef[] }[] {
  const by = new Map<string, { id: string; title: string; best: number; items: SourceRef[] }>();
  for (const s of sources) {
    const d = by.get(s.document_id) ?? { id: s.document_id, title: s.title, best: -Infinity, items: [] };
    d.items.push(s);
    d.best = Math.max(d.best, s.score);
    by.set(s.document_id, d);
  }
  for (const d of by.values()) d.items.sort((a, b) => b.score - a.score);
  return [...by.values()].sort((a, b) => b.best - a.best);
}
