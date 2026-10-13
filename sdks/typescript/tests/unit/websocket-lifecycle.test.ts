import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { EdgeQuakeWebSocket } from "../../src/streaming/websocket.js";

class Socket {
  static instances: Socket[] = [];
  onmessage?: (event: { data: string }) => void;
  onclose?: () => void;
  onerror?: () => void;
  closes = 0;
  constructor(_url: string) { Socket.instances.push(this); }
  close() { this.closes++; } // Real close events are asynchronous, not synchronous callbacks.
}
beforeEach(() => { Socket.instances = []; vi.stubGlobal("WebSocket", Socket); });
afterEach(() => vi.unstubAllGlobals());

it("does not connect when closed before the first pull", async () => {
  const stream = new EdgeQuakeWebSocket("ws://example.invalid/progress");
  stream.close();
  expect(await stream[Symbol.asyncIterator]().next()).toMatchObject({ done: true });
  expect(Socket.instances).toHaveLength(0);
});

it("manual close settles a pending pull before the socket close event", async () => {
  const stream = new EdgeQuakeWebSocket("ws://example.invalid/progress");
  const iterator = stream[Symbol.asyncIterator]();
  const pending = iterator.next();
  stream.close();
  const result = await Promise.race([pending, new Promise((resolve) => setTimeout(() => resolve("timeout"), 100))]);
  Socket.instances[0].onclose?.(); // Bound cleanup on the old implementation.
  await pending;
  expect(result).toMatchObject({ done: true });
  expect(Socket.instances[0].closes).toBe(1);
});

it("returning from iteration closes the socket once", async () => {
  const stream = new EdgeQuakeWebSocket("ws://example.invalid/progress");
  const iterator = stream[Symbol.asyncIterator]();
  const pending = iterator.next();
  Socket.instances[0].onmessage?.({ data: '{"type":"progress"}' });
  await pending;
  await iterator.return?.();
  const closes = Socket.instances[0].closes;
  stream.close();
  expect(closes).toBe(1);
  expect(Socket.instances[0].closes).toBe(1);
});

it("error termination closes the underlying socket", async () => {
  const stream = new EdgeQuakeWebSocket("ws://example.invalid/progress");
  const pending = stream[Symbol.asyncIterator]().next();
  Socket.instances[0].onerror?.();
  expect(await pending).toMatchObject({ done: true });
  expect(Socket.instances[0].closes).toBe(1);
});

it("server close still settles the pending pull without closing twice", async () => {
  const stream = new EdgeQuakeWebSocket("ws://example.invalid/progress");
  const pending = stream[Symbol.asyncIterator]().next();
  Socket.instances[0].onclose?.();
  expect(await pending).toMatchObject({ done: true });
  expect(Socket.instances[0].closes).toBe(0);
});
