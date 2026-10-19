'use client';

import { Card, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import type { LucideIcon } from 'lucide-react';

/** Centered card for the public, token-gated pages (invitation, access link). */
export function LinkShell({ children, testId }: { children: React.ReactNode; testId: string }) {
  return (
    <div className="flex h-full min-h-0 items-center justify-center overflow-y-auto bg-background p-4">
      <Card className="w-full max-w-md" data-testid={testId}>
        {children}
      </Card>
    </div>
  );
}

export function LinkHeader({
  icon: Icon,
  title,
  description,
}: {
  icon: LucideIcon;
  title: React.ReactNode;
  description?: React.ReactNode;
}) {
  return (
    <CardHeader className="text-center">
      <div className="mx-auto mb-3 flex size-12 items-center justify-center rounded-full bg-primary/10">
        <Icon className="size-6 text-primary" aria-hidden />
      </div>
      <CardTitle className="text-xl">{title}</CardTitle>
      {description && <CardDescription>{description}</CardDescription>}
    </CardHeader>
  );
}
