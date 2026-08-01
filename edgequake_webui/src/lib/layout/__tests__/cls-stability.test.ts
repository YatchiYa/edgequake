import { describe, expect, test } from "bun:test";
import { isInitialLoading, shouldReserveSlot } from "../cls-stability";

describe("cls-stability", () => {
  test("shouldReserveSlot only on cold load when signal or hint", () => {
    expect(
      shouldReserveSlot({
        hasContent: true,
        isInitialLoading: true,
        signal: true,
        hint: true,
      }),
    ).toBe(false);

    expect(
      shouldReserveSlot({
        hasContent: false,
        isInitialLoading: false,
        signal: true,
        hint: true,
      }),
    ).toBe(false);

    expect(
      shouldReserveSlot({
        hasContent: false,
        isInitialLoading: true,
        signal: true,
        hint: false,
      }),
    ).toBe(true);

    expect(
      shouldReserveSlot({
        hasContent: false,
        isInitialLoading: true,
        signal: false,
        hint: true,
      }),
    ).toBe(true);

    expect(
      shouldReserveSlot({
        hasContent: false,
        isInitialLoading: true,
        signal: false,
        hint: false,
      }),
    ).toBe(false);
  });

  test("isInitialLoading requires no cached data", () => {
    expect(isInitialLoading(true, false)).toBe(true);
    expect(isInitialLoading(true, true)).toBe(false);
    expect(isInitialLoading(false, false)).toBe(false);
  });
});
