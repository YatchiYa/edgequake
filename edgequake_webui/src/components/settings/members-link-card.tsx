'use client';

import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Users } from 'lucide-react';
import Link from 'next/link';
import { useTranslation } from 'react-i18next';

/** SPEC-169 — people, roles and single sign-on moved out of Settings into `/members`. */
export function MembersLinkCard() {
  const { t } = useTranslation();
  return (
    <Card data-testid="members-link-card">
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <Users className="h-5 w-5" aria-hidden />
          {t('settings.membersCard.title', 'Members, roles and sign-in')}
        </CardTitle>
        <CardDescription>
          {t(
            'settings.membersCard.description',
            'Invite people, manage their roles and configure single sign-on from the Members page.',
          )}
        </CardDescription>
      </CardHeader>
      <CardContent>
        <Button asChild variant="outline">
          <Link href="/members" data-testid="members-link">
            {t('settings.membersCard.cta', 'Open Members')}
          </Link>
        </Button>
      </CardContent>
    </Card>
  );
}
