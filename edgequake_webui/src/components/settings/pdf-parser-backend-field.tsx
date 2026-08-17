'use client';

import { Badge } from '@/components/ui/badge';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
<<<<<<< HEAD
=======
import {
  formatServerDefaultPdfParserLabel,
  getServerDefaultPdfParserBackend,
  pdfParserBackendDisplayName,
} from '@/lib/pdf/resolve-pdf-parser-backend';
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
import { Eye, Gauge } from 'lucide-react';
import { useTranslation } from 'react-i18next';

export type PdfParserBackendChoice = 'none' | 'vision' | 'edgeparse';

function backendLabel(
  value: PdfParserBackendChoice,
<<<<<<< HEAD
  t: (key: string, defaultValue: string) => string,
=======
  t: (key: string, defaultValue: string, options?: { value: string }) => string,
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
) {
  switch (value) {
    case 'edgeparse':
      return t('settings.pdfParser.edgeparse', 'EdgeParse');
    case 'vision':
      return t('settings.pdfParser.vision', 'Vision');
    default:
<<<<<<< HEAD
      return t('settings.pdfParser.serverDefault', 'Server Default');
=======
      return formatServerDefaultPdfParserLabel(t);
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
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
<<<<<<< HEAD
=======
  const serverBackend = getServerDefaultPdfParserBackend();
  const serverLabel = formatServerDefaultPdfParserLabel(t, serverBackend);
  const resolvedName = pdfParserBackendDisplayName(serverBackend);
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042

  if (isEditing) {
    return (
      <Select value={value} onValueChange={(next) => onChange(next as PdfParserBackendChoice)}>
<<<<<<< HEAD
        <SelectTrigger className="w-full">
          <SelectValue />
        </SelectTrigger>
        <SelectContent>
          <SelectItem value="none">
            {t('settings.pdfParser.serverDefault', 'Server Default')}
=======
        <SelectTrigger className="w-full" data-testid="pdf-parser-backend-select">
          <SelectValue />
        </SelectTrigger>
        <SelectContent>
          <SelectItem value="none" data-testid="pdf-parser-option-server-default">
            {serverLabel}
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
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
<<<<<<< HEAD
      {value === 'vision' ? (
=======
      {value === 'vision' || (value === 'none' && serverBackend === 'vision') ? (
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
        <Eye className="h-4 w-4 text-orange-600" />
      ) : (
        <Gauge className="h-4 w-4 text-amber-600" />
      )}
      <div>
<<<<<<< HEAD
        <div className="font-medium">{backendLabel(value, t)}</div>
=======
        <div className="font-medium" data-testid="pdf-parser-backend-label">
          {backendLabel(value, t)}
        </div>
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
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
<<<<<<< HEAD
                  'settings.pdfParser.serverDefaultHint',
                  'Uses the server fallback when no workspace override is set',
=======
                  'settings.pdfParser.serverDefaultHintWithValue',
                  'Uses server config when no workspace override is set (currently {{value}}).',
                  { value: resolvedName },
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
                )}
        </div>
      </div>
      <Badge variant="outline" className="ml-auto">
        {value === 'none'
<<<<<<< HEAD
          ? t('settings.pdfParser.fallbackVision', 'Fallback: Vision')
=======
          ? t('settings.pdfParser.resolvesTo', 'Resolves to {{value}}', {
              value: resolvedName,
            })
>>>>>>> 2e2518aa584f496bca65f772ce322563285ab042
          : backendLabel(value, t)}
      </Badge>
    </div>
  );
}
