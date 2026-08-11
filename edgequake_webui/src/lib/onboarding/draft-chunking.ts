/**
 * SPEC-116 — Map wizard draft ↔ chunking card value (SSOT for all wizard kinds).
 */

import {
  ACC_FAIR_CHUNK_OVERLAP,
  ACC_FAIR_CHUNK_TOKEN_SIZE,
  parseChunkingMode,
  validateFixedChunkPair,
  type ChunkingMode,
} from '@/constants/chunking-policy';
import type { WizardDraft } from '@/lib/onboarding/wizard-state';

export type DraftChunkingValue = {
  mode: ChunkingMode;
  size: number;
  overlap: number;
};

type ChunkingDraftSlice = Pick<
  WizardDraft,
  'chunkingMode' | 'chunkTokenSize' | 'chunkOverlapTokenSize'
>;

export function chunkingValueFromDraft(
  draft: ChunkingDraftSlice,
): DraftChunkingValue {
  return {
    mode: parseChunkingMode(draft.chunkingMode),
    size: draft.chunkTokenSize ?? ACC_FAIR_CHUNK_TOKEN_SIZE,
    overlap: draft.chunkOverlapTokenSize ?? ACC_FAIR_CHUNK_OVERLAP,
  };
}

export function draftPatchFromChunkingValue(
  next: DraftChunkingValue,
): Partial<WizardDraft> {
  return {
    chunkingMode: next.mode === 'inherit' ? null : next.mode,
    chunkTokenSize: next.size,
    chunkOverlapTokenSize: next.overlap,
  };
}

/** Fixed mode must pass size/overlap validation before Next. */
export function isChunkingDraftValid(draft: ChunkingDraftSlice): boolean {
  const value = chunkingValueFromDraft(draft);
  if (value.mode !== 'fixed') return true;
  return validateFixedChunkPair(value.size, value.overlap) === null;
}
