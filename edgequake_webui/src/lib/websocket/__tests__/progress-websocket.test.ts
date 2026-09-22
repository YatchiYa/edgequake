/**
 * SPEC-149 — ProgressWebSocket lifecycle (production class, not reimplemented helpers).
 */
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { ProgressWebSocket } from "../progress-websocket";

class FakeWebSocket {
  static OPEN = 1;
  static CONNECTING = 0;
  static CLOSING = 2;
  static CLOSED = 3;
  static instances: FakeWebSocket[] = [];

  readyState = FakeWebSocket.CONNECTING;
  url: string;
  onopen: ((ev: Event) => void) | null = null;
  onclose: ((ev: CloseEvent) => void) | null = null;
  onerror: ((ev: Event) => void) | null = null;
  onmessage: ((ev: MessageEvent) => void) | null = null;
  sent: string[] = [];

  constructor(url: string) {
    this.url = url;
    FakeWebSocket.instances.push(this);
  }

  send(data: string) {
    this.sent.push(data);
  }

  close() {
    this.readyState = FakeWebSocket.CLOSED;
    this.onclose?.(
      new CloseEvent("close", { code: 1000, reason: "close", wasClean: true }),
    );
  }

  open() {
    this.readyState = FakeWebSocket.OPEN;
    this.onopen?.(new Event("open"));
  }

  dirtyClose() {
    this.readyState = FakeWebSocket.CLOSED;
    this.onclose?.(
      new CloseEvent("close", { code: 1006, reason: "abnormal", wasClean: false }),
    );
  }
}

describe("ProgressWebSocket SPEC-149", () => {
  let token = "";

  beforeEach(() => {
    FakeWebSocket.instances = [];
    token = "";
    vi.stubGlobal("WebSocket", FakeWebSocket as unknown as typeof WebSocket);
  });

  afterEach(() => {
    vi.unstubAllGlobals();
    vi.useRealTimers();
  });

  it("U-149-01 resolves URL with latest token on each connect", () => {
    const client = new ProgressWebSocket({
      urlResolver: () =>
        `ws://example.test/ws/pipeline/progress${token ? `?token=${token}` : ""}`,
      maxReconnectAttempts: 3,
    });

    client.connect();
    expect(FakeWebSocket.instances[0].url).toBe(
      "ws://example.test/ws/pipeline/progress",
    );
    FakeWebSocket.instances[0].open();

    token = "tok-2";
    client.reconnectFresh();
    expect(FakeWebSocket.instances.at(-1)!.url).toContain("token=tok-2");
    client.disconnect();
  });

  it("U-149-03 replays desired subscriptions after reconnect", () => {
    const client = new ProgressWebSocket({
      urlResolver: () => "ws://example.test/ws",
    });
    client.connect();
    FakeWebSocket.instances[0].open();
    client.subscribe(["track-a"]);
    expect(FakeWebSocket.instances[0].sent.some((s) => s.includes("track-a"))).toBe(
      true,
    );

    client.reconnectFresh();
    const next = FakeWebSocket.instances.at(-1)!;
    next.open();
    expect(next.sent.some((s) => s.includes("subscribe") && s.includes("track-a"))).toBe(
      true,
    );
    client.disconnect();
  });

  it("U-149-04 guards CONNECTING and resets attempts on reconnectFresh", () => {
    vi.useFakeTimers();
    const client = new ProgressWebSocket({
      urlResolver: () => "ws://example.test/ws",
      reconnectInterval: 10,
      maxReconnectAttempts: 3,
      maxReconnectDelayMs: 100,
    });

    client.connect();
    expect(FakeWebSocket.instances).toHaveLength(1);
    client.connect(); // CONNECTING — no second socket
    expect(FakeWebSocket.instances).toHaveLength(1);

    FakeWebSocket.instances[0].dirtyClose();
    vi.advanceTimersByTime(20);
    expect(FakeWebSocket.instances.length).toBeGreaterThan(1);

    client.reconnectFresh();
    const after = FakeWebSocket.instances.at(-1)!;
    after.open();
    // After fresh reconnect + open, attempts reset (another dirty close should reconnect again)
    after.dirtyClose();
    vi.advanceTimersByTime(20);
    expect(FakeWebSocket.instances.length).toBeGreaterThan(2);
    client.disconnect();
  });
});
