import pw from "/Users/raphaelmansuy/Github/03-working/edgequake/edgequake_webui/node_modules/playwright-core/index.js";
const { chromium } = pw;

const browser = await chromium.launch();
const ctx = await browser.newContext();
const page = await ctx.newPage();
await page.setContent(`<!doctype html><html><head>
<link rel="preconnect" href="https://fonts.googleapis.com">
<link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
<link href="https://fonts.googleapis.com/css2?family=Space+Grotesk:wght@400;500;600;700&family=IBM+Plex+Sans:wght@400;500;600;700&family=IBM+Plex+Mono:wght@400;500&display=swap" rel="stylesheet">
<style>.p{position:absolute;white-space:nowrap;font-size:100px;line-height:normal}</style></head><body>
  <span class="p" id="sgWeb" style="font-family:'Space Grotesk'">Build to ship Graph RAG 0123456789</span>
  <span class="p" id="sgFb"  style="font-family:Arial">Build to ship Graph RAG 0123456789</span>
  <span class="p" id="plexWeb" style="font-family:'IBM Plex Sans'">Build to ship Graph RAG 0123456789</span>
  <span class="p" id="plexFb"  style="font-family:Arial">Build to ship Graph RAG 0123456789</span>
</body></html>`, { waitUntil: "load" });
await page.evaluate(() => document.fonts.ready);
await page.waitForTimeout(1800);
const m = await page.evaluate(() => {
  const g = (id) => { const r = document.getElementById(id).getBoundingClientRect(); return { w: r.width, h: r.height }; };
  return { sgWeb: g("sgWeb"), sgFb: g("sgFb"), plexWeb: g("plexWeb"), plexFb: g("plexFb") };
});
function rep(name, web, fb) {
  const sa = web.w / fb.w;
  console.log(`${name}: size-adjust=${(sa * 100).toFixed(2)}%  webLine=${web.h} fbLine=${fb.h} (web advance ${web.w.toFixed(0)} / fb ${fb.w.toFixed(0)})`);
}
rep("Space Grotesk / Arial", m.sgWeb, m.sgFb);
rep("IBM Plex Sans / Arial", m.plexWeb, m.plexFb);
console.log("raw", JSON.stringify(m));
await browser.close();
