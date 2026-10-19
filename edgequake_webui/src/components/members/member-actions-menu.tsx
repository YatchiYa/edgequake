'use client';

import { Button } from '@/components/ui/button';
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuSeparator, DropdownMenuTrigger } from '@/components/ui/dropdown-menu';
import type { MemberView, MemberViewer } from '@/lib/api/edgequake/members';
import { canSuspend, linkBlock, removeBlock, viewerCanManage } from '@/lib/members/permissions';
import { Copy, KeyRound, LogOut, MailPlus, MoreHorizontal, PanelRight, Trash2, UserCheck, UserX, WandSparkles } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import { toast } from 'sonner';

export type MemberAction = 'remove' | 'revoke' | 'suspend' | 'restore' | 'resend' | 'details' | 'magic-link' | 'reset-link';

/** Per-row "⋯" menu. Items the viewer cannot perform are omitted rather than disabled. */
export function MemberActionsMenu({
  member,
  viewer,
  onAction,
}: {
  member: MemberView;
  viewer: MemberViewer;
  onAction: (action: MemberAction, member: MemberView) => void;
}) {
  const { t } = useTranslation();
  const manage = viewerCanManage(viewer);
  const isInvite = member.kind === 'invitation';
  const canRemove = !isInvite && removeBlock(viewer, member) === null;
  const canLink = linkBlock(viewer, member) === null;

  const copyEmail = async () => {
    await navigator.clipboard?.writeText(member.email);
    toast.success(t('members.toasts.emailCopied', 'Email copied'));
  };

  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button
          variant="ghost"
          size="icon"
          className="size-8 text-muted-foreground"
          data-testid={`member-actions-${member.id}`}
          aria-label={t('members.actions.open', 'Actions for {{name}}', { name: member.username ?? member.email })}
        >
          <MoreHorizontal className="size-4" aria-hidden />
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end" className="w-52">
        {!isInvite && (
          <DropdownMenuItem onSelect={() => onAction('details', member)} data-testid="action-details">
            <PanelRight className="size-4" aria-hidden /> {t('members.actions.details', 'View details')}
          </DropdownMenuItem>
        )}
        <DropdownMenuItem onSelect={copyEmail} data-testid="action-copy-email">
          <Copy className="size-4" aria-hidden /> {t('members.actions.copyEmail', 'Copy email')}
        </DropdownMenuItem>
        {isInvite && manage && (
          <DropdownMenuItem onSelect={() => onAction('resend', member)} data-testid="action-resend">
            <MailPlus className="size-4" aria-hidden /> {t('members.actions.resend', 'Resend invitation')}
          </DropdownMenuItem>
        )}
        {canLink && (
          <>
            <DropdownMenuSeparator />
            <DropdownMenuItem onSelect={() => onAction('magic-link', member)} data-testid="action-magic-link">
              <WandSparkles className="size-4" aria-hidden /> {t('members.actions.magicLink', 'Create sign-in link')}
            </DropdownMenuItem>
            <DropdownMenuItem onSelect={() => onAction('reset-link', member)} data-testid="action-reset-link">
              <KeyRound className="size-4" aria-hidden /> {t('members.actions.resetPassword', 'Reset password')}
            </DropdownMenuItem>
          </>
        )}
        {(canSuspend(viewer, member) || canRemove || (isInvite && manage)) && <DropdownMenuSeparator />}
        {canSuspend(viewer, member) && (
          <DropdownMenuItem
            onSelect={() => onAction(member.status === 'deactivated' ? 'restore' : 'suspend', member)}
            data-testid="action-suspend"
          >
            {member.status === 'deactivated' ? <UserCheck className="size-4" aria-hidden /> : <UserX className="size-4" aria-hidden />}
            {member.status === 'deactivated'
              ? t('members.actions.restore', 'Restore account')
              : t('members.actions.suspend', 'Suspend account')}
          </DropdownMenuItem>
        )}
        {canRemove && (
          <DropdownMenuItem variant="destructive" onSelect={() => onAction('remove', member)} data-testid="action-remove">
            {member.is_you ? <LogOut className="size-4" aria-hidden /> : <Trash2 className="size-4" aria-hidden />}
            {member.is_you ? t('members.actions.leave', 'Leave organization') : t('members.actions.remove', 'Remove from organization')}
          </DropdownMenuItem>
        )}
        {isInvite && manage && (
          <DropdownMenuItem variant="destructive" onSelect={() => onAction('revoke', member)} data-testid="action-revoke">
            <Trash2 className="size-4" aria-hidden /> {t('members.actions.revoke', 'Revoke invitation')}
          </DropdownMenuItem>
        )}
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
