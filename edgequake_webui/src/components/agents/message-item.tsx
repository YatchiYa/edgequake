'use client';

import { StreamingMarkdown } from '@/components/query/markdown';
import { ARCHITECT_ID, useAgentTools, type Message, type ToolActivity } from '@/lib/agents/api';
import { Copy, Sparkles } from 'lucide-react';
import { memo } from 'react';
import { useTranslation } from 'react-i18next';
import { toast } from 'sonner';
import { AgentAvatar } from './agent-bits';
import { ArtifactCard } from './artifact-card';
import { ActivityLine, SourcesSection } from './run-activity';

function argsPreview(args: Record<string, unknown> | undefined): string {
  if (!args) return '';
  const s = Object.values(args)
    .filter((v) => typeof v === 'string' || typeof v === 'number')
    .join(' · ');
  return s.length > 48 ? `${s.slice(0, 47)}…` : s;
}

const isRouting = (tool: ToolActivity) => tool.name === 'transfer_to_agent';

interface MessageItemProps {
  message: Message;
  conversationId: string;
  streaming: boolean;
  author: { name: string; icon: string };
  /** Agent of the conversation (picks the waiting phrases). */
  agentId?: string;
  lookup: (id: string) => { name: string; icon: string };
  /** Step whose passages the Sources section shows (null = all). */
  sourceFocus?: string | null;
  onSourceFocus?: (messageId: string, step: string | null) => void;
}

export const MessageItem = memo(function MessageItem({
  message,
  conversationId,
  streaming,
  author,
  agentId,
  lookup,
  sourceFocus,
  onSourceFocus,
}: MessageItemProps) {
  const { t } = useTranslation();
  const { data: catalog } = useAgentTools();

  if (message.role === 'user') {
    return (
      <div className="flex justify-end">
        <div className="max-w-[85%] whitespace-pre-wrap break-words rounded-3xl bg-muted px-4 py-2.5 text-[15px] leading-relaxed">
          {message.content}
        </div>
      </div>
    );
  }

  const routing = message.tools.filter(isRouting);
  const work = message.tools.filter((tool) => !isRouting(tool));
  const steps = message.steps ?? [];
  // Hand-off target: the first other agent of this turn (tool args are not kept in history).
  const target = routing.length ? message.agents?.find((a) => a !== message.author) : undefined;
  const routedTo = target ? `${lookup(target).icon} ${lookup(target).name}` : routing.length ? t('agentsPlatform.tools.routed') : undefined;
  const tools = work.map((tool) => ({
    id: tool.id,
    label: catalog?.find((c) => c.id === tool.name)?.name ?? tool.name,
    detail: argsPreview(tool.args) || undefined,
    status: tool.status,
  }));
  const busy = work.length + steps.length > 0;
  const waiting =
    agentId === ARCHITECT_ID
      ? ['phaseAnalyze', 'phaseDesign', 'phaseWrite', 'phaseFinal'].map((k) => t(`agentsPlatform.architect.${k}`))
      : [t(busy ? 'agentsPlatform.run.synthesis' : 'agentsPlatform.chat.thinking')];
  const proposal = message.agent_spec?.spec;
  return (
    <div className="group flex gap-3 sm:gap-4">
      <AgentAvatar icon={author.icon} className="mt-0.5" />
      <div className="flex min-w-0 flex-1 flex-col gap-2">
        <div className="text-sm font-semibold leading-7">{author.name}</div>
        <ActivityLine
          steps={steps}
          tools={tools}
          routedTo={routedTo}
          streaming={streaming}
          answering={!!message.content}
          waiting={waiting}
          onShowSources={(step) => onSourceFocus?.(message.id, step)}
        />
        {message.content && (
          <div className="text-[15px] leading-relaxed">
            <StreamingMarkdown content={message.content} isStreaming={streaming} />
          </div>
        )}
        {proposal && (
          <div className="flex max-w-md items-start gap-3 rounded-2xl border border-primary/25 bg-gradient-to-br from-primary/10 via-card to-card p-3 shadow-sm animate-in fade-in-0 slide-in-from-bottom-1">
            <AgentAvatar icon={proposal.icon} className="size-10 text-xl" />
            <div className="min-w-0 flex-1">
              <div className="flex items-center gap-1 text-xs font-medium text-primary">
                <Sparkles className="size-3.5" aria-hidden="true" /> {t('agentsPlatform.architect.proposedLabel')}
              </div>
              <div className="truncate font-semibold">{proposal.name}</div>
              {proposal.description && <p className="line-clamp-2 text-xs text-muted-foreground">{proposal.description}</p>}
              <div className="mt-1 text-xs text-muted-foreground">
                {t('agentsPlatform.architect.proposedMeta', { tools: proposal.tools.length, starters: proposal.starters.length })}
              </div>
            </div>
          </div>
        )}
        {!!message.sources?.length && !streaming && (
          <SourcesSection
            messageId={message.id}
            sources={message.sources}
            focusStep={sourceFocus}
            onClearFocus={() => onSourceFocus?.(message.id, null)}
          />
        )}
        {message.artifacts.length > 0 && (
          <div className="flex flex-col gap-3">
            {message.artifacts.map((a) => (
              <div key={`${a.name}@${a.version}`} id={`artifact-${message.id}-${a.name}`} className="scroll-mt-6">
                <ArtifactCard conversationId={conversationId} artifact={a} />
              </div>
            ))}
          </div>
        )}
        {!streaming && message.content && (
          <div className="-ml-1.5 flex opacity-0 transition-opacity group-hover:opacity-100 focus-within:opacity-100 max-md:opacity-100">
            <button
              type="button"
              onClick={() => {
                void navigator.clipboard.writeText(message.content);
                toast.success(t('agentsPlatform.chat.copied'));
              }}
              className="flex size-8 items-center justify-center rounded-lg text-muted-foreground hover:bg-accent hover:text-foreground"
              aria-label={t('agentsPlatform.chat.copy')}
              title={t('agentsPlatform.chat.copy')}
            >
              <Copy className="size-4" />
            </button>
          </div>
        )}
      </div>
    </div>
  );
});
