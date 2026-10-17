'use client';

/**
 * SPEC-165 — what a turn did, around its answer: the collapsible activity block (plan,
 * searches, synthesis), the Sources section (modelled on the Query page: document chips,
 * expandable passages, link to the document) and the floating Investigation panel.
 */
import { StreamingMarkdown } from '@/components/query/markdown';
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from '@/components/ui/collapsible';
import type { Message, RunStep, SourceRef } from '@/lib/agents/api';
import { groupSourcesByDocument, splitSnippet } from '@/lib/agents/chat';
import { cn } from '@/lib/utils';
import type { TFunction } from 'i18next';
import { Check, ChevronDown, ChevronRight, CircleAlert, ExternalLink, FileText, Loader2, Minus, Network, Search } from 'lucide-react';
import Link from 'next/link';
import { useEffect, useMemo, useState } from 'react';
import { useTranslation } from 'react-i18next';

export function stepLabel(step: Pick<RunStep, 'id' | 'label'>, t: TFunction): string {
  if (step.id === 'plan') return t('agentsPlatform.run.plan');
  if (step.id === 'synth') return t('agentsPlatform.run.synthesis');
  if (step.id === 'graph') return t('agentsPlatform.run.graph');
  return t('agentsPlatform.run.search', { label: step.label });
}

function StepIcon({ status, className }: { status: RunStep['status']; className?: string }) {
  if (status === 'running') return <Loader2 className={cn('size-3.5 animate-spin text-primary', className)} aria-hidden="true" />;
  if (status === 'error') return <CircleAlert className={cn('size-3.5 text-destructive', className)} aria-hidden="true" />;
  return <Check className={cn('size-3.5 text-emerald-600 dark:text-emerald-400', className)} aria-hidden="true" />;
}

const seconds = (ms: number) => `${Math.max(1, Math.round(ms / 1000))} s`;
const totalMs = (steps: RunStep[] | undefined) => Math.max(0, ...(steps ?? []).map((s) => s.elapsed_ms ?? 0));

/** Live seconds since `startedAt` while `live`, else the run's recorded duration. */
function useElapsed(live: boolean, startedAt: number | undefined, steps: RunStep[] | undefined): number {
  const [now, setNow] = useState(() => Date.now());
  useEffect(() => {
    if (!live) return;
    const id = setInterval(() => setNow(Date.now()), 1000);
    return () => clearInterval(id);
  }, [live]);
  return live && startedAt ? now - startedAt : totalMs(steps);
}

// ---------------------------------------------------------------- activity line

export interface ActivityTool {
  id: string;
  label: string;
  detail?: string;
  status: RunStep['status'];
}

interface ActivityProps {
  steps: RunStep[];
  tools: ActivityTool[];
  /** Coordinator hand-off target, e.g. "🎧 Support". */
  routedTo?: string;
  streaming: boolean;
  /** The answer has started: the line folds and stops announcing work. */
  answering?: boolean;
  /** Status phrases shown while the model works with nothing to report, advancing over time. */
  waiting: string[];
  onShowSources: (stepId: string) => void;
}

const WAIT_STEP_MS = 6000;

/** One compact line per turn, ChatGPT-style: shimmering current action while running,
 * a one-line summary once done; click to unfold the timeline of steps and tools. */
