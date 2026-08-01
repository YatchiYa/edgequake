# Evidence — SPEC-099 Documents screenshots

Captured from local `/documents` (v0.22.x) during SPEC-099 authoring. Used as cross-ref for findings in [`01-finding-register.md`](../01-finding-register.md) and WHY in [`00-why.md`](../00-why.md).

| File | Caption | Primary findings |
|------|---------|------------------|
| [`01-idle-completed.png`](01-idle-completed.png) | **Idle inventory** — 11 completed docs; dual `Completed` + `Ready` pills; large dropzone; Clear All peer to Refresh; NEW badges; Cost column always on | F-099-02, F-099-05, F-099-11, F-099-12 |
| [`02-busy-active-runs.png`](02-busy-active-runs.png) | **Busy multi-upload** — Working/Queued chips; quiet dropzone; tall Active runs cards; table still shows Queued/Uploading; toast “Uploading 7 file(s)…”; header 17 vs All Status 11 | F-099-03, F-099-04, F-099-06, F-099-10 |
| [`03-legacy-active-card.png`](03-legacy-active-card.png) | **Legacy tall active card** — single uploading card with Admit→Materialize stepper + overall %; illustrates card density lineage still present in busy UI | F-099-06 |

## ASCII — idle vs busy (from evidence)

```ascii
01 IDLE                                      02 BUSY
┌ Header · Clear All ──────────┐             ┌ Header · Working/Queued ───────┐
├ Search · Filter · Sort ──────┤             ├ Search · quiet dropzone ───────┤
├ LARGE dropzone ──────────────┤             ├ Active runs (≤35vh scroll) ────┤
├ Table Completed+Ready × N ───┤             ├ Table same docs status again ──┤
└ NEW · Cost always on ────────┘             └ Toast Uploading N… ────────────┘
```
