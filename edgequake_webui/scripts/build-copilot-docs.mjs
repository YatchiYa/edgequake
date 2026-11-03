#!/usr/bin/env node
// Builds src/lib/server/copilot/product-docs.json: the public product documentation the copilot
// searches to answer questions about EdgeQuake itself (search_product_docs tool).
// Run from edgequake_webui/: `pnpm run docs:copilot` after editing the docs below.
// Internal or client documentation (docs/documentation_interne, specs/) is deliberately NOT included:
// every signed-in user can read what the copilot can search.
import { readFileSync, readdirSync, statSync, writeFileSync } from "node:fs";
import { join, relative } from "node:path";

const REPO = new URL("../../", import.meta.url).pathname;
const SOURCES = [
  "tutorial",
  "docs/getting-started",
  "docs/concepts",
  "docs/architecture",
  "docs/tutorials",
  "docs/operations",
  "docs/troubleshooting",
  "docs/providers",
  "docs/security",
  "docs/deep-dives",
  "docs/faq.md",
  "docs/product-limits.md",
  "docs/whats-new.md",
];
const SKIP = /documentation_interne|linkedin|\/formation\//i;
const MAX_CHARS = 2400;

const files = (p) => {
  const abs = join(REPO, p);
  if (statSync(abs, { throwIfNoEntry: false })?.isDirectory()) {
    return readdirSync(abs).flatMap((f) => files(join(p, f)));
  }
  return p.endsWith(".md") && !SKIP.test(p) ? [p] : [];
};

/** One section per heading; long sections are cut on paragraph breaks. */
function sections(path, text) {
  const body = text.replace(/^---\n[\s\S]*?\n---\n/, "");
  const title = body.match(/^#\s+(.+)$/m)?.[1]?.trim() ?? path;
  // "> **Product: v0.32.2**" header: lets the copilot spot pages older than a change (e.g. AGE → plain tables in v0.35).
  const version = text.match(/Product:\s*v(\d+\.\d+(?:\.\d+)?)/)?.[1] ?? null;
  const out = [];
  let heading = title;
  let buf = [];
  const flush = () => {
    let chunk = buf.join("\n").trim();
    while (chunk.length > 40) {
      let cut = chunk.length <= MAX_CHARS ? chunk.length : chunk.lastIndexOf("\n\n", MAX_CHARS);
      if (cut < MAX_CHARS / 2) cut = Math.min(chunk.length, MAX_CHARS);
      out.push({ path, title, heading, version, text: chunk.slice(0, cut).trim() });
      chunk = chunk.slice(cut).trim();
    }
    buf = [];
  };
  for (const line of body.split("\n")) {
    const h = line.match(/^#{1,4}\s+(.+)$/);
    if (h) {
      flush();
      heading = h[1].trim();
    } else buf.push(line);
  }
  flush();
  return out;
}

const all = [...new Set(SOURCES.flatMap(files))].sort();
const docs = all.flatMap((p) => sections(relative(REPO, join(REPO, p)), readFileSync(join(REPO, p), "utf8")));
writeFileSync(new URL("../src/lib/server/copilot/product-docs.json", import.meta.url), JSON.stringify(docs));
console.log(`copilot product docs: ${all.length} files, ${docs.length} sections`);
