'use client';

import { ServerDefaultsCard } from '@/components/onboarding/server-defaults-card';
import { Button } from '@/components/ui/button';
import { Label } from '@/components/ui/label';
import {
  EmbeddingModelSelector,
  type EmbeddingSelection,
} from '@/components/workspace/embedding-model-selector';
import {
  LLMModelSelector,
  type LLMSelection,
} from '@/components/workspace/llm-model-selector';
import { useInheritedModelDefaults } from '@/hooks/use-inherited-model-defaults';
import type { WizardDraft } from '@/lib/onboarding/wizard-state';
import { useEffect } from 'react';
import { useTranslation } from 'react-i18next';

export interface ModelDefaultsStepProps {
  draft: WizardDraft;
  onChange: (patch: Partial<WizardDraft>) => void;
  llm?: LLMSelection;
  embedding?: EmbeddingSelection;
  vision?: LLMSelection;
  onLlmChange: (v: LLMSelection | undefined) => void;
  onEmbeddingChange: (v: EmbeddingSelection | undefined) => void;
  onVisionChange: (v: LLMSelection | undefined) => void;
  advancedOpen: boolean;
  onAdvancedOpenChange: (open: boolean) => void;
  /** When set, resolve LLM/Embed/Vision via tenant → server ladder (Create Workspace). */
  tenantId?: string | null;
}

/**
 * SPEC-101 — Defaults-first model step (LAW-101-2 / LAW-101-3).
 * Happy path never mounts provider chip bars. Card stays visible when Advanced opens.
 */
export function ModelDefaultsStep({
  draft,
  onChange,
  llm,
  embedding,
  vision,
  onLlmChange,
  onEmbeddingChange,
  onVisionChange,
  advancedOpen,
  onAdvancedOpenChange,
  tenantId,
}: ModelDefaultsStepProps) {
  const { t } = useTranslation();
  const useTenantLadder = Boolean(tenantId);
  const inherited = useInheritedModelDefaults(tenantId ?? null);

  const { isLoading, hasConfiguredDefaults } = inherited;

  // Only force Advanced after defaults have finished loading and are incomplete.
  useEffect(() => {
    if (isLoading) return;
    if (!hasConfiguredDefaults && draft.useServerDefaults) {
      onChange({ useServerDefaults: false });
      onAdvancedOpenChange(true);
    }
  }, [
    isLoading,
    hasConfiguredDefaults,
    draft.useServerDefaults,
    onChange,
    onAdvancedOpenChange,
  ]);

  const showAdvanced = advancedOpen && !isLoading;

  return (
    <div className="space-y-3" data-testid="wizard-step-models">
      <ServerDefaultsCard
        showCustomize={!showAdvanced}
        overridden={showAdvanced}
        defaults={{
          isLoading: inherited.isLoading,
          hasConfiguredDefaults: inherited.hasConfiguredDefaults,
          source: inherited.source,
          defaultLlmProvider: inherited.defaultLlmProvider,
          defaultLlmModel: inherited.defaultLlmModel,
          defaultEmbeddingProvider: inherited.defaultEmbeddingProvider,
          defaultEmbeddingModel: inherited.defaultEmbeddingModel,
          defaultVisionProvider: inherited.defaultVisionProvider,
          defaultVisionModel: inherited.defaultVisionModel,
        }}
        onCustomize={() => {
          onChange({ useServerDefaults: false });
          onAdvancedOpenChange(true);
        }}
      />

      {showAdvanced ? (
        <div className="space-y-3" data-testid="wizard-models-advanced">
          <div className="flex items-center justify-between gap-2">
            <p className="text-sm font-medium">
              {t('onboarding.customModels', 'Custom model selection')}
            </p>
            {hasConfiguredDefaults ? (
              <Button
                type="button"
                variant="ghost"
                size="sm"
                className="h-auto px-2 py-1 text-xs"
                onClick={() => {
                  onChange({ useServerDefaults: true });
                  onLlmChange(undefined);
                  onEmbeddingChange(undefined);
                  onVisionChange(undefined);
                  onAdvancedOpenChange(false);
                }}
                data-testid="wizard-models-use-defaults"
              >
                {useTenantLadder
                  ? t('onboarding.backToInheritedDefaults', 'Use tenant defaults')
                  : t('onboarding.backToServerDefaults', 'Use server defaults')}
              </Button>
            ) : null}
          </div>
          <p className="text-xs text-muted-foreground">
            {t(
              'onboarding.customModelsHint',
              'Pick LLM, embedding, and vision. Provider filters stay hidden.',
            )}
          </p>
          <div className="grid gap-3">
            <div className="grid gap-2">
              <Label>{t('workspace.llmModel', 'LLM Model')}</Label>
              <LLMModelSelector
                value={llm}
                onChange={onLlmChange}
                showProviderFilters={false}
                showCapabilityFilters={false}
                showUsageHint
              />
            </div>
            <div className="grid gap-2">
              <Label>{t('workspace.embeddingModel', 'Embedding Model')}</Label>
              <EmbeddingModelSelector
                value={embedding}
                onChange={onEmbeddingChange}
                showProviderFilters={false}
              />
            </div>
            <div className="grid gap-2">
              <Label>{t('workspace.visionLLM', 'Vision LLM')}</Label>
              <LLMModelSelector
                value={vision}
                onChange={onVisionChange}
                filterVision
                showProviderFilters={false}
                showCapabilityFilters={false}
                showUsageHint
              />
            </div>
          </div>
        </div>
      ) : null}

      {!isLoading && !hasConfiguredDefaults && !showAdvanced ? (
        <p
          className="text-sm text-amber-700 dark:text-amber-400"
          data-testid="wizard-models-required-hint"
        >
          {t(
            'onboarding.modelsRequiredHint',
            useTenantLadder
              ? 'Tenant and server defaults are incomplete. Customize models to continue.'
              : 'Server defaults are incomplete. Customize models to continue.',
          )}
        </p>
      ) : null}
    </div>
  );
}
