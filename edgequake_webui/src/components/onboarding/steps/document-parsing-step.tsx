'use client';

import {
  PdfParserBackendField,
  type PdfParserBackendChoice,
} from '@/components/settings/pdf-parser-backend-field';
import {
  shouldShowVisionExtractControls,
  VisionExtractControls,
} from '@/components/settings/vision-extract-controls';
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
 * SPEC-015V — Vision extract toggles + prompts when Vision is selected.
 */
export function DocumentParsingStep({ draft, onChange }: DocumentParsingStepProps) {
  const { t } = useTranslation();
  const serverResolved = pdfParserBackendDisplayName(getServerDefaultPdfParserBackend());
  const serverIsVision = getServerDefaultPdfParserBackend() === 'vision';
  const showVisionExtract = shouldShowVisionExtractControls(
    draft.pdfParserBackend,
    serverIsVision,
  );

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

      {showVisionExtract ? (
        <section className="rounded-lg border p-3 space-y-2">
          <div>
            <h4 className="text-sm font-medium">
              {t('onboarding.visionExtractHeading', 'Vision extraction')}
            </h4>
            <p className="text-xs text-muted-foreground mt-0.5">
              {t(
                'onboarding.visionExtractHint',
                'Choose which visual modalities Vision extracts. Applies to future uploads.',
              )}
            </p>
          </div>
          <VisionExtractControls
            value={{
              extractImages: draft.visionExtractImages,
              extractCharts: draft.visionExtractCharts,
              extractFigures: draft.visionExtractFigures,
              pageSystemPrompt: draft.visionPageSystemPrompt,
              imageSystemPrompt: draft.visionImageSystemPrompt,
              chartSystemPrompt: draft.visionChartSystemPrompt,
              figureSystemPrompt: draft.visionFigureSystemPrompt,
            }}
            onChange={(next) =>
              onChange({
                visionExtractImages: next.extractImages,
                visionExtractCharts: next.extractCharts,
                visionExtractFigures: next.extractFigures,
                visionPageSystemPrompt: next.pageSystemPrompt,
                visionImageSystemPrompt: next.imageSystemPrompt,
                visionChartSystemPrompt: next.chartSystemPrompt,
                visionFigureSystemPrompt: next.figureSystemPrompt,
              })
            }
          />
        </section>
      ) : null}
    </div>
  );
}
