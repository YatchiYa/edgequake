/**
 * SSE (Server-Sent Events) streaming utilities.
 *
 * WHY: SSE is used for query streaming and chat streaming in EdgeQuake.
 * This parser handles the SSE wire format (data: lines, [DONE] sentinel).
 *
 * @module streaming/sse
 */

/**
 * Parse an SSE response body into an async iterable of typed events.
 *
 * Handles:
 * - `data: {...json...}` lines → parsed as T
 * - `data: [DONE]` → ends iteration
 * - Empty lines / comments → skipped
 *
 * @param response - Fetch Response with streaming body
 * @param parser - Function to parse raw data string into T (return null to skip)
 * @param signal - Optional AbortSignal for cancellation
 */
export async function* parseSSEStream<T>(
  response: Response,
  parser: (raw: string) => T | null,
  signal?: AbortSignal,
): AsyncGenerator<T> {
  for await (const data of parseSSEData(response, signal)) {
    const parsed = parser(data);
    if (parsed !== null) yield parsed;
  }
}

/** Read complete SSE data events for both typed and raw transport consumers. */
export async function* parseSSEData(
  response: Response,
  signal?: AbortSignal,
): AsyncGenerator<string> {
  if (!response.body) {
    throw new Error("Response body is null — SSE streaming unavailable");
  }
  const reader = response.body.getReader();
  const decoder = new TextDecoder();
  let buffer = "";
  let cursor = 0;
  let dataLines: string[] = [];
  try {
    while (true) {
      if (signal?.aborted) throw new DOMException("Aborted", "AbortError");
      const { done, value } = await reader.read();
      buffer += done ? decoder.decode() : decoder.decode(value, { stream: true });
      while (cursor < buffer.length) {
        const char = buffer[cursor];
        if (char !== "\r" && char !== "\n") {
          cursor++;
          continue;
        }
        // A CRLF delimiter can arrive in separate network chunks.
        if (char === "\r" && cursor + 1 === buffer.length && !done) break;
        const line = buffer.slice(0, cursor);
        const delimiterLength = char === "\r" && buffer[cursor + 1] === "\n" ? 2 : 1;
        buffer = buffer.slice(cursor + delimiterLength);
        cursor = 0;
        if (line === "") {
          if (dataLines.length) {
            const data = dataLines.join("\n");
            dataLines = [];
            if (data === "[DONE]") return;
            yield data;
          }
        } else {
          const colon = line.indexOf(":");
          const field = colon < 0 ? line : line.slice(0, colon);
          if (field === "data") {
            let data = colon < 0 ? "" : line.slice(colon + 1);
            if (data.startsWith(" ")) data = data.slice(1);
            dataLines.push(data);
          }
        }
      }
      if (done) break;
    }
  } finally {
    reader.releaseLock();
  }
}
