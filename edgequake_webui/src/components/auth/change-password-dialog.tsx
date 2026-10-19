'use client';

import { Button } from '@/components/ui/button';
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from '@/components/ui/dialog';
import { changePassword } from '@/lib/api/edgequake/access-links';
import { memberErrorKey } from '@/lib/members/errors';
import { useAuthStore } from '@/stores/use-auth-store';
import { useMutation } from '@tanstack/react-query';
import { AlertCircle, Loader2 } from 'lucide-react';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { toast } from 'sonner';
import { NewPasswordFields, newPasswordReady } from './new-password-fields';
import { PasswordField } from './password-field';

/** Change your own password. Other devices are signed out; this one gets a fresh session. */
export function ChangePasswordDialog({ open, onClose }: { open: boolean; onClose: () => void }) {
  const { t } = useTranslation();
  const authLogin = useAuthStore((s) => s.login);
  const [current, setCurrent] = useState('');
  const [next, setNext] = useState('');
  const [confirm, setConfirm] = useState('');

  const reset = () => {
    setCurrent('');
    setNext('');
    setConfirm('');
    change.reset();
  };
  const close = () => {
    reset();
    onClose();
  };

  const change = useMutation({
    mutationFn: () => changePassword({ current_password: current, new_password: next }),
    onSuccess: (session) => {
      authLogin(session);
      toast.success(t('auth.password.changed', 'Password changed. Other devices were signed out.'));
      close();
    },
  });

  const reason = change.error ? memberErrorKey(change.error) : null;
  const message = change.error
    ? reason
      ? t(`members.errors.${reason}`)
      : change.error instanceof Error
        ? change.error.message
        : ''
    : null;
  const ready = current !== '' && newPasswordReady(next, confirm);

  return (
    <Dialog open={open} onOpenChange={(o) => !o && !change.isPending && close()}>
      <DialogContent className="sm:max-w-md" data-testid="change-password-dialog">
        <DialogHeader>
          <DialogTitle>{t('auth.password.changeTitle', 'Change password')}</DialogTitle>
          <DialogDescription>
            {t('auth.password.changeBody', 'Choose a new password. You stay signed in here; your other devices are signed out.')}
          </DialogDescription>
        </DialogHeader>
        <form
          className="space-y-4"
          noValidate
          onSubmit={(e) => {
            e.preventDefault();
            if (ready) change.mutate();
          }}
        >
          <PasswordField
            label={t('auth.password.current', 'Current password')}
            value={current}
            onChange={setCurrent}
            autoComplete="current-password"
            autoFocus
            disabled={change.isPending}
            testId="change-current"
          />
          <NewPasswordFields
            password={next}
            confirm={confirm}
            onPassword={setNext}
            onConfirm={setConfirm}
            disabled={change.isPending}
            testIdPrefix="change-new"
          />
          {message && (
            <div role="alert" data-testid="change-password-error" className="flex items-start gap-2 rounded-md border border-destructive/30 bg-destructive/5 p-3 text-sm text-destructive">
              <AlertCircle className="mt-0.5 size-4 shrink-0" aria-hidden />
              {message}
            </div>
          )}
          <DialogFooter>
            <Button type="button" variant="ghost" onClick={close} disabled={change.isPending}>
              {t('common.cancel', 'Cancel')}
            </Button>
            <Button type="submit" disabled={!ready || change.isPending} data-testid="change-password-submit">
              {change.isPending && <Loader2 className="size-4 animate-spin" aria-hidden />}
              {t('auth.password.changeSubmit', 'Change password')}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}