export function ActivityLine({ steps, tools, routedTo, streaming, answering = false, waiting, onShowSources }: ActivityProps) {
  const { t } = useTranslation();
  const [open, setOpen] = useState(false);
  const runningStep = steps.find((s) => s.status === 'running');
  const runningTool = tools.find((x) => x.status === 'running');
  const running = runningStep ?? runningTool;
  const idle = streaming && !running && !answering;
  const items = steps.length + tools.length;
  // Phrase index only moves forward, one step per WAIT_STEP_MS of idle time.
  const [waitIdx, setWaitIdx] = useState(0);
  useEffect(() => {
    if (!idle) return;
    const id = setInterval(() => setWaitIdx((i) => i + 1), WAIT_STEP_MS);
    return () => clearInterval(id);
  }, [idle]);

  const searches = steps.filter((s) => /^r\d+$/.test(s.id));
  const passages = searches.reduce((n, s) => n + (s.sources_count ?? 0), 0);
  const failed = [...steps, ...tools].filter((s) => s.status === 'error').length;
  if (!items && !routedTo && !idle) return null;

  const live = streaming && (running || idle);
  const current = runningStep ? stepLabel(runningStep, t) : (runningTool?.label ?? waiting[Math.min(waitIdx, waiting.length - 1)]);
  const summary = [
    routedTo,
    searches.length ? t('agentsPlatform.run.searched', { count: passages }) : null,
    tools.length ? t('agentsPlatform.tools.used', { count: tools.length }) : null,
    steps.length ? seconds(totalMs(steps)) : null,
  ].filter(Boolean).join(' · ');

  const trigger = (
    <>
      {live ? (
        <span className="text-shimmer min-w-0 truncate">{current}…</span>
      ) : (
        <span className="min-w-0 truncate">
          {summary}
          {failed > 0 && <span className="text-destructive"> · {t('agentsPlatform.tools.failed', { count: failed })}</span>}
        </span>
      )}
      {items > 0 && <ChevronRight className="size-3.5 shrink-0 opacity-60 transition-transform group-data-[state=open]:rotate-90" />}
    </>
  );
  const cls = 'group inline-flex max-w-full items-center gap-1 text-left text-sm text-muted-foreground outline-none focus-visible:ring-[3px] focus-visible:ring-ring/50 rounded';
  if (!items) return <div className={cls} role="status">{trigger}</div>;

  return (
    <Collapsible open={open} onOpenChange={setOpen}>
      <CollapsibleTrigger className={cn(cls, 'hover:text-foreground')} aria-live="polite">{trigger}</CollapsibleTrigger>
      <CollapsibleContent>
        <ol className="relative mt-2 ml-1.5 space-y-1.5 border-l pl-4">
          {steps.map((s) => (
            <TimelineRow key={s.id} status={s.status} label={stepLabel(s, t)} detail={s.status === 'error' ? s.detail : undefined}>
              {s.sources_count !== undefined && (
                <button
                  type="button"
                  onClick={() => onShowSources(s.id)}
                  disabled={!s.sources_count}
                  className="shrink-0 rounded-full px-1.5 text-xs text-muted-foreground tabular-nums hover:bg-accent hover:text-foreground disabled:pointer-events-none"
                >
                  {t('agentsPlatform.run.passages', { count: s.sources_count })}
                </button>
              )}
            </TimelineRow>
          ))}
          {tools.map((x) => (
            <TimelineRow key={x.id} status={x.status} label={x.label} detail={x.detail} />
          ))}
        </ol>
      </CollapsibleContent>
    </Collapsible>
  );
}

function TimelineRow({ status, label, detail, children }: { status: RunStep['status']; label: string; detail?: string; children?: React.ReactNode }) {
  return (
    <li className="relative flex items-start gap-2 text-[13px]">
      <span className="absolute -left-[1.4rem] top-0.5 rounded-full bg-background p-0.5">
        <StepIcon status={status} />
      </span>
      <span className={cn('min-w-0 flex-1 break-words', status === 'running' ? 'text-foreground' : 'text-muted-foreground')}>
        {label}
        {detail && <span className={cn('block text-xs', status === 'error' ? 'text-destructive' : 'opacity-75')}>{detail}</span>}
      </span>
      {children}
    </li>
  );
}

// ---------------------------------------------------------------- sources

interface SourcesProps {
  messageId: string;
  sources: SourceRef[];
  /** Show only the passages found by this step (Investigation panel / activity click). */
  focusStep?: string | null;
  onClearFocus?: () => void;
}

