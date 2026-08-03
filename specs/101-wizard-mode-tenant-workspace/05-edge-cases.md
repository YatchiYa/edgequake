# 05 — Edge Cases (SPEC-101)

| ID | Scenario | Mitigation | Test |
|----|----------|------------|------|
| EC-101-01 | Double-submit Finish | Disable button; initialize idempotent → 409 treated as success if already done | Rust + Playwright |
| EC-101-02 | Refresh mid-wizard | sessionStorage draft for non-secret fields; password never persisted | unit `wizard-draft-storage` (implemented Wave 6) |
| EC-101-03 | Bootstrap env already set | Skip Admin step; `bootstrap_admin_configured=true` | Rust status |
| EC-101-04 | Auth disabled / dev mode | No Admin step; tenant/workspace wizard only | status DTO |
| EC-101-05 | Server defaults missing | Advanced required; validation copy | Playwright |
| EC-101-06 | Tenant auto-creates Default Workspace | Workspace step PATCHes that workspace | create flow |
| EC-101-07 | Upgrade DB with `…0002` Default | `needs_setup=false` | Rust |
| EC-101-08 | Weak password | Reuse existing user password validation | API 400 |
| EC-101-09 | Network fail on finalize | Toast; stay on Review; retry safe | Playwright soft |
| EC-101-10 | `EDGEQUAKE_PROVISION_DEFAULTS=true` | Silent defaults for demos | env gate unit |
| EC-101-11 | Slug collision | Surface API error on Review | Playwright |
| EC-101-12 | Close dialog mid-create | Discard draft or keep sessionStorage; no partial tenant if not finished | LAW-101-4 |
| EC-101-13 | Long tenant+workspace names in header | Two-line truncate + full `title`/`aria-label` | unit + Playwright |
| EC-101-14 | Many tenants bury Workspaces | Workspaces group first; tenant select keeps popover open | Playwright `spec101-context-selector` |
| EC-101-15 | Workspace missing after tenant switch | Line 2 shows “Select workspace”; never omit workspace row | Playwright |
| EC-101-16 | Docs > 0 + LLM/vision change | Review Impact warns; post-save `pendingRebuild.extraction/vision` | Playwright reconfigure |
| EC-101-17 | Docs > 0 + embedding change | Review Impact warns; `pendingRebuild.embeddings` | unit + Playwright |
| EC-101-18 | Zero documents | Soft note on Review; no rebuild urgency | unit diff |
| EC-101-19 | Reset to server defaults | Clear overrides via `""`; Review shows Server default | unit `model-payload` |
| EC-101-20 | Language change remaps entity presets (SPEC-096 L6) | Reuse `applyExtractionLanguageToEntityTypes` | Playwright spec096 |
| EC-101-21 | Dirty cancel on reconfigure | LAW-101-10 confirm; draft keep/discard | Playwright soft |
| EC-101-22 | Refresh mid-reconfigure | sessionStorage draft keyed by `workspaceId` | unit draft storage |
| EC-101-23 | Network fail on PUT | Toast; stay on Review; retry safe | Playwright soft |
| EC-101-24 | Double-submit Apply | Disable footer while pending | Playwright |
| EC-101-25 | Stale workspace / 404 | Close wizard; toast; refetch | soft |
| EC-101-26 | Deeplink `/w/[slug]/workspace` | Same wizard; full field parity | Playwright |
| EC-101-27 | No-op Apply (no diffs) | Disable Apply or short-circuit without rebuild banners | unit + Playwright |
