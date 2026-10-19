'use client';

import { LinkHeader, LinkShell } from '@/components/auth/link-card';
import { NewPasswordFields, newPasswordReady } from '@/components/auth/new-password-fields';
import { SsoButtons } from '@/components/auth/sso-buttons';
import { Button } from '@/components/ui/button';
import { CardContent } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { acceptInvitation, previewInvitation, type InvitationPreview } from '@/lib/api/edgequake/members';
import { login } from '@/lib/api/edgequake';
import { ApiRequestError } from '@/lib/api/client';
import { memberErrorKey } from '@/lib/members/errors';
import { useAuthStore } from '@/stores/use-auth-store';
import { useMutation, useQuery } from '@tanstack/react-query';
import { AlertCircle, Loader2, MailX, UserPlus } from 'lucide-react';
import Link from 'next/link';
import { useRouter } from 'next/navigation';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { RoleBadge } from './role-badge';

const Shell = ({ children }: { children: React.ReactNode }) => <LinkShell testId="invite-card">{children}</LinkShell>;
const Header = LinkHeader;

function Unavailable({ expired }: { expired: boolean }) {
  const { t } = useTranslation();
  return (
    <Shell>
      <Header
        icon={MailX}
        title={expired ? t('members.accept.expiredTitle', 'This invitation has expired') : t('members.accept.invalidTitle', 'This invitation is no longer valid')}
        description={
          expired
            ? t('members.accept.expiredBody', 'Ask the person who invited you to send a new link.')
            : t('members.accept.invalidBody', 'It may have been used already or revoked. Ask for a new invitation.')
        }
      />
      <CardContent>
        <Button asChild variant="outline" className="w-full" data-testid="invite-go-login">
          <Link href="/login">{t('members.accept.toLogin', 'Go to sign in')}</Link>
        </Button>
      </CardContent>
    </Shell>
  );
}

function Summary({ preview }: { preview: InvitationPreview }) {
  const { t } = useTranslation();
  return (
    <div className="space-y-2 rounded-lg border bg-muted/30 p-3 text-sm" data-testid="invite-summary">
      <div className="flex items-center justify-between gap-2">
        <span className="text-muted-foreground">{t('members.accept.email', 'Email')}</span>
        <span className="truncate font-medium" data-testid="invite-summary-email">{preview.email}</span>
      </div>
      <div className="flex items-center justify-between gap-2">
        <span className="text-muted-foreground">{t('members.accept.role', 'Your role')}</span>
        <RoleBadge role={preview.role} />
      </div>
    </div>
  );
}

function CreateAccountForm({ preview, token }: { preview: InvitationPreview; token: string }) {
  const { t } = useTranslation();
  const router = useRouter();
  const authLogin = useAuthStore((s) => s.login);
  const [username, setUsername] = useState(preview.email.split('@')[0]);
  const [password, setPassword] = useState('');
  const [confirm, setConfirm] = useState('');

  const join = useMutation({
    mutationFn: async () => {
      await acceptInvitation(token, { username: username.trim(), password });
      return login({ username: username.trim(), password });
    },
    onSuccess: (session) => {
      authLogin(session);
      router.push('/');
    },
  });

  const reasonKey = join.error ? memberErrorKey(join.error) : null;
  const message = join.error
    ? reasonKey
      ? t(`members.errors.${reasonKey}`)
      : join.error instanceof Error
        ? join.error.message
        : ''
    : null;

  return (
    <form
      className="space-y-4"
      noValidate
      onSubmit={(e) => {
        e.preventDefault();
        if (newPasswordReady(password, confirm) && username.trim()) join.mutate();
      }}
    >
      <div className="space-y-1.5">
        <Label htmlFor="invite-username">{t('members.accept.username', 'Username')}</Label>
        <Input id="invite-username" data-testid="accept-username" autoComplete="username" value={username} onChange={(e) => setUsername(e.target.value)} disabled={join.isPending} />
      </div>
      <NewPasswordFields
        password={password}
        confirm={confirm}
        onPassword={setPassword}
        onConfirm={setConfirm}
        disabled={join.isPending}
        newLabel={t('members.accept.password', 'Password')}
        testIdPrefix="accept-password"
      />
      {message && (
        <div role="alert" data-testid="accept-error" className="flex items-start gap-2 rounded-md border border-destructive/30 bg-destructive/5 p-3 text-sm text-destructive">
          <AlertCircle className="mt-0.5 size-4 shrink-0" aria-hidden />
          {message}
        </div>
      )}
      <Button type="submit" className="w-full" disabled={join.isPending || !newPasswordReady(password, confirm) || !username.trim()} data-testid="accept-submit">
        {join.isPending && <Loader2 className="size-4 animate-spin" aria-hidden />}
        {t('members.accept.join', 'Join {{org}}', { org: preview.tenant_name })}
      </Button>
    </form>
  );
}

