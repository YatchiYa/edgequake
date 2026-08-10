/**
 * @module entity-presets
 * @description Domain entity type presets for workspace creation.
 *
 * Each preset provides a curated set of entity types suited for a specific domain.
 * The General preset matches backend `default_entity_types()` exactly (SPEC-114).
 * Relation lists live in `kg-schema-presets.ts`.
 *
 * @implements SPEC-085: Custom entity configuration from UI
 * @implements SPEC-114: General ≡ Rust defaults
 */

export type PresetKey =
  | 'blank'
  | 'general'
  | 'manufacturing'
  | 'healthcare'
  | 'legal'
  | 'research'
  | 'finance'
  | 'custom';

export interface EntityPreset {
  /** Translation key for the label (falls back to labelFallback). */
  labelKey: string;
  /** English fallback label used before i18n loads. */
  labelFallback: string;
  /** Icon name (Lucide) to render next to the preset. */
  icon: string;
  /** Entity types included in this preset (UPPERCASE_UNDERSCORED). */
  types: string[];
}

/**
 * Preset definitions (DRY: canonical English types).
 * For language-aware tokens use `getPresetTypes(key, language)` from
 * `@/constants/entity-type-catalog` (SPEC-096 LAW-L6).
 *
 * @implements SPEC-085: Domain presets for common industries
 */
export const ENTITY_PRESETS: Record<Exclude<PresetKey, 'custom'>, EntityPreset> = {
  /** Empty slate — start with no entities/relations/edges and add what you need. */
  blank: {
    labelKey: 'entityTypes.presets.blank',
    labelFallback: 'Blank',
    icon: 'SquareDashed',
    types: [],
  },
  general: {
    labelKey: 'entityTypes.presets.general',
    labelFallback: 'General',
    icon: 'Globe',
    // Must match edgequake-pipeline `default_entity_types()` (SPEC-114 LAW-114-4).
    types: [
      'PERSON',
      'CREATURE',
      'ORGANIZATION',
      'LOCATION',
      'EVENT',
      'CONCEPT',
      'METHOD',
      'CONTENT',
      'DATA',
      'ARTIFACT',
      'NATURALOBJECT',
      'OTHER',
    ],
  },
  manufacturing: {
    labelKey: 'entityTypes.presets.manufacturing',
    labelFallback: 'Manufacturing',
    icon: 'Factory',
    types: [
      'PERSON',
      'ORGANIZATION',
      'LOCATION',
      'EVENT',
      'CONCEPT',
      'MACHINE',
      'COMPONENT',
      'DEFECT',
      'MEASUREMENT',
      'PROCESS',
      'MATERIAL',
      'PRODUCT',
    ],
  },
  healthcare: {
    labelKey: 'entityTypes.presets.healthcare',
    labelFallback: 'Healthcare',
    icon: 'HeartPulse',
    types: [
      'PERSON',
      'ORGANIZATION',
      'LOCATION',
      'EVENT',
      'CONCEPT',
      'SYMPTOM',
      'DRUG',
      'DIAGNOSIS',
      'PROCEDURE',
      'PATIENT',
      'CONDITION',
      'DATE',
    ],
  },
  legal: {
    labelKey: 'entityTypes.presets.legal',
    labelFallback: 'Legal',
    icon: 'Scale',
    types: [
      'PERSON',
      'ORGANIZATION',
      'LOCATION',
      'EVENT',
      'CONCEPT',
      'CONTRACT',
      'CLAUSE',
      'PARTY',
      'REGULATION',
      'JURISDICTION',
      'CASE',
      'DATE',
    ],
  },
  research: {
    labelKey: 'entityTypes.presets.research',
    labelFallback: 'Research',
    icon: 'FlaskConical',
    types: [
      'PERSON',
      'ORGANIZATION',
      'LOCATION',
      'EVENT',
      'CONCEPT',
      'PAPER',
      'METHOD',
      'DATASET',
      'HYPOTHESIS',
      'FINDING',
      'METRIC',
      'DATE',
    ],
  },
  finance: {
    labelKey: 'entityTypes.presets.finance',
    labelFallback: 'Finance',
    icon: 'TrendingUp',
    types: [
      'PERSON',
      'ORGANIZATION',
      'LOCATION',
      'EVENT',
      'CONCEPT',
      'FUND',
      'SECURITY',
      'RISK',
      'REGULATION',
      'COUNTERPARTY',
      'DATE',
      'PRODUCT',
    ],
  },
} as const;

/** Maximum number of entity types per workspace (mirrors backend MAX_ENTITY_TYPES). */
export const MAX_ENTITY_TYPES = 50;

/**
 * Normalize a raw entity type string to UPPERCASE_UNDERSCORED format.
 * Mirrors the backend `normalize_entity_types` logic (SPEC-085).
 */
export function normalizeEntityType(raw: string): string {
  return raw.trim().toUpperCase().replace(/[\s-]+/g, '_');
}

/**
 * Return unique types, dropping empty strings and capping at MAX_ENTITY_TYPES.
 */
export function deduplicateTypes(types: string[]): string[] {
  const seen = new Set<string>();
  const result: string[] = [];
  for (const t of types) {
    const normalized = normalizeEntityType(t);
    if (normalized && !seen.has(normalized)) {
      seen.add(normalized);
      result.push(normalized);
    }
    if (result.length >= MAX_ENTITY_TYPES) break;
  }
  return result;
}

/**
 * Detect which preset best matches a given list of entity types (English tokens).
 * For language-aware matching across catalog locales, use `detectCanonicalPreset`
 * from `@/constants/entity-type-catalog` (SPEC-096 LAW-L6).
 * Returns 'custom' if no preset matches exactly.
 */
export function detectPreset(types: string[]): PresetKey {
  const sorted = [...types.map(normalizeEntityType)].filter(Boolean).sort().join(',');
  for (const [key, preset] of Object.entries(ENTITY_PRESETS)) {
    if ([...preset.types].map(normalizeEntityType).sort().join(',') === sorted) {
      return key as PresetKey;
    }
  }
  return 'custom';
}
