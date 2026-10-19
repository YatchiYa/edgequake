'use client';

import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Eye, EyeOff, TriangleAlert } from 'lucide-react';
import { useId, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { PasswordStrength } from './password-strength';

interface PasswordFieldProps {
  label: string;
  value: string;
  onChange: (value: string) => void;
  /** `new-password` (default) or `current-password`. */
  autoComplete?: 'new-password' | 'current-password';
  /** Show the strength meter and rule checklist (new passwords only). */
  showStrength?: boolean;
  disabled?: boolean;
  autoFocus?: boolean;
  invalid?: boolean;
  error?: string | null;
  testId?: string;
}

/**
 * The one password input: reveal toggle, Caps-Lock warning and (optionally) live policy feedback.
 * Shared by invitation accept, password reset, change password.
 */
export function PasswordField({
  label,
  value,
  onChange,
  autoComplete = 'new-password',
  showStrength = false,
  disabled,
  autoFocus,
  invalid,
  error,
  testId,
}: PasswordFieldProps) {
  const { t } = useTranslation();
  const id = useId();
  const [shown, setShown] = useState(false);
  const [caps, setCaps] = useState(false);
  const hintId = `${id}-hint`;
  const errorId = `${id}-error`;

  return (
    <div className="space-y-1.5">
      <Label htmlFor={id}>{label}</Label>
      <div className="relative">
        <Input
          id={id}
          data-testid={testId}
          type={shown ? 'text' : 'password'}
          autoComplete={autoComplete}
          autoFocus={autoFocus}
          disabled={disabled}
          value={value}
          onChange={(e) => onChange(e.target.value)}
          onKeyUp={(e) => setCaps(e.getModifierState?.('CapsLock') ?? false)}
          onBlur={() => setCaps(false)}
          aria-invalid={invalid || !!error || undefined}
          aria-describedby={[showStrength ? hintId : null, error ? errorId : null].filter(Boolean).join(' ') || undefined}
          className="pr-10"
          spellCheck={false}
          autoCapitalize="off"
        />
        <Button
          type="button"
          variant="ghost"
          size="icon"
          className="absolute right-0 top-0 size-9 text-muted-foreground"
          onClick={() => setShown((s) => !s)}
          aria-pressed={shown}
          aria-label={shown ? t('auth.password.hide', 'Hide password') : t('auth.password.show', 'Show password')}
          data-testid={testId ? `${testId}-toggle` : undefined}
          tabIndex={-1}
        >
          {shown ? <EyeOff className="size-4" aria-hidden /> : <Eye className="size-4" aria-hidden />}
        </Button>
      </div>
      {caps && (
        <p className="flex items-center gap-1.5 text-xs text-warning" role="status" data-testid="password-caps">
          <TriangleAlert className="size-3.5" aria-hidden />
          {t('auth.password.capsLock', 'Caps Lock is on')}
        </p>
      )}
      {error && (
        <p id={errorId} role="alert" className="text-xs text-destructive">
          {error}
        </p>
      )}
      {showStrength && <PasswordStrength id={hintId} value={value} />}
    </div>
  );
}
