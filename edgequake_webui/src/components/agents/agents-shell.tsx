'use client';

import { HeaderTenantSelector } from '@/components/layout/header-tenant-selector';
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from '@/components/ui/alert-dialog';
import { Button } from '@/components/ui/button';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuRadioGroup,
  DropdownMenuRadioItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import { Sheet, SheetContent, SheetDescription, SheetTitle } from '@/components/ui/sheet';
import { Skeleton } from '@/components/ui/skeleton';
import { useDebounce } from '@/hooks/use-debounce';
import { getCurrentUser } from '@/lib/api/edgequake/auth';
import { agentsApi, useAgents, useConversations, useOnWorkspaceChange, type Conversation } from '@/lib/agents/api';
import { groupByDate } from '@/lib/agents/chat';
import { cn } from '@/lib/utils';
import { useAgentsUiStore } from '@/stores/use-agents-ui-store';
import { useAuthStore } from '@/stores/use-auth-store';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import {
  ArrowLeft,
  Compass,
  LogOut,
  Menu,
  MoreHorizontal,
  PanelLeftClose,
  PanelLeftOpen,
  Pencil,
  Search,
  SquarePen,
  Trash2,
} from 'lucide-react';
import { useTheme } from 'next-themes';
import Link from 'next/link';
import { usePathname, useRouter } from 'next/navigation';
import { useEffect, useMemo, useRef, useState, type ReactNode } from 'react';
import { useTranslation } from 'react-i18next';
import { toast } from 'sonner';
import { AgentAvatar } from './agent-bits';

const isConversationList = (key: readonly unknown[]) => key[0] === 'agents-platform' && key[2] === 'conversations';

/** Optimistic rename / delete over every cached conversation list (rollback on error). */
function useConversationMutations() {
  const queryClient = useQueryClient();
  const { t } = useTranslation();
  const optimistic = async (patch: (list: Conversation[]) => Conversation[]) => {
    await queryClient.cancelQueries({ predicate: (q) => isConversationList(q.queryKey) });
    const snapshot = queryClient.getQueriesData<Conversation[]>({ predicate: (q) => isConversationList(q.queryKey) });
    queryClient.setQueriesData<Conversation[]>({ predicate: (q) => isConversationList(q.queryKey) }, (old) => (old ? patch(old) : old));
    return { snapshot };
  };
  const rollback = (_e: unknown, _v: unknown, ctx?: { snapshot: [readonly unknown[], Conversation[] | undefined][] }) =>
    ctx?.snapshot.forEach(([key, data]) => queryClient.setQueryData(key, data));
  const settle = () => queryClient.invalidateQueries({ predicate: (q) => isConversationList(q.queryKey) });

  const rename = useMutation({
    mutationFn: ({ id, title }: { id: string; title: string }) => agentsApi.renameConversation(id, title),
    onMutate: ({ id, title }) => optimistic((list) => list.map((c) => (c.id === id ? { ...c, title } : c))),
    onError: (e, v, ctx) => {
      rollback(e, v, ctx);
      toast.error(t('agentsPlatform.sidebar.renameFailed'));
    },
    onSettled: settle,
  });
  const remove = useMutation({
    mutationFn: (id: string) => agentsApi.deleteConversation(id),
    onMutate: (id) => optimistic((list) => list.filter((c) => c.id !== id)),
    onError: (e, v, ctx) => {
      rollback(e, v, ctx);
      toast.error(t('agentsPlatform.sidebar.deleteFailed'));
    },
    onSuccess: () => toast.success(t('agentsPlatform.sidebar.deleted')),
    onSettled: settle,
  });
  return { rename, remove };
}

