/**
 * Empty-state copy for Query UI — Chat (bypass) vs RAG modes.
 * Pure SSOT so UI and tests share the same strings (DRY).
 */

import type { QueryMode } from "@/types/query";

export interface QueryEmptyCopy {
  title: string;
  description: string;
  suggestions: string[];
}

const RAG_COPY: QueryEmptyCopy = {
  title: "Ask about your knowledge graph",
  description:
    "I can help you explore entities, find connections, and uncover insights from your documents.",
  suggestions: [
    "What are the main entities in my knowledge graph?",
    "Summarize the key relationships between documents",
    "Find connections between people and organizations",
    "What topics are covered in my documents?",
  ],
};

const CHAT_COPY: QueryEmptyCopy = {
  title: "Chat with your assistant",
  description:
    "General conversation without document or graph retrieval. Follow-ups use recent chat history.",
  suggestions: [
    "Help me brainstorm ideas",
    "Explain a concept in simple terms",
    "What should I consider before starting a project?",
    "Summarize the trade-offs of two approaches",
  ],
};

export function getQueryEmptyCopy(mode: QueryMode = "mix"): QueryEmptyCopy {
  return mode === "bypass" ? CHAT_COPY : RAG_COPY;
}

export function isChatQueryMode(mode: QueryMode): boolean {
  return mode === "bypass";
}
