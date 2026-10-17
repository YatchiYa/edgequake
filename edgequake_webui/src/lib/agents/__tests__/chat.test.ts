import { describe, expect, it } from 'vitest';
import type { Conversation } from '../api';
import { groupByDate, initialRun, parseSseFrames, reduceRun, settleRun, splitSnippet, groupSourcesByDocument, type StreamEvent } from '../chat';

const run = (events: StreamEvent[]) => events.reduce(reduceRun, initialRun('hi'));

describe('parseSseFrames', () => {
  it('returns complete frames and keeps the partial tail', () => {
    const { events, rest } = parseSseFrames(
      'data: {"type":"delta","text":"a"}\n\n: ping\n\ndata: not json\n\ndata: {"type":"done"}\r\n\r\ndata: {"type":"de',
    );
    expect(events).toEqual([{ type: 'delta', text: 'a' }, { type: 'done' }]);
    expect(rest).toBe('data: {"type":"de');
  });
});

describe('reduceRun', () => {
  it('keeps a whole turn in ONE assistant message (routing + specialist)', () => {
    const s = run([
      { type: 'message_start', author: 'edgequake_assistant' },
      { type: 'delta', author: 'edgequake_assistant', text: 'Hel' },
      { type: 'delta', author: 'edgequake_assistant', text: 'lo' },
      { type: 'tool_start', id: 'r', author: 'edgequake_assistant', name: 'transfer_to_agent', args: { agent_name: 'explorer_agent' } },
      { type: 'message_start', author: 'explorer_agent' },
      { type: 'delta', author: 'explorer_agent', text: 'Graph' },
      { type: 'title', title: 'Hi' },
      { type: 'done' },
    ]);
    expect(s.messages.map((m) => [m.role, m.author, m.content])).toEqual([
      ['user', 'user', 'hi'],
      ['assistant', 'edgequake_assistant', 'Hello\n\nGraph'],
    ]);
    expect(s.messages[1].agents).toEqual(['edgequake_assistant', 'explorer_agent']);
    expect(s.messages[1].tools.map((t) => t.name)).toEqual(['transfer_to_agent']);
    expect(s.title).toBe('Hi');
    expect(s.done).toBe(true);
  });

  it('folds plan, steps and sources of an investigation in order', () => {
    const src = (chunk: string, steps: string[]) => ({ document_id: 'd1', title: 'a.md', snippet: 's', score: 1, chunk_id: chunk, steps });
    const s = run([
      { type: 'message_start', author: 'investigator_agent' },
      { type: 'step', id: 'plan', label: 'Plan', status: 'running' },
      { type: 'step', id: 'plan', label: 'Plan', status: 'done', elapsed_ms: 900 },
      { type: 'plan', items: [{ id: 'r1', label: 'Fees?' }, { id: 'r2', label: 'TRI?' }] },
      { type: 'step', id: 'r1', label: 'Fees?', status: 'running' },
      { type: 'step', id: 'r2', label: 'TRI?', status: 'running' },
      { type: 'step', id: 'r2', label: 'TRI?', status: 'done', sources_count: 3 },
      { type: 'step', id: 'r1', label: 'Fees?', status: 'error', detail: 'timeout' },
      { type: 'sources', items: [src('c1', ['r2']), src('c2', ['r2'])] },
      { type: 'sources', items: [src('c1', ['r2'])] },
      { type: 'step', id: 'synth', label: 'Synthesis', status: 'running' },
      { type: 'message_start', author: 'synthesizer' },
      { type: 'delta', author: 'synthesizer', text: 'Answer ' },
      { type: 'delta', author: 'synthesizer', text: '[1].' },
      { type: 'message_start', author: 'investigator_agent' },
      { type: 'step', id: 'synth', label: 'Synthesis', status: 'done' },
    ]);
    expect(s.messages).toHaveLength(2);
    const m = s.messages[1];
    expect(m.author).toBe('investigator_agent');
    expect(m.agents).toEqual(['investigator_agent', 'synthesizer']);
    expect(m.content).toBe('Answer [1].');
    expect(m.plan?.map((p) => p.id)).toEqual(['r1', 'r2']);
    expect(m.steps?.map((x) => [x.id, x.status, x.sources_count ?? null])).toEqual([
      ['plan', 'done', null],
      ['r1', 'error', null],
      ['r2', 'done', 3],
      ['synth', 'done', null],
    ]);
    expect(m.steps?.[0].elapsed_ms).toBe(900);
    expect(m.sources?.map((x) => x.chunk_id)).toEqual(['c1', 'c2']);
  });

  it('settles steps still running when the run ends', () => {
    const s = settleRun(run([{ type: 'step', id: 'r1', label: 'x', status: 'running' }]));
    expect(s.messages[1].steps?.[0].status).toBe('error');
  });

  it('creates a bubble when deltas arrive without message_start', () => {
    const s = run([{ type: 'delta', author: 'a', text: 'x' }]);
    expect(s.messages[1]).toMatchObject({ role: 'assistant', author: 'a', content: 'x' });
  });

  it('tracks tools and artifacts on the current bubble', () => {
    const s = run([
      { type: 'message_start', author: 'a' },
      { type: 'tool_start', id: 't1', author: 'a', name: 'render_graph', args: { q: 1 } },
      { type: 'tool_start', id: 't2', author: 'a', name: 'eq_search' },
      { type: 'tool_end', id: 't1', ok: true },
      { type: 'tool_end', id: 't2', ok: false },
      { type: 'artifact', name: 'g.png', mime_type: 'image/png', version: 0 },
      { type: 'artifact', name: 'g.png', mime_type: 'image/png', version: 1 },
    ]);
    const m = s.messages[1];
    expect(m.tools.map((t) => [t.id, t.status])).toEqual([
      ['t1', 'done'],
      ['t2', 'error'],
    ]);
    expect(m.artifacts).toEqual([{ name: 'g.png', mime_type: 'image/png', version: 1 }]);
  });

  it('records errors and settles running tools', () => {
    const s = settleRun(run([{ type: 'tool_start', id: 't', name: 'x' }, { type: 'error', message: 'boom' }]));
    expect(s.error).toBe('boom');
    expect(s.done).toBe(true);
    expect(s.messages[1].tools[0].status).toBe('error');
  });
});

