'use client';

import { assessPassword } from '@/lib/auth/password-policy';
import { useTranslation } from 'react-i18next';
import { PasswordField } from './password-field';

/** Can this pair be submitted? (Server re-validates; this keeps the button honest.) */
export const newPasswordReady = (password: string, confirm: string) =>
  assessPassword(password).valid && password === confirm;

/** "New password" + "Confirm" with live policy feedback and a mismatch message. */
export function NewPasswordFields({
  password,
  confirm,
  onPassword,
  onConfirm,
  disabled,
  autoFocus,
  newLabel,
  testIdPrefix = 'new-password',
}: {
  password: string;
  confirm: string;
  onPassword: (v: string) => void;
  onConfirm: (v: string) => void;
  disabled?: boolean;
  autoFocus?: boolean;
  newLabel?: string;
  testIdPrefix?: string;
}) {
  const { t } = useTranslation();
  const mismatch = confirm !== '' && confirm !== password;
  return (
    <>
      <PasswordField
        label={newLabel ?? t('auth.password.new', 'New password')}
        value={password}
        onChange={onPassword}
        showStrength
        disabled={disabled}
        autoFocus={autoFocus}
        testId={`${testIdPrefix}`}
      />
      <PasswordField
        label={t('auth.password.confirm', 'Confirm password')}
        value={confirm}
        onChange={onConfirm}
        disabled={disabled}
        error={mismatch ? t('auth.password.mismatch', 'Passwords do not match.') : null}
        testId={`${testIdPrefix}-confirm`}
      />
    </>
  );
}
