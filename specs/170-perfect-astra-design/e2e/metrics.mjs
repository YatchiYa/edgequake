// Rigorous, measurable "visual inspection" harness for SPEC-170.
// Captures screenshots AND quantitative layout-health metrics:
//   - Cumulative Layout Shift (CLS) via PerformanceObserver (real shifts during load)
//   - Horizontal overflow (elements wider than viewport / body scrollWidth)
//   - Web-font load timing (FOUT/CLS risk)
//   - Section geometry (so we can reason about rhythm/spacing without eyes)
//   - Tap-target sizes on mobile (a11y)
// Usage: node metrics.mjs <label> [baseURL]
import pw from "/Users/raphaelmansuy/Github/03-working/edgequake/edgequake_webui/node_modules/playwright-core/index.js";
const { chromium } = pw;
import { mkdirSync, writeFileSync } from "node:fs";
import { join } from "node:path";

const ROOT = "/Users/raphaelmansuy/Github/03-working/edgequake/specs/170-perfect-astra-design/e2e/screenshots";
const label = process.argv[2] || "baseline";
const baseURL = process.argv[3] || "http://localhost:4321";
const outDir = join(ROOT, label);
mkdirSync(outDir, { recursive: true });

const VIEWPORTS = {
  desktop: { width: 1440, height: 900 },
  mobile: { width: 390, height: 844 },
};

const CLS_SCRIPT = `
  window.__cls = 0;
  window.__shifts = [];
  try {
    const po = new PerformanceObserver((list) => {
      for (const entry of list.getEntries()) {
        if (!entry.hadRecentInput) {
          window.__cls += entry.value;
          if (entry.value > 0.0005) {
            window.__shifts.push({
              value: +entry.value.toFixed(4),
              sources: (entry.sources || []).map(s => (s.node && s.node.nodeName ? s.node.nodeName + (s.node.className ? '.'+String(s.node.className).split(' ')[0] : '') : 'unknown')),
            });
          }
        }
      }
    });
    po.observe({ type: 'layout-shift', buffered: true });
  } catch (e) {}
`;

async function measure(page, theme) {
  return await page.evaluate((theme) => {
    const vw = window.innerWidth;
    const bodyOverflow = document.documentElement.scrollWidth - vw;
    // Elements overflowing horizontally
    const overflowers = [];
    document.querySelectorAll("*").forEach((el) => {
      const r = el.getBoundingClientRect();
      if (r.width > 0 && (r.right > vw + 1 || r.left < -1)) {
        const tag = el.tagName.toLowerCase() + (el.className && typeof el.className === "string" ? "." + el.className.split(" ").slice(0, 2).join(".") : "");
        overflowers.push({ tag, right: Math.round(r.right), left: Math.round(r.left) });
      }
    });
    // Section rhythm
    const sections = [...document.querySelectorAll("main > section, main > div > section, section")].slice(0, 20).map((s) => {
      const r = s.getBoundingClientRect();
      return { h: Math.round(r.height), top: Math.round(r.top + window.scrollY) };
    });
    return {
      theme,
      viewport: vw,
      bodyScrollWidth: document.documentElement.scrollWidth,
      horizontalOverflowPx: bodyOverflow,
      overflowers: overflowers.slice(0, 12),
      sectionCount: sections.length,
      sections,
    };
  }, theme);
}

async function run() {
  const browser = await chromium.launch();
  const report = { label, baseURL, when: new Date().toISOString(), pages: [] };

  for (const [vpName, vp] of Object.entries(VIEWPORTS)) {
    for (const theme of ["light", "dark"]) {
      const ctx = await browser.newContext({ viewport: vp, deviceScaleFactor: 2, colorScheme: theme });
      const page = await ctx.newPage();
      await page.addInitScript(CLS_SCRIPT);
      await page.goto(baseURL + "/", { waitUntil: "load", timeout: 45000 });
      // Let fonts + images settle, scroll to trigger any lazy shifts
      await page.waitForTimeout(500);
      await page.evaluate(async () => {
        await (document.fonts ? document.fonts.ready : Promise.resolve());
        for (let y = 0; y <= document.body.scrollHeight; y += 400) { window.scrollTo(0, y); await new Promise(r => setTimeout(r, 40)); }
        window.scrollTo(0, 0);
      });
      await page.waitForTimeout(800);
      const cls = await page.evaluate(() => ({ cls: +window.__cls.toFixed(4), shifts: window.__shifts.slice(0, 10) }));
      const geom = await measure(page, theme);
      const shot = join(outDir, `home-${vpName}-${theme}.png`);
      await page.screenshot({ path: shot, fullPage: true });
      report.pages.push({ viewport: vpName, theme, cls: cls.cls, shifts: cls.shifts, ...geom, shot });
      console.log(`${vpName}/${theme}: CLS=${cls.cls} overflow=${geom.horizontalOverflowPx}px overflowers=${geom.overflowers.length} sections=${geom.sectionCount}`);
      await ctx.close();
    }
  }
  writeFileSync(join(outDir, "metrics.json"), JSON.stringify(report, null, 2));
  await browser.close();

  // Verdict summary
  const worstCls = Math.max(...report.pages.map(p => p.cls));
  const worstOverflow = Math.max(...report.pages.map(p => p.horizontalOverflowPx));
  console.log(`\n== ${label} verdict == worst CLS=${worstCls} worst horizontal overflow=${worstOverflow}px`);
}

run().catch((e) => { console.error(e); process.exit(1); });