describe('groupByDate', () => {
  const now = new Date(2026, 9, 16, 15, 0);
  const conv = (id: string, d: Date): Conversation => ({
    id,
    agent_id: 'a',
    title: id,
    created_at: d.toISOString(),
    updated_at: d.toISOString(),
  });
  it('buckets by local day boundaries, in display order', () => {
    const groups = groupByDate(
      [
        conv('today', new Date(2026, 9, 16, 0, 5)),
        conv('yesterday', new Date(2026, 9, 15, 23, 59)),
        conv('yesterday-early', new Date(2026, 9, 15, 0, 0)),
        conv('week', new Date(2026, 9, 10, 12, 0)),
        conv('older', new Date(2026, 9, 8, 23, 0)),
      ],
      now,
    );
    expect(groups.map((g) => [g.group, g.items.map((c) => c.id)])).toEqual([
      ['today', ['today']],
      ['yesterday', ['yesterday', 'yesterday-early']],
      ['previous7', ['week']],
      ['older', ['older']],
    ]);
  });
  it('omits empty groups', () => {
    expect(groupByDate([conv('x', new Date(2020, 0, 1))], now).map((g) => g.group)).toEqual(['older']);
  });
});

describe('splitSnippet', () => {
  it('turns leading headings into a section path and keeps the passage text', () => {
    expect(splitSnippet('# CATALOGUE\n## GAMME 1 — ODYSSEY\n### Odyssey 2026\n\n---\n- **Format** : SLP\n- Statut')).toEqual({
      section: 'GAMME 1 — ODYSSEY › Odyssey 2026',
      text: '- Format : SLP\n- Statut',
    });
    expect(splitSnippet('plain text')).toEqual({ section: '', text: 'plain text' });
  });
});

describe('groupSourcesByDocument', () => {
  it('orders documents by their best passage, not by passage count', () => {
    const src = (doc: string, score: number) => ({ document_id: doc, title: doc, snippet: '', score, chunk_id: `${doc}-${score}` });
    const groups = groupSourcesByDocument([src('emails', 2), src('emails', 3), src('emails', 1), src('produits', 5), src('produits', 0.5)]);
    expect(groups.map((g) => [g.id, g.items.map((i) => i.score)])).toEqual([
      ['produits', [5, 0.5]],
      ['emails', [3, 2, 1]],
    ]);
  });
});
