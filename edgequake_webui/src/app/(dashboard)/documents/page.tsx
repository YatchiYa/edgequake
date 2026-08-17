/**
 * @module DocumentsPage
 * @description Document ingestion and management page route.
 *
 * @implements FEAT0001 - Document ingestion
 * @implements SPEC-099 — fill main height so inventory scroll stays internal
 * @see DocumentManager component for full implementation
 */
import { DocumentManager } from '@/components/documents/document-manager';

export default function DocumentsPage() {
  // h-full min-h-0: participate in dashboard main flex height so the table
  // virtualizer scrolls inside Documents, not the whole page (dropzone stays).
  return (
    <div
      className="flex h-full min-h-0 flex-col overflow-clip"
      data-testid="documents-page"
    >
      <DocumentManager />
    </div>
  );
}
