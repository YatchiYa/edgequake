/**
 * @module WorkspaceExtractionLanguageCard
 * @description Workspace-scoped KG extraction output language (SPEC-096 / GH-352).
 *
 * @implements SPEC-096 — Multi-Language Extraction
 */
'use client';

import { Badge } from '@/components/ui/badge';
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from '@/components/ui/card';
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
} from '@/constants/extraction-languages';
import type { Workspace } from '@/types';
import { Languages } from 'lucide-react';
import { useTranslation } from 'react-i18next';

export interface WorkspaceExtractionLanguageCardProps {
  isEditing: boolean;
  workspace: Workspace;
  /** null / server-default sentinel = inherit env/default */
  selectedLanguage: string | null;
  onLanguageChange: (language: string | null) => void;
  disabled?: boolean;
}

export function WorkspaceExtractionLanguageCard({
  isEditing,
  workspace,
  selectedLanguage,
  onLanguageChange,
  disabled = false,
}: WorkspaceExtractionLanguageCardProps) {
  const { t } = useTranslation();
  const configured = workspace.extraction_language ?? null;
  const selectValue =
    selectedLanguage && selectedLanguage !== EXTRACTION_LANGUAGE_SERVER_DEFAULT
      ? selectedLanguage
      : EXTRACTION_LANGUAGE_SERVER_DEFAULT;

  return (
    <Card data-testid="workspace-extraction-language-card">
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <Languages className="h-5 w-5 text-indigo-600" />
          {t('workspace.extractionLanguage.title', 'Extraction Language')}
        </CardTitle>
        <CardDescription>
          {t(
            'workspace.extractionLanguage.description',
            'Language used for entity names, descriptions, and relationship text during extraction.',
          )}
        </CardDescription>
        <p
          className="text-xs text-muted-foreground pt-1"
          data-testid="extraction-language-future-only-hint"
        >
          {t(
            'workspace.extractionLanguage.futureOnlyHint',
            'Applies to future document ingestions. Use Rebuild Knowledge Graph to re-extract existing documents.',
          )}
        </p>
      </CardHeader>
      <CardContent>
        {isEditing ? (
          <Select
            value={selectValue}
            onValueChange={(value) => {
              if (value === EXTRACTION_LANGUAGE_SERVER_DEFAULT) {
                onLanguageChange(null);
              } else {
                onLanguageChange(value);
              }
            }}
            disabled={disabled}
          >
            <SelectTrigger
              className="w-full max-w-sm"
              data-testid="ws-extraction-language-select"
            >
              <SelectValue
                placeholder={t(
                  'workspace.extractionLanguage.serverDefault',
                  'Server default',
                )}
              />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value={EXTRACTION_LANGUAGE_SERVER_DEFAULT}>
                {t(
                  'workspace.extractionLanguage.serverDefault',
                  'Server default',
                )}
              </SelectItem>
              {EXTRACTION_LANGUAGES.map((lang) => (
                <SelectItem key={lang} value={lang}>
                  {lang}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        ) : configured ? (
          <Badge
            variant="secondary"
            className="text-sm"
            data-testid="ws-extraction-language-value"
          >
            {configured}
          </Badge>
        ) : (
          <p
            className="text-sm text-muted-foreground"
            data-testid="ws-extraction-language-value"
          >
            {t(
              'workspace.extractionLanguage.serverDefault',
              'Server default',
            )}
          </p>
        )}
      </CardContent>
    </Card>
  );
}
