/**
 * Product version for marketing badges and schema.org.
 * Prefer repo-root VERSION at build time; fall back to the known pin.
 */
import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

/** Known pin when VERSION cannot be read (must match repo root VERSION). */
export const PRODUCT_VERSION_FALLBACK = "0.36.0";

const __dirname = dirname(fileURLToPath(import.meta.url));

export function getProductVersion(): string {
  try {
    const versionPath = resolve(__dirname, "../../../../VERSION");
    const raw = readFileSync(versionPath, "utf8").trim();
    if (/^\d+\.\d+\.\d+/.test(raw)) {
      return raw;
    }
  } catch {
    // fall through
  }
  return PRODUCT_VERSION_FALLBACK;
}

export const GITHUB_REPO_URL = "https://github.com/raphaelmansuy/edgequake";
export const DEMO_URL = "https://demo.edgequake.com/";
export const QUERY_MODES = [
  "naive",
  "local",
  "global",
  "hybrid",
  "mix",
  "bypass",
] as const;
