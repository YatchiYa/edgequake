# e2e — SPEC-170 capture & verification harness

Measurable "visual inspection" (no eyeballing): CLS, overflow, contrast, motion.

## Scripts

| Script | Purpose |
|--------|---------|
| `metrics.mjs <label> [baseURL]` | Capture full-page screenshots **and** CLS / overflow / geometry (desktop+mobile × light+dark) → `screenshots/<label>/{*.png,metrics.json}` |
| `verify-a11y.mjs` | Gate G3/G4: contrast (composited), tap targets, no-JS + reduced-motion visibility |
| `measure-fonts.mjs` | Empirically measure `size-adjust`/line metrics for the font fallbacks (CLS fix) |
| `capture-product.mjs` | Authenticated product screens from demo.edgequake.com (env creds) |
| `capture-graph.mjs` | Multiple knowledge-graph states (overview/zoom/selection) |
| `capture.mjs` | Generic route capture (public demo + marketing baseline/after) |

## Run (marketing site must be served on :4321)

```bash
cd edgequake-website && npm run dev        # → http://localhost:4321
cd specs/170-perfect-astra-design/e2e
node metrics.mjs baseline                  # before
node metrics.mjs after                     # after each change
node verify-a11y.mjs
```

## Refresh product screenshots (credentials required — never hardcode)

```bash
DEMO_USER=… DEMO_PASS=… node capture-product.mjs
DEMO_USER=… DEMO_PASS=… node capture-graph.mjs
# then promote/resize chosen frames into edgequake-website/public/ (webp + png)
```

The demo uses single-use refresh-token rotation → each run logs in fresh; do not
persist `storageState` to disk. Session tokens / passwords must not be committed.

## Screenshots

- `screenshots/baseline/` — before-redesign (frozen reference)
- `screenshots/after/` — current redesign (regenerated each iteration)
- `screenshots/product/` — authentic demo.edgequake.com captures (graph set + dashboard/query/documents/members)
