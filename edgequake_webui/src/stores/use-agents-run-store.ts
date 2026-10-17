/**
 * SPEC-165 — live agent runs, keyed by conversation id.
 *
 * Runs live outside React components so a stream started on `/agents` keeps going
 * across the navigation to `/agents/c/[id]` (and survives StrictMode remounts).
 */
'use client';

import {
  agentsApi,
  agentsFetch,
  agentsKeys,
  AgentsApiError,
  type Conversation,
  type ConversationDetail,
  type ArchitectContext,
  type Message,
} from '@/lib/agents/api';
import { initialRun, parseSseFrames, reduceRun, settleRun, type RunState } from '@/lib/agents/chat';
import type { QueryClient } from '@tanstack/react-query';
import i18n from '@/lib/i18n';
import { toast } from 'sonner';
import { create } from 'zustand';

export interface LiveRun extends RunState {
  /** Messages shown before this run started (server history at send time). */
  base: Message[];
  controller: AbortController;
}

interface AgentsRunStore {
  runs: Record<string, LiveRun>;
  start: (conversationId: string, content: string, base: Message[], queryClient: QueryClient, context?: ArchitectContext) => Promise<void>;
  /** start() with the conversation's current messages (live run or cached history) as base. */
  send: (conversationId: string, content: string, queryClient: QueryClient, context?: ArchitectContext) => Promise<void>;
  stop: (conversationId: string) => void;
}

function patchTitle(queryClient: QueryClient, id: string, title: string) {
  queryClient.setQueriesData<Conversation[]>({ predicate: (q) => q.queryKey[2] === 'conversations' }, (old) =>
    Array.isArray(old) ? old.map((c) => (c.id === id ? { ...c, title } : c)) : old,
  );
  queryClient.setQueryData<ConversationDetail>(agentsKeys.conversation(id), (old) => (old ? { ...old, title } : old));
}

export const useAgentsRunStore = create<AgentsRunStore>((set, get) => {
  const update = (id: string, fn: (r: LiveRun) => LiveRun) =>
    set((s) => (s.runs[id] ? { runs: { ...s.runs, [id]: fn(s.runs[id]) } } : s));

  return {
    runs: {},
    stop: (id) => get().runs[id]?.controller.abort(),
    send: (id, content, queryClient, context) => {
      const run = get().runs[id];
      const base = run
        ? [...run.base, ...run.messages]
        : (queryClient.getQueryData<ConversationDetail>(agentsKeys.conversation(id))?.messages ?? []);
      return get().start(id, content, base, queryClient, context);
    },
    start: async (id, content, base, queryClient, context) => {
      get().runs[id]?.controller.abort();
      const controller = new AbortController();
      set((s) => ({ runs: { ...s.runs, [id]: { ...initialRun(content), base, controller } } }));
      try {
        const res = await agentsFetch(`/conversations/${encodeURIComponent(id)}/messages`, {
          method: 'POST',
          headers: { Accept: 'text/event-stream' },
          body: JSON.stringify(context ? { content, context } : { content }),
          signal: controller.signal,
        });
        if (!res.ok || !res.body) {
          const body = (await res.json().catch(() => ({}))) as { error?: string };
          throw new AgentsApiError(body.error || `HTTP ${res.status}`, res.status);
        }
        const reader = res.body.getReader();
        const decoder = new TextDecoder();
        let buffer = '';
        for (;;) {
          const { done, value } = await reader.read();
          if (done) break;
          const parsed = parseSseFrames(buffer + decoder.decode(value, { stream: true }));
          buffer = parsed.rest;
          if (!parsed.events.length) continue;
          update(id, (r) => ({ ...parsed.events.reduce(reduceRun, r as RunState), base: r.base, controller: r.controller }));
          const title = [...parsed.events].reverse().find((e) => e.type === 'title');
          if (title?.type === 'title') patchTitle(queryClient, id, title.title);
        }
      } catch (error) {
        if (error instanceof AgentsApiError && error.status === 409) {
          // A reply is already running in this conversation (other tab): drop our optimistic turn.
          toast.info(i18n.t('agentsPlatform.chat.busy'));
          set((s) => {
            const runs = { ...s.runs };
            delete runs[id];
            return { runs };
          });
          return;
        }
        if (!controller.signal.aborted) {
          update(id, (r) => ({ ...r, error: error instanceof Error ? error.message : String(error) }));
        }
      }
      update(id, (r) => ({ ...r, ...settleRun(r) }));
      if (get().runs[id]?.controller !== controller) return; // superseded by a newer run

      // Swap the live view for the server's rebuilt history once it holds everything we showed.
      try {
        const fresh = await agentsApi.getConversation(id);
        const run = get().runs[id];
        if (run?.controller !== controller) return;
        queryClient.setQueryData(agentsKeys.conversation(id), fresh);
        if (!run.error && fresh.messages.length >= run.base.length + run.messages.length) {
          set((s) => {
            const runs = { ...s.runs };
            delete runs[id];
            return { runs };
          });
        }
      } catch {
        /* keep the live messages on screen */
      }
      void queryClient.invalidateQueries({ predicate: (q) => q.queryKey[2] === 'conversations' });
    },
  };
});
