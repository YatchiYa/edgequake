# LENS — UX / UI Designer (SPEC-101)

## Progressive disclosure

| Layer | Content |
|-------|---------|
| Step | One decision |
| Card | Server defaults (always visible on model step) |
| Advanced | Provider select + searchable model (no chip storm) |
| Review | Confirm before mutate; Edit jump-links to prior steps |

## Context selector (LAW-101-11)

| Rule | Detail |
|------|--------|
| Recognition | Header chip is **one line**: `Tenant — Workspace` |
| Truncation | End-biased `smartTruncate` per side; full names in `title` / `aria` / `data-full-name` |
| Popover order | **1 · Organization** → **2 · Workspace** |
| Interaction | Tenant select keeps open + scrolls to workspaces; workspace select closes |
| Copy | Search “organizations and workspaces”; empty “No matches” |
| Scan | Workspace slug secondary; selected row accent; no Current strip |

```ascii
Trigger (always visible)
┌──────────────────────────────┐
│ 📁 Acme — Research         ▾ │
└──────────────────────────────┘
```

## NN/g + Aug 2026 SaaS checklist

- **Recognition over recall** — show actual model IDs  
- **Error prevention** — inline validation + `aria-invalid` (not Next-disabled alone)  
- **User control** — Back always; Cancel confirms if dirty (create); first-run non-dismissible  
- **Redundant entry** — sessionStorage draft for non-secrets (LAW-101-9)  
- **Aesthetic minimalism** — no provider chip storm on happy path  
- **Progress honesty** — `Step N of M` + live region + `aria-valuetext`

## A11y

- Dialog / step `aria-labelledby` = step title  
- Progress: `aria-valuenow` / `aria-valuemax` / `aria-valuetext` + polite live region  
- Focus move to step title on navigation  
- Password fields autocomplete `new-password`; never in draft storage  
- Inline errors use `role="alert"`

## Viewport budget (LAW-101-8)

Primary surfaces captured at **1440 / 768 / 375**; dialog height ≤ viewport.

## Copy SSOT (EN defaults)

| Step       | Title                  | Subtitle                                           |
| ------------| ------------------------| ----------------------------------------------------|
| Admin      | Create admin account   | This password secures your EdgeQuake instance.     |
| Tenant     | Name your organization | Tenants isolate workspaces and data.               |
| Models     | Confirm AI models      | Server defaults apply unless you override.         |
| Workspace  | Name your workspace    | Documents and extractions knowledge live here.     |
| Extraction | Extraction preferences | Language and entity types for the knowledge graph. |
| Review     | Review and create      | Nothing is saved until you confirm.                |
| Doc parse  | Document parsing       | Choose how PDFs are converted to text.             |
| Reconfig R | Review and apply       | Confirm changes. Rebuild may be required for existing docs. |

## Reconfigure Workspace (LAW-101-12)

| Rule | Detail |
|------|--------|
| Entry | Workspace header **Edit Configuration** opens wizard (page stays read-only) |
| Prefill | Current workspace values as defaults (keep-or-change) |
| Steps | Models → Document parsing → Extraction → Review + Impact |
| Impact | Disclose rebuild embeddings / KG / vision when models change and docs > 0 |
| Finish | Footer primary = **Apply** (not Create) |
| Draft | sessionStorage key includes workspace id |
