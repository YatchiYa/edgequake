'use client';

import type { OrgRole } from '@/lib/api/edgequake/members';
import { ROLE_ICON } from '@/lib/members/role-meta';
import { cn } from '@/lib/utils';
import { useRoleLabel } from './use-role-label';

const TONE: Record<OrgRole, string> = {
  owner: 'border-amber-500/30 bg-amber-500/10 text-amber-700 dark:text-amber-300',
  admin: 'border-violet-500/30 bg-violet-500/10 text-violet-700 dark:text-violet-300',
  member: 'border-sky-500/30 bg-sky-500/10 text-sky-700 dark:text-sky-300',
  readonly: 'border-border bg-muted text-muted-foreground',
};

export function RoleBadge({ role, className }: { role: OrgRole; className?: string }) {
  const label = useRoleLabel()(role);
  const Icon = ROLE_ICON[role];
  return (
    <span
      data-testid={`role-badge-${role}`}
      className={cn(
        'inline-flex items-center gap-1 rounded-full border px-2 py-0.5 text-xs font-medium',
        TONE[role],
        className,
      )}
    >
      <Icon className="size-3" aria-hidden />
      {label}
    </span>
  );
}
