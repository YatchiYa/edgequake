# Lens 001 — Product Owner

## Stake

Partners on Docker (v0.24.2+) report “PDF and DOCX broken; only JSON works” ([#370](https://github.com/raphaelmansuy/edgequake/issues/370)). That story collapses three product truths into one bug title and burns support trust.

## Outcome

| Priority | Outcome |
|----------|---------|
| P0 | Publish honest format matrix: MD/text/JSON/image/PDF yes; DOCX/Excel no |
| P0 | PDF admit+convert works in supported Docker topology (or clear env diagnosis) |
| P0 | FAQ/docs stop advertising DOCX as “Planned” |
| P1 | Error taxonomy so partners can self-triage (unsupported vs convert vs proxy) |
| Later | Optional Office converter SPEC — only after v1 matrix is green |

## Acceptance language

> “I can upload Markdown, text, JSON, images, and PDFs. If I drop a Word or Excel file, the product tells me those formats are not supported. If a PDF fails, I can tell whether admission or conversion failed.”

## Non-goals

- Promising DOCX/Excel parity with LightRAG in this release
- Marketing “all Office formats” until a funded follow-up SPEC ships

## Cross-refs

- WHY: [../00-why.md](../00-why.md)
- Acceptance: [../09-acceptance.md](../09-acceptance.md)
