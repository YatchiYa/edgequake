# Task Completion Log — Work-with-Raphael page + LinkedIn demo access

Deployed to https://edgequake.com. Commit fd71d64ec, Pages run 39366901088 = success.

## 1. Lessons Learned
* The docs on the site are a tracked **symlink** `edgequake-website/src/content/docs/docs`
  -> `../../../../docs` (root), so doc edits are made to the tracked root `docs/`
  files and ship via the symlink on CI. No separate docs commit needed.
* The whole `edgequake-website/` tree and all `*.png` are gitignored but the site
  is force-added; any NEW page/component must be `git add -f`'d or CI omits it.
* Adding a 6th top-nav item overflowed the 768-1023px range (Get Started clipped);
  moving the desktop nav breakpoint md->lg (hamburger below lg) is the clean fix.
* A long hero headline that auto-wraps on mobile causes font-swap CLS (0.177);
  a deterministic hard line break (as on the homepage) returns CLS to 0.

## 2. What Went Wrong & Fixes
* New-page mobile CLS 0.177 from auto-wrapped headline -> shortened to a
  two-line hard-broken headline ("Build Graph-RAG with / EdgeQuake's creator.").
* Icon union type error on the engagements array -> typed it to IconName.
* 6-item nav crowding -> md:flex/md:hidden -> lg:flex/lg:hidden on nav + menu.
* a11y harness reported false low contrast in light theme (picks a decorative
  ancestor bg); confirmed real colors are ink-on-white (~19:1). Page reuses the
  already-AA-verified token system, so no new pairings.

## 3. Results (live-verified)
* New /work-with-raphael/ page: honest marketing, proof = EdgeQuake itself, one
  primary CTA (LinkedIn). CLS 0, 0 overflow, 0 defects, AA both themes.
* Demo access now routes through LinkedIn on /demo/, /contact/, and the docs
  (/docs/ and /docs/getting-started/).
* Nav "Collaborate" + footer "Work with Raphael" link the page; nav fits 390-1920px.
* astro check 0 errors; astro build 229 pages; Pages deploy success; all live
  URLs return 200 with the expected content.

## 4. Next Steps
* Optional: add a short, truthful bio/photo of Raphael if one is available.
* Optional: self-host fonts to guarantee first-paint CLS 0 for all users.
* Address repo Dependabot alerts (unrelated to this change).
