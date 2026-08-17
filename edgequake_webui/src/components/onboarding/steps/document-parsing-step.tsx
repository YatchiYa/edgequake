'use client';

import {
  PdfParserBackendField,
  type PdfParserBackendChoice,
} from '@/components/settings/pdf-parser-backend-field';
import type { WizardDraft } from '@/lib/onboarding/wizard-state';
import {
  getServerDefaultPdfParserBackend,
  pdfParserBackendDisplayName,
} from '@/lib/pdf/resolve-pdf-parser-backend';
import { FileText } from 'lucide-react';
import { useTranslation } from 'react-i18next';

export interface DocumentParsingStepProps {
  draft: WizardDraft;
  onChange: (patch: Partial<WizardDraft>) => void;
}

/**
 * SPEC-101 Wave 8 — PDF parser choice (SRP: document parsing only).
 */
export function DocumentParsingStep({ draft, onChange }: DocumentParsingStepProps) {
  const { t } = useTranslation();
  const serverResolved = pdfParserBackendDisplayName(getServerDefaultPdfParserBackend());

  return (
    <div className="space-y-3" data-testid="wizard-step-document-parsing">
      <section className="rounded-lg border p-3 space-y-3">
        <div className="flex items-start gap-2">
          <FileText className="h-4 w-4 mt-0.5 text-muted-foreground shrink-0" />
          <div>
            <h4 className="text-sm font-medium">
              {t('onboarding.pdfParserHeading', 'PDF parser')}
            </h4>
            <p className="text-xs text-muted-foreground mt-0.5">
              {t(
                'onboarding.pdfParserHintWithValue',
                'Choose how PDFs are converted to text. Server default resolves to {{value}}.',
                { value: serverResolved },
              )}
            </p>
          </div>
        </div>
        <PdfParserBackendField
          value={draft.pdfParserBackend as PdfParserBackendChoice}
          isEditing
          onChange={(value) => onChange({ pdfParserBackend: value })}
        />
      </section>
    </div>
  );
}
