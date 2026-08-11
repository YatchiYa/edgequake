# Lens 002 — Full Stack Developer

## Stake

WebUI, API admit, and worker fairness are three layers partners experience as one “upload.” Bugs and UX lies hide in the seams.

## As-is hotspots

| Layer | Path | Risk |
|-------|------|------|
| Transfer bound | `bounded-file-upload.ts` (=3) | Mistaken for ingest parallelism |
| Admit router | `perform-file-upload.ts` | PDF vs text paths |
| Batch API | `batch_upload.rs` | Serial admit; unused by WebUI |
| Workers | `edgequake-tasks/src/worker.rs` | Tenant park |
| Extract/embed | pipeline semaphores | Local=1 |

## Target work

1. Progress/ETA wiring from task status + queue-metrics (Phase A).
2. Shared concurrency constants documentation (DRY) — avoid hardcoding a second matrix in FE.
3. Measurement harness calling same admit APIs as WebUI.
4. Do **not** bypass fairness for “batch mode.”

## Anti-patterns

- Calling `/upload/batch` from WebUI without updating UX semantics
- Marking row Completed on PDF convert alone
- Raising FE transfer concurrency to “speed processing”

## Cross-refs

- Code as-is: [../03-code-as-is.md](../03-code-as-is.md)
- UX: [../06-ux-ui-spec.md](../06-ux-ui-spec.md)
