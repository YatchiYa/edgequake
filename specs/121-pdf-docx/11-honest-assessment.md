# 11 — Honest Assessment

## What #370 got right

- JSON succeeding while other formats fail is a real **user-visible asymmetry**.
- Docker is the right environment class to investigate for PDF (pdfium + networking).
- Blocking ingestion for “documents people actually have” (PDF) is severity-high.

## What #370 got wrong / incomplete

| Claim | Assessment |
|-------|------------|
| DOCX should upload successfully | **Product false** for EdgeQuake v1 — not implemented; reject is correct |
| PDF and DOCX are the same bug | **False** — different code paths and product decisions |
| “Not uploading” for PDF | May mean admit failure **or** convert failure; issue lacks status codes / toasts / logs |

## Residual risks after SPEC-121 docs

1. **Docs/FAQ still drift** until P0 lands in the product tree.
2. **Reporter env** may still break PDF (proxy 413, vision down) after messaging fix.
3. **API-only formats** (CSV/HTML/…) remain invisible in WebUI — secondary confusion.
4. **Office demand** will return; without [12-office-future-study.md](12-office-future-study.md) clarity, FAQ may regress to “Planned”.

## Confidence

| Item | Confidence |
|------|------------|
| DOCX reject-by-design | High (code + tests) |
| PDF separate path | High |
| Specific Docker root cause for reporter | Low until logs / curl evidence |
| undocx as future DOCX adapter | Medium (crate young; needs security review) |

## Recommendation

1. Ship SPEC + GitHub comment now (honesty).  
2. Execute P0 messaging + P1 runbook.  
3. Ask reporter for exact failure artifact (toast text, HTTP status, document status, backend log snippet).  
4. Do **not** implement DOCX to “close the issue”.

## Cross-refs

- WHY: [00-why.md](00-why.md)
- Acceptance: [09-acceptance.md](09-acceptance.md)
