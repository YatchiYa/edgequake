import { describe, expect, it } from "vitest";
import { createRequire } from "node:module";

const require = createRequire(import.meta.url);

describe("SPEC-144 Next.js pin", () => {
  it("pins next and eslint-config-next to 16.3.3 Active LTS", () => {
    const pkg = require("../../../../package.json") as {
      dependencies: Record<string, string>;
      devDependencies: Record<string, string>;
    };
    expect(pkg.dependencies.next).toBe("16.3.3");
    expect(pkg.devDependencies["eslint-config-next"]).toBe("16.3.3");
  });

  it("installs next@16.3.3 from node_modules", () => {
    const nextPkg = require("next/package.json") as { version: string };
    expect(nextPkg.version).toBe("16.3.3");
  });
});
