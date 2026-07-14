# 014 — Implementation Roadmap (P0–P3)

---

## P0 — Truth before chrome (ship first)

**Goal:** stop lying; one busy rule; no dead routes; stage reset on reprocess.

| Work | Owner | AC | Notes |
|------|-------|----|-------|
| Implement or remove `GET /ingestion/{track_id}/progress` | BE | AC-04 | Prefer implement to match FE |
| `PipelineActivity` endpoint + FE pill | BE+FE | AC-01 AC-12 | Replace ad-hoc `is_busy` |
| Reprocess stage field reset | BE | AC-03 | Before first LLM tick |
| WS ChunkProgress (+ GraphStorageProgress) | BE | AC-05 | Bridge pipeline callbacks |
| Banner/row from shared `IngestionRunView` | FE | AC-02 | Even if UI chrome unchanged |
| Feature-detect progress URL | FE | AC-04 | Fail closed |

**Exit:** AC-01…AC-05 green on smoke workspace.

---

## P1 — Transparent run surface

| Work | Owner | AC |
|------|-------|----|
| ActiveRunsPanel + ServerStageStepper | FE UI | AC-06 |
| Morph upload FSM → server stepper | FE | AC-06 |
| Run detail dialog timeline | FE | AC-02 AC-11 |
| `mode` on progress DTO + badge | BE FE | AC-07 |
| Tab title Working vs Queued | FE | AC-08 |
| Filter display-status SSOT | FE | AC-09 |

**Exit:** Screen matches [009-B](./009-screens-ascii.md) for one active PDF.

---

## P2 — Polish & i18n

| Work | Owner | AC |
|------|-------|----|
| Full `ingestion.stage.*` i18n | FE | AC-10 |
| Heartbeat / stuck heuristic | FE FS | AC-11 |
| Mute completed rows while Working | UI | — |
| Cost live alignment | FE | — |
| Duplicate subline cleanup | FE | AC-02 |

---

## P3 — Ops-grade history

| Work | Owner | Notes |
|------|-------|-------|
| Persistent run timeline in KV | BE | Fivetran-like |
| Partial failure retry UX | FE BE | |
| Multi-run queue visualization | UI | |
| Metrics: `ingest.ui.skew` | FS | [008](./008-lens-fullstack.md) |

---

## Suggested sequence (dependency)

```text
  DEF-01/04 APIs ──► RunView FE ──► Banner/Row parity
       │                  │
       ▼                  ▼
  WS ticks (DEF-02)   Stepper morph (DEF-10)
       │                  │
       └────────┬─────────┘
                ▼
         Reprocess reset (DEF-03) + mode badge (P7e)
                ▼
         P1 dialog + P2 i18n
```

---

## Out of scope (reminders)

- SPEC-047 Acc/F1 work  
- Full Documents redesign  
- Replacing WS/React Query wholesale  

---

## First PR suggestion (minimal)

1. BE: progress route + activity DTO + reprocess reset  
2. FE: `buildIngestionRunView` + wire banner/row/pill  
3. Tests: contract AC-01 AC-03 AC-04 + Playwright AC-02  

Then WS ticks + stepper morph as PR2.
