'use client';

import { Button } from '@/components/ui/button';
import { DialogFooter } from '@/components/ui/dialog';
import { Input } from '@/components/ui/input';
import { Check, Copy, type LucideIcon } from 'lucide-react';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { toast } from 'sonner';

/**
 * A single-use link, shown once. Email delivery is out of band, so the link *is* the product:
 * invitations, sign-in links and password-reset links all end here (DRY).
 */
export function ShareLinkPanel({
  url,
  icon: Icon,
  message,
  hint,
  onDone,
  testIdPrefix,
  inputLabel,
}: {
  url: string;
  icon: LucideIcon;
  message: React.ReactNode;
  hint: React.ReactNode;
  onDone: () => void;
  /** `invite` | `access` — keeps test ids stable per flow. */
  testIdPrefix: string;
  inputLabel: string;
}) {
  const { t } = useTranslation();
  const [copied, setCopied] = useState(false);

  const copy = async () => {
    await navigator.clipboard?.writeText(url);
    setCopied(true);
    toast.success(t('members.share.copied', 'Link copied'));
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <div className="space-y-4" data-testid={`${testIdPrefix}-link-panel`}>
      <div className="flex items-start gap-3 rounded-lg border border-success/30 bg-success/10 p-3">
        <Icon className="mt-0.5 size-5 shrink-0 text-success" aria-hidden />
        <p className="text-sm">{message}</p>
      </div>
      <div className="flex gap-2">
        <Input
          readOnly
          value={url}
          data-testid={`${testIdPrefix}-link-input`}
          onFocus={(e) => e.currentTarget.select()}
          className="font-mono text-xs"
          aria-label={inputLabel}
        />
        <Button type="button" onClick={copy} data-testid={`${testIdPrefix}-copy`} variant="secondary" className="shrink-0">
          {copied ? <Check className="size-4" aria-hidden /> : <Copy className="size-4" aria-hidden />}
          {copied ? t('members.share.copiedShort', 'Copied') : t('members.share.copy', 'Copy link')}
        </Button>
      </div>
      <p className="text-xs text-muted-foreground">{hint}</p>
      <DialogFooter>
        <Button type="button" onClick={onDone} data-testid={`${testIdPrefix}-done`}>
          {t('members.share.done', 'Done')}
        </Button>
      </DialogFooter>
    </div>
  );
}
