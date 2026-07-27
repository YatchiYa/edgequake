/**
 * SPEC-048: IngestionRunView — single projection for banner / row / pill / stepper.
 * DIP: UI depends on this model, not raw KV / WS shapes.
 */

import type { Document } from "@/types";
import type { IngestionProgress, IngestionStage } from "@/types/ingestion";
import {
  getDocumentDisplayStatus,
  isProcessingStatus,
} from "@/components/documents/status-badge";
import { bareDocumentId } from "@/lib/documents/reprocess-cache";
import {
  isOrphanAdmissionShell,
  isWaitingStatus,
} from "./pipeline-document-state";
import {
  loadCancelledFromStage,
  rememberCancelledFromStage,
} from "./cancelled-active-run-dismiss";

export { bareDocumentId };

export type IngestionRunMode = "full" | "entities" | "merge";

export type IngestionRunStage =
  | IngestionStage
  | "queued"
  | "cleaning"
  | "stopping"
  | "cancelled";

export type IngestionCountUnit = "pages" | "chunks" | "entities" | "relationships" | "figures";

export interface IngestionRunCounts {
  current: number;
  total: number;
  unit: IngestionCountUnit;
}

/** Stage lifecycle status — cancel is first-class, never overloaded as failed. */
export type IngestionRunStageStatus =
  | "pending"
  | "active"
  | "complete"
  | "failed"
  | "skipped"
  | "stopping"
  | "cancelled";

export interface IngestionRunView {
  documentId: string;
  trackId: string | null;
  filename: string;
  sourceType: "pdf" | "markdown" | "text" | "image" | "unknown";
  stage: IngestionRunStage;
  stageStatus: IngestionRunStageStatus;
  message: string;
  counts?: IngestionRunCounts;
  progress01?: number;
  mode?: IngestionRunMode;
  costUsd?: number;
  updatedAt?: string;
  /**
   * Last non-terminal pipeline stage when cancel stopped the run.
   * Used by the timeline to freeze honest progress (INV-10).
   */
  cancelledAtStage?: IngestionRunStage;
}

const STAGE_LABELS: Record<string, string> = {
  cleaning: "Cleaning",
  queued: "Queued",
  uploading: "Uploading",
  converting: "Converting",
  preprocessing: "Preprocessing",
  chunking: "Chunking",
  extracting: "Extracting Entities",
  gleaning: "Gleaning",
  merging: "Merging Graph",
  summarizing: "Summarizing",
  embedding: "Generating Embeddings",
  storing: "Storing",
  completed: "Completed",
  failed: "Failed",
  pending: "Queued",
  indexing: "Storing",
  processing: "Preprocessing",
  stopping: "Stopping…",
  cancelled: "Cancelled",
};

/** Server UnifiedStage order (+ admission cleaning → queued). */
export const SERVER_STAGE_ORDER: IngestionRunStage[] = [
  "cleaning",
  "queued",
  "uploading",
  "converting",
  "preprocessing",
  "chunking",
  "extracting",
  "gleaning",
  "merging",
  "summarizing",
  "embedding",
  "storing",
  "completed",
];

export function stageDisplayName(
  stage: string,
  sourceType?: IngestionRunView["sourceType"],
): string {
  const key = stage.toLowerCase();
  // SPEC-086 ops: "Converting PDF" only for PDF; never for MD/text.
  if (key === "converting") {
    return sourceType === "pdf" ? "Converting PDF" : "Converting";
  }
  return STAGE_LABELS[key] ?? stage;
}

export function normalizeRunStage(
  currentStage?: string | null,
  status?: string | null,
): IngestionRunStage {
  const raw = (currentStage || status || "uploading").toLowerCase();
  if (raw === "stopping") return "stopping";
  if (raw === "cancelled") return "cancelled";
  if (raw === "pending") return "queued";
  if (raw === "indexing") return "storing";
  if (raw === "processing") return "preprocessing";
  return raw as IngestionRunStage;
}

export function parseCountsFromMessage(
  message: string,
): IngestionRunCounts | undefined {
  const lower = message.toLowerCase();
  const unit: IngestionCountUnit = lower.includes("chunk")
    ? "chunks"
    : lower.includes("figure") || lower.includes("chart")
      ? "figures"
      : lower.includes("page")
        ? "pages"
        : lower.includes("relat")
          ? "relationships"
          : lower.includes("entit")
            ? "entities"
            : "chunks";
  const m = message.match(/(\d+)\s*\/\s*(\d+)/);
  if (!m) return undefined;
  const current = Number(m[1]);
  const total = Number(m[2]);
  if (!total) return undefined;
  return { current, total, unit };
}

