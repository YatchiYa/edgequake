/**
 * @module label-utils
 * @description Human-readable formatting for entity labels stored in normalized
 * UPPERCASE_UNDERSCORE format in the knowledge graph backend.
 *
 * @implements UX-AUDIT-030 F-GR-01 — Entity labels must be human-readable
 * @implements 066 — Drawing/table/equation display_name (identity ≠ presentation)
 */

// ─── Canonical entity type color palette ────────────────────────────────────
// Single source of truth used by graph-renderer, graph-legend, and entity-browser.
// WHY: Multiple components had diverged copies causing visual inconsistency.
export const ENTITY_TYPE_COLORS: Record<string, string> = {
  PERSON: '#3b82f6',          // blue-500
  ORGANIZATION: '#10b981',    // emerald-500
  TECHNOLOGY: '#06b6d4',      // cyan-500
  LOCATION: '#f59e0b',        // amber-500
  EVENT: '#ef4444',           // red-500
  CONCEPT: '#8b5cf6',         // violet-500
  DOCUMENT: '#6366f1',        // indigo-500
  PRODUCT: '#f97316',         // orange-500
  DRAWING: '#475569',         // slate-600 — multimodal figure/chart crop
  TABLE: '#0d9488',           // teal-600
  EQUATION: '#7c3aed',        // violet-600
  LAW: '#64748b',             // slate-500
  REGULATION: '#64748b',      // slate-500
  DEFAULT: '#94a3b8',         // slate-400
};

/**
 * Get the display color for an entity type.
 * Falls back to DEFAULT for unknown types.
 */
export function getEntityTypeColor(entityType: string | undefined): string {
  if (!entityType) return ENTITY_TYPE_COLORS.DEFAULT;
  return ENTITY_TYPE_COLORS[entityType.toUpperCase()] ?? ENTITY_TYPE_COLORS.DEFAULT;
}

/** Strip workspace scope prefix `{uuid}::NAME` → `NAME`. */
export function bareGraphId(raw: string): string {
  const idx = raw.lastIndexOf('::');
  return idx >= 0 ? raw.slice(idx + 2) : raw;
}

/** True when the string looks like a multimodal item id (im-… / IM-…). */
export function isMmItemId(raw: string): boolean {
  const bare = bareGraphId(raw).toLowerCase();
  return bare.startsWith('im-') || /^-?page-\d/.test(bare);
}

/** True when the label already looks human (spaces, middot, mixed case words). */
function looksHumanDisplayName(raw: string): boolean {
  if (!raw) return false;
  if (isMmItemId(raw)) return false;
  // Already has spaces or · separators from API display_name
  if (/\s|·/.test(raw)) return true;
  // Title-ish without underscores (avoid re-casing "Architecture")
  if (!/_/.test(raw) && /[a-z]/.test(raw) && /[A-Z]/.test(raw)) return true;
  return false;
}

/**
 * Convert a normalized entity name to a human-readable label.
 *
 * For 066 multimodal display names (from Graph API `label`), preserve as-is
 * (only truncate). Opaque `IM-…` ids are shown shortened, not title-cased.
 *
 * Examples:
 *   MARKET_SURVEILLANCE_AUTH → "Market Surveillance Auth"
 *   Architecture overview · p.2 · Fig 1 → unchanged (truncated if needed)
 *   IM-019F…-PAGE-0002-FIG-01 → "IM-…-PAGE-0002-FIG-01" short form
 */
export function formatEntityLabel(raw: string, maxLen = 35): string {
  if (!raw) return '';

  const bare = bareGraphId(raw);

  if (looksHumanDisplayName(bare) || looksHumanDisplayName(raw)) {
    const src = looksHumanDisplayName(raw) ? raw : bare;
    if (src.length <= maxLen) return src;
    return src.slice(0, maxLen - 1) + '…';
  }

  if (isMmItemId(bare)) {
    // Keep structural suffix readable; collapse long uuid slug.
    const short = bare.replace(
      /^im-[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}-/i,
      'im-…-',
    );
    if (short.length <= maxLen) return short;
    return short.slice(0, maxLen - 1) + '…';
  }

  const formatted = bare
    .replace(/_/g, ' ')
    .toLowerCase()
    .replace(/\b\w/g, (c) => c.toUpperCase());

  if (formatted.length <= maxLen) return formatted;
  return formatted.slice(0, maxLen - 1) + '…';
}

/**
 * Format an entity type name for display in the UI.
 * Entity types are stored as ALL_CAPS; this converts to Title Case.
 */
export function formatEntityType(raw: string): string {
  if (!raw) return '';
  return raw
    .replace(/_/g, ' ')
    .toLowerCase()
    .replace(/\b\w/g, (c) => c.toUpperCase());
}

/** Read string prop from GraphNode.properties. */
export function graphPropString(
  props: Record<string, unknown> | undefined,
  key: string,
): string | undefined {
  const v = props?.[key];
  return typeof v === 'string' && v.trim() ? v.trim() : undefined;
}

export function graphPropNumber(
  props: Record<string, unknown> | undefined,
  key: string,
): number | undefined {
  const v = props?.[key];
  if (typeof v === 'number' && Number.isFinite(v)) return v;
  if (typeof v === 'string' && v.trim()) {
    const n = Number(v);
    if (Number.isFinite(n)) return n;
  }
  return undefined;
}

/** First source document id from lineage props (array or singular). */
export function graphSourceDocumentId(
  props: Record<string, unknown> | undefined,
): string | undefined {
  const ids = props?.source_document_ids;
  if (Array.isArray(ids) && typeof ids[0] === 'string') return ids[0];
  const singular = props?.source_document_id;
  if (typeof singular === 'string' && singular.trim()) return singular.trim();
  return undefined;
}

/** Subtitle for Drawing/Table/Equation detail rows. */
export function formatMmEntitySubtitle(
  nodeType: string | undefined,
  props: Record<string, unknown> | undefined,
): string | undefined {
  const t = (nodeType ?? '').toLowerCase();
  if (!['drawing', 'table', 'equation'].includes(t)) return undefined;
  const page = graphPropNumber(props, 'page_num');
  const fig = graphPropNumber(props, 'figure_index');
  const subtype = graphPropString(props, 'mm_subtype');
  const parts: string[] = [formatEntityType(nodeType || 'drawing')];
  if (subtype) parts.push(subtype);
  if (page != null) parts.push(`p.${page}`);
  if (fig != null) parts.push(`Fig ${fig}`);
  return parts.join(' · ');
}
