/**
 * Shared scroll assertions for E2E specs.
 *
 * Extracted (DRY) from `spec037-query-settings-scroll.spec.ts` so any spec can
 * assert that a container actually scrolls and can be scrolled to reveal
 * trailing content.
 */
import { expect, type Locator } from "@playwright/test";

export interface ScrollMetrics {
  scrollHeight: number;
  clientHeight: number;
}

/**
 * Assert that `locator` overflows vertically (scrollHeight > clientHeight),
 * i.e. it is genuinely scrollable. Returns the measured metrics so callers can
 * make further assertions.
 */
export async function expectScrollable(
  locator: Locator,
): Promise<ScrollMetrics> {
  await locator.waitFor({ state: "visible" });
  const metrics = await locator.evaluate((el) => ({
    scrollHeight: el.scrollHeight,
    clientHeight: el.clientHeight,
  }));
  expect(
    metrics.scrollHeight,
    `expected element to be scrollable (scrollHeight ${metrics.scrollHeight} > clientHeight ${metrics.clientHeight})`,
  ).toBeGreaterThan(metrics.clientHeight);
  return metrics;
}

/** Scroll `locator` to the very bottom to reveal trailing content. */
export async function scrollToBottom(locator: Locator): Promise<void> {
  await locator.evaluate((el) => {
    el.scrollTop = el.scrollHeight;
  });
}