const ACTIVE_PIPELINE_STAGES = new Set([
  "uploading",
  "converting",
  "preprocessing",
  "chunking",
  "extracting",
  "gleaning",
  "merging",
  "summarizing",
  "embedding",
  "storing",
  "indexing",
  "processing",
]);

/**
 * Resolve Stopping / Cancelled terminals from display status or stage.
 * Shared by list-path and progress-path builders (INV-05 one status story).
 */
export function resolveRunTerminal(
  displayStatus: string,
  stage?: IngestionRunStage,
): { stage: "stopping" | "cancelled"; stageStatus: "stopping" | "cancelled" } | null {
  const s = displayStatus.toLowerCase();
  if (s === "stopping" || stage === "stopping") {
    return { stage: "stopping", stageStatus: "stopping" };
  }
  if (s === "cancelled" || stage === "cancelled") {
    return { stage: "cancelled", stageStatus: "cancelled" };
  }
  return null;
}

/** Infer last honest pipeline stage before a cancel terminal. */
export function resolveCancelledAtStage(
  currentStage?: string | null,
  status?: string | null,
): IngestionRunStage | undefined {
  const raw = (currentStage || status || "").toLowerCase();
  if (!raw || raw === "cancelled" || raw === "stopping" || raw === "failed") {
    return undefined;
  }
  const normalized = normalizeRunStage(currentStage, status);
  if (
    ACTIVE_PIPELINE_STAGES.has(normalized) ||
    normalized === "queued" ||
    normalized === "cleaning"
  ) {
    return normalized;
  }
  return undefined;
}

/**
 * Resolve run stage status.
 * Prefer fine-grained `current_stage` over coarse `status=pending` —
 * otherwise converting/chunking is mislabeled Queued (SPEC-048).
 */
export function stageStatusFor(
  stage: IngestionRunStage,
  status: string,
): IngestionRunView["stageStatus"] {
  const terminal = resolveRunTerminal(status, stage);
  if (terminal) return terminal.stageStatus;
  const s = status.toLowerCase();
  if (s === "failed" || stage === "failed") return "failed";
  if (s === "completed" || stage === "completed") return "complete";
  // Admission only when the stage itself is cleaning/queued
  if (stage === "cleaning" || stage === "queued") return "pending";
  // Fine stage already past admission → active even if coarse status lags as pending
  if (ACTIVE_PIPELINE_STAGES.has(stage)) return "active";
  if (isWaitingStatus(s as never)) return "pending";
  return "active";
}

function sourceTypeOf(doc: Document): IngestionRunView["sourceType"] {
  const t = (doc.source_type || "").toLowerCase();
  if (t === "pdf" || t === "markdown" || t === "text" || t === "image") {
    return t;
  }
  // SPEC-086: infer from filename when taxonomy missing (skip Converting for .md).
  const name = (doc.file_name || doc.title || "").toLowerCase();
  if (name.endsWith(".md") || name.endsWith(".markdown")) return "markdown";
  if (name.endsWith(".pdf")) return "pdf";
  if (
    name.endsWith(".png") ||
    name.endsWith(".jpg") ||
    name.endsWith(".jpeg") ||
    name.endsWith(".webp") ||
    name.endsWith(".gif")
  ) {
    return "image";
  }
  if (name.endsWith(".txt") || name.endsWith(".text")) return "text";
  return "unknown";
}

export type BuildRunViewOpts = {
  hasQueueCoverage?: boolean;
};

