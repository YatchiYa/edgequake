# 11 — Honest Assessment

## What is solid

- Marker SSOT already production-proven for chunking and MM assets.
- Deeplink `?page=` path exists (SPEC-033/142).
- Continuous stack is a well-known react-pdf pattern.

## What is hard

| Hard part | Why |
|-----------|-----|
| Bidirectional sync | Feedback loops without a strict driver lock |
| Windowing | Placeholder heights must match real page metrics under zoom |
| Virtualized MD | Anchors may unmount; MD→PDF may soft-fail |
| Dual viewer shells | Easy to leave dialog half-wired |

## Residual risk

- Scanned PDFs with weak extraction may have markers but sparse MD sections —
  sync will jump to thin sections (honest: still correct page).
- Very large PDFs (>100 pages) may still hitch on first scroll past window.

## What we will not claim

- “Perfect paragraph alignment.”
- “Works without page markers.”
- “Sync without e2e attribute contracts.”

## Cross-refs

- WHY: [00-why.md](00-why.md)
- Acceptance: [10-acceptance.md](10-acceptance.md)
