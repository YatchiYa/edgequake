// Empirically measure metric-matched fallback overrides so the web-font swap
// causes ZERO reflow (CLS). We measure advance width + normal line-box height for
// the web font vs the fallback at identical size, then compute:
//   size-adjust      = webAdvance / fallbackAdvance
//   ascent-override  = webAscent  / (fallbackEm * sizeAdjust)   (via line-box)
// We derive line metrics from measured normal line height.
import pw from "/Users/raphaelmansuy/Github/03-working/edgequake/edgequake_webui/node_modules/playwright-core/index.js";
const { chromium } = pw;

const browser = await chromium.launch();
const ctx = await browser.newContext();
const page = await ctx.newPage();
await page.setContent(`<!doctype html><html><head>
<link rel="preconnect" href="https://fonts.googleapis.com">
<link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
<link href="https://fonts.googleapis.com/css2?family=Source+Sans+3:wght@400;600;700&family=Source+Serif+4:opsz,wght@8..60,600;8..60,700&display=swap" rel="stylesheet">
<style>
  .probe{position:absolute;white-space:nowrap;font-size:100px;line-height:normal;}
</style></head><body>
  <span class="probe" id="serifWeb" style="font-family:'Source Serif 4',serif">Build to ship Graph RAG 0123456789</span>
  <span class="probe" id="serifFb"  style="font-family:Georgia,serif">Build to ship Graph RAG 0123456789</span>
  <span class="probe" id="sansWeb"  style="font-family:'Source Sans 3',sans-serif">Build to ship Graph RAG 0123456789</span>
  <span class="probe" id="sansFb"   style="font-family:Arial,sans-serif">Build to ship Graph RAG 0123456789</span>
</body></html>`, { waitUntil: "load" });

await page.evaluate(() => document.fonts.ready);
await page.waitForTimeout(1500);

const m = await page.evaluate(() => {
  const g = (id) => {
    const el = document.getElementById(id);
    const r = el.getBoundingClientRect();
    return { w: r.width, h: r.height };
  };
  return { serifWeb: g("serifWeb"), serifFb: g("serifFb"), sansWeb: g("sansWeb"), sansFb: g("sansFb") };
});

function report(name, web, fb) {
  const sizeAdjust = web.w / fb.w;          // scale fallback so advance matches web
  // After size-adjust, fallback em = 100*sizeAdjust. Normal line-box scales too.
  const fbAdjustedH = fb.h * sizeAdjust;
  console.log(`\n${name}`);
  console.log(`  web   : advance=${web.w.toFixed(1)} normalLine=${web.h.toFixed(1)}`);
  console.log(`  fb    : advance=${fb.w.toFixed(1)} normalLine=${fb.h.toFixed(1)}`);
  console.log(`  size-adjust = ${(sizeAdjust * 100).toFixed(2)}%`);
  console.log(`  fb line after size-adjust = ${fbAdjustedH.toFixed(1)} (web ${web.h.toFixed(1)}; ratio ${(web.h / fbAdjustedH).toFixed(3)})`);
  return sizeAdjust;
}

report("SERIF  (Source Serif 4 vs Georgia)", m.serifWeb, m.serifFb);
report("SANS   (Source Sans 3 vs system-ui)", m.sansWeb, m.sansFb);
console.log("\nraw:", JSON.stringify(m));
await browser.close();
