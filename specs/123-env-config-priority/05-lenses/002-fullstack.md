# Lens 002 — Full Stack Developer

## SOLID / DRY plan

| Principle | Application |
|-----------|-------------|
| SRP | Pure `resolve_pdf_parser_choice`; routing gate separate; converters unchanged |
| OCP | New `auto` choice without rewriting Vision/EdgeParse converters |
| LSP | Runtime backend remains Vision\|EdgeParse; Auto only affects routing |
| ISP | Call sites take `ResolvedPdfParser`, not raw env |
| DIP | Processor depends on resolved DTO, not workspace/tenant structs |
| DRY | One Rust SSOT + one FE mirror; kill duplicated `or_else` chains |

## Touch points

- `edgequake-pdf` / `edgequake-core`: choice enum + resolver
- `pdf_upload/types.rs` + `helpers.rs`: apply tenant; set `allows_auto_route`
- `large_document_profile.rs` / `pdf_auto_routing.rs`: gate on Auto
- `fallback.rs`: Auto-only degrade
- `pending_doc_task_reconcile.rs`, reprocess, `/parse`, batch upload
- FE: `resolve-pdf-parser-backend.ts`, settings card, dropzone, admission, Replace

## Risks

- Breaking tests that assert auto-route on implicit Vision (`spec038_*`)
- OpenAPI / serde for `auto` string
- Tenant field backward compatible (None)

## Test hooks

See [08-test-protocol.md](../08-test-protocol.md).
