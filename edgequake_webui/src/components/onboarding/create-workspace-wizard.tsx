'use client';

import { WizardShell } from '@/components/onboarding/wizard-shell';
import { STEP_META } from '@/components/onboarding/step-meta';
import { ModelDefaultsStep } from '@/components/onboarding/steps/model-defaults-step';
import { ReviewStep } from '@/components/onboarding/steps/review-step';
import { WorkspaceBasicsStep } from '@/components/onboarding/steps/workspace-basics-step';
import { WorkspaceExtractionStep } from '@/components/onboarding/steps/workspace-extraction-step';
import type { EmbeddingSelection } from '@/components/workspace/embedding-model-selector';
import type { LLMSelection } from '@/components/workspace/llm-model-selector';
import { ENTITY_PRESETS } from '@/constants/entity-presets';
import { useInheritedModelDefaults } from '@/hooks/use-inherited-model-defaults';
import { createWorkspace } from '@/lib/api/edgequake';
import { buildWorkspaceModelPayload } from '@/lib/onboarding/model-payload';
import { useWizardDraftPersistence } from '@/lib/onboarding/use-wizard-draft-persistence';
import {
  EMPTY_WIZARD_DRAFT,
  canProceed,
  clampStepIndex,
  draftForStorage,
  stepsForWizard,
  type WizardDraft,
  type WizardStepId,
} from '@/lib/onboarding/wizard-state';
import type { Workspace } from '@/types';
import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { toast } from 'sonner';

export interface CreateWorkspaceWizardProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  tenantId: string | null;
  onCreated?: (workspace: Workspace) => void;
}

function initialWorkspaceDraft(): WizardDraft {
  return {
    ...EMPTY_WIZARD_DRAFT,
    entityTypes: [...ENTITY_PRESETS.general.types],
  };
}

/**
 * SPEC-101 — Guided Create Workspace wizard.
 */
