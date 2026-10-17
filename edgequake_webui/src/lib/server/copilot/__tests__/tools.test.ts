import { describe, expect, it } from "vitest";
import { describeOperation, parseOperations } from "../tools";

describe("copilot graph operations", () => {
  it("keeps valid operations in order and drops malformed ones", () => {
    const ops = parseOperations([
      { op: "merge_entities", sources: ["ACME_CORP", 3], target: "ACME_CORPORATION" },
      { op: "delete_relationship" },
      { op: "create_relationship", source: "JANE_DOE", target: "PARIS", keywords: "lives_in" },
      { op: "drop_database" },
      "nope",
      { op: "update_entity", name: "PARIS" },
    ]);
    expect(ops).toEqual([
      { op: "merge_entities", sources: ["ACME_CORP"], target: "ACME_CORPORATION", strategy: "merge" },
      {
        op: "create_relationship",
        source: "JANE_DOE",
        target: "PARIS",
        keywords: "lives_in",
        description: "lives_in",
        weight: undefined,
      },
    ]);
    expect(parseOperations("not an array")).toEqual([]);
  });

  it("describes operations for the approval card", () => {
    expect(describeOperation({ op: "merge_entities", sources: ["A", "B"], target: "C" })).toBe(
      "Merge A, B → C (merge descriptions)",
    );
    expect(describeOperation({ op: "update_entity", name: "A", description: "new text" })).toBe(
      "Update entity A · description → “new text”",
    );
    expect(
      describeOperation({ op: "create_relationship", source: "A", target: "B", keywords: "works_at, x", description: "" }),
    ).toBe("Link A —[works_at]→ B");
    expect(describeOperation({ op: "delete_entity", name: "A" })).toBe("Delete entity A and all its relationships");
  });
});
