'use client';

import { calculateLayoutPositions } from '@/lib/graph/layouts';
import { Button } from '@/components/ui/button';
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';
import { useGraphStore } from '@/stores/use-graph-store';
import { useSettingsStore } from '@/stores/use-settings-store';
import { RotateCw } from 'lucide-react';
import { useCallback, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { animateNodes } from 'sigma/utils';

interface LayoutControllerProps {
  className?: string;
}

/**
 * Layout Controller Component
 *
 * Provides instant layout application button for the graph.
 * Applies the selected layout algorithm with smooth animation.
 */
export function LayoutController({ className }: LayoutControllerProps) {
  const { t } = useTranslation();
  const sigmaInstance = useGraphStore((s) => s.sigmaInstance);
  const { graphSettings } = useSettingsStore();

  const [isApplying, setIsApplying] = useState(false);

  const layout = graphSettings.layout ?? 'force';

  /**
   * Apply layout instantly (one-shot, with smooth animation)
   */
  const applyLayout = useCallback(() => {
    if (!sigmaInstance) return;

    const graph = sigmaInstance.getGraph();
    if (!graph || graph.order === 0) return;

    setIsApplying(true);

    try {
      const newPositions = calculateLayoutPositions(graph, layout, 'interactive');

      // Animate to new positions
      animateNodes(graph, newPositions, {
        duration: 300,
        easing: 'quadraticInOut'
      });
    } catch (error) {
      console.error('Error applying layout:', error);
    } finally {
      setTimeout(() => setIsApplying(false), 300);
    }
  }, [sigmaInstance, layout]);

  // Don't render if no sigma instance
  if (!sigmaInstance) {
    return null;
  }

  return (
    <div className={`flex items-center gap-1 ${className ?? ''}`}>
      {/* Apply layout instantly button */}
      <Tooltip>
        <TooltipTrigger asChild>
          <Button
            variant="ghost"
            size="icon"
            onClick={applyLayout}
            className="h-8 w-8"
            disabled={isApplying}
            aria-label={t('graph.layout.apply', 'Apply Layout')}
          >
            <RotateCw className={`h-4 w-4 ${isApplying ? 'animate-spin' : ''}`} />
          </Button>
        </TooltipTrigger>
        <TooltipContent>
          {t('graph.layout.applyTooltip', 'Apply layout instantly')}
        </TooltipContent>
      </Tooltip>
    </div>
  );
}

export default LayoutController;
