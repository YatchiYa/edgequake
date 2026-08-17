'use client';

import { useTranslation } from 'react-i18next';

/**
 * Shared empty body state for document detail surfaces (page + dialog).
 * WHY: StreamingMarkdownRenderer returns null on empty content — without this
 * the detail pane looks like a blank white failure rather than an empty state.
 */
export function DocumentContentEmptyState({
  className,
}: {
  className?: string;
}) {
  const { t } = useTranslation();
  return (
    <p
      className={
        className ?? 'text-muted-foreground text-sm py-8 text-center'
      }
      data-testid="document-content-empty"
    >
      {t('documents.details.noContent', 'No content available')}
    </p>
  );
}
