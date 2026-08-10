import { describe, expect, it } from 'vitest';
import {
  diffWorkspaceConfig,
  toPendingRebuild,
  type WorkspaceConfigSnapshot,
} from '../workspace-config-diff';

const base: WorkspaceConfigSnapshot = {
  useServerDefaults: false,
  llm: { provider: 'ollama', model: 'gemma4:latest' },
  embedding: { provider: 'ollama', model: 'embeddinggemma', dimension: 768 },
  vision: { provider: 'ollama', model: 'gemma4:latest' },
  pdfParserBackend: 'none',
  extractionLanguage: null,
  entityTypes: ['PERSON'],
  entityTypesStrict: true,
};

describe('workspace-config-diff', () => {
  it('returns no changes for identical snapshots (EC-101-27)', () => {
    const diff = diffWorkspaceConfig(base, { ...base }, { documentCount: 10 });
    expect(diff.hasChanges).toBe(false);
    expect(diff.changedKeys).toEqual([]);
    expect(toPendingRebuild(diff.rebuildHints)).toBeNull();
  });

  it('flags LLM rebuild when docs > 0 (EC-101-16)', () => {
    const draft = {
      ...base,
      llm: { provider: 'ollama', model: 'other:latest' },
    };
    const diff = diffWorkspaceConfig(base, draft, { documentCount: 3 });
    expect(diff.changedKeys).toContain('llm');
    expect(diff.rebuildHints.extraction).toBe(true);
    expect(diff.rebuildHints.embeddings).toBe(false);
  });

  it('flags embedding rebuild when docs > 0 (EC-101-17)', () => {
    const draft = {
      ...base,
      embedding: { provider: 'openai', model: 'text-embedding-3-small', dimension: 1536 },
    };
    const diff = diffWorkspaceConfig(base, draft, { documentCount: 1 });
    expect(diff.rebuildHints.embeddings).toBe(true);
  });

  it('softens rebuild when zero documents (EC-101-18)', () => {
    const draft = {
      ...base,
      llm: { provider: 'ollama', model: 'other' },
      embedding: { provider: 'ollama', model: 'other-emb', dimension: 768 },
      vision: { provider: 'ollama', model: 'other-v' },
    };
    const diff = diffWorkspaceConfig(base, draft, { documentCount: 0 });
    expect(diff.hasChanges).toBe(true);
    expect(diff.rebuildHints).toEqual({
      embeddings: false,
      extraction: false,
      vision: false,
    });
  });

  it('detects pdf / language / entity / strict changes', () => {
    const draft: WorkspaceConfigSnapshot = {
      ...base,
      pdfParserBackend: 'edgeparse',
      extractionLanguage: 'French',
      entityTypes: ['PERSONNE'],
      entityTypesStrict: false,
    };
    const diff = diffWorkspaceConfig(base, draft, { documentCount: 0 });
    expect(diff.changedKeys).toEqual(
      expect.arrayContaining([
        'pdfParser',
        'extractionLanguage',
        'entityTypes',
        'entityTypesStrict',
      ]),
    );
  });

  it('detects reset to server defaults as llm/embedding/vision change (EC-101-19)', () => {
    const draft: WorkspaceConfigSnapshot = {
      ...base,
      useServerDefaults: true,
      llm: undefined,
      embedding: undefined,
      vision: undefined,
    };
    const diff = diffWorkspaceConfig(base, draft, { documentCount: 2 });
    expect(diff.changedKeys).toEqual(expect.arrayContaining(['llm', 'embedding', 'vision']));
    expect(diff.rebuildHints.extraction).toBe(true);
    expect(diff.rebuildHints.embeddings).toBe(true);
    expect(diff.rebuildHints.vision).toBe(true);
  });

  it('SPEC-114b flags typed edge changes and suggests KG rebuild', () => {
    const baseline = {
      ...base,
      relationEdges: [
        { source: 'PERSON', relation: 'WORKS_AT', target: 'ORGANIZATION' },
      ],
    };
    const draft = {
      ...base,
      relationEdges: [
        { source: 'PERSON', relation: 'LOCATED_IN', target: 'LOCATION' },
      ],
    };
    const diff = diffWorkspaceConfig(baseline, draft, { documentCount: 3 });
    expect(diff.changedKeys).toContain('relationEdges');
    expect(diff.rebuildHints.extraction).toBe(true);
  });

  it('SPEC-114 flags relation schema changes and suggests KG rebuild', () => {
    const draft: WorkspaceConfigSnapshot = {
      ...base,
      relationTypes: ['WORKS_AT', 'PART_OF'],
      relationTypesStrict: false,
      kgSchemaPreset: 'manufacturing',
    };
    const diff = diffWorkspaceConfig(
      { ...base, relationTypes: [], relationTypesStrict: true },
      draft,
      { documentCount: 3 },
    );
    expect(diff.changedKeys).toEqual(
      expect.arrayContaining(['relationTypes', 'relationTypesStrict', 'kgSchemaPreset']),
    );
    expect(diff.rebuildHints.extraction).toBe(true);
  });
});
