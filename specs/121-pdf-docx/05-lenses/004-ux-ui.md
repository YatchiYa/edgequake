# Lens 004 — UX / UI Designer

## Stake

Users cannot distinguish “format not supported” from “upload broken” from “still converting”. #370 is partly a **language** failure.

## Principles

1. Dropzone states supported formats in plain language (not MIME jargon alone).
2. Rejected Office files get a **definitive** toast — not a spinner.
3. PDF progress: Uploading → Converting → Processing → Completed/Failed.
4. Failed convert shows actionable next step (check vision provider / retry), not “unsupported”.

## Error copy (target)

| Situation | Tone | Example |
|-----------|------|---------|
| DOCX/XLSX | Closed door | “Word and Excel files are not supported. Use PDF, Markdown, text, JSON, or an image.” |
| Oversize | Limit | “File is too large. Maximum size is 50MB.” |
| Bad PDF | File | “This file is not a valid PDF.” |
| Convert fail | Recoverable | “PDF uploaded, but conversion failed. Check the vision/LLM provider and retry.” |
| Proxy 413 | Ops | Surface status + “Ask your admin to raise the reverse-proxy body limit.” |

## Non-goals

- Redesigning Document Manager layout
- Adding Office icons that imply support

## Cross-refs

- UX spec: [../06-ux-ui-spec.md](../06-ux-ui-spec.md)
- Front designer: [005-front-designer.md](005-front-designer.md)
