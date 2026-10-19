'use client';

import { IdentityProvidersCard } from '@/components/settings/identity-providers-card';
import { EmptyState } from '@/components/shared/empty-state';
import { useAuthStore } from '@/stores/use-auth-store';
import { KeyRound, Lock } from 'lucide-react';
import { useTranslation } from 'react-i18next';

const FLOW = ['sso.flow.org', 'sso.flow.role', 'sso.flow.jit'] as const;
const FLOW_FALLBACK: Record<(typeof FLOW)[number], string> = {
  'sso.flow.org': 'A Keycloak Organization maps to an EdgeQuake organization, matched by its alias.',
  'sso.flow.role': "The IdP role claim sets the member's role at each sign-in, capped by the provider's maximum role.",
  'sso.flow.jit': 'New people are created on first sign-in when just-in-time provisioning is on. Pinning a role here stops the IdP overriding it.',
};

/** Platform-admin view of single sign-on; everyone else sees why it is unavailable. */
export function SsoTab() {
  const { t } = useTranslation();
  const user = useAuthStore((s) => s.user);
  const isAdmin = user?.role === 'admin' || user?.roles?.includes('admin') || false;

  if (!isAdmin) {
    return (
      <div className="rounded-lg border bg-card" data-testid="sso-restricted">
        <EmptyState
          icon={Lock}
          title={t('members.sso.restrictedTitle', 'Single sign-on is set up by instance admins')}
          description={t('members.sso.restrictedBody', 'Ask an instance admin to connect or change an identity provider.')}
        />
      </div>
    );
  }

  return (
    <div className="space-y-4" data-testid="sso-tab">
      <div className="rounded-lg border bg-card p-4">
        <h3 className="flex items-center gap-2 text-sm font-semibold">
          <KeyRound className="size-4" aria-hidden />
          {t('members.sso.howTitle', 'How single sign-on maps people')}
        </h3>
        <ul className="mt-2 list-disc space-y-1 pl-5 text-xs text-muted-foreground">
          {FLOW.map((k) => (
            <li key={k}>{t(`members.${k}`, FLOW_FALLBACK[k])}</li>
          ))}
        </ul>
      </div>
      <IdentityProvidersCard />
    </div>
  );
}
