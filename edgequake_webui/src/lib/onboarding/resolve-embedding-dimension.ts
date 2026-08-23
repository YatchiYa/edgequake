/**
 * Resolve embedding vector dimension for wizard PUT payloads (SPEC-101).
 * Mirrors edgequake-core `Workspace::known_embedding_dimension` / detect fallbacks.
 */

const KNOWN_MODEL_DIMENSIONS: Record<string, number> = {
  'text-embedding-3-small': 1536,
  'text-embedding-ada-002': 1536,
  'text-embedding-3-large': 3072,
  'embeddinggemma': 768,
  'embeddinggemma:latest': 768,
  'nomic-embed-text': 768,
  'nomic-embed-text:latest': 768,
  'mistral-embed': 1024,
  'mistral-embed-2312': 1024,
  'codestral-embed': 1024,
  'codestral-embed-2505': 1024,
  'mxbai-embed-large': 1024,
  'mxbai-embed-large:latest': 1024,
};

const DEFAULT_EMBEDDING_DIMENSION = 1536;

/** Catalog row or picker option dimension when present. */
export function knownEmbeddingDimension(model: string): number | undefined {
  const trimmed = model.trim();
  if (!trimmed) return undefined;
  if (KNOWN_MODEL_DIMENSIONS[trimmed] !== undefined) {
    return KNOWN_MODEL_DIMENSIONS[trimmed];
  }
  if (trimmed.includes('768')) return 768;
  if (trimmed.includes('1024')) return 1024;
  if (trimmed.includes('3072')) return 3072;
  return undefined;
}

/**
 * Resolve a positive dimension for PUT payloads.
 * Never returns 0 when provider+model are set (poison `embedding_dimension: 0` clears overrides).
 */
export function resolveEmbeddingDimension(args: {
  provider: string;
  model: string;
  dimension?: number;
  catalogDimension?: number;
  providerDefaultDimension?: number;
}): number {
  if (typeof args.dimension === 'number' && args.dimension > 0) {
    return args.dimension;
  }
  if (typeof args.catalogDimension === 'number' && args.catalogDimension > 0) {
    return args.catalogDimension;
  }
  const known = knownEmbeddingDimension(args.model);
  if (known !== undefined) return known;
  if (
    typeof args.providerDefaultDimension === 'number' &&
    args.providerDefaultDimension > 0
  ) {
    return args.providerDefaultDimension;
  }
  return DEFAULT_EMBEDDING_DIMENSION;
}
