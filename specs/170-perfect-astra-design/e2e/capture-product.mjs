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
const ctx = await browser.newContext({ viewport: { width: 1600, height: 1000 }, deviceScaleFactor: 2, colorScheme: "dark" });
const page = await ctx.newPage();

await page.goto(BASE + "/login", { waitUntil: "networkidle" });
await page.waitForTimeout(1000);
await page.locator('input:not([type="password"]):not([type="hidden"])').first().fill(USER);
await page.locator('input[type="password"]').first().fill(PASS);
await page.getByRole("button", { name: /sign in/i }).click();
await page.waitForTimeout(4000);
console.log("logged in at", await page.evaluate(() => location.pathname));

const results = [];
async function shot(name, { path, clickText, wait = 4000, fullPage = false } = {}) {
  if (path) await page.goto(BASE + path, { waitUntil: "networkidle", timeout: 60000 });
  if (clickText) await page.getByText(clickText, { exact: false }).first().click().catch((e) => console.log("click fail", e));
  await page.waitForTimeout(wait);
  const info = await page.evaluate(() => ({
    path: location.pathname,
    login: /sign in to access/i.test(document.body.innerText),
    svg: document.querySelectorAll("svg").length,
    canvas: document.querySelectorAll("canvas").length,
  }));
  const file = join(OUT, `${name}.png`);
  await page.screenshot({ path: file, fullPage });
  results.push({ name, ...info, file });
  console.log(`✓ ${name} -> ${info.path} login=${info.login} svg=${info.svg} canvas=${info.canvas}`);
  return info;
}

await shot("dashboard", { path: "/" });
await shot("documents", { path: "/documents" });
await shot("query", { path: "/query" });
await page.goto(BASE + "/", { waitUntil: "networkidle" });
await page.waitForTimeout(1500);
await shot("graph", { clickText: "Knowledge Graph", wait: 7000 });
await shot("members", { path: "/members" });

writeFileSync(join(OUT, "capture-index.json"), JSON.stringify(results, null, 2));
await browser.close();
