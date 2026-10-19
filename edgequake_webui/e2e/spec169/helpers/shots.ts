/**
 * SPEC-169 screenshot helper — visual inspection loop.
 * Writes under specs/169-perfect-user-management/e2e/screenshots/{round}/.
 * Theme/viewport helpers are shared with SPEC-168 (DRY).
 */
import fs from "node:fs";
import path from "node:path";
import type { Page } from "@playwright/test";
import { applyTheme, setViewport, type Theme } from "../../spec168/helpers/shots";

export { applyTheme, setViewport };
export type { Theme };

const REPO_ROOT = path.resolve(__dirname, "../../../../");
export const SPEC169_SHOTS_ROOT = path.join(
  REPO_ROOT,
  "specs/169-perfect-user-management/e2e/screenshots",
);

export const ROUND = process.env.SPEC169_ROUND ?? "round-00-baseline";

export function shotPath(name: string, theme: Theme, width: number): string {
  const dir = path.join(SPEC169_SHOTS_ROOT, ROUND);
  fs.mkdirSync(dir, { recursive: true });
  return path.join(dir, `${name}-${theme}-${width}.png`);
}

export async function shot(
  page: Page,
  name: string,
  opts: { theme?: Theme; width?: number; fullPage?: boolean } = {},
): Promise<string> {
  const theme = opts.theme ?? "light";
  const width = opts.width ?? (page.viewportSize()?.width ?? 1440);
  const file = shotPath(name, theme, width);
  const take = () =>
    page.screenshot({
      path: file,
      fullPage: opts.fullPage ?? false,
      animations: "disabled",
      caret: "hide",
      timeout: 15_000,
    });
  try {
    await take();
  } catch {
    await page.waitForTimeout(500);
    await take();
  }
  return file;
}
