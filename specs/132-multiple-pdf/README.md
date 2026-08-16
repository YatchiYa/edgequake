# SPEC-132 — Multiple PDF Upload (#378)

> **Mission:** Make multi-PDF selection **admit honestly** — every selected PDF either receives a durable `task_id` with list presence, or a per-file terminal error — without confusing upload failure with capacity-bound processing ([#361](https://github.com/raphaelmansuy/edgequake/issues/361) / SPEC-122).
>
> **Trigger:** [GitHub #378](https://github.com/raphaelmansuy/edgequake/issues/378) — Docker v0.24.4: multiple PDFs stuck / not uploaded.

## Short verdict

| Layer | Finding |
|-------|---------|
| Symptom | Multi-PDF select → stuck in upload; files appear not uploaded |
| Classification | **Admit honesty + UX isolation + test gap** — not “multi-select missing” |
| WebUI path | N× concurrent `POST /api/v1/documents/pdf` (cap **3**); not `/pdf/batch` |
| Distinct from #361 | #361 = slow processing; #378 = admit hang / mislabeled stuck |
| Fix posture | Reproduce → non-blocking wake enqueue → per-file UI timeout → multi-PDF e2e → docs truth |

```ascii
  WebUI (≤3 parallel admits)
       │  N × POST /documents/pdf
       ▼
  BYTEA + durable task row + wake enqueue
       │
       ├─ hang / no task_id ──► #378 (this spec)
       │
       ▼
  Convert (vision≈2) → Insert → KG
       │
       └─ slow wall clock ──► #361 / SPEC-122 (out of fix scope)
```

## Document map

```ascii
 00-why
  → 01-first-principles (LAW-132-*)
  → 02-cross-ref-matrix
  → 03-code-as-is
  → 04-target-architecture
  → 05-lenses/ (PO, fullstack, DB, UX, front, AI)
  → 06-ux-ui-spec
  → 07-implementation-plan
  → 08-test-protocol
  → 09-acceptance
  → 10-edge-cases
  → 11-honest-assessment
  → 12-reproduction
  → zz-raw.md (intake, not the contract)
```

## Status board

| ID | Item | Status |
|----|------|--------|
| D0 | Intake `zz-raw.md` / #378 | Done |
| D1 | Doc pack (this folder) | Done |
| R1 | Reproduce + classify arms | Done — Plane A OK locally; wake hang residual fixed |
| I1 | Non-blocking wake enqueue SSOT | Done |
| I2 | WebUI per-file timeout isolation | Done |
| I3 | Docs: PDF ≠ `/upload/batch` | Done |
| T1 | Multi-PDF Playwright + Rust admit-non-block | Done |
| C1 | GitHub #378 root-cause comment | Done |

## Related

- [#378](https://github.com/raphaelmansuy/edgequake/issues/378) — this bug
- [#361](https://github.com/raphaelmansuy/edgequake/issues/361) / [#365](https://github.com/raphaelmansuy/edgequake/issues/365) — capacity / slow (SPEC-122)
- [SPEC-014](../014-multi/) — `/documents/pdf/batch`
- [SPEC-054](../054-fix-bugs-17/) — progress identity (`task_id`)
- [SPEC-091](../091-simplify-data-layer/) — F-091-19 / LD-12 wake channel
- [SPEC-098](../098-data-access-hardening/) / GH-350 — WebUI N× admits
- [SPEC-122](../122-implementation/) — bulk latency honesty
- [SPEC-123](../123-env-config-priority/) — `/upload/batch` rejects PDFs

## Non-goals

- Unbounded vision / extract parallelism
- Closing #361/#365 latency SLOs
- Switching WebUI to `/pdf/batch` as the primary path
- Widening `documents_valid_status` CHECK
