# SPEC-048 Screenshot Analysis

Generated: 2026-07-11 (Playwright chromium · mocked API)  
Source: `edgequake_webui/e2e/spec048-ingestion-progress.spec.ts`

## Verdict matrix

| ID | Scenario | Pass? | Notes |
|----|----------|-------|-------|
| S01 | Idle | **PASS** | No pill/banner; state cleared |
| S02 | Working parity | **PASS** | Stage headline + Extraction progress |
| S03 | Server stepper | **PASS** | Full timeline + step detail `42/351` |
| S04 | Queued-only | **PASS** | Amber Queued; admission chip |
| S05 | Stuck | **PASS** | Red only with recovery signal / aged orphan |
| S05b | Fresh upload | **PASS** | Amber Queued — **never red** on new upload |
| S06 | Dialog | **PASS** | 12% parity with banner |
| S07 | Embedding detail | **PASS** | Per-step `80/200` |
| S08 | Merge mode | **PASS** | Early stages skipped; mode badge |
| S09 | Failed extract | **PASS** | ActiveRunsPanel cleared at end |
| S10 | Markdown skip convert | **PASS** | Converting skipped |

## Stuck vs Queued (user-reported fix)

Fresh uploads (`Chanel_Loop.pdf` style) with `pending`/`queued` and no tasks yet are **Queued** (amber), not **Needs attention** (red).

Stuck requires: no queue coverage **and** (recovery message **or** aged >60s without `track_id`).

## Clear at end

- Completed docs drop out of `IngestionRunView` / ActiveRunsPanel
- Failed docs do not keep ActiveRunsPanel open
- Client upload rows pruned when matching docs are terminal

## Regenerate

```bash
cd edgequake_webui
pnpm exec playwright test e2e/spec048-ingestion-progress.spec.ts --project=chromium
```
