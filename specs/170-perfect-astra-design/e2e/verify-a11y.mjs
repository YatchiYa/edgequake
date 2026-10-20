import pw from "/Users/raphaelmansuy/Github/03-working/edgequake/edgequake_webui/node_modules/playwright-core/index.js";
const { chromium } = pw;

const BASE = "http://localhost:4321";
const browser = await chromium.launch();

function luminance([r, g, b]) {
  const a = [r, g, b].map((v) => {
    v /= 255;
    return v <= 0.03928 ? v / 12.92 : Math.pow((v + 0.055) / 1.055, 2.4);
  });
  return 0.2126 * a[0] + 0.7152 * a[1] + 0.0722 * a[2];
}
function contrast(fg, bg) {
  const L1 = luminance(fg), L2 = luminance(bg);
  return (Math.max(L1, L2) + 0.05) / (Math.min(L1, L2) + 0.05);
}
function parseRGBA(s) {
  const m = s.match(/rgba?\(([^)]+)\)/);
  if (!m) return [0, 0, 0, 1];
  const p = m[1].split(",").map((n) => parseFloat(n));
  return [p[0], p[1], p[2], p[3] ?? 1];
}
function composite(fg, bg) {
  // fg over bg using fg alpha
  const a = fg[3];
  return [0, 1, 2].map((i) => Math.round(fg[i] * a + bg[i] * (1 - a)));
}
function parseRGB(s) {
  return parseRGBA(s).slice(0, 3);
}

// ---- G4a: no-JS visibility ----
{
  const ctx = await browser.newContext({ javaScriptEnabled: false, viewport: { width: 1440, height: 900 } });
  const page = await ctx.newPage();
  await page.goto(BASE + "/", { waitUntil: "load" });
  const vis = await page.evaluate(() => {
    const reveals = [...document.querySelectorAll(".reveal")];
    const hidden = reveals.filter((el) => getComputedStyle(el).opacity === "0").length;
    return { total: reveals.length, hidden };
  });
  console.log(`G4a no-JS: ${vis.total} reveals, ${vis.hidden} hidden → ${vis.hidden === 0 ? "PASS" : "FAIL"}`);
  await ctx.close();
}

// ---- G4b: reduced-motion visibility ----
{
  const ctx = await browser.newContext({ reducedMotion: "reduce", viewport: { width: 1440, height: 900 } });
  const page = await ctx.newPage();
  await page.goto(BASE + "/", { waitUntil: "load" });
  await page.waitForTimeout(600);
  const vis = await page.evaluate(() => {
    const reveals = [...document.querySelectorAll(".reveal")];
    const hidden = reveals.filter((el) => parseFloat(getComputedStyle(el).opacity) < 0.99).length;
    return { total: reveals.length, hidden };
  });
  console.log(`G4b reduced-motion: ${vis.total} reveals, ${vis.hidden} not-full-opacity → ${vis.hidden === 0 ? "PASS" : "FAIL"}`);
  await ctx.close();
}

// ---- G3a: contrast of key text tokens, light + dark ----
for (const theme of ["light", "dark"]) {
  const ctx = await browser.newContext({ colorScheme: theme, viewport: { width: 1440, height: 900 } });
  const page = await ctx.newPage();
  await page.goto(BASE + "/", { waitUntil: "load" });
  await page.waitForTimeout(400);
  const samples = await page.evaluate(() => {
    const bodyBg = getComputedStyle(document.body).backgroundColor;
    const pick = (sel) => {
      // first element of this class that actually has visible text
      const els = [...document.querySelectorAll(sel)];
      const el = els.find((e) => e.textContent.trim().length > 1 && e.offsetParent !== null);
      if (!el) return null;
      let bgEl = el, bg = null;
      while (bgEl) {
        const c = getComputedStyle(bgEl).backgroundColor;
        const a = c.match(/rgba?\(([^)]+)\)/);
        if (a) {
          const parts = a[1].split(",").map((n) => parseFloat(n));
          if ((parts[3] ?? 1) > 0.999) { bg = c; break; }
        }
        bgEl = bgEl.parentElement;
      }
      return { sel, color: getComputedStyle(el).color, bg: bg || bodyBg, text: el.textContent.trim().slice(0, 20) };
    };
    return [
      pick("h1"),
      pick("p.mkt-text-muted"),
      pick(".mkt-text-dim"),
      pick(".mkt-accent-text"),
      pick(".mkt-frame-bar span"),
    ].filter(Boolean);
  });
  console.log(`\nG3 contrast (${theme}):`);
  for (const s of samples) {
    const ratio = contrast(parseRGB(s.color), parseRGB(s.bg));
    const pass = ratio >= 4.5 ? "PASS" : ratio >= 3 ? "LARGE-ONLY" : "FAIL";
    console.log(`  ${s.sel.padEnd(16)} ratio=${ratio.toFixed(2)} ${pass}  (${s.text})`);
  }
  await ctx.close();
}

// ---- G3b: tap target sizes on mobile ----
{
  const ctx = await browser.newContext({ viewport: { width: 390, height: 844 } });
  const page = await ctx.newPage();
  await page.goto(BASE + "/", { waitUntil: "load" });
  await page.waitForTimeout(400);
  const small = await page.evaluate(() => {
    const out = [];
    document.querySelectorAll("header a, header button, main a[href], main button").forEach((el) => {
      const r = el.getBoundingClientRect();
      if (r.width > 0 && r.height > 0 && (r.height < 44 || r.width < 24)) {
        out.push({ tag: el.tagName.toLowerCase(), t: el.textContent.trim().slice(0, 18), w: Math.round(r.width), h: Math.round(r.height) });
      }
    });
    return out;
  });
  console.log(`\nG3 tap targets <44px high (mobile): ${small.length}`);
  small.slice(0, 15).forEach((s) => console.log(`  ${s.tag} "${s.t}" ${s.w}x${s.h}`));
  await ctx.close();
}

await browser.close();
