# zz-raw — Intake (#378)

> Not the contract. Contract starts at [00-why.md](00-why.md).

## Source

- **GH:** https://github.com/raphaelmansuy/edgequake/issues/378
- **Author:** ankursingh-devops
- **Opened:** 2026-08-13
- **Label:** bug
- **Env:** EdgeQuake 0.24.4, Docker, PostgreSQL, API v0.24.4

## Reporter text (verbatim summary)

- Multiple PDF files are not uploading successfully.
- Selecting multiple PDFs → process gets stuck → files are not uploaded.
- Expected: all selected PDFs upload and show correct status.
- Additional: files remain stuck in upload; users unable to upload multiple PDFs.

## Sibling signals (not identical)

| Issue | Claim | Spec |
|-------|-------|------|
| #361 / #365 | Bulk upload **too slow** | [SPEC-122](../122-implementation/) |
| #350 | Bulk WebUI ops / agtype | SPEC-098 / WebUI N× admits |
| #236 | Batch API contract | SPEC-014 |

## Agent pre-read (2026-08-16)

Code review: multi-select works; WebUI = N× `POST /documents/pdf` (cap 3); `/pdf/batch` exists but unused by WebUI; wake channel `send().await` can still block HTTP (F-091-19 gap); `spec350` e2e is MD-only.
