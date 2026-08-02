/**
 * Optional extraction language select for create/reconfigure wizards (SPEC-096).
 */
'use client';

import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import {
  EXTRACTION_LANGUAGES,
  EXTRACTION_LANGUAGE_SERVER_DEFAULT,
  formatServerDefaultExtractionLanguageLabel,
} from '@/constants/extraction-languages';
import { useTranslation } from 'react-i18next';

export interface CreateWorkspaceExtractionLanguageFieldProps {
  value: string | null;
  onChange: (language: string | null) => void;
}

export function CreateWorkspaceExtractionLanguageField({
  value,
  onChange,
}: CreateWorkspaceExtractionLanguageFieldProps) {
  const { t } = useTranslation();
  const selectValue =
    value && value !== EXTRACTION_LANGUAGE_SERVER_DEFAULT
      ? value
      : EXTRACTION_LANGUAGE_SERVER_DEFAULT;
  const serverDefaultLabel = formatServerDefaultExtractionLanguageLabel(t);

  return (
    <div className="space-y-2">
      <label className="text-sm font-medium">
        {t('workspace.extractionLanguage.title', 'Extraction Language')}
      </label>
      <p className="text-xs text-muted-foreground">
        {t(
          'workspace.extractionLanguage.description',
          'Language used for entity names, descriptions, and relationship text during extraction.',
        )}
      </p>
      <Select
        value={selectValue}
        onValueChange={(v) => {
          onChange(v === EXTRACTION_LANGUAGE_SERVER_DEFAULT ? null : v);
        }}
      >
        <SelectTrigger
          className="w-full"
          data-testid="create-workspace-extraction-language"
        >
          <SelectValue
            placeholder={serverDefaultLabel}
          />
        </SelectTrigger>
        <SelectContent>
          <SelectItem value={EXTRACTION_LANGUAGE_SERVER_DEFAULT}>
            {serverDefaultLabel}
          </SelectItem>
          {EXTRACTION_LANGUAGES.map((lang) => (
            <SelectItem key={lang} value={lang}>
              {lang}
            </SelectItem>
          ))}
        </SelectContent>
      </Select>
    </div>
  );
}
