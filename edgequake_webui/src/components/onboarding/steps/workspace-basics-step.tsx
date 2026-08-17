'use client';

import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Textarea } from '@/components/ui/textarea';
import type { WizardDraft } from '@/lib/onboarding/wizard-state';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';

export interface WorkspaceBasicsStepProps {
  draft: WizardDraft;
  onChange: (patch: Partial<WizardDraft>) => void;
}

export function WorkspaceBasicsStep({ draft, onChange }: WorkspaceBasicsStepProps) {
  const { t } = useTranslation();
  const [touched, setTouched] = useState(false);
  const nameError =
    touched && draft.workspaceName.trim().length === 0
      ? t('onboarding.workspaceNameRequired', 'Workspace name is required.')
      : null;

  return (
    <div className="space-y-3" data-testid="wizard-step-workspace-basics">
      <div className="grid gap-2">
        <Label htmlFor="wizard-workspace-name">
          {t('workspace.name', 'Name')}
          <span className="text-destructive ml-0.5">*</span>
        </Label>
        <Input
          id="wizard-workspace-name"
          value={draft.workspaceName}
          onChange={(e) => onChange({ workspaceName: e.target.value })}
          onBlur={() => setTouched(true)}
          placeholder={t('workspace.namePlaceholderExample', 'e.g. Project Alpha')}
          aria-invalid={Boolean(nameError)}
          aria-describedby={nameError ? 'wizard-workspace-name-error' : undefined}
          data-testid="wizard-workspace-name"
        />
        {nameError ? (
          <p id="wizard-workspace-name-error" role="alert" className="text-xs text-destructive">
            {nameError}
          </p>
        ) : null}
      </div>

      <div className="grid gap-2">
        <Label htmlFor="wizard-workspace-slug">
          {t('workspace.slug', 'Slug')}{' '}
          <span className="text-muted-foreground font-normal">
            ({t('common.optional', 'optional')})
          </span>
        </Label>
        <Input
          id="wizard-workspace-slug"
          value={draft.workspaceSlug}
          onChange={(e) => onChange({ workspaceSlug: e.target.value })}
          placeholder="auto-generated"
          aria-describedby="wizard-workspace-slug-hint"
          data-testid="wizard-workspace-slug"
        />
        <p id="wizard-workspace-slug-hint" className="text-xs text-muted-foreground">
          {t('workspace.slugHint', 'Used in URLs: `/query?workspace={slug}`')}
        </p>
      </div>

      <div className="grid gap-2">
        <Label htmlFor="wizard-workspace-description">
          {t('workspace.description', 'Description')}
        </Label>
        <Textarea
          id="wizard-workspace-description"
          value={draft.workspaceDescription}
          onChange={(e) => onChange({ workspaceDescription: e.target.value })}
          placeholder={t('workspace.descriptionPlaceholder', 'A brief description…')}
          rows={2}
          data-testid="wizard-workspace-description"
        />
      </div>
    </div>
  );
}
