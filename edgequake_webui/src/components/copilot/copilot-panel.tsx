'use client';

/**
 * EdgeQuake Copilot — docked assistant available on every dashboard page.
 *
 * Knows where the user is (page, workspace, graph selection), streams answers,
 * shows its tool activity, drives the UI (navigate / highlight nodes) and applies
 * graph edits only after the user approves the proposal card.
 */
import { StreamingMarkdown } from '@/components/query/markdown';
import { Button } from '@/components/ui/button';
import { Textarea } from '@/components/ui/textarea';
import { refreshAccessToken } from '@/lib/api/client';
import { getTenantContext, getTokens } from '@/lib/api/client-context';
import { ensureSessionScope } from '@/lib/api/session-scope';
import { invalidateKnowledgeGraph } from '@/lib/cache-manager';
import { cn } from '@/lib/utils';
import { useAuthStore } from '@/stores/use-auth-store';
import {
  useCopilotStore,
  type CopilotMessage,
  type CopilotProposal,
  type CopilotToolEvent,
} from '@/stores/use-copilot-store';
import { useGraphStore } from '@/stores/use-graph-store';
import { useTenantStore } from '@/stores/use-tenant-store';
import { useQueryClient } from '@tanstack/react-query';
import {
  ArrowUp,
  Check,
  CircleAlert,
  Copy,
  Loader2,
  Sparkles,
  Square,
  Trash2,
  X,
} from 'lucide-react';
import { usePathname, useRouter } from 'next/navigation';
import { useCallback, useEffect, useRef, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { toast } from 'sonner';

type CopilotEvent =
  | { type: 'text'; delta: string }
  | ({ type: 'tool' } & CopilotToolEvent)
  | { type: 'proposal'; id: string; title: string; rationale: string; operations: unknown[]; descriptions: string[] }
  | { type: 'ui'; action: 'navigate'; path: string }
  | { type: 'ui'; action: 'highlight'; names: string[] }
  | { type: 'ui'; action: 'refresh_graph' }
  | { type: 'applied'; results: Array<{ ok: boolean; summary: string }> }
  | { type: 'error'; message: string }
  | { type: 'done' };

const NO_MESSAGES: CopilotMessage[] = [];
const uid = () => (typeof crypto !== 'undefined' && crypto.randomUUID ? crypto.randomUUID() : `${Date.now()}-${Math.random()}`);
const clip = (s: string | undefined, n: number) => (s && s.length > n ? `${s.slice(0, n)}…` : s);

/** Snapshot of what the user is looking at, sent with every turn. */
function buildContext(pathname: string) {
  const tenant = useTenantStore.getState();
  const graph = useGraphStore.getState();
  const user = useAuthStore.getState().user;
  const t = tenant.tenants.find((x) => x.id === tenant.selectedTenantId);
  const w = tenant.workspaces.find((x) => x.id === tenant.selectedWorkspaceId);
  const node = (id: string | null | undefined) => {
    const n = id ? graph.nodeMap.get(id) : undefined;
    return n ? { name: n.label, type: n.node_type, description: clip(n.description, 300) } : undefined;
  };
  const onGraph = pathname.includes('/graph');
  const doc = pathname.match(/\/documents\/([^/?#]+)/)?.[1];
  return {
    page: pathname,
    url_query: typeof window !== 'undefined' ? window.location.search : '',
    tenant: t ? { id: t.id, name: t.name } : { id: tenant.selectedTenantId },
    workspace: w ? { id: w.id, name: w.name, slug: w.slug } : { id: tenant.selectedWorkspaceId },
    user: user ? { username: user.username, role: user.role } : undefined,
    graph: onGraph
      ? {
          visible_entities: graph.nodes.length,
          visible_relationships: graph.edges.length,
          selected_entity: node(graph.selectedNodeId),
          multi_selection: [...graph.selectedNodes].slice(0, 25).map((id) => graph.nodeMap.get(id)?.label ?? id),
          search_query: graph.searchQuery || undefined,
        }
      : undefined,
    open_document_id: doc,
    locale: typeof navigator !== 'undefined' ? navigator.language : undefined,
  };
}

/** Highlight entities by name on the loaded graph; true when something matched. */
function highlightOnGraph(names: string[]): boolean {
  const st = useGraphStore.getState();
  const wanted = new Set(names.map((n) => n.trim().toUpperCase()));
  const ids = st.nodes
    .filter((n) => {
      const bare = n.id.split('::').pop() ?? n.id;
      return wanted.has(n.label?.toUpperCase()) || wanted.has(n.id.toUpperCase()) || wanted.has(bare.toUpperCase());
    })
    .map((n) => n.id);
  if (!ids.length) return false;
  st.setEngineFocus({ mode: 'answer', ids });
  st.selectNode(ids[0]);
  return true;
}

function suggestionsFor(pathname: string, hasSelection: boolean, t: (k: string, d: string) => string): string[] {
  if (pathname.includes('/graph')) {
    return hasSelection
      ? [
          t('copilot.suggest.explainSelection', 'Explain the selected entity and its connections'),
          t('copilot.suggest.duplicates', 'Find duplicates of the selected entity and merge them'),
          t('copilot.suggest.badLinks', 'Which of its connections look wrong? Propose a better split'),
        ]
      : [
          t('copilot.suggest.hubs', 'What are the most connected entities?'),
          t('copilot.suggest.cleanup', 'Find duplicate entities I should merge'),
          t('copilot.suggest.clusters', 'Propose a grouping of related entities'),
        ];
  }
  if (pathname.includes('/documents')) {
    return [
      t('copilot.suggest.failedDocs', 'Which documents failed or are still processing?'),
      t('copilot.suggest.docSummary', 'Summarize the documents in this workspace'),
    ];
  }
  return [
    t('copilot.suggest.overview', 'Give me an overview of this workspace'),
    t('copilot.suggest.topics', 'What are the main topics in my documents?'),
    t('copilot.suggest.openGraph', 'Show me the key entities on the graph'),
  ];
}

export function CopilotPanel() {
  const { t } = useTranslation();
  const router = useRouter();
  const pathname = usePathname() ?? '/';
  const queryClient = useQueryClient();
  const open = useCopilotStore((s) => s.open);
  const width = useCopilotStore((s) => s.width);
  const setOpen = useCopilotStore((s) => s.setOpen);
  const toggle = useCopilotStore((s) => s.toggle);
  const setWidth = useCopilotStore((s) => s.setWidth);
  const setThread = useCopilotStore((s) => s.setThread);
  const clearThread = useCopilotStore((s) => s.clearThread);
  const workspaceId = useTenantStore((s) => s.selectedWorkspaceId) ?? 'none';
  const workspaceName = useTenantStore((s) => s.workspaces.find((w) => w.id === s.selectedWorkspaceId)?.name);
  const messages = useCopilotStore((s) => s.threads[workspaceId]) ?? NO_MESSAGES;
  const hasSelection = useGraphStore((s) => !!s.selectedNodeId || s.selectedNodes.size > 0);
  const graphNodes = useGraphStore((s) => s.nodes);

  const [input, setInput] = useState('');
  const [busy, setBusy] = useState(false);
  const abortRef = useRef<AbortController | null>(null);
  const scrollRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLTextAreaElement>(null);
  const pendingHighlight = useRef<string[] | null>(null);
  const awaitingRefresh = useRef(false);

  // ⌘/Ctrl+J toggles the copilot from anywhere; Esc closes it.
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === 'j') {
        e.preventDefault();
        toggle();
      } else if (e.key === 'Escape' && useCopilotStore.getState().open && !busy) {
        const active = document.activeElement;
        if (active && active.closest('[data-copilot-panel]')) setOpen(false);
      }
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [toggle, setOpen, busy]);

  useEffect(() => {
    if (open) setTimeout(() => inputRef.current?.focus(), 50);
  }, [open]);

  // On the graph page the copilot takes the details panel's place (canvas keeps its width).
  useEffect(() => {
    if (open && pathname.includes('/graph') && !useGraphStore.getState().rightPanelCollapsed) {
      useGraphStore.setState({ rightPanelCollapsed: true });
    }
  }, [open, pathname]);

  // Auto-scroll while streaming.
  useEffect(() => {
    scrollRef.current?.scrollTo({ top: scrollRef.current.scrollHeight });
  }, [messages]);

  // Apply a pending highlight once the (re)loaded graph contains the nodes.
  useEffect(() => {
    if (!pendingHighlight.current || !graphNodes.length || !pathname.includes('/graph')) return;
    if (highlightOnGraph(pendingHighlight.current)) {
      pendingHighlight.current = null;
      awaitingRefresh.current = false;
    }
  }, [graphNodes, pathname]);

  const patchLast = useCallback(
    (update: (m: CopilotMessage) => CopilotMessage) =>
      setThread(workspaceId, (list) => {
        if (!list.length) return list;
        const copy = [...list];
        copy[copy.length - 1] = update(copy[copy.length - 1]);
        return copy;
      }),
    [setThread, workspaceId],
  );

  const handleEvent = useCallback(
    (event: CopilotEvent) => {
      switch (event.type) {
        case 'text':
          patchLast((m) => ({ ...m, content: m.content + event.delta }));
          break;
        case 'tool': {
          const tool: CopilotToolEvent = {
            id: event.id,
            name: event.name,
            label: event.label,
            status: event.status,
            detail: event.detail,
          };
          patchLast((m) => {
            const others = m.tools.filter((x) => x.id !== tool.id);
            return { ...m, tools: [...others, tool] };
          });
          break;
        }
        case 'proposal':
          patchLast((m) => ({
            ...m,
            proposal: {
              id: event.id,
              title: event.title,
              rationale: event.rationale,
              operations: event.operations,
              descriptions: event.descriptions,
              state: 'pending',
            },
          }));
          break;
        case 'applied': {
          const failed = event.results.filter((r) => !r.ok).length;
          if (failed) toast.error(t('copilot.applyPartial', '{{n}} change(s) failed', { n: failed }));
          else toast.success(t('copilot.applyOk', 'Graph updated'));
          setThread(workspaceId, (list) =>
            list.map((m) =>
              m.proposal?.state === 'approving'
                ? { ...m, proposal: { ...m.proposal, state: 'approved', results: event.results } }
                : m,
            ),
          );
          break;
        }
        case 'ui':
          if (event.action === 'navigate') {
            router.push(event.path);
          } else if (event.action === 'refresh_graph') {
            awaitingRefresh.current = true;
            invalidateKnowledgeGraph(queryClient);
          } else if (event.action === 'highlight') {
            pendingHighlight.current = event.names;
            if (!pathname.includes('/graph')) {
              router.push(`/graph?entity=${encodeURIComponent(event.names[0] ?? '')}`);
            } else if (!awaitingRefresh.current && highlightOnGraph(event.names)) {
              pendingHighlight.current = null;
            }
          }
          break;
        case 'error':
          patchLast((m) => ({ ...m, error: event.message }));
          break;
        case 'done':
          break;
      }
    },
    [patchLast, router, pathname, queryClient, setThread, workspaceId, t],
  );

  const run = useCallback(
    async (history: CopilotMessage[], approval?: { decision: 'approve' | 'reject'; proposal: CopilotProposal }) => {
      const controller = new AbortController();
      abortRef.current = controller;
      setBusy(true);
      setThread(workspaceId, (list) => [
        ...list,
        { id: uid(), role: 'assistant', content: '', tools: [], streaming: true },
      ]);
      const payload = JSON.stringify({
        messages: history
          .filter((m) => m.content.trim())
          .map((m) => ({ role: m.role, content: m.content })),
        context: buildContext(pathname),
        approval: approval && {
          decision: approval.decision,
          title: approval.proposal.title,
          operations: approval.proposal.operations,
        },
      });
      const send = async () => {
        await ensureSessionScope();
        const { tenantId, workspaceId: ws } = getTenantContext();
        const headers: Record<string, string> = { 'Content-Type': 'application/json' };
        const token = getTokens().accessToken;
        if (token) headers.Authorization = `Bearer ${token}`;
        if (tenantId) headers['X-Tenant-ID'] = tenantId;
        if (ws) headers['X-Workspace-ID'] = ws;
        return fetch('/api/copilot', { method: 'POST', headers, body: payload, signal: controller.signal });
      };
      try {
        let res = await send();
        if (res.status === 401 && (await refreshAccessToken())) res = await send();
        if (!res.ok || !res.body) {
          const body = (await res.json().catch(() => ({}))) as { error?: string };
          throw new Error(body.error || `HTTP ${res.status}`);
        }
        const reader = res.body.getReader();
        const decoder = new TextDecoder();
        let buffer = '';
        for (;;) {
          const { done, value } = await reader.read();
          if (done) break;
          buffer += decoder.decode(value, { stream: true });
          const frames = buffer.split('\n\n');
          buffer = frames.pop() ?? '';
          for (const frame of frames) {
            const line = frame.split('\n').find((l) => l.startsWith('data:'));
            if (!line) continue;
            try {
              handleEvent(JSON.parse(line.slice(5).trim()) as CopilotEvent);
            } catch {
              /* ignore malformed frame */
            }
          }
        }
      } catch (error) {
        if (!controller.signal.aborted) {
          patchLast((m) => ({ ...m, error: error instanceof Error ? error.message : String(error) }));
        }
      } finally {
        patchLast((m) => ({ ...m, streaming: false }));
        setBusy(false);
        abortRef.current = null;
      }
    },
    [handleEvent, pathname, patchLast, setThread, workspaceId],
  );

  const send = useCallback(
    (text: string) => {
      const content = text.trim();
      if (!content || busy) return;
      const userMsg: CopilotMessage = { id: uid(), role: 'user', content, tools: [] };
      const history = [...(useCopilotStore.getState().threads[workspaceId] ?? []), userMsg];
      setThread(workspaceId, () => history);
      setInput('');
      void run(history);
    },
    [busy, run, setThread, workspaceId],
  );

  const decide = useCallback(
    (proposal: CopilotProposal, decision: 'approve' | 'reject') => {
      if (busy) return;
      setThread(workspaceId, (list) =>
        list.map((m) =>
          m.proposal?.id === proposal.id
            ? { ...m, proposal: { ...proposal, state: decision === 'approve' ? 'approving' : 'rejected' } }
            : m,
        ),
      );
      void run(useCopilotStore.getState().threads[workspaceId] ?? [], { decision, proposal });
    },
    [busy, run, setThread, workspaceId],
  );

  // Resize from the left edge.
  const startResize = (e: React.PointerEvent) => {
    e.preventDefault();
    const startX = e.clientX;
    const startW = width;
    const move = (ev: PointerEvent) => setWidth(startW + (startX - ev.clientX));
    const up = () => {
      window.removeEventListener('pointermove', move);
      window.removeEventListener('pointerup', up);
    };
    window.addEventListener('pointermove', move);
    window.addEventListener('pointerup', up);
  };

  if (!open) return null;

  const suggestions = suggestionsFor(pathname, hasSelection, t);

  return (
    <aside
      data-copilot-panel
      aria-label={t('copilot.title', 'Copilot')}
      style={{ ['--copilot-w' as string]: `${width}px` }}
      className="fixed inset-0 z-50 flex min-h-0 flex-col overflow-hidden border-l bg-background md:relative md:inset-auto md:z-auto md:h-full md:max-h-dvh md:w-[var(--copilot-w)] md:shrink-0"
    >
      <div
        role="separator"
        aria-orientation="vertical"
        onPointerDown={startResize}
        className="absolute inset-y-0 -left-1 z-10 hidden w-2 cursor-col-resize hover:bg-primary/20 md:block"
      />

      <header className="flex h-12 shrink-0 items-center gap-2 border-b px-3">
        <span className="flex h-7 w-7 items-center justify-center rounded-md bg-primary text-primary-foreground">
          <Sparkles className="h-4 w-4" />
        </span>
        <div className="min-w-0 flex-1">
          <div className="text-sm font-semibold leading-tight">{t('copilot.title', 'Copilot')}</div>
          <div className="truncate text-xs text-muted-foreground">{workspaceName ?? '—'}</div>
        </div>
        {messages.length > 0 && (
          <Button
            variant="ghost"
            size="icon"
            className="h-8 w-8"
            disabled={busy}
            onClick={() => clearThread(workspaceId)}
            aria-label={t('copilot.clear', 'New conversation')}
            title={t('copilot.clear', 'New conversation')}
          >
            <Trash2 className="h-4 w-4" />
          </Button>
        )}
        <Button
          variant="ghost"
          size="icon"
          className="h-8 w-8"
          onClick={() => setOpen(false)}
          aria-label={t('common.close', 'Close')}
        >
          <X className="h-4 w-4" />
        </Button>
      </header>

      <div ref={scrollRef} className="relative min-h-0 flex-1 overflow-x-hidden overflow-y-auto overscroll-contain px-3 py-4 [contain:paint]" aria-live="polite">
        {messages.length === 0 ? (
          <div className="flex h-full flex-col justify-end gap-3 pb-2">
            <div>
              <p className="text-base font-semibold">{t('copilot.hello', 'How can I help with this workspace?')}</p>
              <p className="mt-1 text-sm text-muted-foreground">
                {t(
                  'copilot.intro',
                  'I can see the page you are on, explore your graph and documents, and propose changes you approve.',
                )}
              </p>
            </div>
            <div className="flex flex-col gap-2">
              {suggestions.map((s) => (
                <button
                  key={s}
                  type="button"
                  onClick={() => send(s)}
                  className="rounded-lg border px-3 py-2 text-left text-sm transition-colors hover:bg-muted"
                >
                  {s}
                </button>
              ))}
            </div>
          </div>
        ) : (
          <ol className="flex flex-col gap-4">
            {messages.map((m) => (
              <MessageItem key={m.id} message={m} busy={busy} onDecide={decide} />
            ))}
          </ol>
        )}
      </div>

      <form
        className="relative z-10 shrink-0 border-t bg-background p-3"
        onSubmit={(e) => {
          e.preventDefault();
          send(input);
        }}
      >
        <div className="flex items-end gap-2 rounded-xl border bg-muted/30 p-2 focus-within:ring-2 focus-within:ring-ring">
          <Textarea
            ref={inputRef}
            value={input}
            onChange={(e) => setInput(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === 'Enter' && !e.shiftKey && !e.nativeEvent.isComposing) {
                e.preventDefault();
                send(input);
              }
            }}
            rows={1}
            placeholder={
              hasSelection
                ? t('copilot.placeholderSelection', 'Ask about the selection, or tell me what to change…')
                : t('copilot.placeholder', 'Ask anything about this workspace…')
            }
            className="max-h-40 min-h-9 resize-none border-0 bg-transparent p-1 shadow-none focus-visible:ring-0"
            aria-label={t('copilot.inputLabel', 'Message the copilot')}
          />
          {busy ? (
            <Button
              type="button"
              size="icon"
              variant="secondary"
              className="h-8 w-8 shrink-0"
              onClick={() => abortRef.current?.abort()}
              aria-label={t('copilot.stop', 'Stop')}
            >
              <Square className="h-3.5 w-3.5" />
            </Button>
          ) : (
            <Button
              type="submit"
              size="icon"
              className="h-8 w-8 shrink-0"
              disabled={!input.trim()}
              aria-label={t('copilot.send', 'Send')}
            >
              <ArrowUp className="h-4 w-4" />
            </Button>
          )}
        </div>
        <p className="mt-1.5 px-1 text-[11px] text-muted-foreground">
          {t('copilot.hint', 'Enter to send · Shift+Enter new line · ⌘/Ctrl+J toggle')}
        </p>
      </form>
    </aside>
  );
}

function MessageItem({
  message,
  busy,
  onDecide,
}: {
  message: CopilotMessage;
  busy: boolean;
  onDecide: (p: CopilotProposal, d: 'approve' | 'reject') => void;
}) {
  const { t } = useTranslation();
  if (message.role === 'user') {
    return (
      <li className="ml-8 self-end whitespace-pre-wrap rounded-2xl rounded-br-sm bg-primary px-3 py-2 text-sm text-primary-foreground">
        {message.content}
      </li>
    );
  }
  const thinking = message.streaming && !message.content && !message.tools.some((x) => x.status === 'running');
  return (
    <li className="group flex min-w-0 flex-col gap-2">
      {message.tools.length > 0 && (
        <ul className="flex flex-wrap gap-1.5">
          {message.tools.map((tool) => (
            <li
              key={tool.id}
              title={tool.detail}
              className={cn(
                'flex max-w-full items-center gap-1 rounded-full border px-2 py-0.5 text-xs text-muted-foreground',
                tool.status === 'error' && 'border-destructive/40 text-destructive',
              )}
            >
              {tool.status === 'running' ? (
                <Loader2 className="h-3 w-3 animate-spin" />
              ) : tool.status === 'done' ? (
                <Check className="h-3 w-3" />
              ) : (
                <CircleAlert className="h-3 w-3" />
              )}
              <span className="truncate">
                {tool.label}
                {tool.detail && tool.status !== 'error' ? ` · ${clip(tool.detail, 32)}` : ''}
              </span>
            </li>
          ))}
        </ul>
      )}
      {thinking && (
        <div className="flex items-center gap-2 text-sm text-muted-foreground">
          <Loader2 className="h-4 w-4 animate-spin" />
          {t('copilot.thinking', 'Thinking…')}
        </div>
      )}
      {message.content && (
        <div className="min-w-0 break-words text-sm">
          <StreamingMarkdown content={message.content} isStreaming={!!message.streaming} />
        </div>
      )}
      {message.proposal && <ProposalCard proposal={message.proposal} busy={busy} onDecide={onDecide} />}
      {message.error && (
        <div className="flex items-start gap-2 rounded-md border border-destructive/40 bg-destructive/5 px-3 py-2 text-xs text-destructive">
          <CircleAlert className="mt-0.5 h-3.5 w-3.5 shrink-0" />
          <span className="break-words">{message.error}</span>
        </div>
      )}
      {!message.streaming && message.content && (
        <button
          type="button"
          onClick={() => {
            void navigator.clipboard.writeText(message.content);
            toast.success(t('copilot.copied', 'Copied'));
          }}
          className="flex w-fit items-center gap-1 text-xs text-muted-foreground opacity-0 transition-opacity hover:text-foreground focus:opacity-100 group-hover:opacity-100"
        >
          <Copy className="h-3 w-3" /> {t('copilot.copy', 'Copy')}
        </button>
      )}
    </li>
  );
}

function ProposalCard({
  proposal,
  busy,
  onDecide,
}: {
  proposal: CopilotProposal;
  busy: boolean;
  onDecide: (p: CopilotProposal, d: 'approve' | 'reject') => void;
}) {
  const { t } = useTranslation();
  const results = proposal.results;
  return (
    <div className="rounded-xl border bg-card p-3 shadow-sm">
      <div className="flex items-center gap-2 text-sm font-semibold">
        <Sparkles className="h-4 w-4 text-primary" />
        <span className="min-w-0 flex-1">{proposal.title}</span>
        <span className="text-xs font-normal text-muted-foreground">
          {t('copilot.opsCount', '{{n}} change(s)', { n: proposal.descriptions.length })}
        </span>
      </div>
      {proposal.rationale && <p className="mt-1 text-xs text-muted-foreground">{proposal.rationale}</p>}
      <ol className="mt-2 flex flex-col gap-1">
        {proposal.descriptions.map((d, i) => {
          const r = results?.[i];
          return (
            <li key={i} className="flex items-start gap-2 rounded-md bg-muted/50 px-2 py-1 font-mono text-xs">
              <span className="mt-px w-4 shrink-0 text-muted-foreground">
                {r ? r.ok ? <Check className="h-3 w-3 text-green-600" /> : <CircleAlert className="h-3 w-3 text-destructive" /> : `${i + 1}.`}
              </span>
              <span className="break-words">{r && !r.ok ? r.summary : d}</span>
            </li>
          );
        })}
      </ol>
      {proposal.state === 'pending' && (
        <div className="mt-3 flex gap-2">
          <Button size="sm" disabled={busy} onClick={() => onDecide(proposal, 'approve')}>
            <Check className="mr-1 h-3.5 w-3.5" />
            {t('copilot.approve', 'Approve')}
          </Button>
          <Button size="sm" variant="outline" disabled={busy} onClick={() => onDecide(proposal, 'reject')}>
            {t('copilot.reject', 'Reject')}
          </Button>
        </div>
      )}
      {proposal.state === 'approving' && (
        <div className="mt-3 flex items-center gap-2 text-xs text-muted-foreground">
          <Loader2 className="h-3.5 w-3.5 animate-spin" /> {t('copilot.applying', 'Applying…')}
        </div>
      )}
      {proposal.state === 'approved' && (
        <div className="mt-2 text-xs text-green-700 dark:text-green-400">{t('copilot.applied', 'Applied')}</div>
      )}
      {proposal.state === 'rejected' && (
        <div className="mt-2 text-xs text-muted-foreground">{t('copilot.rejected', 'Rejected — nothing changed')}</div>
      )}
    </div>
  );
}

/** Header toggle button. */
export function CopilotToggle() {
  const { t } = useTranslation();
  const open = useCopilotStore((s) => s.open);
  const toggle = useCopilotStore((s) => s.toggle);
  return (
    <Button
      variant={open ? 'secondary' : 'ghost'}
      size="sm"
      className="h-8 gap-1.5 px-2"
      onClick={toggle}
      aria-pressed={open}
      title={`${t('copilot.title', 'Copilot')} (⌘/Ctrl+J)`}
    >
      <Sparkles className="h-4 w-4" />
      <span className="hidden text-xs font-medium lg:inline">{t('copilot.title', 'Copilot')}</span>
    </Button>
  );
}
