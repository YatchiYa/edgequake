/**
 * SPEC-143 — Observe markdown `[data-eq-page]` anchors; report active page.
 */

'use client';

import { useEffect, useRef } from 'react';

export interface UseMarkdownPageObserverOptions {
  /** Element that contains `[data-eq-page]` anchors (e.g. ContentRenderer root). */
  containerRef: React.RefObject<HTMLElement | null>;
  enabled: boolean;
  onPage: (page: number) => void;
  /** When set, scroll this page's `#eq-md-page-N` into view. */
  scrollToPage?: number | null;
  /** Skip scroll when driver is markdown itself. */
  skipScroll?: boolean;
}

function resolveScrollRoot(el: HTMLElement | null): HTMLElement | null {
  let p: HTMLElement | null = el?.parentElement ?? null;
  while (p) {
    const style = window.getComputedStyle(p);
    if (/(auto|scroll)/.test(style.overflowY)) return p;
    p = p.parentElement;
  }
  return null;
}

export function useMarkdownPageObserver({
  containerRef,
  enabled,
  onPage,
  scrollToPage,
  skipScroll = false,
}: UseMarkdownPageObserverOptions): void {
  const onPageRef = useRef(onPage);
  onPageRef.current = onPage;
  const lastReported = useRef<number | null>(null);

  // IntersectionObserver → active page
  useEffect(() => {
    if (!enabled) return;
    const container = containerRef.current;
    if (!container) return;
    const root = resolveScrollRoot(container) ?? container;

    const nodes = container.querySelectorAll<HTMLElement>('[data-eq-page]');
    if (nodes.length === 0) return;

    const ratios = new Map<number, number>();

    const io = new IntersectionObserver(
      (entries) => {
        for (const entry of entries) {
          const raw = (entry.target as HTMLElement).getAttribute('data-eq-page');
          const page = raw ? parseInt(raw, 10) : NaN;
          if (!Number.isFinite(page) || page < 1) continue;
          ratios.set(page, entry.isIntersecting ? entry.intersectionRatio : 0);
        }
        let bestPage = 0;
        let bestRatio = 0;
        for (const [p, r] of ratios) {
          if (r > bestRatio) {
            bestRatio = r;
            bestPage = p;
          }
        }
        if (bestPage >= 1 && bestPage !== lastReported.current) {
          lastReported.current = bestPage;
          onPageRef.current(bestPage);
        }
      },
      {
        root,
        rootMargin: '-10% 0px -55% 0px',
        threshold: [0, 0.1, 0.25, 0.5, 1],
      },
    );

    nodes.forEach((n) => io.observe(n));
    return () => io.disconnect();
  }, [containerRef, enabled]);

  // Programmatic scroll when PDF / external drives
  useEffect(() => {
    if (!enabled || skipScroll) return;
    if (scrollToPage == null || scrollToPage < 1) return;
    const container = containerRef.current;
    if (!container) return;
    const el =
      container.querySelector(`#eq-md-page-${scrollToPage}`) ??
      container.querySelector(`[data-eq-page="${scrollToPage}"]`);
    if (!el) return;
    el.scrollIntoView({ behavior: 'smooth', block: 'start' });
    lastReported.current = scrollToPage;
  }, [containerRef, enabled, scrollToPage, skipScroll]);
}
