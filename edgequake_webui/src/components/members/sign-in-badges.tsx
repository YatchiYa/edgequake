'use client';

import type { SignInMethod } from '@/lib/api/edgequake/members';
import { cn } from '@/lib/utils';
import { KeyRound, ShieldCheck } from 'lucide-react';
import { useTranslation } from 'react-i18next';

/** How a person authenticates: password and/or one chip per linked IdP (Keycloak first-class). */
export function SignInBadges({ methods, className }: { methods: SignInMethod[]; className?: string }) {
  const { t } = useTranslation();
  if (methods.length === 0) {
    return <span className="text-xs text-muted-foreground">—</span>;
  }
  return (
    <div className={cn('flex flex-wrap items-center gap-1', className)}>
      {methods.map((m, i) =>
        m.kind === 'password' ? (
          <span
            key={`pw-${i}`}
            data-testid="signin-password"
            className="inline-flex items-center gap-1 rounded-md border bg-background px-1.5 py-0.5 text-[0.6875rem] text-muted-foreground"
          >
            <KeyRound className="size-3" aria-hidden />
            {t('members.signIn.password', 'Password')}
          </span>
        ) : (
          <span
            key={`${m.provider_slug}-${i}`}
            data-testid={`signin-sso-${m.provider_slug}`}
            title={t('members.signIn.ssoVia', 'Single sign-on via {{name}}', { name: m.provider_name ?? m.provider_slug })}
            className="inline-flex items-center gap-1 rounded-md border border-teal-500/30 bg-teal-500/10 px-1.5 py-0.5 text-[0.6875rem] font-medium text-teal-700 dark:text-teal-300"
          >
            <ShieldCheck className="size-3" aria-hidden />
            {m.provider_name ?? m.provider_slug}
          </span>
        ),
      )}
    </div>
  );
}
