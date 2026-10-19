'use client';

import { EmptyState } from '@/components/shared/empty-state';
import { ErrorState } from '@/components/shared/error-state';
import { Button } from '@/components/ui/button';
import { Skeleton } from '@/components/ui/skeleton';
import { ChevronLeft, ChevronRight, Lock, SearchX, UserPlus } from 'lucide-react';
import { useTranslation } from 'react-i18next';

export function MembersSkeleton() {
  return (
    <div className="overflow-hidden rounded-lg border bg-card" data-testid="members-skeleton" aria-busy="true">
      {Array.from({ length: 6 }).map((_, i) => (
        <div key={i} className="flex items-center gap-3 border-b px-4 py-3 last:border-b-0">
          <Skeleton className="size-9 rounded-full" />
          <div className="flex-1 space-y-1.5">
            <Skeleton className="h-3.5 w-40" />
            <Skeleton className="h-3 w-56" />
          </div>
          <Skeleton className="hidden h-5 w-20 rounded-full sm:block" />
          <Skeleton className="hidden h-5 w-16 md:block" />
        </div>
      ))}
    </div>
  );
}

export function MembersRestricted() {
  const { t } = useTranslation();
  return (
    <div data-testid="members-restricted" className="rounded-lg border bg-card">
      <EmptyState
        icon={Lock}
        title={t('members.restricted.title', 'Members are managed by admins')}
        description={t(
          'members.restricted.body',
          'Only owners and admins of this organization can see and manage its members. Ask an owner if you need access.',
        )}
      />
    </div>
  );
}

export function MembersError({ onRetry }: { onRetry: () => void }) {
  const { t } = useTranslation();
  return (
    <div data-testid="members-error">
      <ErrorState
        title={t('members.error.title', "Couldn't load members")}
        description={t('members.error.body', 'Check your connection and try again.')}
        onRetry={onRetry}
      />
    </div>
  );
}

export function MembersNoMatch({ onClear }: { onClear: () => void }) {
  const { t } = useTranslation();
  return (
    <div data-testid="members-no-match" className="rounded-lg border bg-card">
      <EmptyState
        icon={SearchX}
        title={t('members.noMatch.title', 'No one matches these filters')}
        description={t('members.noMatch.body', 'Try a different name, role or status.')}
        action={{ label: t('members.noMatch.clear', 'Clear filters'), onClick: onClear }}
      />
    </div>
  );
}

/** Shown when you are alone in the organization — the moment an invite matters most. */
export function MembersAlone({ canInvite, onInvite }: { canInvite: boolean; onInvite: () => void }) {
  const { t } = useTranslation();
  return (
    <div
      data-testid="members-alone"
      className="mt-4 flex flex-col items-center gap-2 rounded-lg border border-dashed bg-card px-4 py-8 text-center"
    >
      <UserPlus className="size-6 text-muted-foreground" aria-hidden />
      <h3 className="text-sm font-semibold">{t('members.alone.title', "It's just you for now")}</h3>
      <p className="max-w-sm text-xs text-muted-foreground">
        {t('members.alone.body', 'Invite teammates to share documents, graphs and answers in this organization.')}
      </p>
      {canInvite && (
        <Button size="sm" onClick={onInvite} data-testid="members-alone-invite">
          {t('members.invite.cta', 'Invite people')}
        </Button>
      )}
    </div>
  );
}

export function MembersPager({
  page,
  pageSize,
  total,
  onPage,
}: {
  page: number;
  pageSize: number;
  total: number;
  onPage: (p: number) => void;
}) {
  const { t } = useTranslation();
  if (total <= pageSize) return null;
  const from = (page - 1) * pageSize + 1;
  const to = Math.min(total, page * pageSize);
  const last = Math.ceil(total / pageSize);
  return (
    <nav className="flex items-center justify-between pt-3 text-xs text-muted-foreground" aria-label={t('members.pager.label', 'Pagination')}>
      <span data-testid="members-pager-range">{t('members.pager.range', '{{from}}–{{to}} of {{total}}', { from, to, total })}</span>
      <div className="flex gap-1">
        <Button variant="outline" size="icon" className="size-8" disabled={page <= 1} onClick={() => onPage(page - 1)} aria-label={t('members.pager.prev', 'Previous page')}>
          <ChevronLeft className="size-4" />
        </Button>
        <Button variant="outline" size="icon" className="size-8" disabled={page >= last} onClick={() => onPage(page + 1)} aria-label={t('members.pager.next', 'Next page')}>
          <ChevronRight className="size-4" />
        </Button>
      </div>
    </nav>
  );
}