function ConversationRow({
  conversation,
  active,
  onRename,
  onDelete,
}: {
  conversation: Conversation;
  active: boolean;
  onRename: (title: string) => void;
  onDelete: () => void;
}) {
  const { t } = useTranslation();
  const [editing, setEditing] = useState(false);
  const [draft, setDraft] = useState(conversation.title);
  const inputRef = useRef<HTMLInputElement>(null);
  useEffect(() => {
    if (editing) inputRef.current?.select();
  }, [editing]);

  if (editing) {
    const commit = () => {
      setEditing(false);
      const title = draft.trim().slice(0, 120);
      if (title && title !== conversation.title) onRename(title);
    };
    return (
      <li>
        <input
          ref={inputRef}
          value={draft}
          onChange={(e) => setDraft(e.target.value)}
          onBlur={commit}
          onKeyDown={(e) => {
            if (e.key === 'Enter') commit();
            if (e.key === 'Escape') {
              setDraft(conversation.title);
              setEditing(false);
            }
          }}
          aria-label={t('agentsPlatform.sidebar.renameLabel')}
          className="h-9 w-full rounded-lg border bg-background px-2.5 text-sm outline-none focus-visible:ring-[3px] focus-visible:ring-ring/50"
        />
      </li>
    );
  }

  return (
    <li className="group/row relative">
      <Link
        href={`/agents/c/${conversation.id}`}
        aria-current={active ? 'page' : undefined}
        className={cn(
          'flex h-9 items-center rounded-lg px-2.5 pr-9 text-sm outline-none transition-colors hover:bg-sidebar-accent focus-visible:ring-[3px] focus-visible:ring-ring/50',
          active && 'bg-sidebar-accent font-medium',
        )}
        title={conversation.title}
      >
        <span className="truncate">{conversation.title}</span>
      </Link>
      <DropdownMenu>
        <DropdownMenuTrigger
          className={cn(
            'absolute top-1/2 right-1 flex size-7 -translate-y-1/2 items-center justify-center rounded-md text-muted-foreground opacity-0 outline-none hover:text-foreground focus-visible:opacity-100 group-hover/row:opacity-100 data-[state=open]:opacity-100 max-md:opacity-100',
            active && 'opacity-100',
          )}
          aria-label={t('agentsPlatform.sidebar.options', { title: conversation.title })}
        >
          <MoreHorizontal className="size-4" />
        </DropdownMenuTrigger>
        <DropdownMenuContent align="start" className="w-44">
          <DropdownMenuItem
            onSelect={() => {
              setDraft(conversation.title);
              // Let the menu close (and return focus) before the input grabs it.
              setTimeout(() => setEditing(true), 0);
            }}
          >
            <Pencil className="size-4" /> {t('agentsPlatform.sidebar.rename')}
          </DropdownMenuItem>
          <DropdownMenuItem variant="destructive" onSelect={onDelete}>
            <Trash2 className="size-4" /> {t('agentsPlatform.sidebar.delete')}
          </DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>
    </li>
  );
}

