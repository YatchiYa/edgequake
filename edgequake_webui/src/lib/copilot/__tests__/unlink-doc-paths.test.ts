import { describe, expect, it } from "vitest";
import { unlinkDocPaths } from "../unlink-doc-paths";

describe("unlinkDocPaths", () => {
  it("turns relative doc links into their plain text", () => {
    expect(unlinkDocPaths("*Source : [docs/faq.md](docs/faq.md)*, see [§2](docs/a.md#x)")).toBe(
      "*Source : docs/faq.md*, see §2",
    );
  });
  it("keeps web links and in-app routes", () => {
    const md = "[Gephi](https://gephi.org/) [graph](/graph) [top](#top) [mail](mailto:a@b.c)";
    expect(unlinkDocPaths(md)).toBe(md);
  });
});
