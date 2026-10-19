'use client';

import {
  AlertDialog,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from '@/components/ui/alert-dialog';
import { Button } from '@/components/ui/button';
import type { MemberView } from '@/lib/api/edgequake/members';
import { displayName } from '@/lib/members/format';
import { Loader2, ShieldAlert } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import type { MemberAction } from './member-actions-menu';

export type ConfirmableAction = Extract<MemberAction, 'remove' | 'revoke' | 'suspend' | 'restore'>;

export interface PendingConfirm {
  action: ConfirmableAction;
  member: MemberView;
}

const DESTRUCTIVE: ConfirmableAction[] = ['remove', 'revoke', 'suspend'];

export function ConfirmMemberDialog({
  pending,
  busy,
  tenantName,
  onConfirm,
  onClose,
}: {
  pending: PendingConfirm | null;
  busy: boolean;
  tenantName: string;
  onConfirm: (p: PendingConfirm) => void;
  onClose: () => void;
}) {
  const { t } = useTranslation();
  const member = pending?.member;
  const action = pending?.action;
  const name = member ? displayName(member) : '';
  const ssoMember = !!member?.sign_in.some((s) => s.kind === 'sso');
  const leaving = action === 'remove' && member?.is_you;
  const key = leaving ? 'leave' : action;

  return (
    <AlertDialog open={!!pending} onOpenChange={(open) => !open && !busy && onClose()}>
      <AlertDialogContent data-testid="confirm-member-dialog">
        <AlertDialogHeader>
          <AlertDialogTitle>
            {key ? t(`members.confirm.${key}.title`, { name, org: tenantName }) : ''}
          </AlertDialogTitle>
          <AlertDialogDescription>
            {key ? t(`members.confirm.${key}.body`, { name, org: tenantName }) : ''}
          </AlertDialogDescription>
        </AlertDialogHeader>
        {action === 'remove' && ssoMember && !leaving && (
          <div
            role="note"
            data-testid="confirm-sso-warning"
            className="flex items-start gap-2 rounded-md border border-warning/40 bg-warning/10 p-3 text-xs"
          >
            <ShieldAlert className="mt-0.5 size-4 shrink-0 text-warning-foreground" aria-hidden />
            <span>
              {t(
                'members.confirm.ssoWarning',
                'This person signs in through single sign-on. Removing them here is undone the next time the identity provider signs them in. To lock them out for good, suspend the account or remove them in the identity provider.',
              )}
            </span>
          </div>
        )}
        <AlertDialogFooter>
          <AlertDialogCancel disabled={busy}>{t('common.cancel', 'Cancel')}</AlertDialogCancel>
          <Button
            data-testid="confirm-member-submit"
            variant={action && DESTRUCTIVE.includes(action) ? 'destructive' : 'default'}
            disabled={busy}
            onClick={() => pending && onConfirm(pending)}
          >
            {busy && <Loader2 className="size-4 animate-spin" aria-hidden />}
            {key ? t(`members.confirm.${key}.cta`) : ''}
          </Button>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  );
}