function SidebarContent({ onCollapse }: { onCollapse?: () => void }) {
  const { t } = useTranslation();
  const pathname = usePathname();
  const router = useRouter();
  const [search, setSearch] = useState('');
  const q = useDebounce(search.trim(), 250);
  const { data: conversations, isLoading } = useConversations(q);
  const { data: agents } = useAgents();
  const pinned = useAgentsUiStore((s) => s.pinned);
  const { rename, remove } = useConversationMutations();
  const [pendingDelete, setPendingDelete] = useState<Conversation | null>(null);
  const { setTheme, theme } = useTheme();
  const storeUser = useAuthStore((s) => s.user);
  // The auth store is memory-only after a reload (SPEC-154): fall back to /auth/me.
  const { data: me } = useQuery({ queryKey: ['agents-platform', 'me'], queryFn: getCurrentUser, enabled: !storeUser, staleTime: Infinity });
  const user = storeUser ?? (me && 'user' in me ? (me as unknown as { user: typeof me }).user : me);
  const logout = useAuthStore((s) => s.logout);

  // Pinned first, then agents of the most recent conversations — up to 5.
  const shortcuts = useMemo(() => {
    if (!agents) return [];
    const ids = [...pinned, ...(conversations ?? []).map((c) => c.agent_id)];
    return [...new Set(ids)]
      .map((id) => agents.find((a) => a.id === id))
      .filter((a): a is NonNullable<typeof a> => !!a)
      .slice(0, 5);
  }, [agents, pinned, conversations]);

  const activeId = pathname.startsWith('/agents/c/') ? pathname.split('/')[3] : null;
  const groups = groupByDate(conversations ?? []);

  return (
    <div className="flex h-full min-h-0 flex-col bg-sidebar text-sidebar-foreground">
      <div className="flex items-center gap-1 px-3 pt-3 pb-2">
        <Link href="/agents" className="flex min-w-0 flex-1 items-center gap-2 rounded-lg px-1.5 py-1 text-[15px] font-semibold">
          <span aria-hidden="true">✨</span>
          <span className="truncate">{t('agentsPlatform.title')}</span>
        </Link>
        {onCollapse && (
          <Button variant="ghost" size="icon-sm" onClick={onCollapse} aria-label={t('agentsPlatform.sidebar.collapse')} title={t('agentsPlatform.sidebar.collapse')}>
            <PanelLeftClose />
          </Button>
        )}
      </div>

      <div className="flex flex-col gap-0.5 px-3">
        <Link
          href="/agents"
          className="group/new flex h-9 items-center gap-2.5 rounded-lg px-2.5 text-sm font-medium whitespace-nowrap hover:bg-sidebar-accent"
          title={t('agentsPlatform.sidebar.newChatShortcut')}
        >
          <SquarePen className="size-4 shrink-0" /> <span className="truncate">{t('agentsPlatform.sidebar.newChat')}</span>
          <kbd className="ml-auto font-sans text-xs text-muted-foreground opacity-0 transition-opacity group-hover/new:opacity-100">Ctrl⇧O</kbd>
        </Link>
        <label className="relative flex items-center">
          <Search className="pointer-events-none absolute left-2.5 size-4 text-muted-foreground" aria-hidden="true" />
          <input
            type="search"
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            placeholder={t('agentsPlatform.sidebar.search')}
            aria-label={t('agentsPlatform.sidebar.search')}
            className="query-input h-9 w-full rounded-lg bg-transparent! pr-2 pl-8.5 text-sm placeholder:text-muted-foreground hover:bg-sidebar-accent! focus-visible:bg-sidebar-accent!"
          />
        </label>
      </div>

      <nav className="mt-3 min-h-0 flex-1 overflow-y-auto px-3 pb-3" aria-label={t('agentsPlatform.sidebar.navLabel')}>
        {!q && (
          <section className="mb-4">
            <h2 className="px-2.5 pb-1 text-xs font-medium text-muted-foreground">{t('agentsPlatform.sidebar.agents')}</h2>
            <ul className="flex flex-col gap-0.5">
              <li>
                <Link
                  href="/agents/explore"
                  aria-current={pathname === '/agents/explore' ? 'page' : undefined}
                  className={cn(
                    'flex h-9 items-center gap-2.5 rounded-lg px-2.5 text-sm hover:bg-sidebar-accent',
                    pathname === '/agents/explore' && 'bg-sidebar-accent font-medium',
                  )}
                >
                  <Compass className="size-4" /> {t('agentsPlatform.sidebar.explore')}
                </Link>
              </li>
              {shortcuts.map((a) => (
                <li key={a.id}>
                  <Link
                    href={`/agents?agent=${encodeURIComponent(a.id)}`}
                    className="flex h-9 items-center gap-2.5 rounded-lg px-2.5 text-sm hover:bg-sidebar-accent"
                  >
                    <AgentAvatar icon={a.icon} className="size-5 border-0 bg-transparent text-sm" />
                    <span className="truncate">{a.name}</span>
                  </Link>
                </li>
              ))}
            </ul>
          </section>
        )}

        {isLoading && (
          <div className="flex flex-col gap-2 px-2.5" aria-hidden="true">
            {[70, 90, 60, 80].map((w) => (
              <Skeleton key={w} className="h-5" style={{ width: `${w}%` }} />
            ))}
          </div>
        )}
        {!isLoading && !conversations?.length && (
          <p className="px-2.5 py-2 text-sm text-muted-foreground">
            {q ? t('agentsPlatform.sidebar.noResults') : t('agentsPlatform.sidebar.noConversations')}
          </p>
        )}
        {groups.map(({ group, items }) => (
          <section key={group} className="mb-4">
            <h2 className="px-2.5 pb-1 text-xs font-medium text-muted-foreground">{t(`agentsPlatform.groups.${group}`)}</h2>
            <ul className="flex flex-col gap-0.5">
              {items.map((c) => (
                <ConversationRow
                  key={c.id}
                  conversation={c}
                  active={c.id === activeId}
                  onRename={(title) => rename.mutate({ id: c.id, title })}
                  onDelete={() => setPendingDelete(c)}
                />
              ))}
            </ul>
          </section>
        ))}
      </nav>

      <div className="flex flex-col gap-1 border-t border-sidebar-border p-3">
        <HeaderTenantSelector className="w-full" />
        <Link href="/" className="flex h-9 items-center gap-2.5 rounded-lg px-2.5 text-sm text-muted-foreground hover:bg-sidebar-accent hover:text-foreground">
          <ArrowLeft className="size-4" /> {t('agentsPlatform.sidebar.back')}
        </Link>
        <DropdownMenu>
          <DropdownMenuTrigger className="flex h-11 items-center gap-2.5 rounded-lg px-2 text-left outline-none hover:bg-sidebar-accent focus-visible:ring-[3px] focus-visible:ring-ring/50">
            <span className="flex size-7 shrink-0 items-center justify-center rounded-full bg-primary text-xs font-semibold text-primary-foreground uppercase">
              {(user?.username ?? '?').slice(0, 1)}
            </span>
            <span className="min-w-0 flex-1">
              <span className="block truncate text-sm font-medium">{user?.username ?? t('agentsPlatform.sidebar.account')}</span>
              {user?.role && <span className="block truncate text-xs text-muted-foreground">{user.role}</span>}
            </span>
          </DropdownMenuTrigger>
          <DropdownMenuContent side="top" align="start" className="w-56">
            <DropdownMenuLabel>{t('agentsPlatform.sidebar.theme')}</DropdownMenuLabel>
            <DropdownMenuRadioGroup value={theme} onValueChange={setTheme}>
              <DropdownMenuRadioItem value="light">{t('theme.light', 'Light')}</DropdownMenuRadioItem>
              <DropdownMenuRadioItem value="dark">{t('theme.dark', 'Dark')}</DropdownMenuRadioItem>
              <DropdownMenuRadioItem value="system">{t('theme.system', 'System')}</DropdownMenuRadioItem>
            </DropdownMenuRadioGroup>
            <DropdownMenuSeparator />
            <DropdownMenuItem
              onSelect={() => {
                logout();
                router.push('/login');
              }}
            >
              <LogOut className="size-4" /> {t('header.logout', 'Logout')}
            </DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
      </div>

      <AlertDialog open={!!pendingDelete} onOpenChange={(open) => !open && setPendingDelete(null)}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>{t('agentsPlatform.sidebar.deleteTitle')}</AlertDialogTitle>
            <AlertDialogDescription>{t('agentsPlatform.sidebar.deleteBody', { title: pendingDelete?.title ?? '' })}</AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>{t('common.cancel', 'Cancel')}</AlertDialogCancel>
            <AlertDialogAction
              className="bg-destructive text-destructive-foreground hover:bg-destructive/90"
              onClick={() => {
                if (!pendingDelete) return;
                remove.mutate(pendingDelete.id);
                if (pendingDelete.id === activeId) router.push('/agents');
                setPendingDelete(null);
              }}
            >
              {t('agentsPlatform.sidebar.delete')}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  );
}

