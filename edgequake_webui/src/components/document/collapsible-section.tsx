// Collapsible section component with smooth animations
'use client';

import { cn } from '@/lib/utils';
import { ChevronDown } from 'lucide-react';
import { useEffect, useState } from 'react';

interface CollapsibleSectionProps {
  title: string;
  icon?: React.ReactNode;
  children: React.ReactNode;
  defaultOpen?: boolean;
  /** Force open (e.g. citation deeplink selected a chunk). */
  forceOpen?: boolean;
  testId?: string;
}

export function CollapsibleSection({
  title,
  icon,
  children,
  defaultOpen = false,
  forceOpen = false,
  testId,
}: CollapsibleSectionProps) {
  const [isOpen, setIsOpen] = useState(defaultOpen || forceOpen);

  useEffect(() => {
    if (forceOpen) setIsOpen(true);
  }, [forceOpen]);

  return (
    <div className="border rounded-md bg-card overflow-hidden" data-testid={testId}>
      <button
        onClick={() => setIsOpen(!isOpen)}
        className={cn(
          'w-full px-3 py-2 flex items-center justify-between',
          'hover:bg-accent/40 transition-colors',
          'text-xs font-medium'
        )}
      >
        <div className="flex items-center gap-1.5">
          {icon && <div className="text-muted-foreground">{icon}</div>}
          <span>{title}</span>
        </div>
        <ChevronDown
          className={cn(
            'h-3.5 w-3.5 text-muted-foreground transition-transform duration-200',
            isOpen && 'rotate-180'
          )}
        />
      </button>

      <div
        className={cn(
          'grid transition-[grid-template-rows] duration-300 ease-in-out',
          isOpen ? 'grid-rows-[1fr] opacity-100' : 'grid-rows-[0fr] opacity-0',
        )}
      >
        <div className="overflow-hidden">
          <div className="px-3 pb-3 pt-1.5">
            {children}
          </div>
        </div>
      </div>
    </div>
  );
}
