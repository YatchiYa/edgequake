/**
 * Query mode presentation SSOT — labels, tooltips, icons.
 *
 * Backend modes: naive | local | global | hybrid | mix | bypass
 * (EdgeQuake `hybrid` = local+global+naive arms; LightRAG `hybrid` = local+global only.
 *  Tooltips describe EdgeQuake behavior.)
 */

import type { QueryMode } from "@/types/query";
import { QUERY_MODES } from "@/types/query";
import {
  GitBranch,
  Globe,
  Layers,
  MessageSquare,
  Target,
  Zap,
  type LucideIcon,
} from "lucide-react";

export interface QueryModeMeta {
  id: QueryMode;
  /** Short UX label shown on the toggle */
  label: string;
  /** API / docs name */
  apiName: QueryMode;
  /** Tooltip body — what it retrieves and when to use it */
  description: string;
  icon: LucideIcon;
  color: string;
  recommended?: boolean;
}

/**
 * All modes in UX order (focused → broad → graph hybrid → full mix → chunks → chat).
 * Must stay in sync with {@link QUERY_MODES}.
 */
export const QUERY_MODE_META: readonly QueryModeMeta[] = [
  {
    id: "local",
    label: "Focused",
    apiName: "local",
    description:
      "Entity-centric graph search: finds relevant entities and their nearby relationships and source chunks. Best for targeted questions about a known person, org, product, or concept.",
    icon: Target,
    color: "text-blue-500",
  },
  {
    id: "global",
    label: "Broad",
    apiName: "global",
    description:
      "Relationship-centric graph search: follows high-level themes and cross-document links. Best for summaries, trends, and “how things connect” questions.",
    icon: Globe,
    color: "text-green-500",
  },
  {
    id: "hybrid",
    label: "Linked",
    apiName: "hybrid",
    description:
      "Runs Focused and Broad together (and EdgeQuake also includes chunk retrieval), then merges results. Good when you want both entity detail and graph themes.",
    icon: GitBranch,
    color: "text-teal-500",
  },
  {
    id: "mix",
    label: "Smart",
    apiName: "mix",
    description:
      "Full blend (LightRAG mix): Focused + Broad + document chunks in parallel, fused with ranking (RRF). Always runs all three retrieval arms — richest context for most questions. Recommended default.",
    icon: Layers,
    color: "text-primary",
    recommended: true,
  },
  {
    id: "naive",
    label: "Chunks",
    apiName: "naive",
    description:
      "Document-chunk retrieval only (dense vectors + keyword/BM25 when enabled). No knowledge-graph walk. Fastest RAG path for exact wording, numbers, and quotes.",
    icon: Zap,
    color: "text-orange-500",
  },
  {
    id: "bypass",
    label: "Chat",
    apiName: "bypass",
    description:
      "Skips retrieval entirely. Sends the conversation and your question straight to the LLM — no documents or graph context. Use for general chat or when RAG is not needed.",
    icon: MessageSquare,
    color: "text-violet-500",
  },
] as const;

export function getQueryModeMeta(mode: QueryMode): QueryModeMeta {
  const found = QUERY_MODE_META.find((m) => m.id === mode);
  if (!found) {
    // Fallback should never hit if QUERY_MODES stays complete
    return QUERY_MODE_META[3]!; // mix
  }
  return found;
}

/** Dev/test invariant: every backend mode has UI meta. */
export function assertQueryModeMetaComplete(): boolean {
  return QUERY_MODES.every((mode) => QUERY_MODE_META.some((m) => m.id === mode));
}
