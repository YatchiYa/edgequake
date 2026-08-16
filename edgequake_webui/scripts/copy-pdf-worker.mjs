#!/usr/bin/env node
/**
 * Copy pdfjs-dist worker to public/ so PDFViewer uses a same-origin worker
 * (no unpkg). SPEC-128 overlay e2e depends on this.
 */
import { copyFileSync, existsSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const src = join(root, "node_modules/pdfjs-dist/build/pdf.worker.min.mjs");
const dest = join(root, "public/pdf.worker.min.mjs");
if (!existsSync(src)) {
  console.error(`pdf.js worker missing: ${src}`);
  process.exit(1);
}
copyFileSync(src, dest);
