'use client';

import { AgentBuilder } from '@/components/agents/agent-builder';
import { useParams } from 'next/navigation';
import { Suspense } from 'react';

export default function EditAgentPage() {
  const { agentId } = useParams<{ agentId: string }>();
  return (
    <Suspense fallback={null}>
      <AgentBuilder key={agentId} agentId={agentId} />
    </Suspense>
  );
}
