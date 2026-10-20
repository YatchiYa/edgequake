import pw from "/Users/raphaelmansuy/Github/03-working/edgequake/edgequake_webui/node_modules/playwright-core/index.js";
const { chromium } = pw;
import { mkdirSync, writeFileSync } from "node:fs";
import { join } from "node:path";

const OUT = "/Users/raphaelmansuy/Github/03-working/edgequake/specs/170-perfect-astra-design/e2e/screenshots/product";
mkdirSync(OUT, { recursive: true });
const USER = process.env.DEMO_USER;
const PASS = process.env.DEMO_PASS;
if (!USER || !PASS) {
  console.error("Set DEMO_USER and DEMO_PASS env vars (demo.edgequake.com credentials). Never hardcode.");
  process.exit(2);
}
const BASE = "https://demo.edgequake.com";

const browser = await chromium.launch();
const ctx = await browser.newContext({ viewport: { width: 1920, height: 1200 }, deviceScaleFactor: 2, colorScheme: "dark" });
const page = await ctx.newPage();

await page.goto(BASE + "/login", { waitUntil: "networkidle" });
await page.waitForTimeout(1000);
await page.locator('input:not([type="password"]):not([type="hidden"])').first().fill(USER);
await page.locator('input[type="password"]').first().fill(PASS);
await page.getByRole("button", { name: /sign in/i }).click();
await page.waitForTimeout(4000);
console.log("logged in:", await page.evaluate(() => location.pathname));

// Go to the knowledge graph
await page.getByText("Knowledge Graph", { exact: false }).first().click().catch(() => {});
await page.waitForTimeout(8000);
console.log("graph at", await page.evaluate(() => location.pathname), "canvas=", await page.evaluate(() => document.querySelectorAll("canvas").length));

const results = [];
async function snap(name, full = false) {
  const file = join(OUT, `${name}.png`);
  await page.screenshot({ path: file, fullPage: full });
  results.push({ name, file });
  console.log("  ✓", name);
}

// 1) Full overview (as rendered)
await snap("graph-overview");

// locate the graph canvas center for interactions
const box = await page.evaluate(() => {
  const cs = [...document.querySelectorAll("canvas")];
  if (!cs.length) return null;
  // pick the largest canvas
  let best = cs[0], area = 0;
  for (const c of cs) { const r = c.getBoundingClientRect(); const a = r.width * r.height; if (a > area) { area = a; best = c; } }
  const r = best.getBoundingClientRect();
  return { x: r.x + r.width / 2, y: r.y + r.height / 2, w: r.width, h: r.height, left: r.x, top: r.y };
});
console.log("canvas box:", JSON.stringify(box));

if (box) {
  // 2) Zoom in a couple steps for a denser, label-rich view
  await page.mouse.move(box.x, box.y);
  for (let i = 0; i < 3; i++) { await page.mouse.wheel(0, -260); await page.waitForTimeout(500); }
  await page.waitForTimeout(1500);
  await snap("graph-zoom");

  // 3) Click near center to select a node (highlights neighborhood / opens panel)
  await page.mouse.click(box.x, box.y);
  await page.waitForTimeout(1800);
  await snap("graph-node-selected");

  // 4) Reset zoom out a bit and click a dense area for a hero-worthy neighborhood
  for (let i = 0; i < 2; i++) { await page.mouse.wheel(0, 200); await page.waitForTimeout(400); }
  await page.mouse.click(box.left + box.w * 0.42, box.top + box.h * 0.45);
  await page.waitForTimeout(1800);
  await snap("graph-neighborhood");
}

writeFileSync(join(OUT, "graph-index.json"), JSON.stringify(results, null, 2));
await browser.close();
