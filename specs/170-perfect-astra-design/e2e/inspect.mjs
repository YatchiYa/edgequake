// SPEC-170 deep visual-defect inspector + comprehensive capture.
// Saves full-page AND per-section screenshots for every viewport/theme into
// specs/170-perfect-astra-design/e2e/screenshots/<label>/, and emits a defect
// report so "visual inspection" is rigorous and reproducible.
//
// Usage: node inspect.mjs <label> [baseURL]
import pw from "/Users/raphaelmansuy/Github/03-working/edgequake/edgequake_webui/node_modules/playwright-core/index.js";
const { chromium } = pw;
import { mkdirSync, writeFileSync } from "node:fs";
import { join } from "node:path";

const ROOT = "/Users/raphaelmansuy/Github/03-working/edgequake/specs/170-perfect-astra-design/e2e/screenshots";
const label = process.argv[2] || "after";
const baseURL = process.argv[3] || "http://localhost:4321";
const outDir = join(ROOT, label);
mkdirSync(outDir, { recursive: true });

const VIEWPORTS = {
  wide: { width: 1920, height: 1080 },
  desktop: { width: 1440, height: 900 },
  tablet: { width: 834, height: 1112 },
  mobile: { width: 390, height: 844 },
};

const CLS_SCRIPT = `
  window.__cls = 0; window.__shifts = [];
  try {
    new PerformanceObserver((list) => {
      for (const e of list.getEntries()) {
        if (!e.hadRecentInput) {
          window.__cls += e.value;
          if (e.value > 0.0005) window.__shifts.push({ v:+e.value.toFixed(4),
            s:(e.sources||[]).map(x=>x.node&&x.node.nodeName?x.node.nodeName:'?') });
        }
      }
    }).observe({ type:'layout-shift', buffered:true });
  } catch(e){}
`;

// ---- in-page analysis: the "eye" replaced by geometry + color checks ----
const ANALYZE = () => {
  const vw = window.innerWidth;
  const defects = [];
  const push = (kind, detail) => defects.push({ kind, ...detail });

  // 1) broken / empty images
  document.querySelectorAll("img").forEach((img) => {
    if (!img.complete || img.naturalWidth === 0) {
      push("broken-image", { src: img.currentSrc || img.src, alt: img.alt });
    }
  });

  // 2) horizontal overflow of the page body
  const bodyOverflow = document.documentElement.scrollWidth - vw;

  // helper: visible text leaf elements
  const textLeaves = [...document.querySelectorAll("h1,h2,h3,h4,p,span,a,li,td,th,button,figcaption,code,summary")]
    .filter((el) => {
      const r = el.getBoundingClientRect();
      const cs = getComputedStyle(el);
      return r.width > 0 && r.height > 0 && cs.visibility !== "hidden" && cs.display !== "none" &&
        el.textContent.trim().length > 0 && el.offsetParent !== null;
    });

  // 3) unintended clipping / truncation (ellipsis or hidden overflow cutting text)
  textLeaves.forEach((el) => {
    const cs = getComputedStyle(el);
    const clips = cs.textOverflow === "ellipsis" || cs.overflow === "hidden" || cs.overflowX === "hidden";
    if (clips && el.scrollWidth > el.clientWidth + 2 && !el.closest("[data-expected-truncate]")) {
      // allow the frame-bar URL label (intentional truncate)
      if (!el.closest(".mkt-frame-bar")) {
        push("text-clipped", { text: el.textContent.trim().slice(0, 40), tag: el.tagName, scroll: el.scrollWidth, client: el.clientWidth });
      }
    }
  });

  // 4) element escaping viewport horizontally (excluding decorative aria-hidden + overflow-clip hosts)
  document.querySelectorAll("main *").forEach((el) => {
    if (el.getAttribute("aria-hidden") === "true" || el.closest("[aria-hidden='true']")) return;
    const cs = getComputedStyle(el);
    // code blocks scroll internally — allowed
    if (el.closest(".mkt-code-showcase") || cs.overflowX === "auto" || el.closest("[class*='overflow-x-auto']")) return;
    const r = el.getBoundingClientRect();
    if (r.width > 0 && (r.right > vw + 1.5 || r.left < -1.5)) {
      push("escapes-viewport", { tag: el.tagName + "." + (typeof el.className === "string" ? el.className.split(" ")[0] : ""), left: Math.round(r.left), right: Math.round(r.right) });
    }
  });

  // 5) text overlapping another text block (layout bug detector)
  const blocks = textLeaves.filter((el) => ["H1","H2","H3","H4","P"].includes(el.tagName));
  for (let i = 0; i < blocks.length; i++) {
    for (let j = i + 1; j < blocks.length; j++) {
      const a = blocks[i].getBoundingClientRect();
      const b = blocks[j].getBoundingClientRect();
      if (blocks[i].contains(blocks[j]) || blocks[j].contains(blocks[i])) continue;
      const ox = Math.max(0, Math.min(a.right, b.right) - Math.max(a.left, b.left));
      const oy = Math.max(0, Math.min(a.bottom, b.bottom) - Math.max(a.top, b.top));
      if (ox > 6 && oy > 6) {
        push("text-overlap", { a: blocks[i].textContent.trim().slice(0, 24), b: blocks[j].textContent.trim().slice(0, 24), ox: Math.round(ox), oy: Math.round(oy) });
      }
    }
  }

  // 6) section geometry (for rhythm review)
  const sections = [...document.querySelectorAll("main > section, main > div > section")].map((s) => {
    const r = s.getBoundingClientRect();
    const h2 = s.querySelector("h1,h2");
    return { label: h2 ? h2.textContent.trim().slice(0, 28) : s.id || "(section)", h: Math.round(r.height), top: Math.round(r.top + window.scrollY) };
  });

  return { vw, bodyOverflow, defects, defectCount: defects.length, sectionCount: sections.length, sections };
};

