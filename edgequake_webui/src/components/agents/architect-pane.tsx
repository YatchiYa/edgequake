'use client';

import { StreamingMarkdown } from '@/components/query/markdown';
import { Button } from '@/components/ui/button';
import { Skeleton } from '@/components/ui/skeleton';
import { useAgentModels, useAgentTools, type AgentInput, type AgentSpecProposal, type ArchitectContext } from '@/lib/agents/api';
import { diffSpec, type SpecField } from '@/lib/agents/architect';
import { cn } from '@/lib/utils';
import { Check, ChevronDown, ClipboardCheck, FlaskConical, Loader2, Sparkles, Wand2 } from 'lucide-react';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { AgentAvatar } from './agent-bits';
import { ChatView } from './chat-view';
import { Composer } from './composer';

const IDEAS = ['ideaFaq', 'ideaAnalyst', 'ideaGraph'] as const;

/** Left side of the AI mode: the architect conversation (the builder creates it on first send). */
export function ArchitectChat({
  conversationId,
  getContext,
  onStart,
  starting,
}: {
  conversationId: string | null;
  getContext: () => ArchitectContext;
  onStart: (text: string) => void;
  starting: boolean;
}) {
  const { t } = useTranslation();
  if (conversationId) return <ChatView key={conversationId} conversationId={conversationId} compact getContext={getContext} />;
  return (
    <div className="flex min-h-0 flex-1 flex-col">
      <div className="flex flex-1 flex-col items-center justify-center gap-3 overflow-y-auto p-6 text-center">
        <AgentAvatar icon="🏗️" className="size-12 text-2xl" />
        <h2 className="text-lg font-semibold">{t('agentsPlatform.architect.introTitle')}</h2>
        <p className="max-w-md text-sm text-muted-foreground">{t('agentsPlatform.architect.introBody')}</p>
        <ul className="mt-2 flex max-w-lg flex-col gap-2">
          {IDEAS.map((k) => (
            <li key={k}>
              <button
                type="button"
                disabled={starting}
                onClick={() => onStart(t(`agentsPlatform.architect.${k}`))}
                className="w-full rounded-2xl border bg-card px-4 py-2.5 text-left text-sm hover:bg-accent disabled:opacity-60"
              >
                {t(`agentsPlatform.architect.${k}`)}
              </button>
            </li>
          ))}
        </ul>
      </div>
      <div className="p-3">
        <Composer onSend={onStart} disabled={starting} placeholder={t('agentsPlatform.architect.placeholder')} autoFocus />
      </div>
    </div>
  );
}

function Row({ field, label, diff, children }: { field: SpecField; label: string; diff: Set<SpecField>; children: React.ReactNode }) {
  const { t } = useTranslation();
  const changed = diff.has(field);
  return (
    <div className={cn('flex flex-col gap-1 border-l-2 border-transparent pl-3', changed && 'border-amber-500 bg-amber-500/5')} data-changed={changed || undefined}>
      <div className="flex items-center gap-2 text-xs font-medium text-muted-foreground">
        {label}
        {changed && <span className="rounded bg-amber-500/15 px-1.5 text-[10px] font-semibold text-amber-700 uppercase dark:text-amber-300">{t('agentsPlatform.architect.changed')}</span>}
      </div>
      <div className="text-sm">{children}</div>
    </div>
  );
}