function ExistingAccount({ preview, token }: { preview: InvitationPreview; token: string }) {
  const { t } = useTranslation();
  const router = useRouter();
  const signedIn = useAuthStore((s) => s.isAuthenticated);
  const email = useAuthStore((s) => s.user?.email);
  const here = `/invite/${token}`;
  const accept = useMutation({
    mutationFn: () => acceptInvitation(token, {}),
    onSuccess: () => router.push('/'),
  });
  const matches = signedIn && email?.toLowerCase() === preview.email.toLowerCase();

  if (matches) {
    return (
      <div className="space-y-3">
        <Button className="w-full" onClick={() => accept.mutate()} disabled={accept.isPending} data-testid="accept-existing">
          {accept.isPending && <Loader2 className="size-4 animate-spin" aria-hidden />}
          {t('members.accept.join', 'Join {{org}}', { org: preview.tenant_name })}
        </Button>
        {accept.error instanceof ApiRequestError && (
          <p role="alert" className="text-sm text-destructive">{accept.error.message}</p>
        )}
      </div>
    );
  }
  return (
    <div className="space-y-4" data-testid="invite-existing">
      <p className="text-sm text-muted-foreground">
        {signedIn
          ? t('members.accept.wrongAccount', 'You are signed in with a different account. Sign in as {{email}} to accept.', { email: preview.email })
          : t('members.accept.existing', 'You already have an account. Sign in as {{email}} to accept.', { email: preview.email })}
      </p>
      <SsoButtons redirect={here} />
      <Button asChild className="w-full">
        <Link href={`/login?redirect=${encodeURIComponent(here)}`} data-testid="invite-signin">
          {t('members.accept.signIn', 'Sign in to accept')}
        </Link>
      </Button>
    </div>
  );
}

/** Public landing for an invitation link: preview, then create an account or sign in to accept. */
export function InviteAccept({ token }: { token: string }) {
  const { t } = useTranslation();
  const q = useQuery({
    queryKey: ['invitation', token],
    queryFn: () => previewInvitation(token),
    retry: 0,
    staleTime: Infinity,
  });

  if (q.isLoading) {
    return (
      <Shell>
        <CardContent className="flex justify-center py-16" aria-busy="true" data-testid="invite-loading">
          <Loader2 className="size-6 animate-spin text-muted-foreground" aria-hidden />
        </CardContent>
      </Shell>
    );
  }
  if (q.isError || !q.data) {
    return <Unavailable expired={q.error instanceof ApiRequestError && q.error.status === 410} />;
  }
  const preview = q.data;
  return (
    <Shell>
      <Header
        icon={UserPlus}
        title={t('members.accept.title', 'Join {{org}}', { org: preview.tenant_name })}
        description={
          preview.invited_by
            ? t('members.accept.invitedBy', '{{name}} invited you to collaborate.', { name: preview.invited_by })
            : t('members.accept.invited', 'You have been invited to collaborate.')
        }
      />
      <CardContent className="space-y-4">
        <Summary preview={preview} />
        {preview.account_exists ? (
          <ExistingAccount preview={preview} token={token} />
        ) : (
          <CreateAccountForm preview={preview} token={token} />
        )}
      </CardContent>
    </Shell>
  );
}
