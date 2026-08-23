import { describe, expect, it } from 'vitest';
import type { Workspace } from '@/types';
import {
  getWorkspaceEmbeddingSelection,
  getWorkspaceLlmSelection,
  isWorkspaceResolutionSource,
} from '../drafts';

function ws(overrides: Partial<Workspace> = {}): Workspace {
  return {
    id: 'ws-1',
    tenant_id: 't-1',
    name: 'Demo',
    slug: 'demo',
    is_active: true,
    created_at: '',
    llm_provider: 'ollama',
    llm_model: 'gemma4:latest',
    embedding_provider: 'ollama',
    embedding_model: 'embeddinggemma',
    embedding_dimension: 768,
    ...overrides,
  } as Workspace;
}

describe('isWorkspaceResolutionSource', () => {
  it('is true only for workspace', () => {
    expect(isWorkspaceResolutionSource('workspace')).toBe(true);
    expect(isWorkspaceResolutionSource('tenant')).toBe(false);
    expect(isWorkspaceResolutionSource('env')).toBe(false);
    expect(isWorkspaceResolutionSource(undefined)).toBe(false);
  });
});

describe('getWorkspaceLlmSelection overridesOnly', () => {
  it('returns painted model when not filtering', () => {
    expect(getWorkspaceLlmSelection(ws({ llm_resolution_source: 'tenant' }))?.model).toBe(
      'gemma4:latest',
    );
  });

  it('returns undefined for tenant/env when overridesOnly', () => {
    expect(
      getWorkspaceLlmSelection(ws({ llm_resolution_source: 'tenant' }), {
        overridesOnly: true,
      }),
    ).toBeUndefined();
  });

  it('returns pick when source is workspace', () => {
    expect(
      getWorkspaceLlmSelection(ws({ llm_resolution_source: 'workspace' }), {
        overridesOnly: true,
      })?.provider,
    ).toBe('ollama');
  });
});

describe('getWorkspaceEmbeddingSelection overridesOnly', () => {
  it('returns undefined for painted tenant embedding', () => {
    expect(
      getWorkspaceEmbeddingSelection(ws({ embedding_resolution_source: 'tenant' }), {
        overridesOnly: true,
      }),
    ).toBeUndefined();
  });
});
