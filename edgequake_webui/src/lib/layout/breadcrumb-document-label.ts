/**
 * Resolve human-readable document labels for dashboard breadcrumbs.
 * Route segments are UUIDs; chrome must show file_name/title + GUID.
 */

/** UUID (optional staging: prefix) used as document route segments. */
export const DOCUMENT_ID_SEGMENT_RE =
  /^(?:staging:)?[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i;

export function isDocumentIdSegment(segment: string): boolean {
  return DOCUMENT_ID_SEGMENT_RE.test(segment);
}

/** Short GUID for chrome (full id stays in title / data attributes). */
export function formatGuidShort(id: string, head = 8): string {
  const bare = id.replace(/^staging:/i, '');
  return bare.length > head ? `${bare.slice(0, head)}…` : bare;
}

export function resolveDocumentBreadcrumbLabel(
  doc?: { file_name?: string | null; title?: string | null } | null,
): string | null {
  const file = doc?.file_name?.trim();
  if (file) return file;
  const title = doc?.title?.trim();
  if (title) return title;
  return null;
}

/** Extract `/documents/:id` (also under `/w/:slug/documents/:id`). */
export function extractDocumentIdFromPath(pathname: string): string | null {
  const segments = pathname.split('/').filter(Boolean);
  const docsIdx = segments.findIndex((s) => s === 'documents');
  if (docsIdx < 0) return null;
  const next = segments[docsIdx + 1];
  if (!next || !isDocumentIdSegment(next)) return null;
  return next;
}
