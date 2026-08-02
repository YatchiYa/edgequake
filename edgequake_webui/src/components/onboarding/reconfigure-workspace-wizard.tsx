'use client';

import { WizardShell } from '@/components/onboarding/wizard-shell';
import {
  RECONFIGURE_REVIEW_META,
  STEP_META,
} from '@/components/onboarding/step-meta';
import { DocumentParsingStep } from '@/components/onboarding/steps/document-parsing-step';
import { ModelDefaultsStep } from '@/components/onboarding/steps/model-defaults-step';
import { ReviewStep } from '@/components/onboarding/steps/review-step';
import { WorkspaceExtractionStep } from '@/components/onboarding/steps/workspace-extraction-step';
import type { EmbeddingSelection } from '@/components/workspace/embedding-model-selector';
import type { LLMSelection } from '@/components/workspace/llm-model-selector';
import { useInheritedModelDefaults } from '@/hooks/use-inherited-model-defaults';
import { updateWorkspace } from '@/lib/api/edgequake';
import { buildWorkspaceUpdatePayload } from '@/lib/onboarding/model-payload';
import {
  prefillReconfigureFromWorkspace,
  snapshotFromWizardState,
} from '@/lib/onboarding/reconfigure-from-workspace';
import { useWizardDraftPersistence } from '@/lib/onboarding/use-wizard-draft-persistence';
import {
  hydrateWizardDraft,
  loadWizardDraft,
} from '@/lib/onboarding/wizard-draft-storage';
import {
  diffWorkspaceConfig,
  toPendingRebuild,
  type WorkspaceConfigSnapshot,
  type WorkspaceRebuildHints,
} from '@/lib/onboarding/workspace-config-diff';
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

export interface ReconfigureWorkspaceWizardProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  tenantId: string | null;
  workspace: Workspace | null;
  /** Document count for Impact rebuild urgency (EC-101-16…18). */
  documentCount?: number;
  onApplied?: (result: {
    workspace: Workspace;
    pendingRebuild: WorkspaceRebuildHints | null;
    extractionLanguageChanged: boolean;
  }) => void;
}

/**
 * SPEC-101 Wave 8 — Guided Reconfigure Workspace wizard (LAW-101-12).
 */
