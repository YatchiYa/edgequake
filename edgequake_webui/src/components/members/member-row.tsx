'use client';

import { Badge } from '@/components/ui/badge';
import { TableCell, TableRow } from '@/components/ui/table';
import type { MemberView, MemberViewer, OrgRole } from '@/lib/api/edgequake/members';
import { displayName, expiresSoon, relativeTime } from '@/lib/members/format';
import { roleChangeBlock, roleOptions, type BlockReason } from '@/lib/members/permissions';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { MemberActionsMenu, type MemberAction } from './member-actions-menu';
import { MemberAvatar } from './member-avatar';
import { RoleBadge } from './role-badge';
import { RoleSelect } from './role-select';
import { SignInBadges } from './sign-in-badges';
import { MemberStatusBadge } from './status-badge';

/** `members.role.locked.<reason>` — why a role pill is not editable. */
export function useLockedReason() {
  const { t } = useTranslation();
  return (reason: BlockReason) =>
    reason && reason !== 'notPerson'
      ? t(`members.role.locked.${reason}`, {
          notManager: 'Only admins and owners can change roles.',
          outranked: 'You cannot change a role above your own.',
          lastOwner: 'Promote another owner first — an organization needs at least one.',
        }[reason])
      : null;
}

export function MemberRow({
  member,
  viewer,
  onRoleChange,
  onAction,
}: {
  member: MemberView;
  viewer: MemberViewer;
  onRoleChange: (member: MemberView, role: OrgRole) => void;
  onAction: (action: MemberAction, member: MemberView) => void;
}) {
  const { t, i18n } = useTranslation();
  const [now] = useState(() => Date.now());
  const lockedReason = useLockedReason();
  const isInvite = member.kind === 'invitation';
  const block = isInvite ? 'notPerson' : roleChangeBlock(viewer, member);
  const name = displayName(member);

  return (
    <TableRow
      data-testid={`member-row-${member.id}`}
      data-kind={member.kind}
      data-status={member.status}
      className={member.status === 'deactivated' ? 'opacity-70' : undefined}
    >
      <TableCell className="min-w-0 py-3">
        <div className="flex min-w-0 items-center gap-3">
          <MemberAvatar member={member} />
          <div className="min-w-0">
            <div className="flex items-center gap-1.5">
              {isInvite ? (
                <span className="truncate text-sm font-medium">{member.email}</span>
              ) : (
                <button
                  type="button"
                  onClick={() => onAction('details', member)}
                  className="truncate text-left text-sm font-medium hover:underline focus-visible:outline-none focus-visible:underline"
                >
                  {name}
                </button>
              )}
              {member.is_you && (
                <Badge variant="secondary" className="h-4 px-1.5 text-[0.625rem]">
                  {t('members.you', 'You')}
                </Badge>
              )}
            </div>
            {!isInvite && member.username && (
              <div className="truncate text-xs text-muted-foreground">{member.email}</div>
            )}
            {member.status !== 'active' && (
              <div className="mt-1 sm:hidden">
                <MemberStatusBadge status={member.status} testId="status-inline" />
              </div>
            )}
            {isInvite && (
              <div className="truncate text-xs text-muted-foreground">
                {member.invited_by
                  ? t('members.invitedBy', 'Invited by {{name}}', { name: member.invited_by })
                  : t('members.invitationPending', 'Invitation pending')}
              </div>
            )}
          </div>
        </div>
      </TableCell>

      <TableCell className="py-3">
        {isInvite ? (
          <RoleBadge role={member.role} />
        ) : (
          <RoleSelect
            testId={`role-select-${member.id}`}
            value={member.role}
            options={roleOptions(viewer)}
            lockedReason={lockedReason(block)}
            managedBy={member.role_managed_by}
            onChange={(r) => onRoleChange(member, r)}
          />
        )}
      </TableCell>

      <TableCell className="hidden py-3 md:table-cell">
        {isInvite ? <span className="text-xs text-muted-foreground">—</span> : <SignInBadges methods={member.sign_in} />}
      </TableCell>

      <TableCell className="hidden py-3 sm:table-cell">
        <MemberStatusBadge status={member.status} />
      </TableCell>

      <TableCell className="hidden whitespace-nowrap py-3 text-xs text-muted-foreground lg:table-cell">
        {isInvite
          ? member.expires_at
            ? (
                <span className={expiresSoon(member.expires_at) ? 'text-warning-foreground' : undefined}>
                  {member.status === 'expired'
                    ? t('members.expiredAgo', 'Expired {{when}}', { when: relativeTime(member.expires_at, now, i18n.language) })
                    : t('members.expiresIn', 'Expires {{when}}', { when: relativeTime(member.expires_at, now, i18n.language) })}
                </span>
              )
            : '—'
          : member.last_login_at
            ? relativeTime(member.last_login_at, now, i18n.language)
            : t('members.neverSignedIn', 'Never')}
      </TableCell>

      <TableCell className="w-10 py-3 text-right">
        <MemberActionsMenu member={member} viewer={viewer} onAction={onAction} />
      </TableCell>
    </TableRow>
  );
}
