import { describe, expect, it } from 'vitest';
import {
  buildTenantModelPayload,
  buildWorkspaceModelPayload,
  buildWorkspaceUpdatePayload,
  normalizeModelFullId,
} from '../model-payload';

describe('model-payload', () => {
  it('omits all model fields when using server defaults (tenant)', () => {
    expect(
      buildTenantModelPayload({
        useServerDefaults: true,
        llm: { provider: 'ollama', model: 'gemma4:latest' },
        embedding: { provider: 'openai', model: 'text-embedding-3-small', dimension: 1536 },
        vision: { provider: 'ollama', model: 'gemma4:latest' },
      }),
    ).toEqual({});
  });

  it('includes slash-based overrides when not using server defaults (tenant)', () => {
    expect(
      buildTenantModelPayload({
        useServerDefaults: false,
        llm: { provider: 'ollama', model: 'gemma4:latest' },
        embedding: { provider: 'openai', model: 'text-embedding-3-small', dimension: 1536 },
        vision: { provider: 'openai', model: 'gpt-4o' },
      }),
    ).toEqual({
      default_llm_provider: 'ollama',
      default_llm_model: 'gemma4:latest',
      default_embedding_provider: 'openai',
      default_embedding_model: 'text-embedding-3-small',
      default_embedding_dimension: 1536,
      default_vision_llm_provider: 'openai',
      default_vision_llm_model: 'gpt-4o',
    });
  });

  it('omits workspace model fields when using server defaults', () => {
    expect(
      buildWorkspaceModelPayload({
        useServerDefaults: true,
        llm: { provider: 'a', model: 'b' },
      }),
    ).toEqual({});
  });

  it('builds workspace overrides', () => {
    expect(
      buildWorkspaceModelPayload({
        useServerDefaults: false,
        llm: { provider: 'ollama', model: 'x' },
        embedding: { provider: 'openai', model: 'e', dimension: 768 },
        vision: { provider: 'ollama', model: 'v' },
      }),
    ).toEqual({
      llm_provider: 'ollama',
      llm_model: 'x',
      embedding_provider: 'openai',
      embedding_model: 'e',
      embedding_dimension: 768,
      vision_llm_provider: 'ollama',
      vision_llm_model: 'v',
    });
  });

  it('normalizes colon legacy ids to slash', () => {
    expect(normalizeModelFullId('ollama:gemma4:latest')).toEqual({
      provider: 'ollama',
      model: 'gemma4:latest',
      fullId: 'ollama/gemma4:latest',
    });
    expect(normalizeModelFullId('openai/gpt-4o')).toEqual({
      provider: 'openai',
      model: 'gpt-4o',
      fullId: 'openai/gpt-4o',
    });
  });

  it('clears model overrides when useServerDefaults on update (EC-101-19)', () => {
    expect(
      buildWorkspaceUpdatePayload({
        useServerDefaults: true,
        llm: { provider: 'ollama', model: 'x' },
        embedding: { provider: 'openai', model: 'e', dimension: 768 },
        vision: { provider: 'ollama', model: 'v' },
        pdfParserBackend: 'vision',
        extractionLanguage: null,
        entityTypes: ['PERSON'],
        entityTypesStrict: true,
      }),
    ).toEqual({
      llm_model: '',
      llm_provider: '',
      embedding_model: '',
      embedding_provider: '',
      embedding_dimension: 0,
      vision_llm_model: '',
      vision_llm_provider: '',
      pdf_parser_backend: 'vision',
      vision_extract_images: true,
      vision_extract_charts: true,
      vision_extract_figures: true,
      vision_page_system_prompt: '',
      vision_image_system_prompt: '',
      vision_chart_system_prompt: '',
      vision_figure_system_prompt: '',
      entity_types: ['PERSON'],
      entity_types_strict: true,
      extraction_language: 'none',
      chunking_mode: 'inherit',
      extract_budget_mode: 'inherit',
      entity_type_colors: {},
      relation_types: [],
      relation_types_strict: true,
      kg_schema_preset: 'custom',
      relation_edges: [],
    });
  });

  it('builds update payload with overrides', () => {
    expect(
      buildWorkspaceUpdatePayload({
        useServerDefaults: false,
        llm: { provider: 'ollama', model: 'gemma4:latest' },
        embedding: { provider: 'ollama', model: 'embeddinggemma', dimension: 768 },
        vision: { provider: 'ollama', model: 'gemma4:latest' },
        pdfParserBackend: 'edgeparse',
        extractionLanguage: 'Chinese',
        chunkingMode: 'fixed',
        chunkTokenSize: 1200,
        chunkOverlapTokenSize: 100,
        entityTypes: ['PERSON', 'ORGANIZATION'],
        entityTypesStrict: false,
        entityTypeColors: { PERSON: '#112233' },
      }),
    ).toMatchObject({
      llm_provider: 'ollama',
      llm_model: 'gemma4:latest',
      embedding_provider: 'ollama',
      embedding_model: 'embeddinggemma',
      embedding_dimension: 768,
      vision_llm_provider: 'ollama',
      vision_llm_model: 'gemma4:latest',
      pdf_parser_backend: 'edgeparse',
      entity_types_strict: false,
      extraction_language: 'Chinese',
      chunking_mode: 'fixed',
      chunk_token_size: 1200,
      chunk_overlap_token_size: 100,
      extract_budget_mode: 'inherit',
      entity_type_colors: { PERSON: '#112233' },
    });
  });

  it('builds update payload with LightRAG extract budget', () => {
    expect(
      buildWorkspaceUpdatePayload({
        useServerDefaults: false,
        llm: { provider: 'ollama', model: 'gemma4:latest' },
        embedding: { provider: 'ollama', model: 'embeddinggemma', dimension: 768 },
        vision: { provider: 'ollama', model: 'gemma4:latest' },
        pdfParserBackend: 'vision',
        extractionLanguage: null,
        extractBudgetMode: 'custom',
        extractMaxEntities: 40,
        extractMaxRecords: 100,
        entityTypes: ['PERSON'],
        entityTypesStrict: true,
      }),
    ).toMatchObject({
      extract_budget_mode: 'custom',
      extract_max_entities: 40,
      extract_max_records: 100,
    });
  });
});
