import { describe, expect, it } from "vitest";
import { extrasInSameTenant, mergeEntitiesById } from "../merge-entities-by-id";

describe("mergeEntitiesById", () => {
  it("keeps distinct ids (no last-write collapse)", () => {
    const merged = mergeEntitiesById(
      [
        { id: "a", name: "g99-73" },
        { id: "b", name: "g99-72" },
        { id: "c", name: "g99-71" },
      ],
      [],
    );
    expect(merged.map((r) => r.name).sort()).toEqual([
      "g99-71",
      "g99-72",
      "g99-73",
    ]);
  });

  it("server wins on id collision; extras fill gaps", () => {
    const merged = mergeEntitiesById(
      [{ id: "a", name: "server" }],
      [
        { id: "a", name: "optimistic" },
        { id: "b", name: "extra" },
      ],
    );
    expect(merged).toEqual([
      { id: "a", name: "server" },
      { id: "b", name: "extra" },
    ]);
  });

  it("skips missing ids so Map cannot collapse to the last row", () => {
    const merged = mergeEntitiesById(
      [
        { name: "g99-71" },
        { name: "g99-72" },
        { id: "z", name: "g99-73" },
      ],
      [{ name: "ghost" }],
    );
    expect(merged).toEqual([{ id: "z", name: "g99-73" }]);
  });

  it("extrasInSameTenant drops other-tenant leftovers", () => {
    const extras = extrasInSameTenant(
      [
        { id: "keep", tenant_id: "t1" },
        { id: "drop", tenant_id: "t2" },
        { id: "optimistic" },
      ],
      "t1",
    );
    expect(extras.map((r) => r.id)).toEqual(["keep", "optimistic"]);
  });
});
