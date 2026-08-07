/**
 * Pure presentation for PDF progress load errors (SRP / testable without DOM).
 *
 * Nested Active-run meters must not paint a red "pipeline died" card when only
 * the PDF progress row is gone — list SSOT / reconcile owns truth.
 */

export type PdfProgressErrorView =
  | { kind: "reconnecting"; message: string }
  | { kind: "nested_ended"; message: string }
  | { kind: "terminal"; message: string; tone: "danger" };

export function presentPdfProgressError(
  errorMessage: string,
  opts: {
    nested?: boolean;
    isLoading?: boolean;
    isPolling?: boolean;
  } = {},
): PdfProgressErrorView {
  const msg = errorMessage ?? "";
  const isTaskGone = /task not found/i.test(msg);
  const isProgressMiss = /progress not found/i.test(msg);

  if (isProgressMiss && (opts.isLoading || opts.isPolling)) {
    return { kind: "reconnecting", message: "Reconnecting to progress…" };
  }

  if (opts.nested && isTaskGone) {
    return { kind: "nested_ended", message: "Progress tracking ended" };
  }

  if (isTaskGone) {
    return {
      kind: "terminal",
      tone: "danger",
      message:
        "Task ended — progress is no longer available. Refresh the document list or retry.",
    };
  }

  if (isProgressMiss) {
    return {
      kind: "terminal",
      tone: "danger",
      message:
        "Progress unavailable — upload may have completed or the task ended.",
    };
  }

  return {
    kind: "terminal",
    tone: "danger",
    message: `Failed to load progress: ${msg}`,
  };
}
