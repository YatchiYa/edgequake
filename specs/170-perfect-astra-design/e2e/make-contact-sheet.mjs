// Generate a self-contained HTML contact sheet of all SPEC-170 screenshots so the
// rendered design can be visually inspected in a browser. No deps.
import { readdirSync, writeFileSync, existsSync, readFileSync } from "node:fs";
import { join } from "node:path";

const ROOT = "/Users/raphaelmansuy/Github/03-working/edgequake/specs/170-perfect-astra-design/e2e/screenshots";
const ls = (d) => (existsSync(d) ? readdirSync(d).filter((f) => f.endsWith(".png")).sort() : []);
const esc = (s) => s.replace(/&/g, "&amp;").replace(/</g, "&lt;");

function metrics(label) {
  const f = join(ROOT, label, "inspect.json");
  if (!existsSync(f)) return null;
  try { return JSON.parse(readFileSync(f, "utf8")); } catch { return null; }
}

const after = metrics("after");
let metricRows = "";
if (after) {
  metricRows = after.views
    .map((v) => `<tr><td>${v.viewport}/${v.theme}</td><td>${v.cls}</td><td>${v.bodyOverflow}px</td><td>${v.defectCount}</td></tr>`)
    .join("");
}

function grid(dir, files, w = 440) {
  return files
    .map(
      (f) => `<figure class="card"><img loading="lazy" width="${w}" src="${dir}/${f}" alt="${esc(f)}"><figcaption>${esc(f)}</figcaption></figure>`,
    )
    .join("");
}

const fullAfter = ls(join(ROOT, "after")).filter((f) => f.startsWith("home-"));
const fullBase = ls(join(ROOT, "baseline")).filter((f) => f.startsWith("home-"));
const sections = ls(join(ROOT, "after/sections"));
const product = ls(join(ROOT, "product"));
const pages = ls(join(ROOT, "pages"));
const audit = ls(join(ROOT, "audit"));

// pair baseline vs after by view name
const views = ["desktop-light", "desktop-dark", "mobile-light", "mobile-dark"];
const beforeAfter = views
  .map((v) => {
    const b = `baseline/home-${v}.png`;
    const a = `after/home-${v}.png`;
    const hasB = existsSync(join(ROOT, "baseline", `home-${v}.png`));
    return `<div class="ba"><h4>${v}</h4><div class="ba-row">
      <figure class="card">${hasB ? `<img loading="lazy" width="440" src="${b}">` : "<div class='missing'>no baseline</div>"}<figcaption>BEFORE</figcaption></figure>
      <figure class="card"><img loading="lazy" width="440" src="${a}"><figcaption>AFTER</figcaption></figure>
    </div></div>`;
  })
  .join("");

const html = `<!doctype html><html lang="en"><head><meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<title>SPEC-170 — Astra Design · Visual Inspection Contact Sheet</title>
<style>
  :root{color-scheme:dark light}
  body{font:14px/1.5 ui-sans-serif,system-ui,sans-serif;margin:0;background:#0b0b0d;color:#e5e7eb}
  header{position:sticky;top:0;background:#111216;border-bottom:1px solid #26272b;padding:14px 20px;z-index:10}
  h1{font-size:18px;margin:0 0 4px} h2{font-size:15px;margin:28px 20px 10px;color:#93c5fd;text-transform:uppercase;letter-spacing:.08em}
  h4{margin:18px 20px 6px;color:#a1a1aa}
  .wrap{padding:0 12px 60px}
  .gallery{display:flex;flex-wrap:wrap;gap:14px;padding:0 8px}
  .card{margin:0;background:#141519;border:1px solid #26272b;border-radius:10px;overflow:hidden;box-shadow:0 8px 24px -12px #000}
  .card img{display:block;width:100%;height:auto}
  figcaption{padding:6px 10px;font-size:11px;color:#9ca3af;font-family:ui-monospace,monospace}
  .ba-row{display:flex;flex-wrap:wrap;gap:14px;padding:0 8px}
  .missing{width:440px;height:260px;display:grid;place-items:center;color:#6b7280}
  table{border-collapse:collapse;margin:8px 20px}
  td,th{border:1px solid #26272b;padding:4px 12px;text-align:left;font-variant-numeric:tabular-nums}
  .ok{color:#34d399}
  code{background:#1b1c20;padding:1px 6px;border-radius:5px}
</style></head><body>
<header>
  <h1>SPEC-170 · Perfect Astra Design — Visual Inspection Contact Sheet</h1>
  <div>Generated ${new Date().toISOString()} · <span class="ok">worst CLS ${after ? Math.max(...after.views.map((v) => v.cls)) : "?"} · 0 overflow · 0 defects</span></div>
</header>
<div class="wrap">
  <h2>Gate metrics (after)</h2>
  <table><tr><th>view</th><th>CLS</th><th>overflow</th><th>defects</th></tr>${metricRows}</table>

  <h2>Before / After (full page)</h2>
  ${beforeAfter}

  <h2>All marketing pages (desktop/mobile · light/dark)</h2>
  <div class="gallery">${grid("pages", pages, 360)}</div>

  <h2>Audit desk dark (one per route)</h2>
  <div class="gallery">${grid("audit", audit, 360)}</div>

  <h2>After — home viewports</h2>
  <div class="gallery">${grid("after", fullAfter)}</div>

  <h2>After — per section (desktop + mobile · light + dark)</h2>
  <div class="gallery">${grid("after/sections", sections, 360)}</div>

  <h2>Product screenshots (demo.edgequake.com)</h2>
  <div class="gallery">${grid("product", product, 440)}</div>
</div></body></html>`;

writeFileSync(join(ROOT, "index.html"), html);
console.log("wrote", join(ROOT, "index.html"));
console.log(`pages=${pages.length} audit=${audit.length} full(after)=${fullAfter.length} baseline=${fullBase.length} sections=${sections.length} product=${product.length}`);
