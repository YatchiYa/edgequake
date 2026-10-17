'use client';

import { AgentAvatar } from '@/components/agents/agent-bits';
import { AgentsTopBar } from '@/components/agents/agents-shell';
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
import { Skeleton } from '@/components/ui/skeleton';
import { agentsApi, agentsKeys, useAgents, useWorkspaceId, type Agent } from '@/lib/agents/api';
import { cn } from '@/lib/utils';
import { useAgentsUiStore } from '@/stores/use-agents-ui-store';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { Copy, MessageSquarePlus, Pencil, Pin, PinOff, Plus, Trash2, Users } from 'lucide-react';
import Link from 'next/link';
import { useRouter } from 'next/navigation';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { toast } from 'sonner';

function AgentCard({ agent, onDelete }: { agent: Agent; onDelete: (a: Agent) => void }) {
  const { t } = useTranslation();
  const router = useRouter();
  const queryClient = useQueryClient();
  const ws = useWorkspaceId();
  const pinned = useAgentsUiStore((s) => s.pinned.includes(agent.id));
  const togglePin = useAgentsUiStore((s) => s.togglePin);
  const duplicate = useMutation({
    mutationFn: () => agentsApi.duplicateAgent(agent.id),
    onSuccess: (copy) => {
      queryClient.setQueryData<Agent[]>(agentsKeys.agents(ws), (old) => (old ? [...old, copy] : old));
      toast.success(t('agentsPlatform.explore.duplicated', { name: copy.name }));
      router.push(`/agents/${encodeURIComponent(copy.id)}/edit`);
    },
    onError: (e) => toast.error(e.message),
  });

  return (
    <li className="group flex flex-col rounded-2xl border bg-card p-4 text-card-foreground transition-shadow hover:shadow-md">
      <div className="flex items-start gap-3">
        <AgentAvatar icon={agent.icon} className="size-11 text-xl" />
        <div className="min-w-0 flex-1">
          <h3 className="flex items-center gap-1.5 truncate font-semibold">
            {agent.name}
            {agent.visibility === 'workspace' && agent.kind === 'custom' && (
              <Users className="size-3.5 shrink-0 text-muted-foreground" aria-label={t('agentsPlatform.builder.visibilityWorkspace')} />
            )}
          </h3>
          <p className="mt-0.5 line-clamp-2 text-sm text-muted-foreground">{agent.description || t('agentsPlatform.explore.noDescription')}</p>
        </div>
        <Button
          variant="ghost"
          size="icon-sm"
          className={cn('opacity-0 group-hover:opacity-100 focus-visible:opacity-100 max-md:opacity-100', pinned && 'opacity-100')}
          onClick={() => togglePin(agent.id)}
          aria-pressed={pinned}
          aria-label={pinned ? t('agentsPlatform.explore.unpin', { name: agent.name }) : t('agentsPlatform.explore.pin', { name: agent.name })}
          title={pinned ? t('agentsPlatform.explore.unpinShort') : t('agentsPlatform.explore.pinShort')}
        >
          {pinned ? <PinOff /> : <Pin />}
        </Button>
      </div>
      <div className="mt-4 flex flex-wrap gap-2">
        <Button asChild size="sm">
          <Link href={`/agents?agent=${encodeURIComponent(agent.id)}`}>
            <MessageSquarePlus /> {t('agentsPlatform.explore.start')}
          </Link>
        </Button>
        {agent.editable ? (
          <>
            <Button asChild size="sm" variant="outline">
              <Link href={`/agents/${encodeURIComponent(agent.id)}/edit`}>
                <Pencil /> {t('agentsPlatform.explore.edit')}
              </Link>
            </Button>
            <Button size="sm" variant="ghost" onClick={() => onDelete(agent)} aria-label={t('agentsPlatform.explore.deleteNamed', { name: agent.name })}>
              <Trash2 />
            </Button>
          </>
        ) : (
          <Button size="sm" variant="outline" disabled={duplicate.isPending} onClick={() => duplicate.mutate()}>
            <Copy /> {t('agentsPlatform.explore.duplicate')}
          </Button>
        )}
      </div>
    </li>
  );
}

