/**
 * @module api-stream-client
 * @description SSE (Server-Sent Events) streaming fetch client.
 *
 * Extracted from `client.ts` (SPEC-017 LOC guard + SRP): this module owns only
 * the streaming transport — reading a fetch `Response.body` ReadableStream,
 * splitting SSE frames, and yielding parsed JSON or raw token events.
 *
 * SPEC-149: named-event frames + AbortSignal for authenticated PDF progress.
 *
 * @implements FEAT0770 - SSE streaming client
 * @implements SPEC-149 - Auth on every stream (LAW-149-10)
 */
import { getRuntimeApiBaseUrl } from "@/lib/runtime-config";
import { buildHeaders, handleErrorResponse } from "./client";

/** One SSE frame with optional event name (default `"message"`). */
export interface SSEFrame<T = unknown> {
  event: string;
  data: T;
}

/**
 * Streaming API client for SSE (Server-Sent Events) responses.
 *
 * Yields parsed `data:` payloads only (legacy chat/query callers).
 * Prefer {@link streamClientFrames} when the `event:` name matters.
 */
export async function* streamClient<T>(
  endpoint: string,
  options: RequestInit = {},
): AsyncGenerator<T, void, unknown> {
  for await (const frame of streamClientFrames<T>(endpoint, options)) {
    yield frame.data;
  }
}

/**
 * Streaming API client that preserves SSE event names.
 */
export async function* streamClientFrames<T>(
  endpoint: string,
  options: RequestInit = {},
): AsyncGenerator<SSEFrame<T>, void, unknown> {
  const url = endpoint.startsWith("http")
    ? endpoint
    : `${getRuntimeApiBaseUrl()}${endpoint}`;

  const config: RequestInit = {
    ...options,
    headers: buildHeaders(options.headers),
  };

  const response = await fetch(url, config);

  if (!response.ok) {
    throw await handleErrorResponse(response);
  }

  if (!response.body) {
    throw new Error("Response body is null");
  }

  const reader = response.body.getReader();
  const decoder = new TextDecoder();
  let buffer = "";

  try {
    while (true) {
      if (options.signal?.aborted) {
        break;
      }

      const { done, value } = await reader.read();

      if (done) {
        if (buffer.trim()) {
          const frame = parseSSEFrame(buffer);
          if (frame !== null) {
            yield frame as SSEFrame<T>;
          }
        }
        break;
      }

      buffer += decoder.decode(value, { stream: true });

      const events = buffer.split("\n\n");
      buffer = events.pop() || "";

      for (const event of events) {
        const frame = parseSSEFrame(event);
        if (frame !== null) {
          yield frame as SSEFrame<T>;
        }
      }
    }
  } finally {
    try {
      reader.releaseLock();
    } catch {
      /* already released */
    }
  }
}

export default streamClient;

/**
 * Parse a single SSE event block into `{ event, data }`.
 */
export function parseSSEFrame(event: string): SSEFrame<unknown> | null {
  const lines = event.split("\n");
  const dataChunks: string[] = [];
  let eventName = "message";

  for (const line of lines) {
    const trimmed = line.trim();
    if (trimmed.startsWith("event:")) {
      let name = trimmed.slice(6);
      if (name.startsWith(" ")) name = name.slice(1);
      if (name) eventName = name;
      continue;
    }
    if (trimmed.startsWith("data:")) {
      let content = trimmed.slice(5);
      if (content.startsWith(" ")) {
        content = content.slice(1);
      }
      if (content) {
        dataChunks.push(content);
      }
      continue;
    }
    if (trimmed.startsWith("id:") || trimmed.startsWith("retry:")) {
      continue;
    }
    if (trimmed && !trimmed.startsWith(":")) {
      dataChunks.push(trimmed);
    }
  }

  if (dataChunks.length === 0) {
    return null;
  }

  const data = dataChunks.join("");

  try {
    return { event: eventName, data: JSON.parse(data) };
  } catch {
    return { event: eventName, data: { type: "token", content: data } };
  }
}
