/**
 * EdgeQuake Copilot UI state. Conversations are kept in memory per workspace
 * (never persisted: they may quote private document content).
 */
import { create } from "zustand";
import { persist } from "zustand/middleware";

export interface CopilotToolEvent {
  id: string;
  name: string;
  label: string;
  status: "running" | "done" | "error";
  detail?: string;
}

export interface CopilotProposal {
  id: string;
  title: string;
  rationale: string;
  operations: unknown[];
  descriptions: string[];
  state: "pending" | "approving" | "approved" | "rejected";
  results?: Array<{ ok: boolean; summary: string }>;
}

export interface CopilotMessage {
  id: string;
  role: "user" | "assistant";
  content: string;
  tools: CopilotToolEvent[];
  proposal?: CopilotProposal;
  error?: string;
  streaming?: boolean;
}

interface CopilotState {
  open: boolean;
  width: number;
  /** workspaceId → messages */
  threads: Record<string, CopilotMessage[]>;
  setOpen: (open: boolean) => void;
  toggle: () => void;
  setWidth: (width: number) => void;
  setThread: (workspaceId: string, update: (messages: CopilotMessage[]) => CopilotMessage[]) => void;
  clearThread: (workspaceId: string) => void;
}

export const useCopilotStore = create<CopilotState>()(
  persist(
    (set) => ({
      open: false,
      width: 400,
      threads: {},
      setOpen: (open) => set({ open }),
      toggle: () => set((s) => ({ open: !s.open })),
      setWidth: (width) => set({ width: Math.min(720, Math.max(340, Math.round(width))) }),
      setThread: (workspaceId, update) =>
        set((s) => ({ threads: { ...s.threads, [workspaceId]: update(s.threads[workspaceId] ?? []) } })),
      clearThread: (workspaceId) =>
        set((s) => {
          const threads = { ...s.threads };
          delete threads[workspaceId];
          return { threads };
        }),
    }),
    {
      name: "edgequake-copilot-ui",
      // Only layout preferences survive reloads.
      partialize: (s) => ({ open: s.open, width: s.width }),
    },
  ),
);
