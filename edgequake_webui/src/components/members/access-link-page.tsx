'use client';

import { LinkHeader, LinkShell } from '@/components/auth/link-card';
import { NewPasswordFields, newPasswordReady } from '@/components/auth/new-password-fields';
import { Button } from '@/components/ui/button';
import { CardContent } from '@/components/ui/card';
import { previewAccessLink, redeemAccessLink, type AccessLinkPreview } from '@/lib/api/edgequake/access-links';
import { ApiRequestError } from '@/lib/api/client';
import { memberErrorKey } from '@/lib/members/errors';
import { useAuthStore } from '@/stores/use-auth-store';
import { useMutation, useQuery } from '@tanstack/react-query';
import { AlertCircle, KeyRound, Link2Off, Loader2, WandSparkles } from 'lucide-react';
import Link from 'next/link';
import { useRouter } from 'next/navigation';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';

const Shell = ({ children }: { children: React.ReactNode }) => <LinkShell testId="access-card">{children}</LinkShell>;

function Unavailable({ expired }: { expired: boolean }) {
  const { t } = useTranslation();
  return (
    <Shell>
      <LinkHeader
        icon={Link2Off}
        title={expired ? t('members.accessPage.expiredTitle', 'This link has expired') : t('members.accessPage.invalidTitle', 'This link is no longer valid')}
        description={
          expired
            ? t('members.accessPage.expiredBody', 'Ask your administrator to create a new one.')
            : t('members.accessPage.invalidBody', 'It may have been used already or replaced by a newer link. Ask your administrator for a new one.')
        }
      />
      <CardContent>
        <Button asChild variant="outline" className="w-full" data-testid="access-go-login">
          <Link href="/login">{t('members.accessPage.toLogin', 'Go to sign in')}</Link>
        </Button>
      </CardContent>
    </Shell>
  );
}

function WhoFor({ preview }: { preview: AccessLinkPreview }) {
  const { t } = useTranslation();
  const current = useAuthStore((s) => s.user?.username);
  const switching = current && current !== preview.username;
  return (
    <div className="space-y-2">
      <div className="space-y-2 rounded-lg border bg-muted/30 p-3 text-sm" data-testid="access-summary">
        <div className="flex items-center justify-between gap-2">
          <span className="text-muted-foreground">{t('members.accessPage.account', 'Account')}</span>
          <span className="truncate font-medium" data-testid="access-summary-user">{preview.username}</span>
        </div>
        <div className="flex items-center justify-between gap-2">
          <span className="text-muted-foreground">{t('members.accept.email', 'Email')}</span>
          <span className="truncate">{preview.email}</span>
        </div>
        {preview.tenant_name && (
          <div className="flex items-center justify-between gap-2">
            <span className="text-muted-foreground">{t('members.accessPage.organization', 'Organization')}</span>
            <span className="truncate">{preview.tenant_name}</span>
          </div>
        )}
      </div>
      {switching && (
        <p className="text-xs text-muted-foreground" data-testid="access-switching">
          {t('members.accessPage.switching', 'You are signed in as {{current}}. Continuing switches this browser to {{name}}.', { current, name: preview.username })}
        </p>
      )}
    </div>
  );
}

function ErrorNote({ error }: { error: unknown }) {
  const { t } = useTranslation();
  if (!error) return null;
  const key = memberErrorKey(error);
  const message = key ? t(`members.errors.${key}`) : error instanceof Error ? error.message : '';
  return (
    <div role="alert" data-testid="access-error" className="flex items-start gap-2 rounded-md border border-destructive/30 bg-destructive/5 p-3 text-sm text-destructive">
      <AlertCircle className="mt-0.5 size-4 shrink-0" aria-hidden />
      {message}
    </div>
  );
}

/** One redeem = a session. Both flows end by signing in and going home. */
function useRedeem(token: string) {
  const router = useRouter();
  const authLogin = useAuthStore((s) => s.login);
  return useMutation({
    mutationFn: (password?: string) => redeemAccessLink(token, password ? { password } : {}),
    onSuccess: (session) => {
      authLogin(session);
      router.push('/');
    },
  });
}

