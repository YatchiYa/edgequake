import { describe, expect, it } from "vitest";
import { parseSSEStream } from "../../src/streaming/sse.js";
import { FetchTransport } from "../../src/transport/fetch.js";

const wires = [
  'data:{"chunk":"hello"}\n\ndata:[DONE]\n\n',
  'data: {"chunk":\ndata: "hello"}\n\ndata: [DONE]\n\n',
  'data: {"chunk":"hello"}\r\n\r\ndata: [DONE]\r\n\r\n',
  'data:{"chunk":"hello"}\r\rdata:[DONE]\r\r',
  ': heartbeat\nid: 42\nevent: token\nretry: 500\ndata:{"chunk":"hello"}\n\n',
  '\ufeffdata:{"chunk":"hello"}\n\n',
  'data:{"chunk":"héllo 🌍"}\n\n',
];

function response(wire: string, bytewise: boolean): Response {
  const bytes = new TextEncoder().encode(wire);
  return new Response(new ReadableStream({
    start(controller) {
      if (bytewise) for (const byte of bytes) controller.enqueue(new Uint8Array([byte]));
      else controller.enqueue(bytes);
      controller.close();
    },
  }));
}

for (const bytewise of [false, true]) {
  describe(`SSE framing (${bytewise ? "bytewise" : "whole"})`, () => {
    it.each(wires)("typed parser preserves event: %s", async (wire) => {
      const events = [];
      for await (const event of parseSSEStream(response(wire, bytewise), JSON.parse)) events.push(event);
      expect(events).toEqual([{ chunk: wire.includes("héllo") ? "héllo 🌍" : "hello" }]);
    });
    it.each(wires)("transport preserves event: %s", async (wire) => {
      const transport = new FetchTransport({ baseUrl: "https://example.invalid", headers: {}, timeout: 0,
        maxRetries: 0, retryDelay: 0, retryStatusCodes: [], fetchFn: async () => response(wire, bytewise) });
      const events = [];
      for await (const data of transport.stream({ method: "GET", path: "/query" })) events.push(JSON.parse(data));
      expect(events).toEqual([{ chunk: wire.includes("héllo") ? "héllo 🌍" : "hello" }]);
    });
  });
}
