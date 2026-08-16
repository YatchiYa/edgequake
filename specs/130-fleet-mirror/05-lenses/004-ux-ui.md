# Lens 004 — UX / UI

## User-visible bug

| Signal today | After SPEC-130 |
|--------------|----------------|
| Document chip **Failed** after dense KG merge | **Completed** / indexed when spine+fleet succeed |
| Error body blames entity spine | Error (if any) names relationship identity / miss keys |
| Reprocess loops forever Failed | Reprocess succeeds for identity-class failures |
| SQL has edges; UI still Failed | UI matches successful persist |

## UX contract

1. Failed means a true integrity / provider failure — not a self-inflicted identity re-lookup.
2. Miss samples in logs remain useful (`SRC->TGT:TYPE`) for support.
3. No new status vocabulary; use existing Failed / Completed / processing stages.

## Non-goals

- New Documents list filters for “fleet mirror.”
- Surfacing raw UUIDs in the WebUI by default.

## Cross-refs

- UX spec: [../06-ux-ui-spec.md](../06-ux-ui-spec.md)
- PO: [001-product-owner.md](001-product-owner.md)
