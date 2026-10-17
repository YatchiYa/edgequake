/**
 * SPEC-165 — pure helpers for the AI agent builder (unit-tested): proposals found in a
 * conversation, the field diff between two proposals, and the test transcript sent back
 * to the architect by "Improve".
 */
import type { AgentInput, AgentSpecProposal, Message } from './api';

export const SPEC_FIELDS = ['icon', 'name', 'description', 'instructions', 'model', 'tools', 'starters', 'temperature', 'visibility'] as const;
export type SpecField = (typeof SPEC_FIELDS)[number];

/** Every architect proposal of the conversation, oldest first. */
export function proposals(messages: Message[]): AgentSpecProposal[] {
  return messages.flatMap((m) => (m.agent_spec ? [m.agent_spec] : []));
}

export interface SpecDiff {
  changed: Set<SpecField>;
  addedTools: string[];
  removedTools: string[];
}

/** What changed from `prev` to `next` (no previous version → nothing is highlighted). */
export function diffSpec(prev: AgentInput | undefined, next: AgentInput): SpecDiff {
  if (!prev) return { changed: new Set(), addedTools: [], removedTools: [] };
  const same = (a: unknown, b: unknown) => JSON.stringify(a) === JSON.stringify(b);
  const changed = new Set(SPEC_FIELDS.filter((f) => !same(prev[f], next[f])));
  if (changed.has('tools') && same([...prev.tools].sort(), [...next.tools].sort())) changed.delete('tools'); // order only
  return {
    changed,
    addedTools: next.tools.filter((t) => !prev.tools.includes(t)),
    removedTools: prev.tools.filter((t) => !next.tools.includes(t)),
  };
}

/** Same spec as the form? (ignores starter whitespace / tool order) */
export function sameSpec(a: AgentInput, b: AgentInput): boolean {
  return diffSpec(a, b).changed.size === 0;
}

/** Compact Q/A transcript of a preview conversation, capped for the 32 KB message limit. */
export function transcriptSummary(messages: Message[], maxChars = 6000): string {
  const per = Math.max(200, Math.floor(maxChars / Math.max(1, messages.length)));
  const lines = messages
    .filter((m) => m.content.trim())
    .map((m) => {
      const text = m.content.trim().replace(/\s+\n/g, '\n');
      const clipped = text.length > per ? `${text.slice(0, per - 1)}…` : text;
      // Metadata on its own line so the architect does not read it as part of the answer.
      const tools = m.tools.length
        ? `\n(tool calls, not shown to the user: ${m.tools.map((t) => `${t.name}${t.status === 'error' ? ' [error]' : ''}`).join(', ')})`
        : '';
      return `${m.role === 'user' ? 'Q' : 'A'}: ${clipped}${tools}`;
    });
  const out = lines.join('\n\n');
  return out.length > maxChars ? `${out.slice(0, maxChars - 1)}…` : out;
}
