# Lens 001 — Product Owner

## Stake

Partners and users treat Query answers as auditable. A confident answer with a
wrong page link destroys trust faster than “I don’t know.”

## Outcome

| Priority | Outcome |
|----------|---------|
| P0 | Inline answer links show **document name + page** from storage |
| P0 | Click opens that document at that page (no hallucinated page in href) |
| P0 | Same behavior on Query, Chat, API, MCP |
| P1 | Claim-level NLI / DeepCitation-style proof |
| Later | Pixel bbox highlight |

## Acceptance language

> “When I ask a question, the answer includes links like `Report.pdf, p.4`.
> Clicking opens Report.pdf on page 4 and selects the cited chunk. The system
> never invents a page number in those links.”

## Non-goals (v1)

- Marketing “100% hallucination-free prose”
- Changing Acc gold scoring
- Entity-level `page=` fields

## Cross-refs

- Acceptance: [../10-acceptance.md](../10-acceptance.md)
- WHY: [../00-why.md](../00-why.md)
