/**
 * @module copilot-product-docs
 * @description search_product_docs: BM25 over the public EdgeQuake documentation bundled at
 * build time (scripts/build-copilot-docs.mjs → product-docs.json), so the copilot can answer
 * questions about the product itself (architecture, limits, settings, operations), not only
 * about the workspace data.
 */
import DOCS from "./product-docs.json";

interface Section {
  path: string;
  title: string;
  heading: string;
  /** Product version the page was written for (null = unversioned). */
  version: string | null;
  text: string;
}

const K1 = 1.2;
const B = 0.75;

/** Lowercase, accents removed, words of 2+ letters/digits. */
export function tokenize(text: string): string[] {
  return text
    .toLowerCase()
    .normalize("NFD")
    .replace(/[̀-ͯ]/g, "")
    .split(/[^a-z0-9]+/)
    .filter((t) => t.length > 1);
}

// ponytail: index built once per server process, in memory (~1 MB of docs); rebuild on deploy.
let index: { tf: Map<string, number>[]; len: number[]; avg: number; df: Map<string, number> } | null = null;

function build() {
  const sections = DOCS as Section[];
  const tf = sections.map((s) => {
    const m = new Map<string, number>();
    // Headings weigh double: they name what the section is about.
    for (const t of [...tokenize(s.title + " " + s.heading), ...tokenize(s.heading), ...tokenize(s.text)]) {
      m.set(t, (m.get(t) ?? 0) + 1);
    }
    return m;
  });
  const len = tf.map((m) => [...m.values()].reduce((a, b) => a + b, 0));
  const df = new Map<string, number>();
  for (const m of tf) for (const t of m.keys()) df.set(t, (df.get(t) ?? 0) + 1);
  return { tf, len, avg: len.reduce((a, b) => a + b, 0) / Math.max(1, len.length), df };
}

export function searchProductDocs(query: string, limit = 5): (Section & { score: number })[] {
  index ??= build();
  const { tf, len, avg, df } = index;
  const n = tf.length;
  const terms = [...new Set(tokenize(query))];
  const scored = tf.map((m, i) => {
    let score = 0;
    for (const t of terms) {
      const f = m.get(t);
      if (!f) continue;
      const idf = Math.log(1 + (n - (df.get(t) ?? 0) + 0.5) / ((df.get(t) ?? 0) + 0.5));
      score += (idf * f * (K1 + 1)) / (f + K1 * (1 - B + (B * len[i]) / avg));
    }
    return { i, score };
  });
  return scored
    .filter((s) => s.score > 0)
    .sort((a, b) => b.score - a.score)
    .slice(0, limit)
    .map(({ i, score }) => ({ ...(DOCS as Section[])[i], score: Math.round(score * 100) / 100 }));
}