function MagicSignIn({ preview, token }: { preview: AccessLinkPreview; token: string }) {
  const { t } = useTranslation();
  const redeem = useRedeem(token);
  return (
    <>
      <LinkHeader
        icon={WandSparkles}
        title={t('members.accessPage.magicTitle', 'Sign in to EdgeQuake')}
        description={
          preview.issued_by
            ? t('members.accessPage.magicBy', '{{name}} created this one-time sign-in link for you.', { name: preview.issued_by })
            : t('members.accessPage.magicBody', 'This is a one-time sign-in link.')
        }
      />
      <CardContent className="space-y-4">
        <WhoFor preview={preview} />
        <ErrorNote error={redeem.error} />
        <Button className="w-full" onClick={() => redeem.mutate(undefined)} disabled={redeem.isPending} data-testid="access-signin">
          {redeem.isPending && <Loader2 className="size-4 animate-spin" aria-hidden />}
          {t('members.accessPage.signInAs', 'Sign in as {{name}}', { name: preview.username })}
        </Button>
        <p className="text-center text-xs text-muted-foreground">
          {t('members.accessPage.onceNote', 'Opening the link signs you in once. Your password is unchanged.')}
        </p>
      </CardContent>
    </>
  );
}

function ResetPassword({ preview, token }: { preview: AccessLinkPreview; token: string }) {
  const { t } = useTranslation();
  const redeem = useRedeem(token);
  const [password, setPassword] = useState('');
  const [confirm, setConfirm] = useState('');
  const ready = newPasswordReady(password, confirm);
  return (
    <>
      <LinkHeader
        icon={KeyRound}
        title={t('members.accessPage.resetTitle', 'Choose a new password')}
        description={
          preview.issued_by
            ? t('members.accessPage.resetBy', '{{name}} sent you this link to set a new password.', { name: preview.issued_by })
            : t('members.accessPage.resetBody', 'Set a new password for your account.')
        }
      />
      <CardContent>
        <form
          className="space-y-4"
          noValidate
          onSubmit={(e) => {
            e.preventDefault();
            if (ready) redeem.mutate(password);
          }}
        >
          <WhoFor preview={preview} />
          <NewPasswordFields
            password={password}
            confirm={confirm}
            onPassword={setPassword}
            onConfirm={setConfirm}
            disabled={redeem.isPending}
            autoFocus
            testIdPrefix="reset-password"
          />
          <ErrorNote error={redeem.error} />
          <Button type="submit" className="w-full" disabled={redeem.isPending || !ready} data-testid="reset-submit">
            {redeem.isPending && <Loader2 className="size-4 animate-spin" aria-hidden />}
            {t('members.accessPage.resetSubmit', 'Set password and sign in')}
          </Button>
          <p className="text-center text-xs text-muted-foreground">
            {t('members.accessPage.resetNote', 'Setting a new password signs you out everywhere else.')}
          </p>
        </form>
      </CardContent>
    </>
  );
}

/** Public landing for an access link: preview who it is for, then sign in (or set a password). */
export function AccessLinkPage({ token }: { token: string }) {
  const q = useQuery({
    queryKey: ['access-link', token],
    queryFn: () => previewAccessLink(token),
    retry: 0,
    staleTime: Infinity,
  });

  if (q.isLoading) {
    return (
      <Shell>
        <CardContent className="flex justify-center py-16" aria-busy="true" data-testid="access-loading">
          <Loader2 className="size-6 animate-spin text-muted-foreground" aria-hidden />
        </CardContent>
      </Shell>
    );
  }
  if (q.isError || !q.data) {
    return <Unavailable expired={q.error instanceof ApiRequestError && q.error.status === 410} />;
  }
  return (
    <Shell>
      {q.data.purpose === 'password_reset' ? (
        <ResetPassword preview={q.data} token={token} />
      ) : (
        <MagicSignIn preview={q.data} token={token} />
      )}
    </Shell>
  );
}
