/**
 * Allowlisted extraction output languages (SPEC-096 / GH-352).
 *
 * Keep in sync with `edgequake_pipeline::prompts::SUPPORTED_LANGUAGES` (EC-22).
 */
export const EXTRACTION_LANGUAGES = [
  'English',
  'Chinese',
  'Japanese',
  'Korean',
  'Spanish',
  'French',
  'German',
  'Portuguese',
  'Italian',
  'Russian',
] as const;

export type ExtractionLanguage = (typeof EXTRACTION_LANGUAGES)[number];

/** UI sentinel for "inherit env / English" — maps to clear (`""` / `"none"`) on save. */
export const EXTRACTION_LANGUAGE_SERVER_DEFAULT = '__server_default__';

export function isExtractionLanguage(value: string): value is ExtractionLanguage {
  return (EXTRACTION_LANGUAGES as readonly string[]).includes(value);
}

/**
 * Fleet / server extraction language (SPEC-096).
 * Mirrors `EDGEQUAKE_EXTRACTION_LANGUAGE`; falls back to English.
 */
export function getServerDefaultExtractionLanguage(): ExtractionLanguage {
  const raw =
    process.env.NEXT_PUBLIC_EDGEQUAKE_EXTRACTION_LANGUAGE?.trim() ??
    process.env.EDGEQUAKE_EXTRACTION_LANGUAGE?.trim() ??
    '';
  if (!raw) return 'English';
  const match = EXTRACTION_LANGUAGES.find(
    (lang) => lang.toLowerCase() === raw.toLowerCase(),
  );
  return match ?? 'English';
}

/**
 * Never-silent server default label — e.g. `Server default (English)`.
 */
export function formatServerDefaultExtractionLanguageLabel(
  t: (key: string, defaultValue: string, options?: { value: string }) => string,
  resolved: string = getServerDefaultExtractionLanguage(),
): string {
  return t(
    'workspace.extractionLanguage.serverDefaultWithValue',
    `Server default (${resolved})`,
    { value: resolved },
  );
}

/** Map UI select value → API payload (`undefined` omit on create; `""` clear on update). */
export function extractionLanguageToApiPayload(
  selected: string | null | undefined,
): string | undefined {
  if (selected == null || selected === EXTRACTION_LANGUAGE_SERVER_DEFAULT) {
    return undefined;
  }
  return selected;
}

/** Map UI select value → clear/set for update (always send when editing). */
export function extractionLanguageToUpdatePayload(
  selected: string | null | undefined,
): string {
  if (selected == null || selected === EXTRACTION_LANGUAGE_SERVER_DEFAULT) {
    return 'none';
  }
  return selected;
}
