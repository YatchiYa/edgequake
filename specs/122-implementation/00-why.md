# 00 — Why SPEC-122

## Trigger

Partner reports ([#361](https://github.com/raphaelmansuy/edgequake/issues/361), [#365](https://github.com/raphaelmansuy/edgequake/issues/365)):

> When uploading multiple documents in a single batch, the entire upload and ingestion process takes significantly longer than expected.

Environments cited: Docker **v0.12.11** then confirmed **v0.24.1**, PostgreSQL.

Expected by reporter: concurrent/optimized processing; documents available soon after upload finishes.

## Product WHY

```ascii
  User expects: “I selected N files → they finish together soon”
       │
       ▼
  Product truth:
       │
       ├─ Upload (HTTP admit) can be bounded-parallel (WebUI ≤3)
       ├─ Ingest is capacity-governed (tenant lane + provider budget)
       ├─ Searchable = Insert complete (not HTTP 202)
       └─ Local Ollama defaults are intentionally near-serial
              │
              ▼
  Without honesty:
       • Partners file “bug” against intentional clamps
       • Raising concurrency without VRAM/rate headroom makes it worse
       • PDF vision cost is mistaken for “upload broken” or “bad parse”
```

## Two different truths inside one issue

| Claim in #361/#365 | Product truth | Engineering action |
|--------------------|---------------|--------------------|
| Upload takes too long | Admit is usually seconds; UI may conflate with Processing | Separate admit vs ingest timers in UX |
| Processing takes too long | Often true under LLM/vision law | Measure stages; tune only with budget |
| Should process concurrently | Partially — Docker 6-wide; local 1-wide | Publish concurrency SSOT; provider profiles |
| PDF quality causes slowness | Possible amplifier, not root of serial drain | Hypothesis H2/H3 + measurement |

## Gaps

| Artifact | Gap |
|----------|-----|
| Partner expectation | “Batch” implies multiplexed pipeline job — WebUI does N× single admits |
| FAQ / quick-start | Mentions local clamp but not Docker vs make matrix or docs/min SLO |
| UI status | Processing stays opaque; weak queue depth / ETA |
| SPEC-111 note | Measure-only; no full First-Principles pack |
| PDF path | Two tasks (convert→insert) under tenant=1 ⇒ ~2× serial slots per PDF |

## Success

1. Published concurrency SSOT: WebUI / Docker / make-local / cloud (LAW-122-7).
2. Reproduction Arms A/B/C with stage timings; H1–H5 accepted or rejected.
3. Phase A: honest UX + FAQ + measurement harness; no unbounded fan-out.
4. Phase B/C only if measurements justify (provider-aware raise / PDF cost).
5. #361/#365 updated with SPEC-122 link + numbers; close only when [09-acceptance.md](09-acceptance.md) green.

## Cross-refs

- Laws: [01-first-principles.md](01-first-principles.md)
- Code: [03-code-as-is.md](03-code-as-is.md)
- Repro: [10-reproduction.md](10-reproduction.md)
- PDF hypothesis: [12-pdf-quality-hypothesis.md](12-pdf-quality-hypothesis.md)
