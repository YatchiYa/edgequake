'use client';

import { Badge } from '@/components/ui/badge';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { Eye, Gauge } from 'lucide-react';
import { useTranslation } from 'react-i18next';

export type PdfParserBackendChoice = 'none' | 'vision' | 'edgeparse';

function backendLabel(
  value: PdfParserBackendChoice,
  t: (key: string, defaultValue: string) => string,
) {
  switch (value) {
    case 'edgeparse':
      return t('settings.pdfParser.edgeparse', 'EdgeParse');
    case 'vision':
      return t('settings.pdfParser.vision', 'Vision');
    default:
      return t('settings.pdfParser.serverDefault', 'Server Default');
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

  if (isEditing) {
    return (
      <Select value={value} onValueChange={(next) => onChange(next as PdfParserBackendChoice)}>
        <SelectTrigger className="w-full">
          <SelectValue />
        </SelectTrigger>
        <SelectContent>
          <SelectItem value="none">
            {t('settings.pdfParser.serverDefault', 'Server Default')}
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
      {value === 'vision' ? (
        <Eye className="h-4 w-4 text-orange-600" />
      ) : (
        <Gauge className="h-4 w-4 text-amber-600" />
      )}
      <div>
        <div className="font-medium">{backendLabel(value, t)}</div>
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
                  'settings.pdfParser.serverDefaultHint',
                  'Uses the server fallback when no workspace override is set',
                )}
        </div>
      </div>
      <Badge variant="outline" className="ml-auto">
        {value === 'none'
          ? t('settings.pdfParser.fallbackVision', 'Fallback: Vision')
          : backendLabel(value, t)}
      </Badge>
    </div>
  );
}
