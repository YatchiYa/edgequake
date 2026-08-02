'use client';

import { Badge } from '@/components/ui/badge';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import {
  formatServerDefaultPdfParserLabel,
  getServerDefaultPdfParserBackend,
  pdfParserBackendDisplayName,
} from '@/lib/pdf/resolve-pdf-parser-backend';
import { Eye, Gauge } from 'lucide-react';
import { useTranslation } from 'react-i18next';

export type PdfParserBackendChoice = 'none' | 'vision' | 'edgeparse';

function backendLabel(
  value: PdfParserBackendChoice,
  t: (key: string, defaultValue: string, options?: { value: string }) => string,
) {
  switch (value) {
    case 'edgeparse':
      return t('settings.pdfParser.edgeparse', 'EdgeParse');
    case 'vision':
      return t('settings.pdfParser.vision', 'Vision');
    default:
      return formatServerDefaultPdfParserLabel(t);
  }
}

interface PdfParserBackendFieldProps {
  value: PdfParserBackendChoice;
  isEditing: boolean;
  onChange: (value: PdfParserBackendChoice) => void;
}

export function PdfParserBackendField({
  value,
  isEditing,
  onChange,
}: PdfParserBackendFieldProps) {
  const { t } = useTranslation();
  const serverBackend = getServerDefaultPdfParserBackend();
  const serverLabel = formatServerDefaultPdfParserLabel(t, serverBackend);
  const resolvedName = pdfParserBackendDisplayName(serverBackend);

  if (isEditing) {
    return (
      <Select value={value} onValueChange={(next) => onChange(next as PdfParserBackendChoice)}>
        <SelectTrigger className="w-full" data-testid="pdf-parser-backend-select">
          <SelectValue />
        </SelectTrigger>
        <SelectContent>
          <SelectItem value="none" data-testid="pdf-parser-option-server-default">
            {serverLabel}
          </SelectItem>
          <SelectItem value="vision">
            {t('settings.pdfParser.vision', 'Vision')}
          </SelectItem>
          <SelectItem value="edgeparse">
            {t('settings.pdfParser.edgeparse', 'EdgeParse')}
          </SelectItem>
        </SelectContent>
      </Select>
    );
  }

  return (
    <div className="flex items-center gap-3 p-3 bg-muted/50 rounded-lg">
      {value === 'vision' || (value === 'none' && serverBackend === 'vision') ? (
        <Eye className="h-4 w-4 text-orange-600" />
      ) : (
        <Gauge className="h-4 w-4 text-amber-600" />
      )}
      <div>
        <div className="font-medium" data-testid="pdf-parser-backend-label">
          {backendLabel(value, t)}
        </div>
        <div className="text-sm text-muted-foreground">
          {value === 'edgeparse'
            ? t(
                'settings.pdfParser.edgeparseHint',
                'Fast, CPU-only, best for digital-native PDFs',
              )
            : value === 'vision'
              ? t(
                  'settings.pdfParser.visionHint',
                  'Best for scanned and image-heavy PDFs',
                )
              : t(
                  'settings.pdfParser.serverDefaultHintWithValue',
                  'Uses server config when no workspace override is set (currently {{value}}).',
                  { value: resolvedName },
                )}
        </div>
      </div>
      <Badge variant="outline" className="ml-auto">
        {value === 'none'
          ? t('settings.pdfParser.resolvesTo', 'Resolves to {{value}}', {
              value: resolvedName,
            })
          : backendLabel(value, t)}
      </Badge>
    </div>
  );
}
