import pw from "/Users/raphaelmansuy/Github/03-working/edgequake/edgequake_webui/node_modules/playwright-core/index.js";
const { chromium } = pw;
import { mkdirSync, writeFileSync } from "node:fs";
import { join } from "node:path";

const OUT = "/Users/raphaelmansuy/Github/03-working/edgequake/specs/170-perfect-astra-design/e2e/screenshots/product";
mkdirSync(OUT, { recursive: true });
const USER = process.env.DEMO_USER;
const PASS = process.env.DEMO_PASS;
if (!USER || !PASS) { console.error("Set DEMO_USER and DEMO_PASS."); process.exit(2); }
const BASE = "https://demo.edgequake.com";

const browser = await chromium.launch();
const ctx = await browser.newContext({ viewport: { width: 1600, height: 1000 }, deviceScaleFactor: 2 });
const page = await ctx.newPage();

// Fresh login (one session; no storageState reuse due to token rotation).
await page.goto(BASE + "/login", { waitUntil: "networkidle" });
await page.waitForTimeout(900);
await page.locator('input:not([type="password"]):not([type="hidden"])').first().fill(USER);
await page.locator('input[type="password"]').first().fill(PASS);
await page.getByRole("button", { name: /sign in/i }).click();
await page.waitForTimeout(4000);
console.log("logged in at", await page.evaluate(() => location.pathname));

const results = [];
async function shot(name, { path, clickText, wait = 4000 } = {}) {
  if (path) await page.goto(BASE + path, { waitUntil: "networkidle", timeout: 60000 });
  if (clickText) await page.getByText(clickText, { exact: false }).first().click().catch(() => {});
  await page.waitForTimeout(wait);
  const info = await page.evaluate(() => ({
    path: location.pathname,
    login: /sign in to access/i.test(document.body.innerText),
    theme: document.documentElement.getAttribute("data-theme") || document.documentElement.className,
    canvas: document.querySelectorAll("canvas").length,
  }));
  const file = join(OUT, `${name}.png`);
  await page.screenshot({ path: file });
  results.push({ name, ...info, file });
  console.log(`✓ ${name} -> ${info.path} login=${info.login} theme=${info.theme} canvas=${info.canvas}`);
}

async function detectTheme() {
  return await page.evaluate(() => {
    const c = document.documentElement.className;
    if (/\bdark\b/.test(c)) return "dark";
    if (/\blight\b/.test(c)) return "light";
    return document.documentElement.getAttribute("data-theme") || "unknown";
  });
}

async function captureAll(suffix) {
  await shot(`dashboard-${suffix}`, { path: "/" });
  await shot(`documents-${suffix}`, { path: "/documents" });
  await shot(`query-${suffix}`, { path: "/query" });
  await shot(`members-${suffix}`, { path: "/members" });
  // graph: navigate home then click sidebar item
  await page.goto(BASE + "/", { waitUntil: "networkidle" });
  await page.waitForTimeout(1200);
  await shot(`graph-${suffix}`, { clickText: "Knowledge Graph", wait: 7000 });
}

// --- Pass 1: default theme ---
let theme = await detectTheme();
console.log("initial theme:", theme);
const first = theme === "light" ? "light" : "dark";
await captureAll(first);

// --- Toggle theme (robust: button may open a Light/Dark/System menu) ---
async function switchTheme(target) {
  const btn = page.getByRole("button", { name: /toggle theme|theme/i }).first();
  if (await btn.count().catch(() => 0)) {
    await btn.click().catch(() => {});
    await page.waitForTimeout(700);
    // If a menu opened, pick the target option.
    const opt = page.getByRole("menuitem", { name: new RegExp(target, "i") }).first();
    if (await opt.count().catch(() => 0)) {
      await opt.click().catch(() => {});
    } else {
      // maybe a plain toggle already flipped; try an option by text
      const t = page.getByText(new RegExp(`^${target}$`, "i")).first();
      if (await t.count().catch(() => 0)) await t.click().catch(() => {});
    }
    await page.waitForTimeout(1200);
  }
  const now = await detectTheme();
  console.log(`switchTheme(${target}) -> ${now}`);
  return now;
}

await page.goto(BASE + "/", { waitUntil: "networkidle" });
await page.waitForTimeout(1000);
const second = first === "dark" ? "light" : "dark";
await switchTheme(second);
await captureAll(second);

writeFileSync(join(OUT, "themes-index.json"), JSON.stringify(results, null, 2));
await browser.close();
console.log("done");
