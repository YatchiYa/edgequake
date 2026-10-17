'use client';

import { Button } from '@/components/ui/button';
import { Skeleton } from '@/components/ui/skeleton';
import { useStickToBottom } from '@/hooks/use-stick-to-bottom';
import { AgentsApiError, useConversation, type ArchitectContext } from '@/lib/agents/api';
import { cn } from '@/lib/utils';
import { useAgentsRunStore } from '@/stores/use-agents-run-store';
import { useQueryClient } from '@tanstack/react-query';
import { ArrowDown, CircleAlert, MessageSquareOff } from 'lucide-react';
import Link from 'next/link';
import { useCallback, useEffect, useMemo, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { AgentAvatar, useAgentLookup } from './agent-bits';
import { Composer } from './composer';
import { MessageItem } from './message-item';
import { InvestigationPanel } from './run-activity';

const EMPTY: never[] = [];

interface ChatViewProps {
  conversationId: string;
  compact?: boolean;
  /** Sent with every message (agent architect: the builder form state). */
  getContext?: () => ArchitectContext | undefined;
  /** One-click prompts shown above the composer while idle (e.g. suggested test questions). */
  suggestions?: string[];
  /** Extra content under the last message (e.g. a hint card). */
  footer?: React.ReactNode;
}

/** One conversation: history + live run, sticky composer, auto-scroll with "jump to latest". */
export function ChatView({ conversationId, compact = false, getContext, suggestions, footer }: ChatViewProps) {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const { data, isLoading, error } = useConversation(conversationId);
  const run = useAgentsRunStore((s) => s.runs[conversationId]);
  const start = useAgentsRunStore((s) => s.start);
  const stop = useAgentsRunStore((s) => s.stop);
  const lookup = useAgentLookup();

  const streaming = !!run && !run.done;
  // Until the first event arrives, a placeholder reply shows the agent already working.
  const messages = useMemo(() => {
    const list = run ? [...run.base, ...run.messages] : (data?.messages ?? EMPTY);
    if (!streaming || list[list.length - 1]?.role !== 'user') return list;
    return [...list, { id: 'pending', role: 'assistant' as const, author: data?.agent_id ?? 'assistant', content: '', tools: [], artifacts: [], created_at: '' }];
  }, [run, data, streaming]);
  const agent = data ? lookup(data.agent_id) : null;
  const { scrollRef, scrollAnchorRef, showJumpPill, jumpToLatest } = useStickToBottom(streaming ? 'generating' : 'idle', messages);

  // Investigation panel: the latest turn that has a plan. Its step counts focus the Sources section.
  const investigation = useMemo(() => [...messages].reverse().find((m) => m.role === 'assistant' && m.plan?.length), [messages]);
  const [focus, setFocus] = useState<{ messageId: string; step: string | null } | null>(null);
  const showSources = useCallback((messageId: string, step: string | null) => {
    setFocus({ messageId, step });
    if (step) requestAnimationFrame(() => document.getElementById(`sources-${messageId}`)?.scrollIntoView({ behavior: 'smooth', block: 'start' }));
  }, []);
  const graph = investigation?.artifacts.find((a) => a.mime_type.startsWith('image/')) ?? investigation?.artifacts.find((a) => a.mime_type.startsWith('text/html'));

  const send = useCallback(
    (text: string) => void start(conversationId, text, messages, queryClient, getContext?.()),
    [conversationId, messages, queryClient, start, getContext],
  );

  useEffect(() => {
    if (!streaming) return;
    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape') stop(conversationId);
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [streaming, stop, conversationId]);

  const notFound = error instanceof AgentsApiError && (error.status === 404 || error.status === 403);
  const column = compact ? 'px-4' : 'mx-auto w-full max-w-3xl px-4 sm:px-6';

  return (
    <div className="relative flex min-h-0 flex-1 flex-col">
      <div ref={scrollRef} className="min-h-0 flex-1 overflow-y-auto [scrollbar-gutter:stable]">
        <div className={cn(column, 'flex flex-col gap-8 pt-6 pb-10')} role="log" aria-live="polite" aria-busy={streaming}>
          {isLoading && !run && (
            <div className="flex flex-col gap-8" aria-hidden="true">
              <Skeleton className="ml-auto h-10 w-1/2 rounded-3xl" />
              <div className="flex gap-4">
                <Skeleton className="size-7 rounded-full" />
                <div className="flex flex-1 flex-col gap-2">
                  <Skeleton className="h-4 w-1/3" />
                  <Skeleton className="h-4 w-full" />
                  <Skeleton className="h-4 w-5/6" />
                </div>
              </div>
            </div>
          )}
          {error && !run && (
            <div className="flex flex-col items-center gap-3 py-24 text-center text-muted-foreground">
              <MessageSquareOff className="size-8" />
              <p>{notFound ? t('agentsPlatform.chat.notFound') : error.message}</p>
              <Button asChild variant="outline" size="sm">
                <Link href="/agents">{t('agentsPlatform.sidebar.newChat')}</Link>
              </Button>
            </div>
          )}
          {!isLoading && !error && !messages.length && agent && (
            <div className="flex flex-col items-center gap-3 py-16 text-center">
              <AgentAvatar icon={agent.icon} className="size-12 text-2xl" />
              <p className="text-lg font-medium">{agent.name}</p>
              <p className="text-sm text-muted-foreground">{t('agentsPlatform.chat.emptyHint')}</p>
            </div>
          )}
          {messages.map((m, i) => (
            <MessageItem
              key={m.id}
              message={m}
              conversationId={conversationId}
              streaming={streaming && i === messages.length - 1}
              author={lookup(m.role === 'assistant' && m.author ? m.author : (data?.agent_id ?? m.author))}
              agentId={data?.agent_id}
              lookup={lookup}
              sourceFocus={focus?.messageId === m.id ? focus.step : null}
              onSourceFocus={showSources}
            />
          ))}
          {run?.error && (
            <div role="alert" className="ml-10 flex items-start gap-2 rounded-xl border border-destructive/40 bg-destructive/5 px-3 py-2 text-sm text-destructive sm:ml-11">
              <CircleAlert className="mt-0.5 size-4 shrink-0" />
              <span className="break-words">{run.error}</span>
            </div>
          )}
          {footer}
          <div ref={scrollAnchorRef} />
        </div>
      </div>

      <div className={cn('relative shrink-0 bg-gradient-to-t from-background via-background to-transparent pb-3', compact ? 'px-3 pt-2' : 'pt-2')}>
        {investigation && !compact && (
          <InvestigationPanel
            key={investigation.id}
            message={investigation}
            streaming={streaming && investigation === messages[messages.length - 1]}
            startedAt={run?.startedAt}
            onShowSources={(step) => showSources(investigation.id, step)}
            onShowGraph={
              graph
                ? () => document.getElementById(`artifact-${investigation.id}-${graph.name}`)?.scrollIntoView({ behavior: 'smooth', block: 'center' })
                : undefined
            }
          />
        )}
        {showJumpPill && (
          <Button
            variant="outline"
            size="icon-sm"
            onClick={jumpToLatest}
            className="absolute -top-10 left-1/2 -translate-x-1/2 rounded-full shadow-md"
            aria-label={t('agentsPlatform.chat.jumpToLatest')}
            title={t('agentsPlatform.chat.jumpToLatest')}
          >
            <ArrowDown />
          </Button>
        )}
        <div className={compact ? '' : 'mx-auto w-full max-w-3xl px-4 sm:px-6'}>
          {!!suggestions?.length && !streaming && !(error && !run) && (
            <ul className="mb-2 flex flex-wrap gap-1.5" aria-label={t('agentsPlatform.chat.suggestions')}>
              {suggestions.map((s) => (
                <li key={s}>
                  <button
                    type="button"
                    onClick={() => send(s)}
                    className="rounded-full border bg-card px-3 py-1 text-left text-xs hover:bg-accent focus-visible:ring-[3px] focus-visible:ring-ring/50 focus-visible:outline-none"
                  >
                    {s}
                  </button>
                </li>
              ))}
            </ul>
          )}
          <Composer
            onSend={send}
            onStop={() => stop(conversationId)}
            streaming={streaming}
            disabled={!!error && !run}
            placeholder={agent ? t('agentsPlatform.composer.placeholderAgent', { name: agent.name }) : undefined}
            autoFocus={!compact}
          />
          {!compact && <p className="mt-2 text-center text-xs text-muted-foreground">{t('agentsPlatform.composer.disclaimer')}</p>}
        </div>
      </div>
    </div>
  );
}
