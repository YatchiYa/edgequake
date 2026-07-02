import { ApiRequestError } from "@/lib/api/client";

/** Detect stale or missing conversation errors (404 / not found). */
export function isConversationNotFoundError(error: unknown): boolean {
  if (error instanceof ApiRequestError && error.status === 404) {
    return true;
  }
  if (error instanceof Error) {
    const message = error.message.toLowerCase();
    return message.includes("not found") && message.includes("conversation");
  }
  return false;
}

/** SSE / API errors when the conversation row was deleted mid-stream. */
export function isConversationGoneError(error: unknown): boolean {
  if (error instanceof Error) {
    const message = error.message.toLowerCase();
    return (
      message.includes("conversation no longer exists") ||
      message.includes("conversation expired")
    );
  }
  return false;
}

export function isConversationGoneStreamCode(code: string | undefined): boolean {
  return code === "CONVERSATION_GONE";
}

/** Optimistic/local messages are never persisted server-side. */
export function isServerPersistedMessageId(id: string): boolean {
  return !id.startsWith("optimistic-");
}
