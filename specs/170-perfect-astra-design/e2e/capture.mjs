// Playwright capture harness for SPEC-170 Perfect Astra Design.
// Usage: node capture.mjs <configName>
// Requires chromium from the webui playwright install.
import pw from "/Users/raphaelmansuy/Github/03-working/edgequake/edgequake_webui/node_modules/playwright-core/index.js";
const { chromium } = pw;
import { mkdirSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";

const ROOT = "/Users/raphaelmansuy/Github/03-working/edgequake/specs/170-perfect-astra-design/e2e/screenshots";

const VIEWPORTS = {
  desktop: { width: 1440, height: 900 },
  mobile: { width: 390, height: 844 },
};

/** @type {Record<string, {baseURL:string, outDir:string, routes:{name:string,path:string,fullPage?:boolean,wait?:number,viewport?:keyof typeof VIEWPORTS}[]}>} */
const CONFIGS = {
  demo: {
    baseURL: "https://demo.edgequake.com",
    outDir: join(ROOT, "product"),
    routes: [
      { name: "home", path: "/", fullPage: false },
      { name: "documents", path: "/documents", fullPage: false },
      { name: "query", path: "/query", fullPage: false },
      { name: "knowledge", path: "/knowledge", fullPage: false },
      { name: "login", path: "/login", fullPage: false },
    ],
  },
  baseline: {
    baseURL: "http://localhost:4321",
    outDir: join(ROOT, "baseline"),
    routes: [
      { name: "home-desktop", path: "/", fullPage: true },
      { name: "home-mobile", path: "/", fullPage: true, viewport: "mobile" },
    ],
  },
  after: {
    baseURL: "http://localhost:4321",
    outDir: join(ROOT, "after"),
    routes: [
      { name: "home-desktop", path: "/", fullPage: true },
      { name: "home-mobile", path: "/", fullPage: true, viewport: "mobile" },
    ],
  },
};

async function run() {
  const configName = process.argv[2] || "demo";
  const cfg = CONFIGS[configName];
  if (!cfg) throw new Error(`Unknown config: ${configName}`);
  mkdirSync(cfg.outDir, { recursive: true });

  const browser = await chromium.launch();
  const results = [];
  for (const route of cfg.routes) {
    const vp = VIEWPORTS[route.viewport || "desktop"];
    const ctx = await browser.newContext({
      viewport: vp,
      deviceScaleFactor: 2,
      colorScheme: "light",
    });
    const page = await ctx.newPage();
    const url = cfg.baseURL + route.path;
    let status = 0;
    try {
      const resp = await page.goto(url, { waitUntil: "networkidle", timeout: 45000 });
      status = resp ? resp.status() : 0;
      await page.waitForTimeout(route.wait ?? 1200);
      const file = join(cfg.outDir, `${route.name}.png`);
      mkdirSync(dirname(file), { recursive: true });
      await page.screenshot({ path: file, fullPage: route.fullPage ?? false });
      const title = await page.title();
      results.push({ name: route.name, url, status, title, file });
      console.log(`✓ ${route.name} [${status}] ${title}`);
    } catch (err) {
      results.push({ name: route.name, url, status, error: String(err) });
      console.log(`✗ ${route.name} ${err}`);
    }
    await ctx.close();
  }
  await browser.close();
  writeFileSync(join(cfg.outDir, "index.json"), JSON.stringify(results, null, 2));
}

run().catch((e) => {
  console.error(e);
  process.exit(1);
});
