'use client';

import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Textarea } from '@/components/ui/textarea';
import type { WizardDraft } from '@/lib/onboarding/wizard-state';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';

export interface TenantBasicsStepProps {
  draft: WizardDraft;
  onChange: (patch: Partial<WizardDraft>) => void;
}

export function TenantBasicsStep({ draft, onChange }: TenantBasicsStepProps) {
  const { t } = useTranslation();
  const [touched, setTouched] = useState(false);
  const nameError =
    touched && draft.tenantName.trim().length === 0
      ? t('onboarding.tenantNameRequired', 'Organization name is required.')
      : null;

  return (
    <div className="space-y-3" data-testid="wizard-step-tenant-basics">
      <div className="grid gap-2">
        <Label htmlFor="wizard-tenant-name">
          {t('tenant.name', 'Name')}
          <span className="text-destructive ml-0.5">*</span>
        </Label>
        <Input
          id="wizard-tenant-name"
          value={draft.tenantName}
          onChange={(e) => onChange({ tenantName: e.target.value })}
          onBlur={() => setTouched(true)}
          placeholder={t('tenant.namePlaceholder', 'My Organization')}
          aria-invalid={Boolean(nameError)}
          aria-describedby={nameError ? 'wizard-tenant-name-error' : undefined}
          data-testid="wizard-tenant-name"
        />
        {nameError ? (
          <p id="wizard-tenant-name-error" role="alert" className="text-xs text-destructive">
            {nameError}
          </p>
        ) : null}
      </div>
      <div className="grid gap-2">
        <Label htmlFor="wizard-tenant-description">
          {t('tenant.description', 'Description')}
        </Label>
        <Textarea
          id="wizard-tenant-description"
          value={draft.tenantDescription}
          onChange={(e) => onChange({ tenantDescription: e.target.value })}
          placeholder={t('tenant.descriptionPlaceholder', 'A brief description…')}
          rows={3}
          data-testid="wizard-tenant-description"
        />
      </div>
    </div>
  );
}
