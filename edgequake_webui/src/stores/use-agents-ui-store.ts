/** SPEC-165 — per-browser agents UI preferences (pinned agents, sidebar state). */
'use client';

import { create } from 'zustand';
import { persist } from 'zustand/middleware';

interface AgentsUiStore {
  pinned: string[];
  sidebarCollapsed: boolean;
  /** Mobile drawer (not persisted). */
  mobileOpen: boolean;
  togglePin: (agentId: string) => void;
  setSidebarCollapsed: (collapsed: boolean) => void;
  setMobileOpen: (open: boolean) => void;
}

export const useAgentsUiStore = create<AgentsUiStore>()(
  persist(
    (set) => ({
      pinned: [],
      sidebarCollapsed: false,
      mobileOpen: false,
      togglePin: (id) => set((s) => ({ pinned: s.pinned.includes(id) ? s.pinned.filter((p) => p !== id) : [id, ...s.pinned] })),
      setSidebarCollapsed: (sidebarCollapsed) => set({ sidebarCollapsed }),
      setMobileOpen: (mobileOpen) => set({ mobileOpen }),
    }),
    { name: 'edgequake-agents-ui', partialize: ({ pinned, sidebarCollapsed }) => ({ pinned, sidebarCollapsed }) },
  ),
);
