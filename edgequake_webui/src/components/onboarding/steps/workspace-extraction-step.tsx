'use client';

import { EntityTypeSelector } from '@/components/shared/entity-type-selector';
import { CreateWorkspaceExtractionLanguageField } from '@/components/workspace/create-workspace-extraction-language-field';
import { applyExtractionLanguageToEntityTypes } from '@/lib/workspace/remap-entity-types-on-language-change';
import type { WizardDraft } from '@/lib/onboarding/wizard-state';
import { useCallback } from 'react';
import { useTranslation } from 'react-i18next';

export interface WorkspaceExtractionStepProps {
  draft: WizardDraft;
  onChange: (patch: Partial<WizardDraft>) => void;
  /** When true (reconfigure), show entity_types_strict checkbox. */
  showStrict?: boolean;
}

export function WorkspaceExtractionStep({
  draft,
  onChange,
  showStrict = false,
}: WorkspaceExtractionStepProps) {
  const { t } = useTranslation();

  const handleLanguageChange = useCallback(
    (next: string | null) => {
      const { types, remapped } = applyExtractionLanguageToEntityTypes(
        draft.entityTypes,
        draft.extractionLanguage,
        next,
      );
      onChange({
        extractionLanguage: next,
        ...(remapped ? { entityTypes: types } : {}),
      });
    },
    [draft.entityTypes, draft.extractionLanguage, onChange],
  );

  return (
    <div className="space-y-4" data-testid="wizard-step-extraction">
      <section className="rounded-lg border p-3 space-y-2" data-testid="wizard-extraction-language">
        <CreateWorkspaceExtractionLanguageField
          value={draft.extractionLanguage}
          onChange={handleLanguageChange}
        />
      </section>

      <section className="space-y-2" data-testid="wizard-extraction-entity-types">
        <div>
          <h4 className="text-sm font-medium">
            {t('onboarding.entityTypesHeading', 'Entity types for extraction')}
          </h4>
          <p className="text-xs text-muted-foreground mt-0.5">
            {t(
              'onboarding.entityTypesHint',
              'Choose which kinds of entities to extract into the knowledge graph.',
            )}
          </p>
        </div>
        <EntityTypeSelector
          value={draft.entityTypes}
          onChange={(types) => onChange({ entityTypes: types })}
          extractionLanguage={draft.extractionLanguage}
          compactPresets
          strictLimit={draft.entityTypesStrict}
          onStrictLimitChange={
            showStrict
              ? (strict) => onChange({ entityTypesStrict: strict })
              : undefined
          }
          colors={draft.entityTypeColors}
          onColorsChange={(entityTypeColors) => onChange({ entityTypeColors })}
        />
      </section>
    </div>
  );
}