/** Build one run view from a document list row (KV poll SSOT). */
export function buildIngestionRunView(
  doc: Document,
  opts?: BuildRunViewOpts,
): IngestionRunView | null {
  const status = getDocumentDisplayStatus(doc);

  // SPEC-050: delete is a terminal operation — feedback zone owns progress,
  // not the ingest ActiveRuns stepper.
  if (status === "deleting") {
    return null;
  }

  // SPEC-057 / 086 ops: Cancel → Stopping… then Cancelled on ActiveRuns.
  // Cancel is first-class — never encoded as stageStatus=failed (INV-05).
  const cancelTerminal = resolveRunTerminal(status);
  if (cancelTerminal) {
    // Prefer durable KV cancelled_from_stage, then live stage, then session cache.
    const fromApi = resolveCancelledAtStage(
      doc.cancelled_from_stage,
      doc.cancelled_from_stage,
    );
    const fromLive = resolveCancelledAtStage(doc.current_stage, doc.status);
    const fromSession = resolveCancelledAtStage(
      loadCancelledFromStage(doc.id),
      loadCancelledFromStage(doc.id),
    );
    const freezeStage = fromApi ?? fromLive ?? fromSession;
    if (freezeStage) {
      rememberCancelledFromStage(doc.id, freezeStage);
    }
    return {
      documentId: doc.id,
      trackId: doc.track_id ?? null,
      filename: doc.file_name || doc.title || doc.id,
      sourceType: sourceTypeOf(doc),
      stage: cancelTerminal.stage,
      stageStatus: cancelTerminal.stageStatus,
      message:
        (doc.stage_message && doc.stage_message.trim()) ||
        stageDisplayName(cancelTerminal.stage),
      // Honest freeze: do not carry a near-100% stage_progress as "alive".
      progress01: undefined,
      updatedAt: doc.updated_at,
      cancelledAtStage: freezeStage,
    };
  }

  const isLive =
    isProcessingStatus(status) ||
    isWaitingStatus(status) ||
    Boolean(doc.track_id && doc.current_stage && doc.current_stage !== "completed");

  // SPEC-048: clear completed from live run chrome — only project failed for attention
  if (status === "completed" || status === "indexed") {
    return null;
  }

  if (!isLive && status !== "failed") {
    return null;
  }

  const orphanShell = isOrphanAdmissionShell(doc, Date.now(), {
    hasQueueCoverage: opts?.hasQueueCoverage,
  });
  const stage = orphanShell
    ? ("failed" as IngestionRunStage)
    : normalizeRunStage(doc.current_stage, doc.status);
  // Prefer display status (current_stage) so coarse status=pending cannot
  // force Queued while converting/chunking is already underway.
  const displayStatus = orphanShell ? "failed" : String(status);
  const message = orphanShell
    ? // Prefer re-upload guidance; prefix so ActiveRuns "Needs attention"
      // is clearly a prior shell, not a second card for the current PDF.
      (() => {
        const raw =
          /please re-upload|upload interrupted|orphaned staging/i.test(
            doc.stage_message || "",
          )
            ? (doc.stage_message as string).trim()
            : "please re-upload the document.";
        if (/prior interrupted upload/i.test(raw)) return raw;
        return `Prior interrupted upload — ${raw.replace(/^Upload interrupted[^—]*—?\s*/i, "")}`;
      })()
    : (doc.stage_message && doc.stage_message.trim()) ||
      stageDisplayName(stage);
  const counts = orphanShell ? undefined : parseCountsFromMessage(message);
  const progress01 = orphanShell
    ? 0
    : typeof doc.stage_progress === "number"
      ? doc.stage_progress
      : undefined;

  const modeRaw = (doc.reprocess_mode || "").toLowerCase();
  const mode: IngestionRunMode | undefined =
    modeRaw === "full" || modeRaw === "entities" || modeRaw === "merge"
      ? modeRaw
      : undefined;

  return {
    documentId: doc.id,
    trackId: doc.track_id ?? null,
    filename: doc.file_name || doc.title || doc.id,
    sourceType: sourceTypeOf(doc),
    stage,
    stageStatus: stageStatusFor(stage, displayStatus),
    message,
    counts,
    progress01,
    mode,
    costUsd: doc.cost_usd,
    updatedAt: doc.updated_at,
  };
}

/**
 * SPEC-086: build run view from live IngestionProgress (upload panel / poll+WS).
 */
export function buildIngestionRunViewFromProgress(
  progress: IngestionProgress,
  opts: {
    sourceType: IngestionRunView["sourceType"];
    filename?: string;
    mode?: IngestionRunMode;
  },
): IngestionRunView {
  const rawStage = normalizeRunStage(
    progress.progress?.current_stage,
    progress.status,
  );
  const cancelTerminal = resolveRunTerminal(String(progress.status), rawStage);
  const stage = cancelTerminal?.stage ?? rawStage;
  const message =
    (progress.progress?.latest_message &&
      progress.progress.latest_message.trim()) ||
    stageDisplayName(stage);
  const counts = cancelTerminal ? undefined : parseCountsFromMessage(message);
  const pct = progress.progress?.completion_percentage ?? progress.overall_progress;
  const progress01 = cancelTerminal
    ? undefined
    : typeof pct === "number"
      ? pct > 1
        ? pct / 100
        : pct
      : undefined;
  const cancelledAtStage = cancelTerminal
    ? resolveCancelledAtStage(
        progress.progress?.current_stage,
        progress.status,
      ) ??
      (ACTIVE_PIPELINE_STAGES.has(rawStage) ||
      rawStage === "queued" ||
      rawStage === "cleaning"
        ? rawStage
        : undefined) ??
      resolveCancelledAtStage(
        loadCancelledFromStage(progress.document_id),
        loadCancelledFromStage(progress.document_id),
      )
    : undefined;
  if (cancelledAtStage) {
    rememberCancelledFromStage(progress.document_id, cancelledAtStage);
  }

  return {
    documentId: progress.document_id,
    trackId: progress.track_id,
    filename: opts.filename || progress.document_name || progress.document_id,
    sourceType: opts.sourceType,
    stage,
    stageStatus: cancelTerminal
      ? cancelTerminal.stageStatus
      : stageStatusFor(stage, String(progress.status)),
    message,
    counts,
    progress01,
    mode: opts.mode,
    updatedAt: progress.updated_at,
    cancelledAtStage,
  };
}

