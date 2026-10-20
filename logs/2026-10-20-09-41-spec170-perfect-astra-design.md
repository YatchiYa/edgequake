# Task Completion Log — SPEC-170 Perfect Astra Design

Redesign of the published EdgeQuake marketing site (Astro → "Astra") homepage.
Spec set: [specs/170-perfect-astra-design](../specs/170-perfect-astra-design).

## 1. Lessons Learned
* "Astra" = the Astro marketing site (`edgequake-website/`), not a component.
* Without an image-viewer tool, "visual inspection" must be **measurable**: CLS via
  PerformanceObserver, overflow scans, composited contrast, tap sizes, motion states.
  This is more rigorous than eyeballing and avoids AI slop.
* The dark-mode CLS (0.023) root cause was web-font **swap reflow**; because text
  uses explicit line-heights, `size-adjust` on metric-matched fallbacks (Georgia/Arial,
  values measured empirically) was the decisive fix → CLS ≈ 0.
* demo.edgequake.com interior is auth-walled with **single-use refresh-token
  rotation**; all captures must run in one fresh-login session (no `storageState` reuse).
* DRY/SOLID on a static site = one token source + `SectionHeader`/`Icon`/`Reveal`/
  `Section`/`Button` primitives; sections become data + composition only.

## 2. What Went Wrong & Fixes
* `playwright-core` is CommonJS → `import pw from …; const { chromium } = pw`.
* Reused `storageState` logged out (token rotation) → single-session capture script.
* Magic link was a **registration** invite (not 1-click) → scripted account creation,
  then login with created creds for all captures.
* Initial a11y harness false-FAILed muted contrast (empty element + non-composited
  alpha) → picker now selects text-bearing elements and composites alpha.
* Real contrast misses: frame-bar URL label (dark 3.08) and `--mkt-text-dim` dark
  (~4.1) → switched labels to `mkt-text-muted`, bumped dim to `#7e7e87` → all AA.
* Secrets hygiene: removed `storage-state.json`, scrubbed demo password/username from
  capture scripts (env-only), deleted invite-flow screenshots (PII).

## 3. Results (gates)
* CLS worst **0.0229 → 0.001** (≈0) across desktop/mobile × light/dark.
* Horizontal overflow **0 px**; desktop overflowers 1 → 0.
* Contrast **AA** both themes; no-JS + reduced-motion content visible.
* `astro check` 0 errors/0 warnings; `astro build` 228 pages.
* Product imagery: authentic demo.edgequake.com graph set (overview/zoom/selection)
  + dashboard/query, promoted to `public/` as webp+png with intrinsic dims (no image CLS).

## 4. Next Steps
* Optional: self-host fonts (`font-display: optional`) to drop the third-party
  request entirely and guarantee CLS 0 on first paint for all users.
* Optional: bump header icon buttons to 44 px (AAA 2.5.5) for mobile ergonomics.
* Optional: refresh `og-default.png` to match the new hero; add analytics `data-*` wiring.
* Refresh interior product screenshots periodically via `e2e/capture-product.mjs`
  (DEMO_USER/DEMO_PASS) as the product evolves.
