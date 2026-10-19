'use client';

import { assessPassword, type RuleId, type Strength } from '@/lib/auth/password-policy';
import { cn } from '@/lib/utils';
import { Check, Circle, X } from 'lucide-react';
import { useTranslation } from 'react-i18next';

const SEGMENT_TONE: Record<Strength, string> = {
  0: 'bg-muted',
  1: 'bg-destructive',
  2: 'bg-warning',
  3: 'bg-success',
  4: 'bg-success',
};

const LABELS: Record<Strength, [string, string]> = {
  0: ['empty', ''],
  1: ['weak', 'Weak'],
  2: ['fair', 'Fair'],
  3: ['good', 'Good'],
  4: ['excellent', 'Excellent'],
};

const RULE_TEXT: Record<RuleId, [string, string]> = {
  length: ['length', '8 to 128 characters'],
  variety: ['variety', '3 of 4: uppercase, lowercase, number, symbol'],
  common: ['common', 'No common patterns such as "password"'],
};

/** Live strength meter plus the three server rules, ticked as they are met. */
export function PasswordStrength({ value, id }: { value: string; id: string }) {
  const { t } = useTranslation();
  const { rules, strength } = assessPassword(value);
  const [labelKey, labelDefault] = LABELS[strength];
  const typing = value.length > 0;

  return (
    <div id={id} className="space-y-2" data-testid="password-strength" data-strength={strength}>
      <div className="flex items-center gap-2">
        <div className="flex flex-1 gap-1" aria-hidden>
          {[1, 2, 3, 4].map((n) => (
            <span key={n} className={cn('h-1.5 flex-1 rounded-full transition-colors', n <= strength ? SEGMENT_TONE[strength] : 'bg-muted')} />
          ))}
        </div>
        <span className="min-w-14 text-right text-xs font-medium text-muted-foreground" aria-live="polite" data-testid="password-strength-label">
          {typing ? t(`auth.password.strength.${labelKey}`, labelDefault) : ''}
        </span>
      </div>
      <ul className="space-y-1">
        {rules.map((rule) => {
          const [key, text] = RULE_TEXT[rule.id];
          const Icon = !typing ? Circle : rule.ok ? Check : X;
          return (
            <li
              key={rule.id}
              data-testid={`password-rule-${rule.id}`}
              data-ok={typing ? rule.ok : undefined}
              className={cn(
                'flex items-center gap-1.5 text-xs',
                !typing ? 'text-muted-foreground' : rule.ok ? 'text-success' : 'text-muted-foreground',
              )}
            >
              <Icon className="size-3.5 shrink-0" aria-hidden />
              {t(`auth.password.rules.${key}`, text)}
            </li>
          );
        })}
      </ul>
    </div>
  );
}