export function SourcesSection({ messageId, sources, focusStep, onClearFocus }: SourcesProps) {
  const { t } = useTranslation();
  const visible = focusStep ? sources.filter((s) => s.steps?.includes(focusStep)) : sources;
  const docs = useMemo(() => groupSourcesByDocument(visible), [visible]);
  const [expanded, setExpanded] = useState<string | null>(null);
  const maxScore = Math.max(...sources.map((s) => s.score), 0) || 1;
  const open = focusStep ? docs.map((d) => d.id) : expanded ? [expanded] : [];

  if (!sources.length) return null;
  return (
    <section id={`sources-${messageId}`} className="scroll-mt-6 flex flex-col gap-2" aria-label={t('agentsPlatform.sources.title')}>
      <div className="flex items-baseline gap-2 text-sm">
        <h3 className="font-semibold">{t('agentsPlatform.sources.title')}</h3>
        <span className="text-xs text-muted-foreground">
          {t('agentsPlatform.sources.documents', { count: docs.length })} · {t('agentsPlatform.run.passages', { count: visible.length })}
        </span>
        {focusStep && (
          <button type="button" onClick={onClearFocus} className="ml-auto text-xs text-primary hover:underline">
            {t('agentsPlatform.sources.showAll')}
          </button>
        )}
      </div>
      <ul className="flex flex-wrap gap-1.5">
        {docs.map((d) => {
          const isOpen = open.includes(d.id);
          return (
            <li key={d.id}>
              <button
                type="button"
                onClick={() => (focusStep ? onClearFocus?.() : setExpanded(isOpen ? null : d.id))}
                aria-expanded={isOpen}
                className={cn(
                  'flex max-w-[16rem] items-center gap-1.5 rounded-full border bg-card px-2.5 py-1 text-xs transition-colors hover:bg-accent focus-visible:ring-[3px] focus-visible:ring-ring/50 focus-visible:outline-none',
                  isOpen && 'border-primary/50 bg-primary/5',
                )}
                title={d.title}
              >
                <FileText className="size-3.5 shrink-0 text-muted-foreground" aria-hidden="true" />
                <span className="truncate font-medium">{d.title}</span>
                <span className="rounded-full bg-muted px-1.5 tabular-nums text-muted-foreground">{d.items.length}</span>
              </button>
            </li>
          );
        })}
      </ul>
      {docs
        .filter((d) => open.includes(d.id))
        .map((d) => (
          <div key={d.id} className="overflow-hidden rounded-xl border bg-card">
            <div className="flex items-center gap-2 border-b px-3 py-1.5 text-sm">
              <FileText className="size-4 shrink-0 text-muted-foreground" aria-hidden="true" />
              <span className="min-w-0 flex-1 truncate font-medium">{d.title}</span>
              {d.id && (
                <Link href={`/documents/${encodeURIComponent(d.id)}`} className="flex shrink-0 items-center gap-1 text-xs text-primary hover:underline">
                  {t('agentsPlatform.sources.open')} <ExternalLink className="size-3" aria-hidden="true" />
                </Link>
              )}
            </div>
            <ul className="divide-y">
              {d.items.map((s, i) => (
                <Passage key={s.chunk_id ?? i} source={s} relative={s.score / maxScore} />
              ))}
            </ul>
          </div>
        ))}
    </section>
  );
}

function Passage({ source, relative }: { source: SourceRef; relative: number }) {
  const { t } = useTranslation();
  const [full, setFull] = useState(false);
  const legacy = source.section === undefined ? splitSnippet(source.snippet) : null; // history from before UX v3
  const section = legacy ? legacy.section : source.section;
  const text = legacy ? legacy.text : source.snippet;
  const pct = Math.round(Math.min(1, relative) * 100);
  return (
    <li className="flex gap-3 px-3 py-2.5 text-sm">
      {source.ref !== undefined && (
        <span className="mt-0.5 flex size-5 shrink-0 items-center justify-center rounded-full bg-primary/10 text-[11px] font-semibold text-primary">
          {source.ref}
        </span>
      )}
      <div className="min-w-0 flex-1 space-y-1">
        <div className="flex items-center gap-2">
          {section && (
            <span className="min-w-0 flex-1 truncate text-xs text-muted-foreground" title={section}>
              {section}
            </span>
          )}
          <span className="ml-auto flex shrink-0 items-center gap-1.5 text-[11px] text-muted-foreground" title={t('agentsPlatform.sources.score', { score: pct })}>
            <span className="h-1 w-12 overflow-hidden rounded-full bg-muted">
              <span className="block h-full rounded-full bg-primary/60" style={{ width: `${pct}%` }} />
            </span>
            <span className="w-8 text-right font-medium tabular-nums">{pct}%</span>
          </span>
        </div>
        {full && source.content ? (
          <div className="max-h-96 overflow-y-auto rounded-lg bg-muted/40 px-3 py-2 text-[13px] [&_:is(h1,h2,h3,h4)]:mt-2! [&_:is(h1,h2,h3,h4)]:mb-1! [&_:is(h1,h2)]:text-sm! [&_:is(h3,h4)]:text-[13px]! [&_:is(h1,h2,h3,h4)]:font-semibold! [&_hr]:my-2!">
            <StreamingMarkdown content={source.content} />
          </div>
        ) : (
          <p className="break-words text-foreground/90">{text || source.snippet}</p>
        )}
        {source.content && (
          <button type="button" onClick={() => setFull((v) => !v)} className="text-xs text-primary hover:underline" aria-expanded={full}>
            {full ? t('agentsPlatform.sources.hidePassage') : t('agentsPlatform.sources.showPassage')}
          </button>
        )}
      </div>
    </li>
  );
}

// ---------------------------------------------------------------- investigation panel

interface PanelProps {
  message: Message;
  streaming: boolean;
  startedAt?: number;
  onShowSources: (stepId: string) => void;
  onShowGraph?: () => void;
}

