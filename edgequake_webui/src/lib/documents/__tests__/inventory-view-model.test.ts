/**
 * SPEC-099 LAW-099-7 / LAW-099-8 — inventory view-model honesty.
 */
import { describe, expect, it } from "vitest";
import {
  VIRTUAL_PAGE_SIZE,
  buildInventoryViewModel,
  countClientStatusCounts,
} from "../inventory-view-model";
import type { Document } from "@/types";

function doc(partial: Partial<Document> & { id: string; status: string }): Document {
  return {
    title: partial.id,
    file_name: `${partial.id}.pdf`,
    current_stage: partial.status,
    ...partial,
  } as Document;
}

describe("spec099-scale-overflow / filter-count-parity", () => {
  it("shows N+ when fetch hits VIRTUAL_PAGE_SIZE without API total", () => {
    const items = Array.from({ length: VIRTUAL_PAGE_SIZE }, (_, i) =>
      doc({ id: `d${i}`, status: "completed" }),
    );
    const vm = buildInventoryViewModel({
      fetchedItems: items,
      filteredRows: items,
      pageSize: VIRTUAL_PAGE_SIZE,
    });
    expect(vm.isTruncated).toBe(true);
    expect(vm.countLabel).toBe("100+");
    expect(vm.overflowLabel).toMatch(/Showing 100\+/);
  });

  it("shows N of M when API total exceeds page", () => {
    const items = Array.from({ length: 17 }, (_, i) =>
      doc({ id: `d${i}`, status: "completed" }),
    );
    const vm = buildInventoryViewModel({
      fetchedItems: items,
      filteredRows: items,
      pageSize: VIRTUAL_PAGE_SIZE,
      apiTotal: 240,
    });
    expect(vm.countLabel).toBe("17 of 240");
    expect(vm.overflowLabel).toBe("Showing 17 of 240");
  });

  it("keeps header filteredCount aligned with chip all on client counts", () => {
    const items = [
      doc({ id: "a", status: "completed" }),
      doc({ id: "b", status: "failed" }),
      doc({ id: "c", status: "extracting", current_stage: "extracting" }),
    ];
    const filtered = items.filter((d) => d.status !== "failed");
    const vm = buildInventoryViewModel({
      fetchedItems: items,
      filteredRows: filtered,
    });
    expect(vm.filteredCount).toBe(2);
    expect(vm.statusCounts.all).toBe(2);
    // Honest: filtered view of a larger fetch → "N of M"
    expect(vm.countLabel).toBe("2 of 3");
  });

  it("counts delete_failed via domain display status", () => {
    const counts = countClientStatusCounts([
      { status: "delete_failed", current_stage: "delete_failed" },
      { status: "failed" },
    ]);
    expect(counts.failed).toBe(2);
  });
});
