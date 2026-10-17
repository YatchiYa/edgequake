'use client';

import { AgentAvatar, AgentPicker } from '@/components/agents/agent-bits';
import { AgentsTopBar } from '@/components/agents/agents-shell';
import { Composer } from '@/components/agents/composer';
import { Skeleton } from '@/components/ui/skeleton';
import { agentsApi, agentsKeys, useAgents, type Conversation, type ConversationDetail } from '@/lib/agents/api';
import { useAgentsRunStore } from '@/stores/use-agents-run-store';
import { useQueryClient } from '@tanstack/react-query';
import { useRouter, useSearchParams } from 'next/navigation';
import { Suspense, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { toast } from 'sonner';

const DEFAULT_AGENT = 'edgequake_assistant';

function NewChat() {
  const { t } = useTranslation();
  const router = useRouter();
  const params = useSearchParams();
  const queryClient = useQueryClient();
  const start = useAgentsRunStore((s) => s.start);
  const { data: agents, isLoading, error } = useAgents();
  const [picked, setPicked] = useState<string | null>(null);
  const [creating, setCreating] = useState(false);

  const wanted = picked ?? params.get('agent') ?? DEFAULT_AGENT;
  const agent = agents?.find((a) => a.id === wanted) ?? agents?.find((a) => a.id === DEFAULT_AGENT) ?? agents?.[0];

  const send = async (text: string) => {
    if (!agent || creating) return;
    setCreating(true);
    try {
      const conversation = await agentsApi.createConversation(agent.id);
      queryClient.setQueryData<ConversationDetail>(agentsKeys.conversation(conversation.id), { ...conversation, messages: [] });
      queryClient.setQueriesData<Conversation[]>(
        { predicate: (q) => q.queryKey[0] === 'agents-platform' && q.queryKey[2] === 'conversations' && !q.queryKey[3] },
        (old) => (old ? [conversation, ...old] : old),
      );
      void start(conversation.id, text, [], queryClient);
      router.push(`/agents/c/${conversation.id}`);
    } catch (e) {
      toast.error(e instanceof Error ? e.message : String(e));
      setCreating(false);
    }
  };

  return (
    <>
      <AgentsTopBar>{agents && agents.length > 0 && <AgentPicker agents={agents} value={agent} onChange={(a) => setPicked(a.id)} />}</AgentsTopBar>
      <div className="flex min-h-0 flex-1 flex-col overflow-y-auto">
        <div className="mx-auto flex w-full max-w-3xl flex-1 flex-col justify-center px-4 pt-6 pb-[12vh] sm:px-6">
          {isLoading ? (
            <div className="flex flex-col items-center gap-4" aria-hidden="true">
              <Skeleton className="size-12 rounded-full" />
              <Skeleton className="h-8 w-64" />
              <Skeleton className="h-14 w-full rounded-3xl" />
            </div>
          ) : error ? (
            <p role="alert" className="text-center text-muted-foreground">
              {t('agentsPlatform.home.unavailable')} <span className="block text-sm">{error.message}</span>
            </p>
          ) : (
            <>
              <div className="mb-8 flex flex-col items-center gap-3 text-center">
                <AgentAvatar icon={agent?.icon} className="size-12 text-2xl" />
                <h1 className="text-2xl font-semibold tracking-tight sm:text-3xl">{t('agentsPlatform.home.greeting')}</h1>
                {agent?.description && <p className="max-w-lg text-sm text-muted-foreground">{agent.description}</p>}
              </div>
              <Composer
                onSend={(text) => void send(text)}
                disabled={!agent || creating}
                autoFocus
                placeholder={agent ? t('agentsPlatform.composer.placeholderAgent', { name: agent.name }) : undefined}
              />
              {!!agent?.starters.length && (
                <ul className="mt-6 grid gap-3 sm:grid-cols-2" aria-label={t('agentsPlatform.home.starters')}>
                  {agent.starters.slice(0, 4).map((s) => (
                    <li key={s}>
                      <button
                        type="button"
                        disabled={creating}
                        onClick={() => void send(s)}
                        className="h-full w-full rounded-2xl border bg-card px-4 py-3 text-left text-sm text-card-foreground transition-colors hover:bg-accent disabled:opacity-60"
                      >
                        {s}
                      </button>
                    </li>
                  ))}
                </ul>
              )}
            </>
          )}
        </div>
      </div>
    </>
  );
}

export default function AgentsHomePage() {
  return (
    <Suspense fallback={null}>
      <NewChat />
    </Suspense>
  );
}
