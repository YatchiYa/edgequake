'use client';

import { Progress } from '@/components/ui/progress';
import type { MemberListResponse } from '@/lib/api/edgequake/members';
import { useTranslation } from 'react-i18next';

/** "12 people in Acme · 2 pending" plus seat usage when the plan caps users. */
export function MembersSummary({ data }: { data: MemberListResponse }) {
  const { t } = useTranslation();
  const { summary, seats, tenant_name } = data;
  const people = summary.active + summary.deactivated;
  const pct = seats.max ? Math.min(100, Math.round((seats.used / seats.max) * 100)) : null;
  return (
    <div className="flex flex-wrap items-center gap-x-4 gap-y-1 text-sm" data-testid="members-summary">
      <p className="text-muted-foreground">
        <span className="font-medium text-foreground tabular-nums">
          {t('members.summary.people', { count: people, defaultValue: '{{count}} people' })}
        </span>{' '}
        {t('members.summary.in', 'in {{org}}', { org: tenant_name })}
        {summary.invited > 0 && (
          <>
            {' · '}
            <span data-testid="members-pending-count">
              {t('members.summary.pending', { count: summary.invited, defaultValue: '{{count}} pending' })}
            </span>
          </>
        )}
      </p>
      {seats.max != null && (
        <div className="flex items-center gap-2 text-xs text-muted-foreground" data-testid="members-seats">
          <Progress value={pct ?? 0} className="h-1.5 w-24" aria-label={t('members.seats.label', 'Seats used')} />
          {t('members.seats.usage', '{{used}} of {{max}} seats', { used: seats.used, max: seats.max })}
        </div>
      )}
    </div>
  );
}
