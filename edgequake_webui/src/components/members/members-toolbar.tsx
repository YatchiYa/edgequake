'use client';

import { Input } from '@/components/ui/input';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import type { MemberStatus, MemberSummary, OrgRole } from '@/lib/api/edgequake/members';
import { ORG_ROLES } from '@/lib/members/role-meta';
import { cn } from '@/lib/utils';
import { Search, X } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import type { MemberFilters } from './use-member-filters';
import { useRoleLabel } from './use-role-label';

const STATUSES: (MemberStatus | 'all')[] = ['all', 'active', 'invited', 'expired', 'deactivated'];
const STATUS_FALLBACK: Record<string, string> = {
  all: 'All statuses',
  active: 'Active',
  invited: 'Invited',
  expired: 'Expired',
  deactivated: 'Suspended',
};

function Segment({
  active,
  onClick,
  children,
  testId,
}: {
  active: boolean;
  onClick: () => void;
  children: React.ReactNode;
  testId: string;
}) {
  return (
    <button
      type="button"
      role="radio"
      aria-checked={active}
      data-testid={testId}
      onClick={onClick}
      className={cn(
        'inline-flex h-7 items-center gap-1.5 rounded-md px-2.5 text-xs font-medium transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring',
        active ? 'bg-background text-foreground shadow-sm' : 'text-muted-foreground hover:text-foreground',
      )}
    >
      {children}
    </button>
  );
}

export function MembersToolbar({
  filters,
  summary,
}: {
  filters: MemberFilters;
  summary?: MemberSummary;
}) {
  const { t } = useTranslation();
  const roleLabel = useRoleLabel();
  const count = (r: OrgRole) => summary?.by_role[r];

  return (
    <div className="flex flex-col gap-3 lg:flex-row lg:items-center" data-testid="members-toolbar">
      <div className="relative min-w-0 flex-1 lg:max-w-sm">
        <Search className="pointer-events-none absolute left-2.5 top-1/2 size-4 -translate-y-1/2 text-muted-foreground" aria-hidden />
        <Input
          data-testid="members-search"
          value={filters.search}
          onChange={(e) => filters.setSearch(e.target.value)}
          placeholder={t('members.toolbar.search', 'Search by name or email')}
          aria-label={t('members.toolbar.search', 'Search by name or email')}
          className="h-9 pl-8 pr-8"
        />
        {filters.search && (
          <button
            type="button"
            aria-label={t('members.toolbar.clearSearch', 'Clear search')}
            onClick={() => filters.setSearch('')}
            className="absolute right-2 top-1/2 -translate-y-1/2 rounded p-0.5 text-muted-foreground hover:text-foreground"
          >
            <X className="size-3.5" />
          </button>
        )}
      </div>

      <div
        role="radiogroup"
        aria-label={t('members.toolbar.roleFilter', 'Filter by role')}
        className="flex w-fit max-w-full items-center gap-0.5 overflow-x-auto rounded-lg bg-muted p-0.5"
      >
        <Segment testId="role-filter-all" active={filters.role === 'all'} onClick={() => filters.setRole('all')}>
          {t('members.toolbar.allRoles', 'All')}
          {summary && <span className="tabular-nums text-muted-foreground">{summary.total}</span>}
        </Segment>
        {ORG_ROLES.map((r) => (
          <Segment key={r} testId={`role-filter-${r}`} active={filters.role === r} onClick={() => filters.setRole(r)}>
            {roleLabel(r)}
            {count(r) !== undefined && <span className="tabular-nums text-muted-foreground">{count(r)}</span>}
          </Segment>
        ))}
      </div>

      <Select value={filters.status} onValueChange={(v) => filters.setStatus(v as MemberStatus | 'all')}>
        <SelectTrigger data-testid="members-status-filter" className="h-9 w-full lg:w-40" aria-label={t('members.toolbar.statusFilter', 'Filter by status')}>
          <SelectValue />
        </SelectTrigger>
        <SelectContent>
          {STATUSES.map((s) => (
            <SelectItem key={s} value={s}>
              {s === 'all'
                ? t('members.toolbar.allStatuses', STATUS_FALLBACK.all)
                : t(`members.status.${s}`, STATUS_FALLBACK[s])}
            </SelectItem>
          ))}
        </SelectContent>
      </Select>
    </div>
  );
}