/** Page top bar: mobile menu / expand-sidebar + new chat when the sidebar is hidden. */
export function AgentsTopBar({ children, actions }: { children?: ReactNode; actions?: ReactNode }) {
  const { t } = useTranslation();
  const collapsed = useAgentsUiStore((s) => s.sidebarCollapsed);
  const setCollapsed = useAgentsUiStore((s) => s.setSidebarCollapsed);
  const setMobileOpen = useAgentsUiStore((s) => s.setMobileOpen);
  return (
    <header className="flex h-14 shrink-0 items-center gap-1 px-2 sm:px-3">
      <Button variant="ghost" size="icon" className="md:hidden" onClick={() => setMobileOpen(true)} aria-label={t('agentsPlatform.sidebar.open')}>
        <Menu />
      </Button>
      {collapsed && (
        <Button variant="ghost" size="icon" className="max-md:hidden" onClick={() => setCollapsed(false)} aria-label={t('agentsPlatform.sidebar.expand')} title={t('agentsPlatform.sidebar.expand')}>
          <PanelLeftOpen />
        </Button>
      )}
      <Button
        asChild
        variant="ghost"
        size="icon"
        className={cn(!collapsed && 'md:hidden')}
        aria-label={t('agentsPlatform.sidebar.newChat')}
        title={t('agentsPlatform.sidebar.newChatShortcut')}
      >
        <Link href="/agents">
          <SquarePen />
        </Link>
      </Button>
      <div className="flex min-w-0 flex-1 items-center gap-2">{children}</div>
      {actions}
    </header>
  );
}

