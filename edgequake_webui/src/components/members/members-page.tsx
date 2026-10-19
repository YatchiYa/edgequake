'use client';

import { PageHeader } from '@/components/shared/page-header';
import { PageShell } from '@/components/shared/page-shell';
import { ScrollArea } from '@/components/ui/scroll-area';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { KeyRound, ShieldCheck, Users } from 'lucide-react';
import { usePathname, useRouter, useSearchParams } from 'next/navigation';
import { useTranslation } from 'react-i18next';
import { MembersTab } from './members-tab';
import { RolesTab } from './roles-tab';
import { SsoTab } from './sso-tab';

const TABS = ['members', 'roles', 'sso'] as const;
type TabId = (typeof TABS)[number];

const isTab = (v: string | null): v is TabId => !!v && (TABS as readonly string[]).includes(v);

/** `/members` — people, roles and single sign-on of the current organization. */
export function MembersPage() {
  const { t } = useTranslation();
  const router = useRouter();
  const pathname = usePathname();
  const params = useSearchParams();
  const raw = params.get('tab');
  const tab: TabId = isTab(raw) ? raw : 'members';

  const setTab = (next: string) => {
    const qs = new URLSearchParams(params.toString());
    if (next === 'members') qs.delete('tab');
    else qs.set('tab', next);
    router.replace(qs.size ? `${pathname}?${qs}` : pathname, { scroll: false });
  };

  return (
    <ScrollArea className="h-full" data-testid="members-scroll">
    <PageShell>
      <Tabs value={tab} onValueChange={setTab}>
        <PageHeader
          title={t('members.title', 'Members')}
          description={t('members.description', 'Manage who can access this organization, what they can do, and how they sign in.')}
          tabs={
            <TabsList aria-label={t('members.title', 'Members')} data-testid="members-tabs" className="max-w-full justify-start overflow-x-auto">
              <TabsTrigger value="members" data-testid="tab-members">
                <Users aria-hidden /> {t('members.tabs.members', 'Members')}
              </TabsTrigger>
              <TabsTrigger value="roles" data-testid="tab-roles">
                <ShieldCheck aria-hidden /> {t('members.tabs.roles', 'Roles & permissions')}
              </TabsTrigger>
              <TabsTrigger value="sso" data-testid="tab-sso">
                <KeyRound aria-hidden /> {t('members.tabs.sso', 'Single sign-on')}
              </TabsTrigger>
            </TabsList>
          }
        />
        <TabsContent value="members">
          <MembersTab />
        </TabsContent>
        <TabsContent value="roles">
          <RolesTab />
        </TabsContent>
        <TabsContent value="sso">
          <SsoTab />
        </TabsContent>
      </Tabs>
    </PageShell>
    </ScrollArea>
  );
}
