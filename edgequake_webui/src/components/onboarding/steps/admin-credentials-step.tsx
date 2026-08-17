'use client';

import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import type { WizardDraft } from '@/lib/onboarding/wizard-state';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';

export interface AdminCredentialsStepProps {
  draft: WizardDraft;
  onChange: (patch: Partial<WizardDraft>) => void;
}

export function AdminCredentialsStep({ draft, onChange }: AdminCredentialsStepProps) {
  const { t } = useTranslation();
  const [touched, setTouched] = useState({
    username: false,
    password: false,
    confirm: false,
  });

  const usernameError =
    touched.username && draft.adminUsername.trim().length < 3
      ? t('onboarding.adminUsernameError', 'Username must be at least 3 characters.')
      : null;
  const passwordError =
    touched.password && draft.adminPassword.length > 0 && draft.adminPassword.length < 8
      ? t('onboarding.adminPasswordError', 'Password must be at least 8 characters.')
      : touched.password && draft.adminPassword.length === 0
        ? t('onboarding.adminPasswordRequired', 'Password is required.')
        : null;
  const confirmError =
    touched.confirm && draft.adminPasswordConfirm !== draft.adminPassword
      ? t('onboarding.adminPasswordMismatch', 'Passwords do not match.')
      : null;

  return (
    <div className="space-y-3" data-testid="wizard-step-admin">
      <div className="grid gap-2">
        <Label htmlFor="wizard-admin-username">
          {t('onboarding.adminUsername', 'Username')}
        </Label>
        <Input
          id="wizard-admin-username"
          autoComplete="username"
          value={draft.adminUsername}
          onChange={(e) => onChange({ adminUsername: e.target.value })}
          onBlur={() => setTouched((s) => ({ ...s, username: true }))}
          aria-invalid={Boolean(usernameError)}
          aria-describedby={usernameError ? 'wizard-admin-username-error' : undefined}
          data-testid="wizard-admin-username"
        />
        {usernameError ? (
          <p id="wizard-admin-username-error" role="alert" className="text-xs text-destructive">
            {usernameError}
          </p>
        ) : null}
      </div>
      <div className="grid gap-2">
        <Label htmlFor="wizard-admin-email">
          {t('onboarding.adminEmail', 'Email (optional)')}
        </Label>
        <Input
          id="wizard-admin-email"
          type="email"
          autoComplete="email"
          value={draft.adminEmail}
          onChange={(e) => onChange({ adminEmail: e.target.value })}
          placeholder="admin@localhost"
          data-testid="wizard-admin-email"
        />
      </div>
      <div className="grid gap-2">
        <Label htmlFor="wizard-admin-password">
          {t('onboarding.adminPassword', 'Password')}
        </Label>
        <Input
          id="wizard-admin-password"
          type="password"
          autoComplete="new-password"
          value={draft.adminPassword}
          onChange={(e) => onChange({ adminPassword: e.target.value })}
          onBlur={() => setTouched((s) => ({ ...s, password: true }))}
          aria-invalid={Boolean(passwordError)}
          aria-describedby={
            passwordError ? 'wizard-admin-password-error' : 'wizard-admin-password-hint'
          }
          data-testid="wizard-admin-password"
        />
        <p id="wizard-admin-password-hint" className="text-xs text-muted-foreground">
          {t('onboarding.adminPasswordHint', 'At least 8 characters. Store it securely.')}
        </p>
        {passwordError ? (
          <p id="wizard-admin-password-error" role="alert" className="text-xs text-destructive">
            {passwordError}
          </p>
        ) : null}
      </div>
      <div className="grid gap-2">
        <Label htmlFor="wizard-admin-password-confirm">
          {t('onboarding.adminPasswordConfirm', 'Confirm password')}
        </Label>
        <Input
          id="wizard-admin-password-confirm"
          type="password"
          autoComplete="new-password"
          value={draft.adminPasswordConfirm}
          onChange={(e) => onChange({ adminPasswordConfirm: e.target.value })}
          onBlur={() => setTouched((s) => ({ ...s, confirm: true }))}
          aria-invalid={Boolean(confirmError)}
          aria-describedby={confirmError ? 'wizard-admin-password-confirm-error' : undefined}
          data-testid="wizard-admin-password-confirm"
        />
        {confirmError ? (
          <p
            id="wizard-admin-password-confirm-error"
            role="alert"
            className="text-xs text-destructive"
          >
            {confirmError}
          </p>
        ) : null}
      </div>
    </div>
  );
}
