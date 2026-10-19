'use client';

import { Table, TableBody, TableHead, TableHeader, TableRow } from '@/components/ui/table';
import type { MemberSort, MemberView, MemberViewer, OrgRole } from '@/lib/api/edgequake/members';
import { cn } from '@/lib/utils';
import { ArrowDown, ArrowUp, ChevronsUpDown } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import type { MemberAction } from './member-actions-menu';
import { MemberRow } from './member-row';

function SortHead({
  field,
  label,
  active,
  dir,
  onSort,
  className,
}: {
  field: MemberSort;
  label: string;
  active: boolean;
  dir: 'asc' | 'desc';
  onSort: (f: MemberSort) => void;
  className?: string;
}) {
  const Icon = active ? (dir === 'asc' ? ArrowUp : ArrowDown) : ChevronsUpDown;
  return (
    <TableHead
      className={className}
      aria-sort={active ? (dir === 'asc' ? 'ascending' : 'descending') : 'none'}
    >
      <button
        type="button"
        data-testid={`sort-${field}`}
        onClick={() => onSort(field)}
        className={cn(
          'inline-flex items-center gap-1 text-xs font-medium hover:text-foreground focus-visible:outline-none focus-visible:underline',
          active ? 'text-foreground' : 'text-muted-foreground',
        )}
      >
        {label}
        <Icon className="size-3" aria-hidden />
      </button>
    </TableHead>
  );
}

export function MembersTable({
  members,
  viewer,
  sort,
  dir,
  onSort,
  onRoleChange,
  onAction,
}: {
  members: MemberView[];
  viewer: MemberViewer;
  sort: MemberSort;
  dir: 'asc' | 'desc';
  onSort: (f: MemberSort) => void;
  onRoleChange: (m: MemberView, r: OrgRole) => void;
  onAction: (a: MemberAction, m: MemberView) => void;
}) {
  const { t } = useTranslation();
  const head = (field: MemberSort, label: string, className?: string) => (
    <SortHead field={field} label={label} active={sort === field} dir={dir} onSort={onSort} className={className} />
  );
  return (
    <div className="overflow-hidden rounded-lg border bg-card" data-testid="members-table">
      <Table>
        <TableHeader className="bg-muted/40">
          <TableRow className="hover:bg-transparent">
            {head('name', t('members.columns.person', 'Person'))}
            {head('role', t('members.columns.role', 'Role'))}
            <TableHead className="hidden text-xs font-medium text-muted-foreground md:table-cell">
              {t('members.columns.signIn', 'Sign-in')}
            </TableHead>
            <TableHead className="hidden text-xs font-medium text-muted-foreground sm:table-cell">{t('members.columns.status', 'Status')}</TableHead>
            {head('last_active', t('members.columns.lastActive', 'Last active'), 'hidden lg:table-cell')}
            <TableHead className="w-10">
              <span className="sr-only">{t('members.columns.actions', 'Actions')}</span>
            </TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {members.map((m) => (
            <MemberRow key={`${m.kind}-${m.id}`} member={m} viewer={viewer} onRoleChange={onRoleChange} onAction={onAction} />
          ))}
        </TableBody>
      </Table>
    </div>
  );
}
