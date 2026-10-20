# Task Completion Log — SPEC-170 · frontend-design skill + typography overhaul

Applied Anthropic's **frontend-design** skill to the EdgeQuake marketing homepage,
replaced the disliked typography, removed AI-generated "tells," and added
theme-aware product screenshots captured from demo.edgequake.com in light + dark.

## 1. Lessons Learned
* The installed skill ([.github/skills/frontend-design](../.github/skills/frontend-design/SKILL.md))
  named the exact tells our prior pass had: ALL-CAPS tracked eyebrows, `→` on
  buttons, middle-dot meta strings, single-phrase headline color accent, the
  uniform SaaS-card kit, fade-up-on-every-section, and the generic high-contrast
  **serif** display pairing (the font the client hated).
* "Spend boldness in one place": the knowledge graph is now the one loud element;
  motion collapsed from a reveal on every band to a single hero reveal.
* Subject-grounded type: Space Grotesk (engineered grotesque) + IBM Plex Sans
  (infrastructure heritage) reads like systems software, unlike Source Serif/Sans.
* Metric-matched `size-adjust` fallbacks must be re-measured per font — new values
  (SG 103.77%, Plex 101.23% vs Arial) kept CLS ≈0 through the font change.
* Demo uses single-use refresh-token rotation → capture both themes in ONE session
  (login, shoot dark, toggle the "Toggle theme" menu to dark/light, shoot again).

## 2. What Went Wrong & Fixes
* First theme-capture pass produced two light sets (demo defaulted to light; the
  naive toggle click missed the menu). Fixed by opening the Toggle-theme menu and
  selecting the Dark menuitem, verifying `<html class>` flipped to `dark`.
* Inspector flagged lazy product images as `broken-image` on tall viewports →
  forced all `loading=lazy` images eager + `decode()` before capture → 0 defects.
* Removed now-unused assets (product-hero.png, single-theme graph/dashboard/query)
  and dead primitives (Badge, Stat) after refactor.

## 3. Results (gates, local site :4321)
* Typography: Space Grotesk + IBM Plex Sans verified loaded; headline monochrome.
* Tells on homepage (non-code): `→` 0, middots 0, ALL-CAPS eyebrows 0; em-dashes
  reduced 12→8 (legitimate prose only).
* CLS worst **0.0004**; horizontal overflow **0 px**; **0 visual defects** across
  4 viewports × 2 themes ([e2e/inspect.mjs](../specs/170-perfect-astra-design/e2e/inspect.mjs)).
* Contrast AA in light + dark (h1 16–20, muted 7.1–7.6, dim 4.8–5.6, accent
  7.6–9.7, button 5.19). no-JS + reduced-motion content visible.
* Product screenshots: demo.edgequake.com captured in **light and dark** (graph,
  dashboard, query, documents, members) and wired as **theme-aware** images on the
  site (dark shot on dark theme, light shot on light theme) with no CLS.
* `astro check` 0 errors; `astro build` 228 pages.

## 4. Next Steps
* Optional: self-host Space Grotesk + IBM Plex Sans (drop third-party request,
  guarantee first-paint CLS 0 offline).
* Optional: distinct graph *states* per theme for hero vs showcase (currently the
  same theme-aware graph in both).
* Refresh demo shots via `e2e/capture-themes.mjs` (DEMO_USER/DEMO_PASS) as the
  product evolves.
