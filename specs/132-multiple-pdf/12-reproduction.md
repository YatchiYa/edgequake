# 12 — Reproduction

## Environment

| Field | Value |
|-------|-------|
| Date | 2026-08-16 |
| Stack | `make status` — backend `:8090` v0.24.4, frontend `:3010`, Postgres 18, Ollama |
| Version | 0.24.4 (`git_hash` 9f9b9a218) |
| Auth | `auth_enabled=false` (dev) |
| Vision host | Ollama local |

## Arms

| Arm | Procedure | Result (admit 2xx + task_id?) | Classification |
|-----|-----------|-------------------------------|----------------|
| A | 1 small PDF (`enable_vision=false`) | **Yes** — `200` in ~97ms, `status=queued`, `task_id=pdf-…` | Plane A OK |
| B | 2 unique PDFs concurrent | **Yes** — both `200` + distinct `task_id`s (~70–135ms) | Plane A OK; #378 not “multi missing” |
| B′ | 2 identical content hashes | `200` but `status=duplicate` | Not hang; UI must show duplicate dialog |
| C | 5 unique PDFs concurrent | **Yes** — all `200` within ~120ms (cap-3 is WebUI-only; API unbounded) | Plane A OK under API fan-out |
| D | 2 PDFs `enable_vision=true` | **Yes** — both admitted with `task_id` | LAW-132-1: vision tax is Plane B |
| E | Code path: `ChannelTaskQueue::send` = `send().await` | **Hang risk confirmed in source** when capacity full; QW2 e2e does not prove non-block | F-091-19 residual — fix in WP-2 |

## Network evidence

- Endpoint: `POST http://localhost:8090/api/v1/documents/pdf`
- Headers: `X-Tenant-ID`, `X-Workspace-ID`
- Same-byte PDF → duplicate admit message (not stuck)
- Distinct bytes → distinct `task_id`s

## Verdict

```ascii
  Happy-path multi-PDF admit on local 0.24.4: WORKS (Plane A)
       │
       ├─ Reporter “not uploading” ≠ missing multi-select
       ├─ Likely confusion: duplicate / Plane B queue / Docker vision
       └─ Still ship: wake non-block (E) + UI isolation + multi-PDF e2e + docs
```

**#378 is real as honesty/edge-case class**, not as “API cannot admit two PDFs” on this stack. Capacity wall-clock remains SPEC-122 (#361/#365).

## Cross-refs

- Why: [00-why.md](00-why.md)
- Acceptance A2: [09-acceptance.md](09-acceptance.md)
