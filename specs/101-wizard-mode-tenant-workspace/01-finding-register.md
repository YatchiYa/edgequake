# 01 — Finding Register (SPEC-101)

| ID | Severity | Finding | Target fix |
|----|----------|---------|------------|
| F-101-01 | P0 | Triple create-dialog implementations diverge (required models, ID format, vision) | LAW-101-1 wizard shell |
| F-101-02 | P0 | Tenant create mounts 3× provider chip bars (cognitive overload) | LAW-101-3 defaults-first step |
| F-101-03 | P0 | Server defaults not explicit on tenant create; vision often missing from summary | LAW-101-2 ServerDefaultsCard |
| F-101-04 | P0 | Silent `ensure_defaults` on every boot; no guided admin password on fresh install | LAW-101-4 /setup/initialize |
| F-101-05 | P1 | `needsOnboarding` dead in Zustand store | Wire from GET /setup/status |
| F-101-06 | P1 | Orphan `TenantWorkspaceSelector` maintains unused create UX | Delete |
| F-101-07 | P1 | Guard uses legacy `ModelSelector` (`provider:model`) vs slash IDs | Unified slash payload helpers |
| F-101-08 | P1 | `WorkspaceCreateModelSection` returns null while loading (flash) | Skeleton |
| F-101-09 | P2 | Post-tenant-create auto Default Workspace can collide with named workspace step | PATCH auto workspace |
| F-101-10 | P2 | E2E soft-skips when create buttons missing | Hard gates in spec101-* |
| F-101-11 | P1 | Wizard drafts lost on refresh (no sessionStorage) | LAW-101-9 draft storage |
| F-101-12 | P1 | First-run Cancel/X look dismissible but noop | LAW-101-10 hide Cancel + close |
| F-101-13 | P1 | Create cancel resets without dirty confirm | LAW-101-10 AlertDialog |
| F-101-14 | P1 | No live region / step `aria-labelledby` for progress | Wave 6 a11y shell |
| F-101-15 | P1 | Next disabled only — no inline `aria-invalid` errors | Step validation messages |
| F-101-16 | P2 | Slug hint under description; review has no Edit jump-links | workspace-basics + review |
| F-101-17 | P1 | Evidence after PNGs / multi-viewport QC missing | LAW-101-8 capture spec |
| F-101-18 | P1 | Soft e2e: first-run API-only; tenant create open-only; defaults no regex | Harden spec101-* |
| F-101-19 | P1 | Header trigger truncates to ambiguous single line — workspace name invisible | LAW-101-11 two-line chip |
| F-101-20 | P1 | Tenant select closes popover / buries Workspaces below long tenant list | LAW-101-11 keep-open + workspaces-first |
| F-101-21 | P2 | Search/empty copy says “workspaces” while listing tenants | LAW-101-11 copy fix |
| F-101-22 | P0 | Workspace inline edit mode duplicates create-wizard concerns; no guided Impact Review | LAW-101-12 reconfigure wizard |
| F-101-23 | P1 | Deeplink `/w/[slug]/workspace` missing vision/PDF edit parity | LAW-101-12 same wizard entry |
| F-101-24 | P1 | PDF parser absent from guided create/reconfigure flows | DocumentParsingStep on reconfigure |
| F-101-25 | P1 | Model/extraction changes lack rebuild consequence disclosure before save | Review Impact + pendingRebuild |
