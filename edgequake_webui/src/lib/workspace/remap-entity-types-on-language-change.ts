/**
 * SPEC-096 LAW-L6 — remap preset-backed entity types when extraction language changes.
 * Custom/mixed lists are left unchanged.
 */

import { remapPresetTypes } from '@/constants/entity-type-catalog';

export interface RemapEntityTypesResult {
  types: string[];
  /** True when types were replaced with a localized preset. */
  remapped: boolean;
}

/**
 * If `currentTypes` match a known preset (any language variant), return the same
 * preset in `toLang`. Otherwise return the original list unchanged.
 */
export function applyExtractionLanguageToEntityTypes(
  currentTypes: string[],
  fromLang: string | null | undefined,
  toLang: string | null | undefined,
): RemapEntityTypesResult {
  const next = remapPresetTypes(currentTypes, fromLang, toLang);
  if (next == null) {
    return { types: currentTypes, remapped: false };
  }
  const same =
    next.length === currentTypes.length &&
    next.every((t, i) => t === currentTypes[i]);
  if (same) {
    return { types: currentTypes, remapped: false };
  }
  return { types: next, remapped: true };
}
