/**
 * SPEC-143 — Shared active-page controller for PDF ↔ Markdown sync.
 *
 * SRP: owns activePage + syncEnabled + driver lock only.
 * Does not render panes or write the URL (parent owns router).
 */

'use client';

import { useCallback, useRef, useState } from 'react';

export type PageSyncDriver = 'none' | 'pdf' | 'md' | 'external';

export interface UsePageSyncControllerOptions {
  /** Seed page (1-indexed). */
  initialPage?: number;
  /** Lock window after a driver update (ms). Default 200. */
  settleMs?: number;
  /** Sync enabled by default (side-by-side). Default true. */
  initialSyncEnabled?: boolean;
}

export interface PageSyncController {
  activePage: number;
  syncEnabled: boolean;
  driver: PageSyncDriver;
  toggleSync: () => void;
  setSyncEnabled: (on: boolean) => void;
  setPageFromPdf: (page: number) => void;
  setPageFromMd: (page: number) => void;
  setPageFromExternal: (page: number) => void;
}

function clampPage(page: number): number {
  if (!Number.isFinite(page)) return 1;
  return Math.max(1, Math.floor(page));
}

export function usePageSyncController(
  options: UsePageSyncControllerOptions = {},
): PageSyncController {
  const settleMs = options.settleMs ?? 200;
  const [activePage, setActivePage] = useState(() =>
    clampPage(options.initialPage ?? 1),
  );
  const [syncEnabled, setSyncEnabled] = useState(options.initialSyncEnabled ?? true);
  const [driver, setDriver] = useState<PageSyncDriver>('none');
  const lockUntilRef = useRef(0);
  const driverRef = useRef<PageSyncDriver>('none');

  const apply = useCallback(
    (source: PageSyncDriver, page: number) => {
      const next = clampPage(page);
      const now = Date.now();
      const locked = now < lockUntilRef.current;
      const currentDriver = driverRef.current;
      if (locked && currentDriver !== 'none' && currentDriver !== source) {
        return;
      }
      setActivePage((prev) => (prev === next ? prev : next));
      driverRef.current = source;
      setDriver(source);
      lockUntilRef.current = now + settleMs;
    },
    [settleMs],
  );

  const setPageFromPdf = useCallback(
    (page: number) => apply('pdf', page),
    [apply],
  );
  const setPageFromMd = useCallback(
    (page: number) => {
      if (!syncEnabled) return;
      apply('md', page);
    },
    [apply, syncEnabled],
  );
  const setPageFromExternal = useCallback(
    (page: number) => apply('external', page),
    [apply],
  );

  const toggleSync = useCallback(() => {
    setSyncEnabled((v) => !v);
  }, []);

  return {
    activePage,
    syncEnabled,
    driver,
    toggleSync,
    setSyncEnabled,
    setPageFromPdf,
    setPageFromMd,
    setPageFromExternal,
  };
}
