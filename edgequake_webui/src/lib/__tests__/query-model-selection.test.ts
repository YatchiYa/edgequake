import { describe, expect, it } from 'vitest';
import { sanitizeQueryModelSelection } from '../query-model-selection';

describe('sanitizeQueryModelSelection', () => {
  const models = [
    { provider: 'mock', name: 'mock-smart' },
    { provider: 'openai', name: 'gpt-4o-mini' },
  ];

  const health = [
    { name: 'mock', enabled: true, health: { available: true } },
    { name: 'ollama', enabled: true, health: { available: false } },
  ];

  it('keeps a healthy existing selection', () => {
    expect(
      sanitizeQueryModelSelection({ provider: 'mock', model: 'mock-smart' }, models, health)
    ).toEqual({ provider: 'mock', model: 'mock-smart' });
  });

  it('clears a stale or unhealthy persisted selection back to server default', () => {
    expect(
      sanitizeQueryModelSelection({ provider: 'ollama', model: 'gemma3:12b' }, models, health)
    ).toEqual({ provider: undefined, model: undefined });
  });

  it('clears partial invalid state where only one field is present', () => {
    expect(sanitizeQueryModelSelection({ provider: 'mock' }, models, health)).toEqual({
      provider: undefined,
      model: undefined,
    });
  });
});
