/**
 * SPEC-020 artifact paths — single source for QC screenshots and proof JSON.
 */
import fs from "node:fs";
import path from "node:path";
import type { Page } from "@playwright/test";

export const SPEC020_ROOT = path.resolve(
  __dirname,
  "../../../specs/020-e2e-quality-control",
);

export const SPEC020_SCREENSHOTS = path.join(SPEC020_ROOT, "e2e/screenshots");
export const SPEC020_PROOF_DIR = path.join(SPEC020_ROOT, "e2e");

export function ensureSpec020Artifacts(): void {
  fs.mkdirSync(SPEC020_SCREENSHOTS, { recursive: true });
}

export function spec020Screenshot(name: string): string {
  return path.join(SPEC020_SCREENSHOTS, name);
}

export async function captureSpec020(
  page: Page,
  filename: string,
  options?: { fullPage?: boolean; locator?: ReturnType<Page["locator"]> },
): Promise<void> {
  ensureSpec020Artifacts();
  const target = spec020Screenshot(filename);
  if (options?.locator) {
    await options.locator.screenshot({ path: target });
  } else {
    await page.screenshot({ path: target, fullPage: options?.fullPage ?? false });
  }
}

export function writeSpec020Json(filename: string, data: unknown): void {
  ensureSpec020Artifacts();
  fs.writeFileSync(
    path.join(SPEC020_PROOF_DIR, filename),
    `${JSON.stringify(data, null, 2)}\n`,
  );
}
