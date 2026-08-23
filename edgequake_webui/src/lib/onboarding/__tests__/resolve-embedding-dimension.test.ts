import { describe, expect, it } from 'vitest';
import {
  knownEmbeddingDimension,
  resolveEmbeddingDimension,
} from '../resolve-embedding-dimension';
import { buildWorkspaceUpdatePayload } from '../model-payload';

describe('resolveEmbeddingDimension', () => {
  it('prefers explicit positive dimension', () => {
    expect(
      resolveEmbeddingDimension({
        provider: 'mistral',
        model: 'mistral-embed',
        dimension: 1024,
      }),
    ).toBe(1024);
  });

  it('resolves mistral-embed when catalog omits dimension', () => {
    expect(
      resolveEmbeddingDimension({
        provider: 'mistral',
        model: 'mistral-embed',
        dimension: 0,
      }),
    ).toBe(1024);
  });

  it('knownEmbeddingDimension covers embeddinggemma', () => {
    expect(knownEmbeddingDimension('embeddinggemma')).toBe(768);
  });
});

describe('buildWorkspaceUpdatePayload embedding dimension', () => {
  it('never emits dimension 0 when mistral embedding override is set', () => {
    const payload = buildWorkspaceUpdatePayload({
      useServerDefaults: false,
      llm: { provider: 'mistral', model: 'mistral-small-latest' },
      embedding: {
        provider: 'mistral',
        model: 'mistral-embed',
        dimension: 0,
      },
      pdfParserBackend: 'none',
      extractionLanguage: null,
      entityTypes: [],
      entityTypesStrict: true,
    });
    expect(payload.embedding_provider).toBe('mistral');
    expect(payload.embedding_model).toBe('mistral-embed');
    expect(payload.embedding_dimension).toBeGreaterThan(0);
    expect(payload.embedding_dimension).toBe(1024);
  });
});
