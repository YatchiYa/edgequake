import pw from "/Users/raphaelmansuy/Github/03-working/edgequake/edgequake_webui/node_modules/playwright-core/index.js";
const { chromium } = pw;
import { mkdirSync } from "node:fs";
import { join } from "node:path";
const ROOT = "/Users/raphaelmansuy/Github/03-working/edgequake/specs/170-perfect-astra-design/e2e/screenshots/pages";
const path = process.argv[2] || "/work-with-raphael/";
const name = process.argv[3] || "work-with-raphael";
mkdirSync(ROOT, { recursive: true });
const VP = { desktop: { width: 1440, height: 900 }, mobile: { width: 390, height: 844 } };
const b = await chromium.launch();
for (const [vn, vp] of Object.entries(VP)) {
  for (const theme of ["light", "dark"]) {
    const ctx = await b.newContext({ viewport: vp, deviceScaleFactor: 2, colorScheme: theme });
    const p = await ctx.newPage();
    await p.addInitScript(`window.__cls=0;new PerformanceObserver(l=>{for(const e of l.getEntries())if(!e.hadRecentInput)window.__cls+=e.value}).observe({type:'layout-shift',buffered:true});`);
    await p.goto("http://localhost:4321" + path, { waitUntil: "load", timeout: 45000 });
    await p.evaluate(async()=>{await (document.fonts?document.fonts.ready:0);document.querySelectorAll('img[loading=lazy]').forEach(i=>i.loading='eager');for(let y=0;y<=document.body.scrollHeight;y+=350){window.scrollTo(0,y);await new Promise(r=>setTimeout(r,30))}window.scrollTo(0,0);});
    await p.waitForTimeout(700);
    const a = await p.evaluate(() => {
      const vw = innerWidth; const esc = [];
      document.querySelectorAll("main *,header *").forEach(el=>{const r=el.getBoundingClientRect();if(el.getAttribute("aria-hidden")==="true"||el.closest("[aria-hidden=true]"))return;if(el.closest("[class*=overflow-x-auto]"))return;if(r.width>0&&(r.right>vw+1.5||r.left<-1.5))esc.push(el.tagName+"."+(typeof el.className==="string"?el.className.split(" ")[0]:""));});
      const broken=[...document.images].filter(i=>!i.complete||i.naturalWidth===0).map(i=>i.src);
      return { cls:+window.__cls.toFixed(4), overflow: document.documentElement.scrollWidth-vw, escapes: esc.slice(0,6), broken };
    });
    await p.screenshot({ path: join(ROOT, `${name}-${vn}-${theme}.png`), fullPage: true });
    console.log(`${vn}/${theme}: CLS=${a.cls} overflow=${a.overflow}px escapes=${a.escapes.length} broken=${a.broken.length}${a.escapes.length?" "+a.escapes.join(","):""}`);
    await ctx.close();
  }
}
await b.close();
