/**
 * SPEC-116 — Workspace chunking policy helpers (Acc-fair pin + validation).
 */

export type ChunkingMode = 'inherit' | 'adaptive' | 'fixed';

export const ACC_FAIR_CHUNK_TOKEN_SIZE = 1200;
export const ACC_FAIR_CHUNK_OVERLAP = 100;

export function parseChunkingMode(raw: string | null | undefined): ChunkingMode {
  const v = (raw ?? '').trim().toLowerCase();
  if (!v || v === 'inherit' || v === 'none' || v === 'default' || v === 'auto') {
    return 'inherit';
  }
  if (v === 'adaptive' || v === 'on') return 'adaptive';
  if (v === 'fixed' || v === 'off' || v === 'fair' || v === 'lightrag' || v === 'acc') {
    return 'fixed';
  }
  return 'inherit';
}

export function validateFixedChunkPair(
  size: number,
  overlap: number,
): string | null {
  if (!Number.isFinite(size) || size < 1) {
    return 'Chunk size must be at least 1.';
  }
  if (!Number.isFinite(overlap) || overlap < 0) {
    return 'Overlap must be 0 or greater.';
  }
  if (overlap >= size) {
    return `Overlap (${overlap}) must be less than chunk size (${size}).`;
  }
  return null;
}

/** Payload fields for create/update workspace (clear inherit → "inherit"). */
export function chunkingToUpdatePayload(args: {
  mode: ChunkingMode;
  size?: number | null;
  overlap?: number | null;
}): {
  chunking_mode: string;
  chunk_token_size?: number;
  chunk_overlap_token_size?: number;
} {
  if (args.mode === 'inherit') {
    return { chunking_mode: 'inherit' };
  }
  if (args.mode === 'adaptive') {
    return { chunking_mode: 'adaptive' };
  }
  return {
    chunking_mode: 'fixed',
    chunk_token_size: args.size ?? ACC_FAIR_CHUNK_TOKEN_SIZE,
    chunk_overlap_token_size: args.overlap ?? ACC_FAIR_CHUNK_OVERLAP,
  };
}

export function formatChunkingBadge(args: {
  mode: ChunkingMode;
  size?: number | null;
  overlap?: number | null;
}): string {
  if (args.mode === 'inherit') return 'Inherit';
  if (args.mode === 'adaptive') return 'Adaptive';
  const size = args.size ?? ACC_FAIR_CHUNK_TOKEN_SIZE;
  const overlap = args.overlap ?? ACC_FAIR_CHUNK_OVERLAP;
  return `Fixed · ${size}/${overlap}`;
}