export function ReconfigureWorkspaceWizard({
  open,
  onOpenChange,
  tenantId,
  workspace,
  documentCount = 0,
  onApplied,
}: ReconfigureWorkspaceWizardProps) {
  const { t } = useTranslation();
  const steps = useMemo(() => stepsForWizard('reconfigure-workspace'), []);
  const baselineRef = useRef(EMPTY_WIZARD_DRAFT);
  const baselineSnapshotRef = useRef<WorkspaceConfigSnapshot | null>(null);
  const [stepIndex, setStepIndex] = useState(0);
  const [draft, setDraft] = useState<WizardDraft>(EMPTY_WIZARD_DRAFT);
  const [llm, setLlm] = useState<LLMSelection | undefined>();
  const [embedding, setEmbedding] = useState<EmbeddingSelection | undefined>();
  const [vision, setVision] = useState<LLMSelection | undefined>();
  const [advancedOpen, setAdvancedOpen] = useState(false);
  const [submitting, setSubmitting] = useState(false);
  const workspaceId = workspace?.id ?? null;
  const inherited = useInheritedModelDefaults(tenantId);
  const { hasConfiguredDefaults } = inherited;
  const prefilledForId = useRef<string | null>(null);

  const { clearDraft } = useWizardDraftPersistence(
    'reconfigure-workspace',
    open,
    draft,
    setDraft,
    stepIndex,
    setStepIndex,
    steps.length,
    workspaceId,
    false, // caller hydrates + prefills (EC-101-22)
  );

  // Prefill from workspace, or restore session draft for this workspace id.
  useEffect(() => {
    if (!open || !workspace) {
      if (!open) prefilledForId.current = null;
      return;
    }
    if (prefilledForId.current === workspace.id) return;
    const prefill = prefillReconfigureFromWorkspace(workspace);
    baselineRef.current = prefill.draft;
    baselineSnapshotRef.current = prefill.snapshot;
    const stored = loadWizardDraft('reconfigure-workspace', workspace.id);
    if (stored) {
      setDraft(hydrateWizardDraft(prefill.draft, stored));
      setStepIndex(clampStepIndex(stored.stepIndex, steps.length));
      // Keep model picks from workspace baseline; Advanced mirrors overrides.
      setLlm(prefill.llm);
      setEmbedding(prefill.embedding);
      setVision(prefill.vision);
      setAdvancedOpen(prefill.advancedOpen || !prefill.draft.useServerDefaults);
    } else {
      setDraft(prefill.draft);
      setLlm(prefill.llm);
      setEmbedding(prefill.embedding);
      setVision(prefill.vision);
      setAdvancedOpen(prefill.advancedOpen);
      setStepIndex(0);
    }
    prefilledForId.current = workspace.id;
  }, [open, workspace, steps.length]);

  const stepId = steps[clampStepIndex(stepIndex, steps.length)];
  const meta =
    stepId === 'review'
      ? RECONFIGURE_REVIEW_META
      : STEP_META[stepId];

  const patchDraft = useCallback((patch: Partial<WizardDraft>) => {
    setDraft((d) => ({ ...d, ...patch }));
  }, []);

  const currentSnapshot = useMemo(
    () =>
      snapshotFromWizardState({
        draft,
        llm,
        embedding,
        vision,
      }),
    [draft, llm, embedding, vision],
  );

  const impact = useMemo(() => {
    const baseline = baselineSnapshotRef.current;
    if (!baseline) {
      return diffWorkspaceConfig(currentSnapshot, currentSnapshot, {
        documentCount,
      });
    }
    return diffWorkspaceConfig(baseline, currentSnapshot, { documentCount });
  }, [currentSnapshot, documentCount]);

  // Vision optional on reconfigure (empty clears override → server default).
  const advancedValid = Boolean(llm?.provider && embedding?.provider);
  const canGoNext = canProceed(stepId, draft, {
    hasConfiguredDefaults,
    advancedModelsValid: advancedValid,
    hasConfigChanges: stepId === 'review' ? impact.hasChanges : undefined,
  });

  const isDirty =
    stepIndex > 0 ||
    JSON.stringify(draftForStorage(draft)) !==
      JSON.stringify(draftForStorage(baselineRef.current)) ||
    impact.hasChanges;

  const reset = useCallback(() => {
    prefilledForId.current = null;
    baselineRef.current = EMPTY_WIZARD_DRAFT;
    baselineSnapshotRef.current = null;
    setStepIndex(0);
    setDraft(EMPTY_WIZARD_DRAFT);
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
    if (!tenantId || !workspace) {
      toast.error(t('workspace.noWorkspaceSelected', 'No Workspace Selected'));
      return;
    }
    if (!impact.hasChanges) {
      toast.info(t('onboarding.noChanges', 'No changes to apply'));
      return;
    }
    setSubmitting(true);
    try {
      const payload = buildWorkspaceUpdatePayload({
        useServerDefaults: draft.useServerDefaults,
        llm,
        embedding,
        vision,
        pdfParserBackend: draft.pdfParserBackend,
        extractionLanguage: draft.extractionLanguage,
        entityTypes: draft.entityTypes,
        entityTypesStrict: draft.entityTypesStrict,
        entityTypeColors: draft.entityTypeColors,
      });
      const updated = await updateWorkspace(tenantId, workspace.id, payload);
      const pendingRebuild = toPendingRebuild(impact.rebuildHints);
      const extractionLanguageChanged = impact.changedKeys.includes('extractionLanguage');
      onApplied?.({
        workspace: updated,
        pendingRebuild,
        extractionLanguageChanged,
      });
      handleOpenChange(false);
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Unknown error';
      if (/404|not found/i.test(message)) {
        toast.error(t('workspace.notFound', 'Workspace Not Found'));
        handleOpenChange(false);
        return;
      }
      toast.error(t('workspace.updateFailed', 'Failed to update workspace'), {
        description: message,
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
      case 'document-parsing':
        return <DocumentParsingStep draft={draft} onChange={patchDraft} />;
      case 'extraction':
        return (
          <WorkspaceExtractionStep draft={draft} onChange={patchDraft} showStrict />
        );
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
            hideWorkspaceIdentity
            showDocumentParsing
            impact={impact}
            documentCount={documentCount}
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
      title={t('onboarding.reconfigureTitle', 'Reconfigure workspace')}
      description={t(
        'onboarding.reconfigureDescription',
        'Update models, document parsing, and extraction for this workspace.',
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
      finishLabel={t('onboarding.applyChanges', 'Apply')}
      testId="reconfigure-workspace-wizard"
    >
      {body}
    </WizardShell>
  );
}
