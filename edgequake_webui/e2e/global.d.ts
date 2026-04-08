/**
 * Global type augmentations for Playwright E2E test helpers.
 *
 * These declarations extend the browser `Window` interface with custom
 * properties injected by test fixtures or page-level scripts used during
 * E2E test runs. They must NOT be imported into production source code.
 */

declare global {
  interface Window {
    /**
     * Accumulated list of network request URLs captured by the test harness.
     * Populated via `page.evaluate()` in Playwright tests that need to inspect
     * which network calls were triggered by a page interaction.
     */
    __requestUrls: string[];
  }
}

// Make this file a module so the `declare global` block is treated as an
// augmentation rather than a script-level declaration.
export { };

