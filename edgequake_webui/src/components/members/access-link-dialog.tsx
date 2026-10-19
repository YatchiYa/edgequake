'use client';

import { Button } from '@/components/ui/button';
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from '@/components/ui/dialog';
import { accessUrl, type AccessLinkIssued, type AccessPurpose } from '@/lib/api/edgequake/access-links';
import type { MemberView } from '@/lib/api/edgequake/members';
import { displayName } from '@/lib/members/format';
import { KeyRound, Link2, Loader2, ShieldAlert, WandSparkles } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import { ShareLinkPanel } from './share-link-panel';

export interface PendingLink {
  purpose: AccessPurpose;
  member: MemberView;
}

const ICON = { magic_login: WandSparkles, password_reset: KeyRound } as const;

function duration(minutes: number, t: ReturnType<typeof useTranslation>['t']): string {
  return minutes >= 60
    ? t('members.access.hours', { count: Math.round(minutes / 60) })
    : t('members.access.minutes', { count: minutes });
}

/** Two steps: explain what the link grants → show it once. */
export function AccessLinkDialog({
  pending,
  issued,
  busy,
  onCreate,
  onClose,
}: {
  pending: PendingLink | null;
  issued: AccessLinkIssued | null;
  busy: boolean;
  onCreate: (p: PendingLink) => void;
  onClose: () => void;
}) {
  const { t } = useTranslation();
  const purpose = pending?.purpose ?? 'magic_login';
  const name = pending ? displayName(pending.member) : '';
  const kind = purpose === 'magic_login' ? 'magic' : 'reset';
  const Icon = ICON[purpose];

  return (
    <Dialog open={!!pending} onOpenChange={(o) => !o && !busy && onClose()}>
      <DialogContent className="sm:max-w-lg" data-testid="access-link-dialog">
        <DialogHeader>
          <DialogTitle>{t(`members.access.${kind}.title`, { name })}</DialogTitle>
          <DialogDescription>{t(`members.access.${kind}.description`, { name })}</DialogDescription>
        </DialogHeader>

        {issued && pending ? (
          <ShareLinkPanel
            testIdPrefix="access"
            icon={Link2}
            url={accessUrl(issued.token)}
            onDone={onClose}
            inputLabel={t('members.access.link', 'Access link')}
            message={t(`members.access.${kind}.created`, {
              name,
              duration: duration(issued.expires_in_minutes, t),
            })}
            hint={t('members.access.shownOnce', 'This link is shown only once. Creating another one replaces it.')}
          />
        ) : (
          <>
            <div className="space-y-3 text-sm">
              <div className="flex items-start gap-2 rounded-md border border-warning/40 bg-warning/10 p-3 text-xs" role="note" data-testid="access-link-warning">
                <ShieldAlert className="mt-0.5 size-4 shrink-0 text-warning-foreground" aria-hidden />
                <span>{t('members.access.warning', 'Anyone who opens this link signs in as {{name}}. Send it privately, never in a shared channel.', { name })}</span>
              </div>
              <ul className="list-disc space-y-1 pl-5 text-muted-foreground">
                <li>{t(`members.access.${kind}.rule1`)}</li>
                <li>{t(`members.access.${kind}.rule2`)}</li>
                <li>{t('members.access.rule3', 'It works once. A new link replaces the previous one.')}</li>
              </ul>
            </div>
            <DialogFooter>
              <Button type="button" variant="ghost" onClick={onClose} disabled={busy}>
                {t('common.cancel', 'Cancel')}
              </Button>
              <Button type="button" disabled={busy || !pending} onClick={() => pending && onCreate(pending)} data-testid="access-link-create">
                {busy ? <Loader2 className="size-4 animate-spin" aria-hidden /> : <Icon className="size-4" aria-hidden />}
                {t('members.access.create', 'Create link')}
              </Button>
            </DialogFooter>
          </>
        )}
      </DialogContent>
    </Dialog>
  );
}
