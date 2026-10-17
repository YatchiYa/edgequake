'use client';

import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import { useAllAgents, type Agent } from '@/lib/agents/api';
import { humanizeAuthor } from '@/lib/agents/chat';
import { cn } from '@/lib/utils';
import { Check, ChevronDown, Compass } from 'lucide-react';
import Link from 'next/link';
import { useCallback } from 'react';
import { useTranslation } from 'react-i18next';

export function AgentAvatar({ icon, className }: { icon?: string; className?: string }) {
  return (
    <span
      aria-hidden="true"
      className={cn(
        'flex size-7 shrink-0 select-none items-center justify-center rounded-full border bg-background text-sm leading-none',
        className,
      )}
    >
      {icon || '🤖'}
    </span>
  );
}

/** Mirrors agents/…/catalog.py `agent_name()` (the event author of a custom agent). */
const slugify = (name: string) => {
  const s = name
    .normalize('NFKD')
    .replace(/[^\x00-\x7f]/g, '')
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '_')
    .replace(/^_+|_+$/g, '')
    .slice(0, 40);
  return /^[a-z_][a-z0-9_]*$/.test(s) && s !== 'user' ? s : `agent_${s}`.replace(/_+$/, '');
};

/** Resolve an author / agent id to its display name + icon (sub-agents included). */
export function useAgentLookup() {
  const { data } = useAllAgents();
  return useCallback(
    (id: string): { name: string; icon: string } => {
      // Built-ins stream their ADK id; custom agents stream a slug of their name.
      const agent = data?.find((a) => a.id === id) ?? data?.find((a) => slugify(a.name) === id);
      return agent ? { name: agent.name, icon: agent.icon } : { name: humanizeAuthor(id), icon: '🤖' };
    },
    [data],
  );
}

/** "Current agent" chip + dropdown (home page). */
export function AgentPicker({ agents, value, onChange }: { agents: Agent[]; value?: Agent; onChange: (a: Agent) => void }) {
  const { t } = useTranslation();
  const builtins = agents.filter((a) => a.kind === 'builtin');
  const custom = agents.filter((a) => a.kind !== 'builtin');
  const item = (a: Agent) => (
    <DropdownMenuItem key={a.id} onSelect={() => onChange(a)} className="items-start gap-3 py-2">
      <AgentAvatar icon={a.icon} className="mt-0.5" />
      <span className="min-w-0 flex-1">
        <span className="block truncate font-medium">{a.name}</span>
        {a.description && <span className="line-clamp-1 text-xs text-muted-foreground">{a.description}</span>}
      </span>
      {value?.id === a.id && <Check className="mt-1 size-4" />}
    </DropdownMenuItem>
  );
  return (
    <DropdownMenu>
      <DropdownMenuTrigger
        className="inline-flex max-w-full items-center gap-2 rounded-full px-3 py-1.5 text-base font-medium outline-none transition-colors hover:bg-accent focus-visible:ring-[3px] focus-visible:ring-ring/50"
        aria-label={t('agentsPlatform.picker.label')}
      >
        <span aria-hidden="true">{value?.icon ?? '🤖'}</span>
        <span className="truncate">{value?.name ?? t('agentsPlatform.picker.choose')}</span>
        <ChevronDown className="size-4 text-muted-foreground" />
      </DropdownMenuTrigger>
      <DropdownMenuContent align="start" className="max-h-[60vh] w-80 overflow-y-auto">
        {builtins.length > 0 && <DropdownMenuLabel>{t('agentsPlatform.explore.builtin')}</DropdownMenuLabel>}
        {builtins.map(item)}
        {custom.length > 0 && (
          <>
            <DropdownMenuSeparator />
            <DropdownMenuLabel>{t('agentsPlatform.explore.mine')}</DropdownMenuLabel>
            {custom.map(item)}
          </>
        )}
        <DropdownMenuSeparator />
        <DropdownMenuItem asChild>
          <Link href="/agents/explore" className="gap-3">
            <Compass className="size-4" /> {t('agentsPlatform.sidebar.explore')}
          </Link>
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
