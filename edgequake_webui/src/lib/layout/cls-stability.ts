/**
 * SPEC-100 — Shared CLS / layout-stability helpers.
 *
 * Reserve geometry before async chrome paints so primary content does not jump.
 * @see https://web.dev/articles/optimize-cls
 */

/** Generic sessionStorage hint helpers (route-scoped keys). */
export function readSessionHint(key: string): boolean {
  if (typeof window === "undefined") return false;
  try {
    return window.sessionStorage.getItem(key) === "1";
  } catch {
    return false;
  }
}

export function writeSessionHint(key: string, active: boolean): void {
  if (typeof window === "undefined") return;
  try {
    if (active) {
      window.sessionStorage.setItem(key, "1");
    } else {
      window.sessionStorage.removeItem(key);
    }
  } catch {
    /* private mode / quota */
  }
}

/**
 * Reserve a slot on cold load when a signal or prior hint says content is coming.
 * When `hasContent` is true the real UI already owns the geometry.
 */
export function shouldReserveSlot(opts: {
  hasContent: boolean;
  isInitialLoading: boolean;
  signal: boolean;
  hint: boolean;
}): boolean {
  if (opts.hasContent) return false;
  if (!opts.isInitialLoading) return false;
  return opts.signal || opts.hint;
}

/** Soft-refresh guard: only show full skeletons when there is no cached data. */
export function isInitialLoading(isLoading: boolean, hasData: boolean): boolean {
  return isLoading && !hasData;
}
