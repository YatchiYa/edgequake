'use client';

import { StatusBadge as ToneBadge } from '@/components/shared/status-badge';
import type { MemberStatus } from '@/lib/api/edgequake/members';
import { STATUS_TONE } from '@/lib/members/role-meta';
import { useTranslation } from 'react-i18next';

const FALLBACK: Record<MemberStatus, string> = {
  active: 'Active',
  invited: 'Invited',
  expired: 'Expired',
  deactivated: 'Suspended',
};

/** Member lifecycle pill — the shared tone badge with member-specific labels. */
export function MemberStatusBadge({ status, testId }: { status: MemberStatus; testId?: string }) {
  const { t } = useTranslation();
  return (
    <span data-testid={testId ?? `status-${status}`}>
      <ToneBadge
        label={t(`members.status.${status}`, FALLBACK[status])}
        tone={STATUS_TONE[status]}
        className="normal-case"
      />
    </span>
  );
}