export function AgentsShell({ children }: { children: ReactNode }) {
  const { t } = useTranslation();
  const router = useRouter();
  const pathname = usePathname();
  const collapsed = useAgentsUiStore((s) => s.sidebarCollapsed);
  const setCollapsed = useAgentsUiStore((s) => s.setSidebarCollapsed);
  const mobileOpen = useAgentsUiStore((s) => s.mobileOpen);
  const setMobileOpen = useAgentsUiStore((s) => s.setMobileOpen);

  useEffect(() => setMobileOpen(false), [pathname, setMobileOpen]);

  // Conversations live in the workspace they were created in. The shell sits outside
  // TenantGuard (which remounts pages on a switch), so it is the one that notices:
  // drop cached conversations and leave any page showing one from the old workspace.
  const queryClient = useQueryClient();
  useOnWorkspaceChange(() => {
    void queryClient.resetQueries({ queryKey: ['agents-platform', 'conversation'] });
    const search = new URLSearchParams(window.location.search);
    if (pathname.startsWith('/agents/c/')) router.replace('/agents');
    else if (search.has('design')) {
      search.delete('design');
      router.replace(`${pathname}?${search.toString()}`);
    }
  });

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.shiftKey && e.key.toLowerCase() === 'o') {
        e.preventDefault();
        router.push('/agents');
      }
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [router]);

  return (
    <div className="flex h-dvh max-h-dvh min-h-0 overflow-clip bg-background text-foreground">
      <aside
        className={cn(
          'hidden shrink-0 border-r border-sidebar-border transition-[width] duration-200 md:block',
          collapsed ? 'w-0 overflow-hidden border-r-0' : 'w-[260px]',
        )}
        aria-label={t('agentsPlatform.sidebar.navLabel')}
        aria-hidden={collapsed || undefined}
        inert={collapsed || undefined}
      >
        <div className="h-full w-[260px]">
          <SidebarContent onCollapse={() => setCollapsed(true)} />
        </div>
      </aside>
      <Sheet open={mobileOpen} onOpenChange={setMobileOpen}>
        <SheetContent side="left" className="w-[85vw] max-w-[300px] p-0">
          <SheetTitle className="sr-only">{t('agentsPlatform.sidebar.navLabel')}</SheetTitle>
          <SheetDescription className="sr-only">{t('agentsPlatform.sidebar.navLabel')}</SheetDescription>
          <SidebarContent />
        </SheetContent>
      </Sheet>
      <main id="main-content" className="flex min-h-0 min-w-0 flex-1 flex-col overflow-clip">
        {children}
      </main>
    </div>
  );
}
