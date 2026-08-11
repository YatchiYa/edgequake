# Lens 003 — Database Expert

## Storage

| Layer | Field | Persistence |
|-------|-------|-------------|
| Workspace | `pdf_parser_backend` | Already in workspace metadata / column path |
| Tenant | `pdf_parser_backend` | **NEW** — prefer metadata key `"pdf_parser_backend"` (same string values) to avoid migration; optional first-class column later |
| Document / task | `pdf_parser_backend`, `pdf_parser_backend_explicit` (evolve to `allows_auto_route`) | Task payload JSON |

## Values

`vision` | `edgeparse` | `auto` | absent (`none` / null)

## Compatibility

- Existing rows: unset tenant → inherit env → Vision.
- Existing workspaces with `None`: after fix, resolve to Vision **without** auto-route (behavior change intentional — LAW-123-3).
- Workspaces with `Some(Vision)`: unchanged.

## Indexes / migrations

No index required. If first-class tenant column preferred: additive nullable column + backfill from metadata; v1 may use metadata-only.

## Integrity

Lineage `pdf_extraction_method` remains the audit of what ran; optional new field `pdf_parser_choice` / warning for Auto provenance.