/** Floating checklist of the investigation (bottom-right; bottom sheet on phones). Minimises to a pill. */
export function InvestigationPanel({ message, streaming, startedAt, onShowSources, onShowGraph }: PanelProps) {
  const { t } = useTranslation();
  const [open, setOpen] = useState<boolean | null>(null); // null = follow the run (keyed by message id)
  const steps = message.steps ?? [];
  const elapsed = useElapsed(streaming, startedAt, steps);
  const byId = new Map(steps.map((s) => [s.id, s]));
  const rows: RunStep[] = [
    ...(message.plan ?? []).map((p) => byId.get(p.id) ?? { ...p, status: 'running' as const }),
    ...['synth', 'graph'].flatMap((id) => (byId.has(id) ? [byId.get(id)!] : [])),
  ];
  const done = rows.filter((r) => r.status === 'done').length;
  const isOpen = open ?? (streaming && !message.content); // folds to a pill once the answer streams

  if (!isOpen) {
    return (
      <button
        type="button"
        onClick={() => setOpen(true)}
        className="absolute right-4 bottom-full z-20 mb-3 flex items-center gap-2 rounded-full border bg-card px-3 py-1.5 text-sm shadow-md hover:bg-accent focus-visible:ring-[3px] focus-visible:ring-ring/50 focus-visible:outline-none"
        aria-label={t('agentsPlatform.investigation.open')}
      >
        {streaming ? <Loader2 className="size-4 animate-spin text-primary" /> : <Search className="size-4 text-primary" />}
        <span className="font-medium">{t('agentsPlatform.investigation.title')}</span>
        <span className="tabular-nums text-muted-foreground">
          {done}/{rows.length} · {seconds(elapsed)}
        </span>
      </button>
    );
  }

  return (
    <aside
      aria-label={t('agentsPlatform.investigation.title')}
      className={cn(
        'z-20 flex flex-col overflow-hidden border bg-card shadow-xl',
        'absolute right-4 bottom-full mb-3 max-h-[min(28rem,60vh)] w-80 rounded-2xl',
        'max-sm:fixed max-sm:inset-x-0 max-sm:right-0 max-sm:bottom-0 max-sm:mb-0 max-sm:max-h-[70vh] max-sm:w-auto max-sm:rounded-b-none max-sm:rounded-t-2xl',
      )}
    >
      <header className="flex items-center gap-2 border-b px-3 py-2">
        {streaming ? <Loader2 className="size-4 animate-spin text-primary" /> : <Check className="size-4 text-emerald-600 dark:text-emerald-400" />}
        <h2 className="min-w-0 flex-1 truncate text-sm font-semibold">{t('agentsPlatform.investigation.title')}</h2>
        <span className="text-xs tabular-nums text-muted-foreground">{seconds(elapsed)}</span>
        <button
          type="button"
          onClick={() => setOpen(false)}
          className="flex size-7 items-center justify-center rounded-lg text-muted-foreground hover:bg-accent hover:text-foreground"
          aria-label={t('agentsPlatform.investigation.minimize')}
          title={t('agentsPlatform.investigation.minimize')}
        >
          <Minus className="size-4 max-sm:hidden" />
          <ChevronDown className="size-4 sm:hidden" />
        </button>
      </header>
      <ol className="min-h-0 flex-1 space-y-1 overflow-y-auto p-2">
        {rows.map((r) => (
          <li key={r.id} className="flex items-start gap-2 rounded-lg px-2 py-1.5 text-sm">
            <StepIcon status={r.status} className="mt-0.5 shrink-0" />
            <span className={cn('min-w-0 flex-1 break-words', r.status === 'done' && 'text-muted-foreground')}>
              {/^r\d+$/.test(r.id) ? r.label : stepLabel(r, t)}
            </span>
            {r.sources_count !== undefined && (
              <button
                type="button"
                onClick={() => onShowSources(r.id)}
                disabled={!r.sources_count}
                className="shrink-0 rounded-full border px-1.5 text-xs tabular-nums text-muted-foreground hover:bg-accent hover:text-foreground disabled:opacity-50"
                aria-label={t('agentsPlatform.investigation.showSources', { count: r.sources_count })}
                title={t('agentsPlatform.investigation.showSources', { count: r.sources_count })}
              >
                {r.sources_count}
              </button>
            )}
          </li>
        ))}
      </ol>
      {onShowGraph && (
        <footer className="border-t p-2">
          <button
            type="button"
            onClick={onShowGraph}
            className="flex w-full items-center justify-center gap-2 rounded-lg border px-3 py-1.5 text-sm hover:bg-accent focus-visible:ring-[3px] focus-visible:ring-ring/50 focus-visible:outline-none"
          >
            <Network className="size-4" /> {t('agentsPlatform.investigation.showGraph')}
          </button>
        </footer>
      )}
    </aside>
  );
}
