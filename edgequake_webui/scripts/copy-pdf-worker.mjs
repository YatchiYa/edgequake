#!/usr/bin/env node
/**
 * Copy pdfjs-dist worker to public/ so PDFViewer uses a same-origin worker
 * (no unpkg). SPEC-128 overlay e2e depends on this.
 *
 * Resolve via Node (pnpm/.pnpm layout) rather than a hardcoded
 * node_modules/pdfjs-dist path — CI `pnpm install --frozen-lockfile`
 * does not hoist the transitive worker unless pdfjs-dist is a direct dep.
 */
import { copyFileSync, existsSync } from "node:fs";
import { createRequire } from "node:module";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const require = createRequire(join(root, "package.json"));

let src;
try {
  src = require.resolve("pdfjs-dist/build/pdf.worker.min.mjs");
} catch {
  src = join(root, "node_modules/pdfjs-dist/build/pdf.worker.min.mjs");
}

const dest = join(root, "public/pdf.worker.min.mjs");
if (!existsSync(src)) {
  console.error(`pdf.js worker missing: ${src}`);
  process.exit(1);
}
copyFileSync(src, dest);
