/**
 * SPEC-165 — full-screen agents platform. Providers (query, theme, i18n, tenant)
 * come from the root layout; this layout adds the session guard and its own shell.
 */
import { AgentsShell } from '@/components/agents/agents-shell';
import { AuthGuard } from '@/components/layout/auth-guard';
import { TenantGuard } from '@/components/layout/tenant-guard';
import type { Metadata } from 'next';

export const metadata: Metadata = { title: 'Agents · EdgeQuake' };

export default function AgentsLayout({ children }: { children: React.ReactNode }) {
  return (
    <AuthGuard>
      <AgentsShell>
        <TenantGuard>{children}</TenantGuard>
      </AgentsShell>
    </AuthGuard>
  );
}