async function run() {
  const browser = await chromium.launch();
  const report = { label, baseURL, when: new Date().toISOString(), views: [] };

  for (const [vpName, vp] of Object.entries(VIEWPORTS)) {
    for (const theme of ["light", "dark"]) {
      const ctx = await browser.newContext({ viewport: vp, deviceScaleFactor: 2, colorScheme: theme });
      const page = await ctx.newPage();
      await page.addInitScript(CLS_SCRIPT);
      await page.goto(baseURL + "/", { waitUntil: "load", timeout: 60000 });
      await page.evaluate(async () => {
        await (document.fonts ? document.fonts.ready : Promise.resolve());
        for (let y = 0; y <= document.body.scrollHeight; y += 350) { window.scrollTo(0, y); await new Promise((r) => setTimeout(r, 35)); }
        window.scrollTo(0, 0);
      });
      // Force every image (incl. lazy) to fully load before capture + analysis.
      await page.evaluate(() => {
        document.querySelectorAll('img[loading="lazy"]').forEach((i) => { i.loading = "eager"; });
      });
      await page.waitForLoadState("networkidle").catch(() => {});
      await page.evaluate(async () => {
        await Promise.all([...document.images].map((img) =>
          img.complete && img.naturalWidth > 0
            ? img.decode().catch(() => {})
            : new Promise((res) => { img.onload = img.onerror = res; })
        ));
      });
      await page.waitForTimeout(400);

      const cls = await page.evaluate(() => ({ cls: +window.__cls.toFixed(4), shifts: window.__shifts.slice(0, 6) }));
      const analysis = await page.evaluate(ANALYZE);

      // full-page screenshot
      const shot = join(outDir, `home-${vpName}-${theme}.png`);
      await page.screenshot({ path: shot, fullPage: true });

      // per-section screenshots (desktop + mobile only to limit volume)
      if (vpName === "desktop" || vpName === "mobile") {
        const secDir = join(outDir, "sections");
        mkdirSync(secDir, { recursive: true });
        const handles = await page.$$("main > section, main > div > section");
        let idx = 0;
        for (const h of handles) {
          idx++;
          const id = await h.evaluate((el) => {
            const t = el.querySelector("h1,h2");
            return (t ? t.textContent.trim() : el.id || "section").toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/^-|-$/g, "").slice(0, 28);
          });
          try {
            await h.scrollIntoViewIfNeeded();
            await page.waitForTimeout(120);
            await h.screenshot({ path: join(secDir, `${String(idx).padStart(2, "0")}-${id}-${vpName}-${theme}.png`) });
          } catch {}
        }
      }

      report.views.push({ viewport: vpName, theme, cls: cls.cls, shifts: cls.shifts, ...analysis, shot });
      const bad = analysis.defectCount;
      console.log(`${vpName}/${theme}: CLS=${cls.cls} overflow=${analysis.bodyOverflow}px defects=${bad}${bad ? " → " + analysis.defects.map((d) => d.kind).join(",") : ""}`);
      await ctx.close();
    }
  }

  writeFileSync(join(outDir, "inspect.json"), JSON.stringify(report, null, 2));
  const totalDefects = report.views.reduce((s, v) => s + v.defectCount, 0);
  const worstCls = Math.max(...report.views.map((v) => v.cls));
  const worstOverflow = Math.max(...report.views.map((v) => v.bodyOverflow));
  console.log(`\n== ${label} == worstCLS=${worstCls} worstOverflow=${worstOverflow}px totalDefects=${totalDefects}`);
  await browser.close();
}
run().catch((e) => { console.error(e); process.exit(1); });