export default function ExplorePage() {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const ws = useWorkspaceId();
  const { data: agents, isLoading, error } = useAgents();
  const [pendingDelete, setPendingDelete] = useState<Agent | null>(null);
  const remove = useMutation({
    mutationFn: (id: string) => agentsApi.deleteAgent(id),
    onMutate: async (id) => {
      const key = agentsKeys.agents(ws);
      await queryClient.cancelQueries({ queryKey: key });
      const previous = queryClient.getQueryData<Agent[]>(key);
      queryClient.setQueryData<Agent[]>(key, (old) => old?.filter((a) => a.id !== id));
      return { previous };
    },
    onError: (e, _id, ctx) => {
      queryClient.setQueryData(agentsKeys.agents(ws), ctx?.previous);
      toast.error(e.message);
    },
    onSuccess: () => toast.success(t('agentsPlatform.explore.deleted')),
    onSettled: () => queryClient.invalidateQueries({ queryKey: agentsKeys.agents(ws) }),
  });

  const sections: { key: string; title: string; items: Agent[] }[] = [
    { key: 'builtin', title: t('agentsPlatform.explore.builtin'), items: agents?.filter((a) => a.kind === 'builtin') ?? [] },
    { key: 'mine', title: t('agentsPlatform.explore.mine'), items: agents?.filter((a) => a.kind === 'custom' && a.editable) ?? [] },
    { key: 'shared', title: t('agentsPlatform.explore.shared'), items: agents?.filter((a) => a.kind === 'custom' && !a.editable) ?? [] },
  ];

  return (
    <>
      <AgentsTopBar
        actions={
          <Button asChild size="sm" className="mr-1">
            <Link href="/agents/new">
              <Plus /> {t('agentsPlatform.explore.create')}
            </Link>
          </Button>
        }
      />
      <div className="min-h-0 flex-1 overflow-y-auto">
        <div className="mx-auto w-full max-w-5xl px-4 pt-4 pb-16 sm:px-6">
          <h1 className="text-2xl font-semibold tracking-tight sm:text-3xl">{t('agentsPlatform.explore.title')}</h1>
          <p className="mt-1 text-muted-foreground">{t('agentsPlatform.explore.subtitle')}</p>

          {isLoading && (
            <ul className="mt-8 grid gap-4 sm:grid-cols-2 lg:grid-cols-3" aria-hidden="true">
              {[0, 1, 2, 3, 4, 5].map((i) => (
                <Skeleton key={i} className="h-36 rounded-2xl" />
              ))}
            </ul>
          )}
          {error && (
            <p role="alert" className="mt-8 text-muted-foreground">
              {t('agentsPlatform.home.unavailable')} {error.message}
            </p>
          )}

          {agents &&
            sections.map(
              (s) =>
                (s.items.length > 0 || s.key === 'mine') && (
                  <section key={s.key} className="mt-10" aria-labelledby={`agents-${s.key}`}>
                    <h2 id={`agents-${s.key}`} className="mb-3 text-sm font-semibold text-muted-foreground">
                      {s.title}
                    </h2>
                    {s.items.length ? (
                      <ul className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
                        {s.items.map((a) => (
                          <AgentCard key={a.id} agent={a} onDelete={setPendingDelete} />
                        ))}
                      </ul>
                    ) : (
                      <div className="flex flex-col items-start gap-3 rounded-2xl border border-dashed p-6 text-sm text-muted-foreground">
                        <p>{t('agentsPlatform.explore.mineEmpty')}</p>
                        <Button asChild size="sm" variant="outline">
                          <Link href="/agents/new">
                            <Plus /> {t('agentsPlatform.explore.create')}
                          </Link>
                        </Button>
                      </div>
                    )}
                  </section>
                ),
            )}
        </div>
      </div>

      <AlertDialog open={!!pendingDelete} onOpenChange={(open) => !open && setPendingDelete(null)}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>{t('agentsPlatform.explore.deleteTitle')}</AlertDialogTitle>
            <AlertDialogDescription>{t('agentsPlatform.explore.deleteBody', { name: pendingDelete?.name ?? '' })}</AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>{t('common.cancel', 'Cancel')}</AlertDialogCancel>
            <AlertDialogAction
              className="bg-destructive text-destructive-foreground hover:bg-destructive/90"
              onClick={() => {
                if (pendingDelete) remove.mutate(pendingDelete.id);
                setPendingDelete(null);
              }}
            >
              {t('agentsPlatform.explore.delete')}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </>
  );
}