export function CreateWorkspaceWizard({
  open,
  onOpenChange,
  tenantId,
  onCreated,
}: CreateWorkspaceWizardProps) {
  const { t } = useTranslation();
  const steps = useMemo(
    () => stepsForWizard('create-workspace', { includeAdmin: false, includeExtraction: true }),
    [],
  );
  const baselineRef = useRef(initialWorkspaceDraft());
  const [stepIndex, setStepIndex] = useState(0);
  const [draft, setDraft] = useState<WizardDraft>(initialWorkspaceDraft);
  const [llm, setLlm] = useState<LLMSelection | undefined>();
  const [embedding, setEmbedding] = useState<EmbeddingSelection | undefined>();
  const [vision, setVision] = useState<LLMSelection | undefined>();
  const [advancedOpen, setAdvancedOpen] = useState(false);
  const [submitting, setSubmitting] = useState(false);
  const inherited = useInheritedModelDefaults(tenantId);
  const { hasConfiguredDefaults } = inherited;
  const languagePrefillDone = useRef(false);
  const { clearDraft } = useWizardDraftPersistence(
    'create-workspace',
    open,
    draft,
    setDraft,
    stepIndex,
    setStepIndex,
    steps.length,
  );

  const stepId = steps[clampStepIndex(stepIndex, steps.length)];
  const meta = STEP_META[stepId];

  const patchDraft = useCallback((patch: Partial<WizardDraft>) => {
    setDraft((d) => ({ ...d, ...patch }));
  }, []);

  // Prefill extraction language from tenant Default Workspace once (do not overwrite edits).
  useEffect(() => {
    if (!open) {
      languagePrefillDone.current = false;
      return;
    }
    if (languagePrefillDone.current || inherited.isLoading) return;
    if (inherited.extractionLanguage) {
      setDraft((d) =>
        d.extractionLanguage == null
          ? { ...d, extractionLanguage: inherited.extractionLanguage }
          : d,
      );
    }
    languagePrefillDone.current = true;
  }, [open, inherited.isLoading, inherited.extractionLanguage]);

  const advancedValid = Boolean(llm?.provider && embedding?.provider && vision?.provider);
  const canGoNext = canProceed(stepId, draft, {
    hasConfiguredDefaults,
    advancedModelsValid: advancedValid,
  });

  const isDirty =
    stepIndex > 0 ||
    JSON.stringify(draftForStorage(draft)) !==
      JSON.stringify(draftForStorage(baselineRef.current));

  const reset = useCallback(() => {
    const next = initialWorkspaceDraft();
    baselineRef.current = next;
    languagePrefillDone.current = false;
    setStepIndex(0);
    setDraft(next);
    setLlm(undefined);
    setEmbedding(undefined);
    setVision(undefined);
    setAdvancedOpen(false);
    clearDraft();
  }, [clearDraft]);

  const handleOpenChange = (next: boolean) => {
    if (!next) reset();
    onOpenChange(next);
  };

  const goToStep = (target: WizardStepId) => {
    const idx = steps.indexOf(target);
    if (idx >= 0) setStepIndex(idx);
  };

  const finalize = async () => {
    if (!tenantId) {
      toast.error(t('workspace.noTenant', 'No tenant selected'));
      return;
    }
    setSubmitting(true);
    try {
      const models = buildWorkspaceModelPayload({
        useServerDefaults: draft.useServerDefaults,
        llm,
        embedding,
        vision,
      });
      const workspace = await createWorkspace(tenantId, {
        name: draft.workspaceName.trim(),
        slug: draft.workspaceSlug.trim() || undefined,
        description: draft.workspaceDescription.trim() || undefined,
        ...models,
        entity_types: draft.entityTypes.length > 0 ? draft.entityTypes : undefined,
        extraction_language: draft.extractionLanguage ?? undefined,
        entity_type_colors:
          Object.keys(draft.entityTypeColors).length > 0
            ? draft.entityTypeColors
            : undefined,
      });
      // Success toast + navigation CTA owned by caller (header / tenant-guard).
      onCreated?.(workspace);
      handleOpenChange(false);
    } catch (error) {
      toast.error(t('workspace.createFailed', 'Failed to create workspace'), {
        description: error instanceof Error ? error.message : 'Unknown error',
      });
    } finally {
      setSubmitting(false);
    }
  };

  const onNext = () => {
    if (stepIndex >= steps.length - 1) {
      void finalize();
      return;
    }
    setStepIndex((i) => clampStepIndex(i + 1, steps.length));
  };

  const body = (() => {
    switch (stepId) {
      case 'workspace-basics':
        return <WorkspaceBasicsStep draft={draft} onChange={patchDraft} />;
      case 'models':
        return (
          <ModelDefaultsStep
            draft={draft}
            onChange={patchDraft}
            tenantId={tenantId}
            llm={llm}
            embedding={embedding}
            vision={vision}
            onLlmChange={setLlm}
            onEmbeddingChange={setEmbedding}
            onVisionChange={setVision}
            advancedOpen={advancedOpen}
            onAdvancedOpenChange={setAdvancedOpen}
          />
        );
      case 'extraction':
        return <WorkspaceExtractionStep draft={draft} onChange={patchDraft} />;
      case 'review':
        return (
          <ReviewStep
            draft={draft}
            includeAdmin={false}
            includeTenant={false}
            tenantId={tenantId}
            llm={llm}
            embedding={embedding}
            vision={vision}
            onEditStep={goToStep}
          />
        );
      default:
        return null;
    }
  })();

  return (
    <WizardShell
      open={open}
      onOpenChange={handleOpenChange}
      title={t('workspace.createNew', 'Create New Workspace')}
      description={t(
        'workspace.createDescription',
        'Create a new workspace within the current tenant to organize your documents.',
      )}
      stepIndex={stepIndex}
      stepCount={steps.length}
      stepTitle={t(meta.titleKey, meta.title)}
      stepDescription={t(meta.descriptionKey, meta.description)}
      canGoNext={canGoNext}
      isLastStep={stepIndex >= steps.length - 1}
      isSubmitting={submitting}
      isDirty={isDirty}
      onBack={() => setStepIndex((i) => clampStepIndex(i - 1, steps.length))}
      onNext={onNext}
      finishLabel={t('onboarding.createWorkspace', 'Create workspace')}
      testId="create-workspace-wizard"
    >
      {body}
    </WizardShell>
  );
}
