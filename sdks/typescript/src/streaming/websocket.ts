/**
 * WebSocket wrapper for async iteration.
 *
 * WHY: EdgeQuake uses WebSocket for real-time pipeline progress updates.
 * This wrapper enables `for await...of` syntax over WebSocket events.
 *
 * @module streaming/websocket
 */

import type { WebSocketEvent } from "../types/health.js";

/**
 * Wraps a WebSocket connection as an AsyncIterable.
 *
 * Usage:
 *   const ws = new EdgeQuakeWebSocket('ws://localhost:8080/ws/pipeline/progress');
 *   for await (const event of ws) {
 *     console.log(event.type, event.progress);
 *   }
 */
export class EdgeQuakeWebSocket implements AsyncIterable<WebSocketEvent> {
  private events: WebSocketEvent[] = [];
  private resolve?: (value: IteratorResult<WebSocketEvent>) => void;
  private closed = false;
  private wsInstance?: WebSocket;

  constructor(private readonly url: string) {}

  /**
   * Connect to the WebSocket and start receiving events.
   *
   * WHY: Lazy connection — don't open WebSocket until iteration begins.
   */
  private connect(): void {
    if (this.closed || this.wsInstance) return;

    this.wsInstance = new WebSocket(this.url);

    this.wsInstance.onmessage = (event: MessageEvent) => {
      if (this.closed) return;
      try {
        const parsed = JSON.parse(String(event.data)) as WebSocketEvent;
        if (this.resolve) {
          this.resolve({ value: parsed, done: false });
          this.resolve = undefined;
        } else {
          this.events.push(parsed);
        }
      } catch {
        // WHY: Skip malformed messages
      }
    };

    this.wsInstance.onclose = () => this.finish();
    this.wsInstance.onerror = () => this.close();
  }

  private finish(): void {
    this.closed = true;
    this.resolve?.({ value: undefined as unknown as WebSocketEvent, done: true });
    this.resolve = undefined;
  }

  async *[Symbol.asyncIterator](): AsyncIterator<WebSocketEvent> {
    this.connect();

    try {
      while (!this.closed) {
        if (this.events.length > 0) {
          yield this.events.shift()!;
        } else {
          const result = await new Promise<IteratorResult<WebSocketEvent>>(
            (resolve) => {
              this.resolve = resolve;
            },
          );
          if (result.done) return;
          yield result.value;
        }
      }

      // WHY: Drain any remaining buffered events
      while (this.events.length > 0) {
        yield this.events.shift()!;
      }
    } finally {
      this.close();
    }
  }

  /** Close the WebSocket connection. */
  close(): void {
    if (this.closed) return;
    this.finish();
    this.wsInstance?.close();
  }
}
