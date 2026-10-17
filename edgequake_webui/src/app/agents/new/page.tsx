'use client';

import { AgentBuilder } from '@/components/agents/agent-builder';
import { Suspense } from 'react';

export default function NewAgentPage() {
  return (
    <Suspense fallback={null}>
      <AgentBuilder />
    </Suspense>
  );
}