/** Right side of the AI mode: the latest proposal, with what changed since the previous one. */
export function SpecCard({
  proposal,
  drafting = false,
  previous,
  version,
  applied,
  saved,
  saving,
  editing,
  readOnly,
  onApply,
  onSave,
  onTest,
}: {
  proposal: AgentSpecProposal | undefined;
  /** The architect is working on a (new) proposal. */
  drafting?: boolean;
  previous: AgentInput | undefined;
  version: number;
  /** The form already holds this spec. */
  applied: boolean;
  /** The saved agent already matches this spec. */
  saved: boolean;
  saving: boolean;
  editing: boolean;
  readOnly: boolean;
  onApply: () => void;
  onSave: () => void;
  onTest: () => void;
}) {
  const { t } = useTranslation();
  const { data: tools } = useAgentTools();
  const { data: models } = useAgentModels();
  const [showInstructions, setShowInstructions] = useState(false);

  if (!proposal && drafting) {
    return (
      <div className="flex flex-1 flex-col gap-4 p-4" role="status" aria-busy="true">
        <p className="text-shimmer text-sm text-muted-foreground">{t('agentsPlatform.architect.drafting')}…</p>
        <div className="flex items-start gap-3">
          <Skeleton className="size-12 rounded-full" />
          <div className="flex flex-1 flex-col gap-2 pt-1">
            <Skeleton className="h-5 w-1/2" />
            <Skeleton className="h-4 w-5/6" />
          </div>
        </div>
        <div className="grid grid-cols-3 gap-3">
          {[0, 1, 2].map((i) => <Skeleton key={i} className="h-10" />)}
        </div>
        <Skeleton className="h-7 w-2/3 rounded-full" />
        <Skeleton className="h-40" />
      </div>
    );
  }
  if (!proposal) {
    return (
      <div className="flex flex-1 flex-col items-center justify-center gap-3 p-8 text-center text-sm text-muted-foreground">
        <Sparkles className="size-8" />
        <p className="max-w-xs">{t('agentsPlatform.architect.noSpec')}</p>
      </div>
    );
  }
  const spec = proposal.spec;
  const { changed, addedTools, removedTools } = diffSpec(previous, spec);
  const toolName = (id: string) => tools?.find((x) => x.id === id)?.name ?? id;
  const modelName = models?.find((m) => m.id === spec.model)?.label ?? spec.model;

  return (
    <div className="flex min-h-0 flex-1 flex-col">
      <div className="min-h-0 flex-1 overflow-y-auto p-4" data-testid="spec-card">
        {drafting && (
          <p role="status" className="text-shimmer mb-3 rounded-lg bg-primary/5 px-3 py-1.5 text-sm text-primary">
            {t('agentsPlatform.architect.revising', { n: version + 1 })}…
          </p>
        )}
        <div className="flex items-start gap-3">
          <AgentAvatar icon={spec.icon} className="size-12 text-2xl" />
          <div className="min-w-0 flex-1">
            <div className="flex flex-wrap items-center gap-2">
              <h3 className={cn('text-base font-semibold', changed.has('name') && 'text-amber-700 dark:text-amber-300')}>{spec.name}</h3>
              <span className="rounded-full border px-2 text-xs text-muted-foreground">{t('agentsPlatform.architect.version', { n: version })}</span>
              {changed.size > 0 && (
                <span className="text-xs text-amber-700 dark:text-amber-300">{t('agentsPlatform.architect.changedCount', { count: changed.size })}</span>
              )}
            </div>
            <p className={cn('text-sm text-muted-foreground', changed.has('description') && 'text-amber-700 dark:text-amber-300')}>{spec.description}</p>
          </div>
        </div>

        <div className="mt-4 flex flex-col gap-3">
          <div className="grid grid-cols-3 gap-2">
            <Row field="model" label={t('agentsPlatform.builder.model')} diff={changed}>
              {modelName}
            </Row>
            <Row field="temperature" label={t('agentsPlatform.builder.temperature')} diff={changed}>
              {spec.temperature.toFixed(1)}
            </Row>
            <Row field="visibility" label={t('agentsPlatform.builder.visibility')} diff={changed}>
              {t(spec.visibility === 'workspace' ? 'agentsPlatform.builder.visibilityWorkspace' : 'agentsPlatform.builder.visibilityPrivate')}
            </Row>
          </div>
          <Row field="tools" label={t('agentsPlatform.builder.tools')} diff={changed}>
            <ul className="flex flex-wrap gap-1.5">
              {spec.tools.map((id) => (
                <li
                  key={id}
                  className={cn(
                    'rounded-full border bg-muted/40 px-2.5 py-0.5 text-xs',
                    addedTools.includes(id) && 'border-emerald-500/60 bg-emerald-500/10 text-emerald-700 dark:text-emerald-300',
                  )}
                  title={id}
                >
                  {addedTools.includes(id) && '+ '}
                  {toolName(id)}
                </li>
              ))}
              {removedTools.map((id) => (
                <li key={id} className="rounded-full border border-dashed px-2.5 py-0.5 text-xs text-muted-foreground line-through" title={id}>
                  {toolName(id)}
                </li>
              ))}
            </ul>
          </Row>
          <Row field="starters" label={t('agentsPlatform.builder.starters')} diff={changed}>
            <ul className="list-disc pl-4 text-sm">
              {spec.starters.map((s) => (
                <li key={s}>{s}</li>
              ))}
            </ul>
          </Row>
          <Row field="instructions" label={t('agentsPlatform.builder.instructions')} diff={changed}>
            <pre
              className={cn(
                'overflow-hidden rounded-lg border bg-muted/30 p-3 font-mono text-xs leading-relaxed whitespace-pre-wrap',
                !showInstructions && 'max-h-40',
              )}
            >
              {spec.instructions}
            </pre>
            <button type="button" onClick={() => setShowInstructions((v) => !v)} className="mt-1 inline-flex items-center gap-1 text-xs text-muted-foreground hover:text-foreground">
              <ChevronDown className={cn('size-3.5 transition-transform', showInstructions && 'rotate-180')} />
              {showInstructions ? t('agentsPlatform.architect.collapse') : t('agentsPlatform.architect.expand', { count: spec.instructions.length })}
            </button>
          </Row>
          {proposal.rationale && (
            <div className="rounded-xl border bg-muted/30 p-3 text-sm">
              <div className="mb-1 text-xs font-medium text-muted-foreground">{t('agentsPlatform.architect.rationale')}</div>
              <StreamingMarkdown content={proposal.rationale} isStreaming={false} />
            </div>
          )}
          {proposal.tests.length > 0 && (
            <div className="text-sm">
              <div className="mb-1 text-xs font-medium text-muted-foreground">{t('agentsPlatform.architect.tests')}</div>
              <ul className="list-disc pl-4">
                {proposal.tests.map((q) => (
                  <li key={q}>{q}</li>
                ))}
              </ul>
            </div>
          )}
        </div>
      </div>
      {!readOnly && (
        <div className="flex flex-wrap items-center gap-2 border-t bg-background p-3">
          <Button variant="outline" size="sm" onClick={onApply} disabled={applied}>
            {applied ? <Check /> : <ClipboardCheck />} {applied ? t('agentsPlatform.architect.applied') : t('agentsPlatform.architect.apply')}
          </Button>
          <Button size="sm" onClick={onSave} disabled={saving || saved}>
            {saving ? <Loader2 className="animate-spin" /> : <Wand2 />}
            {saved ? t('agentsPlatform.architect.upToDate') : editing ? t('agentsPlatform.architect.update') : t('agentsPlatform.architect.create')}
          </Button>
          {editing && (
            <Button variant="secondary" size="sm" onClick={onTest} className="ml-auto">
              <FlaskConical /> {t('agentsPlatform.architect.test')}
            </Button>
          )}
        </div>
      )}
    </div>
  );
}
