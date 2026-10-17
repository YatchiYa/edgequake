'use client';

import { useAgentLookup } from '@/components/agents/agent-bits';
import { AgentsTopBar } from '@/components/agents/agents-shell';
import { ChatView } from '@/components/agents/chat-view';
import { Button } from '@/components/ui/button';
import { ARCHITECT_ID, useConversation } from '@/lib/agents/api';
import { Sparkles } from 'lucide-react';
import Link from 'next/link';
import { useParams } from 'next/navigation';
import { useTranslation } from 'react-i18next';

export default function ConversationPage() {
  const { t } = useTranslation();
  const { id } = useParams<{ id: string }>();
  const { data } = useConversation(id);
  const agent = useAgentLookup()(data?.agent_id ?? '');
  return (
    <>
      <AgentsTopBar
        actions={
          data?.agent_id === ARCHITECT_ID ? (
            <Button asChild size="sm" variant="outline" className="mr-1">
              <Link href={`/agents/new?mode=ai&design=${encodeURIComponent(id)}`}>
                <Sparkles /> {t('agentsPlatform.architect.openInBuilder')}
              </Link>
            </Button>
          ) : undefined
        }
      >
        {data && (
          <h1 className="min-w-0 truncate px-1 text-[15px]">
            <span className="font-medium">{data.title}</span>
            <span className="text-muted-foreground max-sm:hidden"> · {agent.icon} {agent.name}</span>
          </h1>
        )}
      </AgentsTopBar>
      <ChatView key={id} conversationId={id} />
    </>
  );
}