function runStageRank(stage: string): number {
  const normalized = normalizeRunStage(stage, stage);
  const idx = SERVER_STAGE_ORDER.indexOf(normalized);
  return idx < 0 ? -1 : idx;
}

/** Carry freeze stage across Stopping→Cancelled merges (INV-10). */
function withPreservedCancelFreeze(
  winner: IngestionRunView,
  other: IngestionRunView,
): IngestionRunView {
  if (
    (winner.stageStatus === "cancelled" ||
      winner.stageStatus === "stopping") &&
    !winner.cancelledAtStage &&
    other.cancelledAtStage
  ) {
    return { ...winner, cancelledAtStage: other.cancelledAtStage };
  }
  // Stopping→Cancelled: keep prior freeze even if winner has a weaker one.
  if (
    winner.stageStatus === "cancelled" &&
    other.stageStatus === "stopping" &&
    other.cancelledAtStage
  ) {
    return {
      ...winner,
      cancelledAtStage: winner.cancelledAtStage ?? other.cancelledAtStage,
    };
  }
  return winner;
}

function preferRunView(
  a: IngestionRunView,
  b: IngestionRunView,
): IngestionRunView {
  // Terminal cancel / stopping always wins (INV-03 / INV-05).
  if (b.stageStatus === "cancelled" || b.stage === "cancelled") {
    return withPreservedCancelFreeze(b, a);
  }
  if (a.stageStatus === "cancelled" || a.stage === "cancelled") {
    return withPreservedCancelFreeze(a, b);
  }
  if (b.stageStatus === "stopping" || b.stage === "stopping") {
    return withPreservedCancelFreeze(b, a);
  }
  if (a.stageStatus === "stopping" || a.stage === "stopping") {
    return withPreservedCancelFreeze(a, b);
  }

  const ra = runStageRank(a.stage);
  const rb = runStageRank(b.stage);
  if (rb > ra) return b;
  if (ra > rb) return a;
  // Prefer active over pending/failed at same stage.
  if (b.stageStatus === "active" && a.stageStatus !== "active") return b;
  if (a.stageStatus === "active" && b.stageStatus !== "active") return a;
  return b;
}

/**
 * Project documents → ActiveRuns map.
 * Dedupe by bare document id / track_id (defense against staging: id drift).
 */
export function buildIngestionRunViews(
  documents: Document[] | undefined,
  opts?: BuildRunViewOpts,
): Map<string, IngestionRunView> {
  const map = new Map<string, IngestionRunView>();
  for (const doc of documents ?? []) {
    const view = buildIngestionRunView(doc, opts);
    if (!view) continue;

    const bareId = bareDocumentId(doc.id);
    let key = bareId;
    if (view.trackId) {
      for (const [existingKey, existing] of map) {
        if (existing.trackId && existing.trackId === view.trackId) {
          key = existingKey;
          break;
        }
      }
    }

    const normalized: IngestionRunView = {
      ...view,
      documentId: bareId,
    };
    const existing = map.get(key);
    if (!existing) {
      map.set(key, normalized);
      continue;
    }
    map.set(key, preferRunView(existing, normalized));
  }
  return map;
}

/** Primary active run for banner (first working, else first queued). */
export function selectPrimaryRun(
  runs: Map<string, IngestionRunView>,
): IngestionRunView | null {
  const list = [...runs.values()];
  const working = list.find(
    (r) =>
      (r.stageStatus === "active" || r.stageStatus === "stopping") &&
      r.stage !== "completed",
  );
  if (working) return working;
  return list.find((r) => r.stageStatus === "pending") ?? null;
}

export function formatRunHeadline(run: IngestionRunView): string {
  const stage = stageDisplayName(run.stage);
  if (run.counts) {
    return `${stage} · ${run.counts.current}/${run.counts.total} ${run.counts.unit}`;
  }
  return `${stage} · ${run.filename}`;
}
