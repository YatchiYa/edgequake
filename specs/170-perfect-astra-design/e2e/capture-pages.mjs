// Capture every marketing page × desktop/mobile × light/dark into screenshots/pages/.
import pw from "/Users/raphaelmansuy/Github/03-working/edgequake/edgequake_webui/node_modules/playwright-core/index.js";
const { chromium } = pw;
import { mkdirSync, writeFileSync } from "node:fs";
import { join } from "node:path";

const ROOT =
  "/Users/raphaelmansuy/Github/03-working/edgequake/specs/170-perfect-astra-design/e2e/screenshots/pages";
mkdirSync(ROOT, { recursive: true });
const BASE = process.argv[2] || "http://127.0.0.1:4321";

const ROUTES = [
  ["home", "/"],
  ["demo", "/demo/"],
  ["ecosystem", "/ecosystem/"],
  ["enterprise", "/enterprise/"],
  ["contact", "/contact/"],
  ["work-with-raphael", "/work-with-raphael/"],
  ["tutorial-index", "/tutorial/"],
  ["tutorial-chapter", "/tutorial/01-what-this-is/"],
  ["404", "/this-page-does-not-exist/"],
];

const VP = {
  desktop: { width: 1440, height: 900 },
  mobile: { width: 390, height: 844 },
};

async function run() {
  const browser = await chromium.launch();
  const index = [];
  for (const [name, path] of ROUTES) {
    for (const [vn, vp] of Object.entries(VP)) {
      for (const theme of ["light", "dark"]) {
        const ctx = await browser.newContext({
          viewport: vp,
          deviceScaleFactor: 1,
          colorScheme: theme,
        });
        const page = await ctx.newPage();
        await page.addInitScript((t) => {
          localStorage.setItem("starlight-theme", t);
          document.documentElement.setAttribute("data-theme", t);
        }, theme);
        const resp = await page.goto(BASE + path, {
          waitUntil: "load",
          timeout: 45000,
        });
        await page.evaluate(async () => {
          await (document.fonts ? document.fonts.ready : 0);
          document
            .querySelectorAll("img[loading=lazy]")
            .forEach((i) => (i.loading = "eager"));
          document
            .querySelectorAll("[data-reveal]")
            .forEach((el) => el.classList.add("is-visible"));
          for (let y = 0; y <= document.body.scrollHeight; y += 500) {
            scrollTo(0, y);
            await new Promise((r) => setTimeout(r, 20));
          }
          scrollTo(0, 0);
        });
        await page.waitForTimeout(400);
        const file = `${name}-${vn}-${theme}.png`;
        await page.screenshot({ path: join(ROOT, file), fullPage: true });
        index.push({
          name,
          path,
          vn,
          theme,
          status: resp?.status() ?? 0,
          file,
        });
        console.log(`✓ ${file}`);
        await ctx.close();
      }
    }
  }
  writeFileSync(join(ROOT, "index.json"), JSON.stringify(index, null, 2));
  await browser.close();
}

run().catch((e) => {
  console.error(e);
  process.exit(1);
});
