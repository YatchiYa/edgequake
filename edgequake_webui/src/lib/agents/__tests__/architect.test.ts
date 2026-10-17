import { describe, expect, it } from 'vitest';
import type { AgentInput, Message } from '../api';
import { diffSpec, proposals, sameSpec, transcriptSummary } from '../architect';
import { initialRun, reduceRun } from '../chat';

const spec = (over: Partial<AgentInput> = {}): AgentInput => ({
  name: 'Fund FAQ',
  description: 'Answers fund questions.',
  icon: '💼',
  instructions: '## Role',
  model: 'mistral/mistral-small-latest',
  tools: ['eq_retrieve', 'eq_search'],
  starters: ['Fees?'],
  temperature: 0.1,
  visibility: 'private',
  ...over,
});

describe('diffSpec', () => {
  it('highlights nothing for the first version', () => {
    expect(diffSpec(undefined, spec()).changed.size).toBe(0);
  });
  it('lists changed fields and tool additions/removals', () => {
    const d = diffSpec(spec(), spec({ name: 'Fund helper', tools: ['eq_retrieve', 'render_graph'], temperature: 0.2 }));
    expect([...d.changed].sort()).toEqual(['name', 'temperature', 'tools']);
    expect(d.addedTools).toEqual(['render_graph']);
    expect(d.removedTools).toEqual(['eq_search']);
  });
  it('ignores tool order', () => {
    expect(sameSpec(spec(), spec({ tools: ['eq_search', 'eq_retrieve'] }))).toBe(true);
    expect(sameSpec(spec(), spec({ starters: ['Fees?', 'More'] }))).toBe(false);
  });
});

describe('agent_spec stream event', () => {
  it('attaches the proposal to the latest assistant bubble, newest wins', () => {
    let run = initialRun('Build me an agent');
    run = reduceRun(run, { type: 'message_start', author: 'agent_architect' });
    run = reduceRun(run, { type: 'tool_start', id: 'p1', author: 'agent_architect', name: 'propose_agent_spec' });
    run = reduceRun(run, { type: 'tool_end', id: 'p1', ok: true });
    run = reduceRun(run, { type: 'agent_spec', spec: spec(), rationale: 'Because', tests: ['T1', 'T2'] });
    run = reduceRun(run, { type: 'agent_spec', spec: spec({ name: 'V2' }), rationale: 'Better', tests: [] });
    run = reduceRun(run, { type: 'delta', author: 'agent_architect', text: 'Done.' });
    const bot = run.messages[1];
    expect(bot.agent_spec?.spec.name).toBe('V2');
    expect(bot.content).toBe('Done.');
    expect(run.messages[0].agent_spec).toBeUndefined();
    expect(proposals(run.messages).map((p) => p.spec.name)).toEqual(['V2']);
  });
  it('creates a bubble when no message_start came first', () => {
    const run = reduceRun(initialRun('x'), { type: 'agent_spec', spec: spec(), rationale: '', tests: ['T'] });
    expect(run.messages).toHaveLength(2);
    expect(run.messages[1].agent_spec?.tests).toEqual(['T']);
  });
});

describe('transcriptSummary', () => {
  const msg = (role: 'user' | 'assistant', content: string, tools: Message['tools'] = []): Message => ({
    id: content,
    role,
    author: role,
    content,
    tools,
    artifacts: [],
    created_at: '',
  });
  it('formats Q/A with tools and caps the size', () => {
    const out = transcriptSummary([msg('user', 'Fees?'), msg('assistant', 'x'.repeat(5000), [{ id: '1', name: 'eq_retrieve', status: 'error' }])], 1000);
    expect(out.startsWith('Q: Fees?\n\nA: x')).toBe(true);
    expect(out).toContain('\n(tool calls, not shown to the user: eq_retrieve [error])');
    expect(out.length).toBeLessThanOrEqual(1000);
  });
});
