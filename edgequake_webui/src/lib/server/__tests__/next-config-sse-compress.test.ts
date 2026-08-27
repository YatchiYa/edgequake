import { readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";

describe("next.config SSE gzip lock (PR #389)", () => {
  it("disables Next compression so proxied text/event-stream is not buffered", () => {
    const configPath = path.resolve(
      path.dirname(fileURLToPath(import.meta.url)),
      "../../../../next.config.ts",
    );
    const source = readFileSync(configPath, "utf8");
    expect(source).toMatch(/compress:\s*false/);
  });
});
