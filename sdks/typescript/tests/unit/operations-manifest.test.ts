import { readFileSync, readdirSync, statSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";

const root = join(import.meta.dirname, "../../../..");
const manifest = JSON.parse(readFileSync(join(root, "sdks/operation-manifest.json"), "utf8"));
const generated = readFileSync(
  join(root, "sdks/typescript/src/resources/operations-generated.ts"),
  "utf8",
);

function walk(dir: string, out: string[]): void {
  for (const name of readdirSync(dir)) {
    if (name === "node_modules" || name === "dist") continue;
    const path = join(dir, name);
    if (statSync(path).isDirectory()) walk(path, out);
    else if (path.endsWith(".ts")) out.push(readFileSync(path, "utf8"));
  }
}

describe("OpenAPI operation manifest", () => {
  it("mentions every HTTP path and records the verb on generated methods", () => {
    const files: string[] = [];
    walk(join(root, "sdks/typescript/src"), files);
    const blob = files.join("\n");
    for (const op of manifest) {
      if (op.kind === "websocket") {
        expect(generated, op.path).toContain(op.path);
        continue;
      }
      expect(blob, op.path).toContain(op.path);
      if (op.missing.typescript) {
        expect(generated).toContain(`${op.method} ${op.path}`);
      }
    }
  });
});
